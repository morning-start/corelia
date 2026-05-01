// 插件注册表模块
// 负责：维护插件实例状态、生命周期管理、多索引查询系统
//
// 核心功能：
// - 线程安全的插件注册/注销（RwLock 多读单写）
// - 双重索引：ID 索引 + 前缀索引
// - 状态机管理：合法性校验防止非法状态转换
// - 便捷查询：按前缀搜索、获取活跃插件等

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::RwLock;

use crate::plugins::loader::{PluginInstance, PluginState, PluginManifest};

// ==================== 数据结构定义 ====================

/// 注册表内部数据结构
struct RegistryData {
    /// 按 ID 索引：plugin_id -> instances 数组索引
    by_id: HashMap<String, usize>,

    /// 按前缀索引：prefix -> [instances 数组索引]
    by_prefix: HashMap<String, Vec<usize>>,

    /// 所有插件实例
    instances: Vec<PluginInstance>,
}

// ==================== PluginRegistry 主结构体 ====================

/// 插件注册表（线程安全）
///
/// 使用 RwLock 实现多读单写的并发访问模式：
/// - 读操作（查询）可以并发执行
/// - 写操作（注册/注销/更新状态）独占访问
///
/// # Example
/// ```rust
/// let registry = PluginRegistry::new();
/// registry.register(instance)?;
/// let plugins = registry.search_by_prefix("hw");
/// ```
pub struct PluginRegistry {
    inner: RwLock<RegistryData>,
}

impl PluginRegistry {
    /// 创建新的空注册表
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(RegistryData {
                by_id: HashMap::new(),
                by_prefix: HashMap::new(),
                instances: Vec::new(),
            }),
        }
    }

    /// 注册插件实例
    ///
    /// 将插件添加到注册表中，同时维护 ID 索引和前缀索引。
    /// 如果插件已存在（相同 ID），返回错误。
    ///
    /// # Arguments
    /// - `instance`: 要注册的插件实例
    ///
    /// # Returns
    /// - `Ok(())`: 注册成功
    /// - `Err(String)`: 错误信息（如插件已存在、锁获取失败等）
    pub fn register(&self, instance: PluginInstance) -> Result<(), String> {
        let mut data = self.inner.write().map_err(|e| e.to_string())?;

        let plugin_id = instance.manifest.name.clone();

        // 检查是否已存在
        if data.by_id.contains_key(&plugin_id) {
            return Err(format!("Plugin '{}' already registered", plugin_id));
        }

        // 获取新索引
        let index = data.instances.len();

        // 添加到 ID 索引
        data.by_id.insert(plugin_id.clone(), index);

        // 如果有前缀，添加到前缀索引
        if let Some(ref prefix) = instance.manifest.prefix {
            data.by_prefix
                .entry(prefix.clone())
                .or_insert_with(Vec::new)
                .push(index);
        }

        // 添加到实例列表
        data.instances.push(instance);

        println!("[Registry] Registered plugin '{}' (index={})", plugin_id, index);
        Ok(())
    }

    /// 注销插件
    ///
    /// 从注册表中移除指定插件，并清理所有相关索引。
    /// 移除后会更新所有后续索引以保持一致性。
    ///
    /// # Arguments
    /// - `id`: 要注销的插件 ID
    ///
    /// # Returns
    /// - `Ok(())`: 注销成功
    /// - `Err(String)`: 错误信息（如插件不存在、锁获取失败等）
    pub fn unregister(&self, id: &str) -> Result<(), String> {
        let mut data = self.inner.write().map_err(|e| e.to_string())?;

        // 从 ID 索引查找
        let index = match data.by_id.remove(id) {
            Some(idx) => idx,
            None => return Err(format!("Plugin '{}' not found", id)),
        };

        // 获取实例以清理前缀索引（先提取 prefix，避免借用冲突）
        let prefix_to_clean = data.instances.get(index)
            .and_then(|inst| inst.manifest.prefix.clone());

        if let Some(ref prefix) = prefix_to_clean {
            if let Some(indices) = data.by_prefix.get_mut(prefix) {
                indices.retain(|&i| i != index);
                if indices.is_empty() {
                    data.by_prefix.remove(prefix);
                }
            }
        }

        // 移除实例
        data.instances.remove(index);

        // 更新所有后续索引（因为删除了元素）
        for (_, idx) in data.by_id.iter_mut() {
            if *idx > index {
                *idx -= 1;
            }
        }
        for indices in data.by_prefix.values_mut() {
            for idx in indices.iter_mut() {
                if *idx > index {
                    *idx -= 1;
                }
            }
        }

        println!("[Registry] Unregistered plugin '{}'", id);
        Ok(())
    }

    /// 根据 ID 获取插件（返回克隆）
    ///
    /// # Arguments
    /// - `id`: 插件标识符
    ///
    /// # Returns
    /// - `Some(PluginInstance)`: 找到的插件实例（克隆）
    /// - `None`: 插件不存在或锁获取失败
    pub fn get(&self, id: &str) -> Option<PluginInstance> {
        let data = self.inner.read().ok()?;
        let index = data.by_id.get(id)?;
        data.instances.get(*index).cloned()
    }

    /// 根据前缀查询匹配的插件列表
    ///
    /// 支持双向部分匹配：
    /// - 如果输入前缀是某个插件 prefix 的前缀，则匹配
    /// - 如果某个插件的 prefix 是输入前缀的前缀，也匹配
    ///
    /// # Arguments
    /// - `query`: 要搜索的前缀字符串
    ///
    /// # Returns
    /// 匹配到的插件列表（已去重）
    ///
    /// # Example
    /// ```rust
    /// // 假设有插件 prefix 为 "hw"
    /// let results = registry.search_by_prefix("h");      // 匹配到 hello-world
    /// let results = registry.search_by_prefix("helloworld"); // 也匹配到 hello-world
    /// ```
    pub fn search_by_prefix(&self, query: &str) -> Vec<PluginInstance> {
        let data = match self.inner.read() {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        let mut results = Vec::new();

        // 遍历所有前缀进行模糊匹配
        for (prefix, indices) in &data.by_prefix {
            // 支持双向部分匹配
            if prefix.starts_with(query) || query.starts_with(prefix.as_str()) {
                for &index in indices {
                    if let Some(instance) = data.instances.get(index) {
                        results.push(instance.clone());
                    }
                }
            }
        }

        // 去重（同一个插件可能有多个匹配的前缀）
        results.dedup_by(|a, b| a.manifest.name == b.manifest.name);

        results
    }

    /// 获取所有活跃的插件（Ready/Cached 状态）
    ///
    /// # Returns
    /// 状态为 Ready 或 Cached 的插件列表
    pub fn get_active_plugins(&self) -> Vec<PluginInstance> {
        let data = match self.inner.read() {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        data.instances
            .iter()
            .filter(|p| p.state == PluginState::Ready || p.state == PluginState::Cached)
            .cloned()
            .collect()
    }

    /// 获取所有已注册插件
    ///
    /// # Returns
    /// 所有已注册插件的列表
    pub fn list_all(&self) -> Vec<PluginInstance> {
        let data = match self.inner.read() {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        data.instances.clone()
    }

    /// 更新插件状态
    ///
    /// 执行状态转换时会进行合法性校验，
    /// 非法转换会被拒绝并返回错误。
    ///
    /// # Arguments
    /// - `id`: 插件标识符
    /// - `new_state`: 目标状态
    ///
    /// # Returns
    /// - `Ok(())`: 状态更新成功
    /// - `Err(String)`: 错误信息（如插件不存在、非法状态转换等）
    ///
    /// # State Machine
    /// ```
    /// MetaLoaded ──→ Loading ──→ Ready/Cached/Error
    ///     ↑                      │
    ///     └──────── Unloaded ←───┘
    /// ```
    pub fn update_state(&self, id: &str, new_state: PluginState) -> Result<(), String> {
        let mut data = self.inner.write().map_err(|e| e.to_string())?;

        let index = match data.by_id.get(id) {
            Some(&idx) => idx,
            None => return Err(format!("Plugin '{}' not found", id)),
        };

        if let Some(instance) = data.instances.get_mut(index) {
            let old_state = instance.state.clone();

            // 状态机校验
            if !Self::is_valid_transition(&old_state, &new_state) {
                return Err(format!(
                    "Invalid state transition: {:?} -> {:?} for plugin '{}'",
                    old_state, new_state, id
                ));
            }

            instance.state = new_state.clone();
            println!(
                "[Registry] Plugin '{}': {:?} -> {:?}",
                id, old_state, new_state
            );
        }

        Ok(())
    }

    /// 验证状态转换是否合法
    ///
    /// 合法转换规则：
    /// - MetaLoaded → Loading | Unloaded
    /// - Loading → Ready | Cached | Error
    /// - Ready → Cached | Unloaded | Error
    /// - Cached → Loading | Unloaded
    /// - Unloaded → Loading
    /// - Error → Loading | Unloaded
    /// - 相同状态（幂等操作允许）
    fn is_valid_transition(from: &PluginState, to: &PluginState) -> bool {
        use PluginState::*;
        match (from, to) {
            // 合法转换
            (MetaLoaded, Loading) | (MetaLoaded, Unloaded) => true,
            (Loading, Ready) | (Loading, Cached) | (Loading, Error(_)) => true,
            (Ready, Cached) | (Ready, Unloaded) | (Ready, Error(_)) => true,
            (Cached, Loading) | (Cached, Unloaded) => true,
            (Unloaded, Loading) => true,
            (Error(_), Loading) | (Error(_), Unloaded) => true,

            // 相同状态（幂等操作允许）
            _ if from == to => true,

            // 其他情况非法
            _ => false,
        }
    }

    /// 获取注册的插件数量
    ///
    /// # Returns
    /// 已注册插件的总数
    pub fn count(&self) -> usize {
        let data = match self.inner.read() {
            Ok(d) => d,
            Err(_) => return 0,
        };
        data.instances.len()
    }

    /// 清空注册表
    ///
    /// 移除所有插件和索引，重置为初始状态
    pub fn clear(&self) {
        if let Ok(mut data) = self.inner.write() {
            data.by_id.clear();
            data.by_prefix.clear();
            data.instances.clear();
            println!("[Registry] Registry cleared");
        }
  }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::loader::{PluginManifest, PluginState, PluginInstance};
    use std::path::PathBuf;

    fn create_test_manifest(name: &str, prefix: Option<&str>) -> PluginManifest {
        PluginManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            plugin_type: "quickjs".to_string(),
            logo: None,
            prefix: prefix.map(|p| p.to_string()),
            main: Some("index.js".to_string()),
            description: Some(format!("Test plugin: {}", name)),
            author: Some("Test Author".to_string()),
            patches: Vec::new(),
            features: None,
        }
    }

    fn create_test_instance(name: &str, prefix: Option<&str>) -> PluginInstance {
        PluginInstance {
            id: name.to_string(),
            manifest: create_test_manifest(name, prefix),
            state: PluginState::MetaLoaded,
            vm_id: None,
            plugin_dir: PathBuf::from("/tmp/test"),
            loaded_at: None,
            last_used: None,
            registered_features: Vec::new(),
            on_ready_callback: None,
            on_out_callback: None,
            load_error_count: 0,
            max_retries: 3,
            last_error: None,
            retry_after: None,
            retry_backoff_ms: 1000,
        }
    }

    #[test]
    fn test_create_registry() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_register_plugin() {
        let registry = PluginRegistry::new();
        let instance = create_test_instance("test-plugin", Some("tp"));
        
        let result = registry.register(instance.clone());
        assert!(result.is_ok());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_register_duplicate_plugin() {
        let registry = PluginRegistry::new();
        let instance = create_test_instance("test-plugin", Some("tp"));
        
        registry.register(instance.clone()).unwrap();
        let result = registry.register(instance);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("已存在"));
    }

    #[test]
    fn test_unregister_plugin() {
        let registry = PluginRegistry::new();
        let instance = create_test_instance("test-plugin", Some("tp"));
        
        registry.register(instance).unwrap();
        assert_eq!(registry.count(), 1);
        
        let result = registry.unregister("test-plugin");
        assert!(result.is_ok());
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_unregister_nonexistent_plugin() {
        let registry = PluginRegistry::new();
        
        let result = registry.unregister("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不存在"));
    }

    #[test]
    fn test_get_plugin() {
        let registry = PluginRegistry::new();
        let instance = create_test_instance("test-plugin", Some("tp"));
        
        registry.register(instance.clone()).unwrap();
        
        let retrieved = registry.get("test-plugin");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().manifest.name, "test-plugin");
    }

    #[test]
    fn test_search_by_prefix_exact() {
        let registry = PluginRegistry::new();
        
        registry.register(create_test_instance("plugin1", Some("p1"))).unwrap();
        registry.register(create_test_instance("plugin2", Some("p2"))).unwrap();
        
        let results = registry.search_by_prefix("p1");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].manifest.name, "plugin1");
    }

    #[test]
    fn test_search_by_prefix_partial() {
        let registry = PluginRegistry::new();
        
        registry.register(create_test_instance("plugin1", Some("test-prefix"))).unwrap();
        
        // 支持双向部分匹配
        let results1 = registry.search_by_prefix("test");
        assert_eq!(results1.len(), 1);
        
        let results2 = registry.search_by_prefix("test-prefix-longer");
        assert_eq!(results2.len(), 1);
    }

    #[test]
    fn test_search_by_prefix_no_match() {
        let registry = PluginRegistry::new();
        
        registry.register(create_test_instance("plugin1", Some("p1"))).unwrap();
        
        let results = registry.search_by_prefix("nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_update_state() {
        let registry = PluginRegistry::new();
        
        let mut instance = create_test_instance("test-plugin", Some("tp"));
        instance.state = PluginState::MetaLoaded;
        registry.register(instance).unwrap();
        
        let result = registry.update_state("test-plugin", PluginState::Ready);
        assert!(result.is_ok());
        
        let updated = registry.get("test-plugin").unwrap();
        assert!(matches!(updated.state, PluginState::Ready));
    }

    #[test]
    fn test_invalid_state_transition() {
        let registry = PluginRegistry::new();
        
        let mut instance = create_test_instance("test-plugin", Some("tp"));
        instance.state = PluginState::Ready;
        registry.register(instance).unwrap();
        
        // Ready -> MetaLoaded 是无效的转换
        let result = registry.update_state("test-plugin", PluginState::MetaLoaded);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_active_plugins() {
        let registry = PluginRegistry::new();
        
        let mut instance1 = create_test_instance("plugin1", Some("p1"));
        instance1.state = PluginState::Ready;
        registry.register(instance1).unwrap();
        
        let mut instance2 = create_test_instance("plugin2", Some("p2"));
        instance2.state = PluginState::Cached;
        registry.register(instance2).unwrap();
        
        let mut instance3 = create_test_instance("plugin3", Some("p3"));
        instance3.state = PluginState::Error("test".to_string());
        registry.register(instance3).unwrap();
        
        let active = registry.get_active_plugins();
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_list_all() {
        let registry = PluginRegistry::new();
        
        registry.register(create_test_instance("plugin1", Some("p1"))).unwrap();
        registry.register(create_test_instance("plugin2", Some("p2"))).unwrap();
        
        let all = registry.list_all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_clear() {
        let registry = PluginRegistry::new();
        
        registry.register(create_test_instance("plugin1", Some("p1"))).unwrap();
        registry.register(create_test_instance("plugin2", Some("p2"))).unwrap();
        
        assert_eq!(registry.count(), 2);
        registry.clear();
        assert_eq!(registry.count(), 0);
    }
}

// ==================== Tauri Commands ====================
//
// 这些命令供前端通过 IPC 调用，提供便捷的插件查询接口

/// 根据前缀搜索插件
///
/// 前端调用示例：
/// ```typescript
/// const results = await invoke('search_plugins_by_prefix', { prefix: 'hw' });
/// results.forEach(p => console.log(p.name));
/// ```
#[tauri::command]
pub async fn search_plugins_by_prefix(
    registry: tauri::State<'_, std::sync::RwLock<PluginRegistry>>,
    prefix: String,
) -> Result<Vec<PluginManifest>, String> {
    let registry = registry.read().map_err(|e| e.to_string())?;
    let results = registry.search_by_prefix(&prefix);
    Ok(results.into_iter().map(|p| p.manifest).collect())
}

/// 获取所有活跃插件
///
/// 前端调用示例：
/// ```typescript
/// const activePlugins = await invoke('get_active_plugins');
/// console.log(`活跃插件数: ${activePlugins.length}`);
/// ```
#[tauri::command]
pub async fn get_active_plugins(
    registry: tauri::State<'_, std::sync::RwLock<PluginRegistry>>,
) -> Result<Vec<PluginManifest>, String> {
    let registry = registry.read().map_err(|e| e.to_string())?;
    let plugins = registry.get_active_plugins();
    Ok(plugins.into_iter().map(|p| p.manifest).collect())
}

/// 获取插件状态
///
/// 前端调用示例：
/// ```typescript
/// const state = await invoke('get_plugin_state', { id: 'hello-world' });
/// console.log(state); // "Ready" 或 "Loading" 等
/// ```
#[tauri::command]
pub async fn get_plugin_state(
    registry: tauri::State<'_, std::sync::RwLock<PluginRegistry>>,
    id: String,
) -> Result<String, String> {
    let registry = registry.read().map_err(|e| e.to_string())?;
    match registry.get(&id) {
        Some(instance) => Ok(format!("{:?}", instance.state)),
        None => Err(format!("Plugin '{}' not found", id)),
    }
}

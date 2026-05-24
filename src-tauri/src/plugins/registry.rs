// 插件注册表模块

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
// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::plugins::loader::PluginState;

    fn create_test_instance(name: &str, prefix: Option<&str>, state: PluginState) -> PluginInstance {
        PluginInstance {
            id: name.to_string(),
            manifest: PluginManifest {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                plugin_type: "app".to_string(),
                logo: None,
                prefix: prefix.map(|s| s.to_string()),
                main: Some("index.js".to_string()),
                description: Some(format!("Test plugin {}", name)),
                author: Some("test".to_string()),
                patches: vec![],
                features: None,
            },
            state,
            vm_id: None,
            plugin_dir: PathBuf::from("/tmp/test-plugins").join(name),
            loaded_at: None,
            last_used: None,
            registered_features: vec![],
            on_ready_callback: None,
            on_out_callback: None,
            load_error_count: 0,
            max_retries: 3,
            last_error: None,
            retry_after: None,
            retry_backoff_ms: 1000,
        }
    }

    // ==================== 注册测试 ====================

    #[test]
    fn test_register_new_plugin() {
        let registry = PluginRegistry::new();
        let instance = create_test_instance("hello-world", None, PluginState::MetaLoaded);

        assert!(registry.register(instance).is_ok());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_register_duplicate_plugin() {
        let registry = PluginRegistry::new();
        let instance = create_test_instance("hello-world", None, PluginState::MetaLoaded);

        registry.register(instance).unwrap();
        let duplicate = create_test_instance("hello-world", None, PluginState::MetaLoaded);

        let err = registry.register(duplicate).unwrap_err();
        assert!(err.contains("already registered"));
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_register_plugin_with_prefix() {
        let registry = PluginRegistry::new();
        let instance = create_test_instance("hello-world", Some("hw"), PluginState::MetaLoaded);

        assert!(registry.register(instance).is_ok());
        let results = registry.search_by_prefix("hw");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].manifest.name, "hello-world");
    }

    #[test]
    fn test_register_multiple_plugins() {
        let registry = PluginRegistry::new();

        registry.register(create_test_instance("plugin-a", Some("pa"), PluginState::MetaLoaded)).unwrap();
        registry.register(create_test_instance("plugin-b", Some("pb"), PluginState::MetaLoaded)).unwrap();
        registry.register(create_test_instance("plugin-c", None, PluginState::MetaLoaded)).unwrap();

        assert_eq!(registry.count(), 3);
        assert_eq!(registry.list_all().len(), 3);
    }

    // ==================== 注销测试 ====================

    #[test]
    fn test_unregister_existing_plugin() {
        let registry = PluginRegistry::new();
        let instance = create_test_instance("hello-world", None, PluginState::MetaLoaded);
        registry.register(instance).unwrap();

        assert!(registry.unregister("hello-world").is_ok());
        assert_eq!(registry.count(), 0);
        assert!(registry.get("hello-world").is_none());
    }

    #[test]
    fn test_unregister_nonexistent_plugin() {
        let registry = PluginRegistry::new();
        let err = registry.unregister("non-existent").unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_unregister_plugin_with_prefix_cleanup() {
        let registry = PluginRegistry::new();
        registry.register(create_test_instance("hello-world", Some("hw"), PluginState::MetaLoaded)).unwrap();

        assert!(registry.unregister("hello-world").is_ok());
        // 卸载后通过前缀搜索应返回空
        assert!(registry.search_by_prefix("hw").is_empty());
    }

    // ==================== 获取测试 ====================

    #[test]
    fn test_get_existing_plugin() {
        let registry = PluginRegistry::new();
        registry.register(create_test_instance("hello-world", None, PluginState::MetaLoaded)).unwrap();

        let instance = registry.get("hello-world");
        assert!(instance.is_some());
        assert_eq!(instance.unwrap().manifest.name, "hello-world");
    }

    #[test]
    fn test_get_nonexistent_plugin() {
        let registry = PluginRegistry::new();
        assert!(registry.get("non-existent").is_none());
    }

    // ==================== 前缀搜索测试 ====================

    #[test]
    fn test_search_by_prefix_exact_match() {
        let registry = PluginRegistry::new();
        registry.register(create_test_instance("hello-world", Some("hw"), PluginState::Ready)).unwrap();

        let results = registry.search_by_prefix("hw");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_by_prefix_partial_match() {
        let registry = PluginRegistry::new();
        registry.register(create_test_instance("hello-world", Some("hw"), PluginState::Ready)).unwrap();

        let results = registry.search_by_prefix("h");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_by_prefix_reverse_match() {
        let registry = PluginRegistry::new();
        registry.register(create_test_instance("hello-world", Some("hw"), PluginState::Ready)).unwrap();

        let results = registry.search_by_prefix("hw-tool");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_by_prefix_no_match() {
        let registry = PluginRegistry::new();
        registry.register(create_test_instance("hello-world", Some("hw"), PluginState::Ready)).unwrap();

        let results = registry.search_by_prefix("zzz");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_by_prefix_dedup() {
        let registry = PluginRegistry::new();
        // 同一个插件有多个前缀匹配时应该去重
        // 注册两个不同插件，其中一个前缀是另一个的前缀
        registry.register(create_test_instance("plugin-a", Some("pa"), PluginState::Ready)).unwrap();
        registry.register(create_test_instance("plugin-b", Some("pb"), PluginState::Ready)).unwrap();

        let results = registry.search_by_prefix("p");
        assert_eq!(results.len(), 2);
        let mut names: Vec<&str> = results.iter().map(|p| p.manifest.name.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["plugin-a", "plugin-b"]);
    }

    // ==================== 活跃插件测试 ====================

    #[test]
    fn test_get_active_plugins_filters_correctly() {
        let registry = PluginRegistry::new();
        registry.register(create_test_instance("ready-plugin", None, PluginState::Ready)).unwrap();
        registry.register(create_test_instance("cached-plugin", None, PluginState::Cached)).unwrap();
        registry.register(create_test_instance("loading-plugin", None, PluginState::Loading)).unwrap();
        registry.register(create_test_instance("error-plugin", None, PluginState::Error("fail".to_string()))).unwrap();
        registry.register(create_test_instance("unloaded-plugin", None, PluginState::Unloaded)).unwrap();

        let active = registry.get_active_plugins();
        assert_eq!(active.len(), 2);
        let names: Vec<&str> = active.iter().map(|p| p.manifest.name.as_str()).collect();
        assert!(names.contains(&"ready-plugin"));
        assert!(names.contains(&"cached-plugin"));
        assert!(!names.contains(&"loading-plugin"));
    }

    // ==================== 状态转换测试 ====================

    #[test]
    fn test_update_state_valid_transition() {
        let registry = PluginRegistry::new();
        registry.register(create_test_instance("test", None, PluginState::MetaLoaded)).unwrap();

        assert!(registry.update_state("test", PluginState::Loading).is_ok());
        assert!(registry.update_state("test", PluginState::Ready).is_ok());
    }

    #[test]
    fn test_update_state_invalid_transition() {
        let registry = PluginRegistry::new();
        registry.register(create_test_instance("test", None, PluginState::MetaLoaded)).unwrap();

        let err = registry.update_state("test", PluginState::Ready).unwrap_err();
        assert!(err.contains("Invalid state transition"));
    }

    #[test]
    fn test_update_state_nonexistent_plugin() {
        let registry = PluginRegistry::new();
        let err = registry.update_state("non-existent", PluginState::Loading).unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_update_state_same_state_idempotent() {
        let registry = PluginRegistry::new();
        registry.register(create_test_instance("test", None, PluginState::MetaLoaded)).unwrap();

        assert!(registry.update_state("test", PluginState::MetaLoaded).is_ok());
    }

    // ==================== 状态机合法性测试 ====================

    #[test]
    fn test_is_valid_transition_all_valid() {
        use PluginState::*;

        let valid_pairs = &[
            (MetaLoaded, Loading),
            (MetaLoaded, Unloaded),
            (Loading, Ready),
            (Loading, Cached),
            (Loading, Error("".to_string())),
            (Ready, Cached),
            (Ready, Unloaded),
            (Ready, Error("".to_string())),
            (Cached, Loading),
            (Cached, Unloaded),
            (Unloaded, Loading),
            (Error("".to_string()), Loading),
            (Error("".to_string()), Unloaded),
        ];

        for (from, to) in valid_pairs {
            assert!(
                PluginRegistry::is_valid_transition(from, to),
                "Expected valid: {:?} -> {:?}",
                from, to
            );
        }
    }

    #[test]
    fn test_is_valid_transition_all_invalid() {
        use PluginState::*;

        let invalid_pairs = &[
            (MetaLoaded, Ready),
            (MetaLoaded, Cached),
            (MetaLoaded, Error("".to_string())),
            (Loading, MetaLoaded),
            (Loading, Unloaded),
            (Ready, MetaLoaded),
            (Ready, Loading),
            (Cached, MetaLoaded),
            (Cached, Ready),
            (Cached, Error("".to_string())),
            (Unloaded, MetaLoaded),
            (Unloaded, Ready),
            (Unloaded, Cached),
            (Unloaded, Error("".to_string())),
            (Error("".to_string()), MetaLoaded),
            (Error("".to_string()), Ready),
            (Error("".to_string()), Cached),
        ];

        for (from, to) in invalid_pairs {
            assert!(
                !PluginRegistry::is_valid_transition(from, to),
                "Expected invalid: {:?} -> {:?}",
                from, to
            );
        }
    }

    #[test]
    fn test_is_valid_transition_idempotent() {
        use PluginState::*;

        let all_states = &[MetaLoaded, Loading, Ready, Cached, Unloaded, Error("x".to_string())];
        for state in all_states {
            assert!(
                PluginRegistry::is_valid_transition(state, state),
                "Expected idempotent: {:?} -> {:?}",
                state, state
            );
        }
    }

    // ==================== 综合性测试 ====================

    #[test]
    fn test_full_lifecycle() {
        let registry = PluginRegistry::new();

        // 注册 → 查询 → 状态转换 → 活跃检查 → 注销 → 空
        assert_eq!(registry.count(), 0);

        registry.register(create_test_instance("my-plugin", Some("mp"), PluginState::MetaLoaded)).unwrap();
        assert_eq!(registry.count(), 1);

        let instance = registry.get("my-plugin").unwrap();
        assert_eq!(instance.manifest.name, "my-plugin");

        registry.update_state("my-plugin", PluginState::Loading).unwrap();
        registry.update_state("my-plugin", PluginState::Ready).unwrap();

        let active = registry.get_active_plugins();
        assert_eq!(active.len(), 1);

        let by_prefix = registry.search_by_prefix("mp");
        assert_eq!(by_prefix.len(), 1);

        registry.unregister("my-plugin").unwrap();
        assert_eq!(registry.count(), 0);
        assert!(registry.get_active_plugins().is_empty());
    }

    #[test]
    fn test_clear_resets_registry() {
        let registry = PluginRegistry::new();
        registry.register(create_test_instance("plugin-a", None, PluginState::MetaLoaded)).unwrap();
        registry.register(create_test_instance("plugin-b", None, PluginState::MetaLoaded)).unwrap();
        assert_eq!(registry.count(), 2);

        registry.clear();
        assert_eq!(registry.count(), 0);
        assert!(registry.list_all().is_empty());
        assert!(registry.get("plugin-a").is_none());
    }

    #[test]
    fn test_count_after_multiple_operations() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.count(), 0);

        registry.register(create_test_instance("a", None, PluginState::MetaLoaded)).unwrap();
        assert_eq!(registry.count(), 1);

        registry.register(create_test_instance("b", None, PluginState::MetaLoaded)).unwrap();
        assert_eq!(registry.count(), 2);

        registry.unregister("a").unwrap();
        assert_eq!(registry.count(), 1);

        registry.unregister("b").unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_search_by_prefix_empty_registry() {
        let registry = PluginRegistry::new();
        let results = registry.search_by_prefix("anything");
        assert!(results.is_empty());
    }

    #[test]
    fn test_get_active_plugins_empty_registry() {
        let registry = PluginRegistry::new();
        assert!(registry.get_active_plugins().is_empty());
    }

    #[test]
    fn test_list_all_empty_registry() {
        let registry = PluginRegistry::new();
        assert!(registry.list_all().is_empty());
    }
}

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

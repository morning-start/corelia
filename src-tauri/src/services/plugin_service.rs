//! 插件服务层
//!
//! 职责：
//! - 统一处理插件相关的业务逻辑
//! - 协调 PluginLoader 和 QuickJSRuntime
//! - 提供清晰的服务接口给 Commands 层
//!
//! 注意：此层仅包含业务逻辑，不涉及 Tauri State 或 Command 相关

use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use crate::plugins::loader::{PluginLoader, PluginManifest, PluginState, PluginHealth, PluginInstance};
use crate::plugins::quickjs_runtime::QuickJSRuntime;

/// 插件服务
pub struct PluginService {
    loader: Arc<Mutex<PluginLoader>>,
    runtime: Arc<QuickJSRuntime>,
}

impl PluginService {
    /// 创建新的插件服务
    pub fn new(loader: Arc<Mutex<PluginLoader>>, runtime: Arc<QuickJSRuntime>) -> Self {
        Self { loader, runtime }
    }

    /// 扫描并加载插件元数据
    pub fn scan_plugins(&self) -> Result<Vec<PluginManifest>, String> {
        let mut loader = self.loader.lock().map_err(|e| e.to_string())?;
        let _ = loader.scan_plugins()?;
        Ok(loader.list_manifests())
    }

    /// 获取所有插件列表
    pub fn list_plugins(&self) -> Result<Vec<PluginManifest>, String> {
        let loader = self.loader.lock().map_err(|e| e.to_string())?;
        Ok(loader.list_manifests())
    }

    /// 加载指定插件
    pub fn load_plugin(&self, plugin_id: &str) -> Result<(String, Option<String>), String> {
        let mut loader = self.loader.lock().map_err(|e| e.to_string())?;
        let (state, vm_id) = loader.load_plugin(plugin_id)?;
        Ok((format!("{}", state), vm_id))
    }

    /// 卸载指定插件
    pub fn unload_plugin(&self, plugin_id: &str) -> Result<(), String> {
        let mut loader = self.loader.lock().map_err(|e| e.to_string())?;
        loader.unload_plugin(plugin_id)
    }

    /// 按前缀搜索插件
    pub fn find_plugins_by_prefix(&self, prefix: &str) -> Result<Vec<PluginManifest>, String> {
        let loader = self.loader.lock().map_err(|e| e.to_string())?;
        Ok(loader
            .find_by_prefix(prefix)
            .into_iter()
            .map(|p| p.manifest.clone())
            .collect())
    }

    /// 清理闲置插件
    pub fn cleanup_idle_plugins(&self) -> Result<usize, String> {
        let mut loader = self.loader.lock().map_err(|e| e.to_string())?;
        let timeout = loader.idle_timeout_secs();
        Ok(loader.cleanup_idle_plugins(timeout))
    }

    /// 获取插件健康状态
    pub fn get_plugin_health(&self) -> Result<Vec<PluginHealth>, String> {
        let loader = self.loader.lock().map_err(|e| e.to_string())?;
        Ok(loader.get_plugin_health())
    }

    /// 执行插件代码
    pub fn execute_plugin_code(
        &self,
        plugin_id: &str,
        code: &str,
    ) -> Result<serde_json::Value, String> {
        let loader = self.loader.lock().map_err(|e| e.to_string())?;

        let instance = loader
            .get_plugin(plugin_id)
            .ok_or_else(|| format!("插件不存在: {}", plugin_id))?;

        let vm_id = instance
            .vm_id
            .as_ref()
            .ok_or_else(|| format!("插件 {} 未加载或 VM 未创建", plugin_id))?;

        if !matches!(instance.state, PluginState::Ready | PluginState::Cached) {
            return Err(format!(
                "插件 {} 当前状态不可用: {:?}",
                plugin_id, instance.state
            ));
        }

        self.runtime
            .execute(vm_id, code)
            .map_err(|e| format!("执行失败: {}", e))
    }

    /// 获取插件实例（用于内部使用）
    pub fn get_plugin_instance(&self, plugin_id: &str) -> Result<Option<PluginInstance>, String> {
        let loader = self.loader.lock().map_err(|e| e.to_string())?;
        Ok(loader.get_plugin(plugin_id).cloned())
    }

    /// 获取已加载插件数量
    pub fn loaded_plugin_count(&self) -> Result<usize, String> {
        let loader = self.loader.lock().map_err(|e| e.to_string())?;
        Ok(loader.loaded_count())
    }

    /// 获取总插件数量
    pub fn total_plugin_count(&self) -> Result<usize, String> {
        let loader = self.loader.lock().map_err(|e| e.to_string())?;
        Ok(loader.total_count())
    }
}

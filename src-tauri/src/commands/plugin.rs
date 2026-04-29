//! 插件相关命令
//!
//! 职责：
//! - 仅做参数验证和转发
//! - 不包含业务逻辑
//! - 业务逻辑委托给 PluginService

use std::sync::{Arc, Mutex};

use crate::plugins::loader::{PluginLoader, PluginManifest, PluginHealth, LoadResult};
use crate::plugins::quickjs_runtime::QuickJSRuntime;
use crate::services::PluginService;

/// 扫描并加载插件元数据
#[tauri::command]
pub fn scan_plugins(
    plugin_service: tauri::State<'_, PluginService>,
) -> Result<Vec<PluginManifest>, String> {
    plugin_service.scan_plugins()
}

/// 获取所有插件列表
#[tauri::command]
pub fn get_plugin_list(
    plugin_service: tauri::State<'_, PluginService>,
) -> Result<Vec<PluginManifest>, String> {
    plugin_service.list_plugins()
}

/// 加载指定插件
#[tauri::command]
pub fn load_plugin(
    plugin_service: tauri::State<'_, PluginService>,
    id: String,
) -> Result<LoadResult, String> {
    let (state, vm_id) = plugin_service.load_plugin(&id)?;
    Ok(LoadResult { state, vm_id })
}

/// 卸载指定插件
#[tauri::command]
pub fn unload_plugin(
    plugin_service: tauri::State<'_, PluginService>,
    id: String,
) -> Result<(), String> {
    plugin_service.unload_plugin(&id)
}

/// 按前缀搜索插件
#[tauri::command]
pub fn find_plugins_by_prefix(
    plugin_service: tauri::State<'_, PluginService>,
    prefix: String,
) -> Result<Vec<PluginManifest>, String> {
    plugin_service.find_plugins_by_prefix(&prefix)
}

/// 清理闲置插件
#[tauri::command]
pub fn cleanup_idle_plugins(
    plugin_service: tauri::State<'_, PluginService>,
) -> Result<usize, String> {
    plugin_service.cleanup_idle_plugins()
}

/// 获取插件健康状态
#[tauri::command]
pub fn get_plugin_health(
    plugin_service: tauri::State<'_, PluginService>,
) -> Result<Vec<PluginHealth>, String> {
    plugin_service.get_plugin_health()
}

/// 执行插件代码
#[tauri::command]
pub fn plugin_execute(
    plugin_service: tauri::State<'_, PluginService>,
    plugin_id: String,
    code: String,
) -> Result<serde_json::Value, String> {
    plugin_service.execute_plugin_code(&plugin_id, &code)
}

use std::sync::Mutex;
use crate::plugins::loader::types::{PluginManifest, PluginHealth, LoadResult, PluginState};
use crate::plugins::loader::PluginLoader;

/// 扫描插件目录
#[tauri::command]
pub fn scan_plugins(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
) -> Result<Vec<PluginManifest>, String> {
    let mut loader = loader.lock().map_err(|e| e.to_string())?;
    let _ids = loader.scan_plugins()?;
    Ok(loader.list_manifests())
}

/// 获取插件列表
#[tauri::command]
pub fn get_plugin_list(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
) -> Result<Vec<PluginManifest>, String> {
    let loader = loader.lock().map_err(|e| e.to_string())?;
    Ok(loader.list_manifests())
}

/// 加载指定插件
#[tauri::command]
pub fn load_plugin(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
    id: String,
) -> Result<LoadResult, String> {
    let mut loader = loader.lock().map_err(|e| e.to_string())?;
    let (state, vm_id) = loader.load_plugin(&id)?;
    Ok(LoadResult { state: format!("{}", state), vm_id })
}

/// 卸载指定插件
#[tauri::command]
pub fn unload_plugin(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
    id: String,
) -> Result<(), String> {
    let mut loader = loader.lock().map_err(|e| e.to_string())?;
    loader.unload_plugin(&id)
}

/// 根据前缀搜索插件
#[tauri::command]
pub fn find_plugins_by_prefix(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
    prefix: String,
) -> Result<Vec<PluginManifest>, String> {
    let loader = loader.lock().map_err(|e| e.to_string())?;
    Ok(loader.find_by_prefix(&prefix)
        .into_iter()
        .map(|p| p.manifest.clone())
        .collect())
}

/// 清理闲置插件
#[tauri::command]
pub fn cleanup_idle_plugins(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
) -> Result<usize, String> {
    let mut loader = loader.lock().map_err(|e| e.to_string())?;
    let timeout = loader.idle_timeout_secs();
    Ok(loader.cleanup_idle_plugins(timeout))
}

/// 获取插件健康状态
#[tauri::command]
pub fn get_plugin_health(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
) -> Result<Vec<PluginHealth>, String> {
    let loader = loader.lock().map_err(|e| e.to_string())?;
    Ok(loader.get_plugin_health())
}

/// 在指定插件的 VM 中执行 JS 代码
#[tauri::command]
pub fn plugin_execute(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
    plugin_id: String,
    code: String,
) -> Result<serde_json::Value, String> {
    let loader = loader.lock().map_err(|e| e.to_string())?;

    let instance = loader.get_plugin(&plugin_id)
        .ok_or_else(|| format!("插件不存在: {}", plugin_id))?;

    let vm_id = instance.vm_id.as_ref()
        .ok_or_else(|| format!("插件 {} 未加载或 VM 未创建", plugin_id))?;

    if !matches!(instance.state, PluginState::Ready | PluginState::Cached) {
        return Err(format!("插件 {} 当前状态不可用: {:?}", plugin_id, instance.state));
    }

    let result = loader.runtime().execute(vm_id, &code)
        .map_err(|e| format!("执行失败: {}", e))?;

    Ok(result)
}
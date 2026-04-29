mod manifest;
mod state;
mod instance;
mod scanner;
mod lifecycle;
mod health;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::plugins::quickjs_runtime::QuickJSRuntime;

pub use manifest::{PluginManifest, FeatureConfig, FeatureItem, RegisteredFeature};
pub use state::PluginState;
pub use instance::PluginInstance;
pub use health::PluginHealth;

pub struct PluginLoader {
    plugins_dir: PathBuf,
    instances: HashMap<String, PluginInstance>,
    quickjs_runtime: Arc<QuickJSRuntime>,
}

impl PluginLoader {
    pub fn new(plugins_dir: PathBuf, runtime: Arc<QuickJSRuntime>) -> Self {
        Self {
            plugins_dir,
            instances: HashMap::new(),
            quickjs_runtime: runtime,
        }
    }

    pub fn runtime(&self) -> &Arc<QuickJSRuntime> {
        &self.quickjs_runtime
    }

    pub fn idle_timeout_secs(&self) -> u64 {
        300
    }

    pub fn list_plugins(&self) -> Vec<&PluginInstance> {
        self.instances.values().collect()
    }

    pub fn list_manifests(&self) -> Vec<PluginManifest> {
        self.instances.values().map(|p| p.manifest.clone()).collect()
    }

    pub fn get_plugin(&self, id: &str) -> Option<&PluginInstance> {
        self.instances.get(id)
    }

    pub fn loaded_count(&self) -> usize {
        self.instances.values()
            .filter(|p| matches!(p.state, PluginState::Ready | PluginState::Cached))
            .count()
    }

    pub fn total_count(&self) -> usize {
        self.instances.len()
    }

    pub fn get_plugin_health(&self) -> Vec<PluginHealth> {
        self.instances.values()
            .map(|inst| PluginHealth {
                id: inst.id.clone(),
                state: format!("{}", inst.state),
                vm_id: inst.vm_id.clone(),
                loaded_at: inst.loaded_at.map(|t| t.elapsed().as_secs()),
                last_used: inst.last_used.map(|t| t.elapsed().as_secs()),
                error_count: inst.load_error_count,
                last_error: inst.last_error.clone(),
            })
            .collect()
    }
}

#[tauri::command]
pub fn scan_plugins(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
) -> Result<Vec<PluginManifest>, String> {
    let mut loader = loader.lock().map_err(|e| e.to_string())?;
    let _ids = loader.scan_plugins()?;
    Ok(loader.list_manifests())
}

#[tauri::command]
pub fn get_plugin_list(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
) -> Result<Vec<PluginManifest>, String> {
    let loader = loader.lock().map_err(|e| e.to_string())?;
    Ok(loader.list_manifests())
}

#[tauri::command]
pub fn load_plugin(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
    id: String,
) -> Result<LoadResult, String> {
    let mut loader = loader.lock().map_err(|e| e.to_string())?;
    let (state, vm_id) = loader.load_plugin(&id)?;
    Ok(LoadResult { state: format!("{}", state), vm_id })
}

#[derive(serde::Serialize)]
pub struct LoadResult {
    pub state: String,
    pub vm_id: Option<String>,
}

#[tauri::command]
pub fn unload_plugin(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
    id: String,
) -> Result<(), String> {
    let mut loader = loader.lock().map_err(|e| e.to_string())?;
    loader.unload_plugin(&id)
}

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

#[tauri::command]
pub fn cleanup_idle_plugins(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
) -> Result<usize, String> {
    let mut loader = loader.lock().map_err(|e| e.to_string())?;
    let timeout = loader.idle_timeout_secs();
    Ok(loader.cleanup_idle_plugins(timeout))
}

#[tauri::command]
pub fn get_plugin_health(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
) -> Result<Vec<PluginHealth>, String> {
    let loader = loader.lock().map_err(|e| e.to_string())?;
    Ok(loader.get_plugin_health())
}

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

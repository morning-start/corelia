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

/// 插件加载器
/// 
/// 职责：
/// - 管理插件实例和状态
/// - 协调插件生命周期
/// - （VM 管理完全委托给 QuickJSRuntime）
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

// 注意：Tauri Commands 现在在 commands/plugin.rs 中
// 这里仅包含 PluginLoader 的核心实现

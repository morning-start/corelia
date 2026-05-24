pub mod types;
mod scanner;
mod lifecycle;
mod query;
mod cleanup;
mod commands;

pub use types::{PluginManifest, PluginInstance, PluginState};
pub use commands::{scan_plugins, get_plugin_list, load_plugin, unload_plugin, find_plugins_by_prefix, cleanup_idle_plugins, get_plugin_health, plugin_execute};

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use crate::plugins::quickjs_runtime::QuickJSRuntime;

/// 插件加载器
///
/// 负责管理插件的生命周期：扫描 → 解析 → 加载 → 执行 → 卸载
pub struct PluginLoader {
    pub(crate) plugins_dir: PathBuf,
    pub(crate) instances: HashMap<String, types::PluginInstance>,
    pub(crate) quickjs_runtime: Arc<QuickJSRuntime>,
}

unsafe impl Send for PluginLoader {}
unsafe impl Sync for PluginLoader {}

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
}
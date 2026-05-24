pub mod types;
mod scanner;
mod lifecycle;
mod query;
mod cleanup;
pub mod commands;

pub use types::{PluginManifest, PluginInstance, PluginState};

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
    /// 上次扫描时的目录修改时间，用于 IO 缓存
    last_scan_mtime: Option<std::time::SystemTime>,
    /// 上次扫描发现的插件 ID 列表缓存
    last_discovered_ids: Vec<String>,
}

impl PluginLoader {
    pub fn new(plugins_dir: PathBuf, runtime: Arc<QuickJSRuntime>) -> Self {
        Self {
            plugins_dir,
            instances: HashMap::new(),
            quickjs_runtime: runtime,
            last_scan_mtime: None,
            last_discovered_ids: Vec::new(),
        }
    }

    pub fn runtime(&self) -> &Arc<QuickJSRuntime> {
        &self.quickjs_runtime
    }

    pub fn idle_timeout_secs(&self) -> u64 {
        300
    }
}
mod manifest;
mod state;
mod instance;
mod scanner;
mod lifecycle;
mod health;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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

    pub fn get_plugin_mut(&mut self, id: &str) -> Option<&mut PluginInstance> {
        self.instances.get_mut(id)
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

    /// 强制重置插件状态（用于调试或紧急恢复）
    pub fn reset_plugin(&mut self, id: &str) -> Result<(), String> {
        let instance = self.instances.get_mut(id)
            .ok_or_else(|| format!("插件不存在: {}", id))?;
        
        println!("[PluginLoader] 🔄 强制重置插件状态: {}", id);
        
        // 销毁 VM（如果存在）
        if let Some(ref vm_id) = instance.vm_id {
            if let Err(e) = self.quickjs_runtime.destroy_vm(vm_id) {
                println!("[PluginLoader] ⚠️ 销毁 VM 时出现错误: {}", e);
            }
        }
        
        instance.vm_id = None;
        instance.state = PluginState::MetaLoaded;
        instance.load_error_count = 0;
        instance.last_error = None;
        instance.retry_after = None;
        instance.retry_backoff_ms = 1000;
        instance.loaded_at = None;
        instance.last_used = None;
        
        Ok(())
    }

    /// 获取插件的详细状态信息
    pub fn get_plugin_status(&self, id: &str) -> Option<PluginStatus> {
        self.instances.get(id).map(|inst| PluginStatus {
            id: inst.id.clone(),
            state: inst.state.clone(),
            vm_id: inst.vm_id.clone(),
            error_count: inst.load_error_count,
            last_error: inst.last_error.clone(),
            is_ready: matches!(inst.state, PluginState::Ready),
            has_vm: inst.vm_id.is_some(),
            last_used_ago: inst.last_used.map(|t| t.elapsed()),
            loaded_ago: inst.loaded_at.map(|t| t.elapsed()),
        })
    }

    /// 注册新插件实例
    pub fn register_plugin(&mut self, manifest: PluginManifest, plugin_dir: PathBuf) -> Result<(), String> {
        let id = manifest.name.clone();
        
        if self.instances.contains_key(&id) {
            return Err(format!("插件已存在: {}", id));
        }
        
        let instance = PluginInstance {
            id: id.clone(),
            manifest,
            state: PluginState::MetaLoaded,
            vm_id: None,
            plugin_dir,
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
        };
        
        self.instances.insert(id.clone(), instance);
        println!("[PluginLoader] ✅ 插件已注册: {}", id);
        Ok(())
    }
}

/// 插件状态详情
#[derive(Debug, Clone)]
pub struct PluginStatus {
    pub id: String,
    pub state: PluginState,
    pub vm_id: Option<String>,
    pub error_count: u32,
    pub last_error: Option<String>,
    pub is_ready: bool,
    pub has_vm: bool,
    pub last_used_ago: Option<Duration>,
    pub loaded_ago: Option<Duration>,
}

// 注意：Tauri Commands 现在在 commands/plugin.rs 中
// 这里仅包含 PluginLoader 的核心实现

use crate::plugins::loader::types::{PluginHealth, PluginState};
use crate::plugins::loader::PluginLoader;

impl PluginLoader {
    /// 清理所有闲置插件（将其 VM 销毁并状态转为 Cached/Unloaded）
    pub fn cleanup_idle_plugins(&mut self, idle_timeout_secs: u64) -> usize {
        let mut cleaned = 0;
        let ids: Vec<String> = self.instances.keys().cloned().collect();

        for id in ids {
            if let Some(instance) = self.instances.get_mut(&id) {
                if instance.state != PluginState::Ready {
                    continue;
                }

                let should_cleanup = match instance.last_used {
                    Some(last) => last.elapsed().as_secs() >= idle_timeout_secs,
                    None => true,
                };

                if should_cleanup {
                    println!("[PluginLoader] 插件 {} 闲置超过 {}s，执行缓存清理", id, idle_timeout_secs);

                    if let Some(ref vm_id) = instance.vm_id {
                        if let Err(e) = self.quickjs_runtime.destroy_vm(vm_id) {
                            eprintln!("[PluginLoader] 清理 VM 失败 ({}): {}", id, e);
                        }
                    }

                    instance.vm_id = None;
                    instance.state = PluginState::Cached;
                    instance.loaded_at = None;
                    cleaned += 1;
                }
            }
        }

        if cleaned > 0 {
            println!("[PluginLoader] 共清理 {} 个闲置插件", cleaned);
        }
        cleaned
    }

    /// 扫描并上报所有插件的 VM 健康状态
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
use std::path::Path;

use super::manifest::PluginManifest;
use super::state::PluginState;
use super::instance::PluginInstance;

impl super::PluginLoader {
    pub fn scan_plugins(&mut self) -> Result<Vec<String>, String> {
        if !self.plugins_dir.exists() {
            return Err(format!("插件目录不存在: {}", self.plugins_dir.display()));
        }

        if !self.plugins_dir.is_dir() {
            return Err(format!("插件路径不是目录: {}", self.plugins_dir.display()));
        }

        let entries = std::fs::read_dir(&self.plugins_dir)
            .map_err(|e| format!("无法读取插件目录: {}", e))?;

        let mut discovered_ids = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("plugin.json");
            if !manifest_path.exists() {
                continue;
            }

            match self.parse_plugin_json(&path) {
                Ok(manifest) => {
                    let plugin_id = manifest.name.clone();

                    let instance = PluginInstance {
                        id: plugin_id.clone(),
                        manifest,
                        state: PluginState::MetaLoaded,
                        vm_id: None,
                        plugin_dir: path.clone(),
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

                    self.instances.insert(plugin_id.clone(), instance);

                    println!("[PluginLoader] 发现插件: {}", plugin_id);
                    discovered_ids.push(plugin_id);
                }
                Err(e) => {
                    eprintln!("[PluginLoader] 解析失败 ({}): {:?}", path.display(), e);
                }
            }
        }

        Ok(discovered_ids)
    }

    fn parse_plugin_json(&self, plugin_path: &Path) -> Result<PluginManifest, String> {
        let json_path = plugin_path.join("plugin.json");

        let content = std::fs::read_to_string(&json_path)
            .map_err(|e| format!("无法读取 plugin.json: {}", e))?;

        let mut manifest: PluginManifest = serde_json::from_str(&content)
            .map_err(|e| format!("JSON 解析失败: {}", e))?;

        if manifest.name.is_empty() {
            return Err("插件名称 (name) 不能为空".to_string());
        }
        if manifest.version.is_empty() {
            return Err("插件版本 (version) 不能为空".to_string());
        }
        if manifest.plugin_type.is_empty() {
            return Err("插件类型 (type) 不能为空".to_string());
        }

        if manifest.main.is_none() {
            manifest.main = Some("index.js".to_string());
        }
        if manifest.features.is_none() {
            manifest.features = Some(Vec::new());
        }

        Ok(manifest)
    }

    pub fn find_by_prefix(&self, prefix: &str) -> Vec<&PluginInstance> {
        let mut matches = Vec::new();
        let prefix_lower = prefix.to_lowercase();

        for instance in self.instances.values() {
            if let Some(ref p) = instance.manifest.prefix {
                let p_lower = p.to_lowercase();
                if p_lower.starts_with(&prefix_lower) || prefix_lower.starts_with(&p_lower) {
                    matches.push(instance);
                }
            }
        }
        matches
    }
}

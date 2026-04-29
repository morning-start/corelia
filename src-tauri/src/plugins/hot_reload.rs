use std::path::PathBuf;
use std::time::Duration;
use std::collections::HashSet;
use tauri::{AppHandle, Emitter};

pub struct HotReloader {
    plugins_dir: PathBuf,
    known_plugins: HashSet<String>,
}

impl HotReloader {
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins_dir,
            known_plugins: HashSet::new(),
        }
    }

    pub fn initial_scan(&mut self) {
        if let Ok(entries) = std::fs::read_dir(&self.plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("plugin.json").exists() {
                    if let Ok(content) = std::fs::read_to_string(path.join("plugin.json")) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                                self.known_plugins.insert(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn check_for_changes(&mut self) -> Vec<HotReloadEvent> {
        let mut events = Vec::new();
        let mut current_plugins = HashSet::new();

        if let Ok(entries) = std::fs::read_dir(&self.plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("plugin.json").exists() {
                    if let Ok(content) = std::fs::read_to_string(path.join("plugin.json")) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                                let name = name.to_string();
                                current_plugins.insert(name.clone());
                                
                                if !self.known_plugins.contains(&name) {
                                    events.push(HotReloadEvent::PluginAdded(name.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }

        for old_plugin in &self.known_plugins {
            if !current_plugins.contains(old_plugin) {
                events.push(HotReloadEvent::PluginRemoved(old_plugin.clone()));
            }
        }

        self.known_plugins = current_plugins;
        events
    }

    pub fn start_watching(app: AppHandle, plugins_dir: PathBuf) {
        let mut reloader = Self::new(plugins_dir);
        reloader.initial_scan();

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(2));

                let events = reloader.check_for_changes();
                for event in events {
                    match event {
                        HotReloadEvent::PluginAdded(id) => {
                            println!("[HotReloader] 📦 检测到新插件: {}", id);
                            let _ = app.emit("plugin-hot-reload", serde_json::json!({
                                "type": "added",
                                "pluginId": id
                            }));
                        }
                        HotReloadEvent::PluginRemoved(id) => {
                            println!("[HotReloader] 🗑️ 检测到插件移除: {}", id);
                            let _ = app.emit("plugin-hot-reload", serde_json::json!({
                                "type": "removed",
                                "pluginId": id
                            }));
                        }
                        HotReloadEvent::PluginModified(id) => {
                            println!("[HotReloader] 🔄 检测到插件修改: {}", id);
                            let _ = app.emit("plugin-hot-reload", serde_json::json!({
                                "type": "modified",
                                "pluginId": id
                            }));
                        }
                    }
                }
            }
        });
    }
}

#[derive(Debug, Clone)]
pub enum HotReloadEvent {
    PluginAdded(String),
    PluginRemoved(String),
    PluginModified(String),
}

#[tauri::command]
pub fn start_hot_reload(
    app: AppHandle,
) -> Result<(), String> {
    let plugins_dir = std::env::current_exe()
        .map(|p| p.parent().unwrap_or(&p).join("plugins"))
        .unwrap_or_else(|_| PathBuf::from("plugins"));

    HotReloader::start_watching(app, plugins_dir);

    println!("[HotReloader] ✅ 热重载监听已启动");
    Ok(())
}

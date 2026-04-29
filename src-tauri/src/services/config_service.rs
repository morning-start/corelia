//! 配置服务模块
//!
//! 封装三层配置的管理操作
//!
//! 存储路径：$APPDATA/morningstart.corelia/
//! - config.system.json  (系统级配置)
//! - config.user.json    (用户级配置)
//! - cache/app.json      (应用级配置)

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tauri_plugin_store::StoreExt;

fn get_app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录：{}", e))
}

pub struct ConfigService;

impl ConfigService {
    const BASE_DIR: &'static str = "morningstart.corelia";

    fn system_path() -> String {
        format!("{}/config.system.json", Self::BASE_DIR)
    }

    fn user_path() -> String {
        format!("{}/config.user.json", Self::BASE_DIR)
    }

    fn app_path() -> String {
        format!("{}/cache/app.json", Self::BASE_DIR)
    }

    fn load_config(app: &tauri::AppHandle, store_path: &str, default: Value) -> Result<Value, String> {
        let store = app.store(store_path)
            .map_err(|e| format!("无法打开配置文件：{}", e))?;

        match store.get("config") {
            Some(v) => Ok(v.clone()),
            None => {
                let default_config = default;
                Self::save_config(app, store_path, default_config.clone())?;
                Ok(default_config)
            }
        }
    }

    fn save_config(app: &tauri::AppHandle, store_path: &str, config: Value) -> Result<(), String> {
        let store = app.store(store_path)
            .map_err(|e| format!("无法保存配置文件：{}", e))?;
        store.set("config", config);
        store.save().map_err(|e| format!("保存配置文件失败：{}", e))
    }

    fn reset_config(app: &tauri::AppHandle, store_path: &str, default: Value) -> Result<(), String> {
        let store = app.store(store_path)
            .map_err(|e| format!("无法重置配置文件：{}", e))?;
        store.set("config", default);
        store.save().map_err(|e| format!("重置配置文件失败：{}", e))
    }

    // ==================== 配置目录初始化 ====================

    pub fn init_config_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        let app_data = get_app_data_dir(app)?;

        let base_dir = app_data.join(Self::BASE_DIR);
        let cache_dir = base_dir.join("cache");
        let logs_dir = base_dir.join("logs");

        fs::create_dir_all(&base_dir)
            .map_err(|e| format!("无法创建配置目录：{}", e))?;
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("无法创建缓存目录：{}", e))?;
        fs::create_dir_all(&logs_dir)
            .map_err(|e| format!("无法创建日志目录：{}", e))?;

        Self::init_config_file(&Self::system_config_path(&base_dir), Self::default_system_config())?;
        Self::init_config_file(&Self::user_config_path(&base_dir), Self::default_user_config())?;
        Self::init_config_file(&Self::app_config_path(&cache_dir), Self::default_app_config())?;

        Ok(base_dir)
    }

    fn system_config_path(base_dir: &Path) -> PathBuf {
        base_dir.join("config.system.json")
    }

    fn user_config_path(base_dir: &Path) -> PathBuf {
        base_dir.join("config.user.json")
    }

    fn app_config_path(cache_dir: &Path) -> PathBuf {
        cache_dir.join("app.json")
    }

    fn init_config_file(path: &Path, default: Value) -> Result<(), String> {
        if !path.exists() {
            let json = serde_json::to_string_pretty(&default)
                .map_err(|e| e.to_string())?;
            fs::write(path, json)
                .map_err(|e| format!("无法创建配置文件：{}", e))?;
        }
        Ok(())
    }

    // ==================== 系统级配置 ====================

    pub fn load_system(app: &tauri::AppHandle) -> Result<Value, String> {
        Self::load_config(app, &Self::system_path(), Self::default_system_config())
    }

    pub fn save_system(app: &tauri::AppHandle, config: Value) -> Result<(), String> {
        Self::validate_system_config(&config)?;
        Self::save_config(app, &Self::system_path(), config)
    }

    fn default_system_config() -> Value {
        serde_json::json!({
            "shortcut": { "summon": "Alt+Space" },
            "startup": { "enabled": false, "minimizeToTray": true },
            "advanced": { "debugMode": false }
        })
    }

    fn validate_system_config(config: &Value) -> Result<(), String> {
        let obj = config.as_object()
            .ok_or("系统配置必须是对象")?;

        if let Some(shortcut) = obj.get("shortcut") {
            let summon = shortcut.get("summon")
                .and_then(|s| s.as_str())
                .ok_or("shortcut.summon 必须是字符串")?;

            if summon.is_empty() {
                return Err("快捷键不能为空".to_string());
            }
        }

        Ok(())
    }

    // ==================== 用户级配置 ====================

    pub fn load_user(app: &tauri::AppHandle) -> Result<Value, String> {
        Self::load_config(app, &Self::user_path(), Self::default_user_config())
    }

    pub fn save_user(app: &tauri::AppHandle, config: Value) -> Result<(), String> {
        Self::save_config(app, &Self::user_path(), config)
    }

    pub fn reset_user(app: &tauri::AppHandle) -> Result<(), String> {
        Self::reset_config(app, &Self::user_path(), Self::default_user_config())
    }

    fn default_user_config() -> Value {
        serde_json::json!({
            "theme": "dark",
            "behavior": { "autoHide": true, "autoHideDelay": 3000 },
            "window": { "width": 600, "height": 420 },
            "search": { "defaultCategory": "all", "maxResults": 20, "maxHistoryCapacity": 100 }
        })
    }

    // ==================== 应用级配置 ====================

    pub fn load_app(app: &tauri::AppHandle) -> Result<Value, String> {
        Self::load_config(app, &Self::app_path(), Self::default_app_config())
    }

    pub fn save_app(app: &tauri::AppHandle, config: Value) -> Result<(), String> {
        Self::save_config(app, &Self::app_path(), config)
    }

    pub fn clear_app(app: &tauri::AppHandle) -> Result<(), String> {
        Self::reset_config(app, &Self::app_path(), Self::default_app_config())
    }

    fn default_app_config() -> Value {
        serde_json::json!({
            "searchHistory": [],
            "plugins": { "cache": {}, "enabled": [] },
            "runtime": { "lastState": {}, "usageStats": { "launchCount": 0, "totalUsageTime": 0 } }
        })
    }
}

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
use std::path::PathBuf;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

/// 获取应用数据目录
fn get_app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录：{}", e))
}

/// 配置服务 (无状态，通过 AppHandle 操作)
pub struct ConfigService;

impl ConfigService {
    // ==================== 配置目录初始化 ====================
    
    /// 初始化配置目录结构
    /// 
    /// 在应用首次启动时调用，创建必要的目录和默认配置文件
    pub fn init_config_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        // 获取应用数据目录
        let app_data = get_app_data_dir(app)?;
        
        let base_dir = app_data.join("morningstart.corelia");
        let cache_dir = base_dir.join("cache");
        let logs_dir = base_dir.join("logs");
        
        // 创建目录结构
        fs::create_dir_all(&base_dir)
            .map_err(|e| format!("无法创建配置目录：{}", e))?;
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("无法创建缓存目录：{}", e))?;
        fs::create_dir_all(&logs_dir)
            .map_err(|e| format!("无法创建日志目录：{}", e))?;
        
        // 检查是否需要创建默认配置文件
        let system_config = base_dir.join("config.system.json");
        let user_config = base_dir.join("config.user.json");
        let app_cache = cache_dir.join("app.json");
        
        // 创建默认系统配置（如果不存在）
        if !system_config.exists() {
            let default_config = serde_json::json!({
                "config": {
                    "shortcut": { "summon": "Alt+Space" },
                    "startup": { "enabled": false, "minimizeToTray": true },
                    "advanced": { "debugMode": false }
                }
            });
            fs::write(&system_config, serde_json::to_string_pretty(&default_config).map_err(|e| e.to_string())?)
                .map_err(|e| format!("无法创建系统配置文件：{}", e))?;
        }
        
        // 创建默认用户配置（如果不存在）
        if !user_config.exists() {
            let default_config = serde_json::json!({
                "config": {
                    "theme": "dark",
                    "behavior": { "autoHide": true, "autoHideDelay": 3000 },
                    "window": { "width": 600, "height": 400 },
                    "search": { "defaultCategory": "all", "maxResults": 20 }
                }
            });
            fs::write(&user_config, serde_json::to_string_pretty(&default_config).map_err(|e| e.to_string())?)
                .map_err(|e| format!("无法创建用户配置文件：{}", e))?;
        }
        
        // 创建默认应用缓存（如果不存在）
        if !app_cache.exists() {
            let default_config = serde_json::json!({
                "config": {
                    "searchHistory": [],
                    "plugins": { "cache": {}, "enabled": [] },
                    "runtime": { "lastState": {}, "usageStats": { "launchCount": 0, "totalUsageTime": 0 } }
                }
            });
            fs::write(&app_cache, serde_json::to_string_pretty(&default_config).map_err(|e| e.to_string())?)
                .map_err(|e| format!("无法创建应用缓存：{}", e))?;
        }
        
        Ok(base_dir)
    }
    
    // ==================== 系统级配置 ====================
    
    /// 加载系统级配置
    /// 
    /// 文件位置：$APPDATA/morningstart.corelia/config.system.json
    pub fn load_system(app: &tauri::AppHandle) -> Result<Value, String> {
        let store = app.store("morningstart.corelia/config.system.json")
            .map_err(|e| format!("无法打开系统配置文件：{}", e))?;
        
        match store.get("config") {
            Some(v) => Ok(v.clone()),
            None => {
                // 首次启动，创建默认配置
                let default_config = Self::default_system_config();
                Self::save_system(app, default_config.clone())?;
                Ok(default_config)
            }
        }
    }
    
    /// 保存系统级配置
    pub fn save_system(app: &tauri::AppHandle, config: Value) -> Result<(), String> {
        // 验证配置
        Self::validate_system_config(&config)?;
        
        let store = app.store("morningstart.corelia/config.system.json")
            .map_err(|e| format!("无法保存系统配置：{}", e))?;
        store.set("config", config);
        store.save().map_err(|e| format!("保存系统配置失败：{}", e))
    }
    
    fn default_system_config() -> Value {
        serde_json::json!({
            "shortcut": { "summon": "Alt+Space" },
            "startup": { "enabled": false, "minimizeToTray": true },
            "advanced": { "debugMode": false }
        })
    }
    
    fn validate_system_config(config: &Value) -> Result<(), String> {
        // 验证必填字段
        let obj = config.as_object()
            .ok_or("系统配置必须是对象")?;
        
        // 验证快捷键
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
    
    /// 加载用户级配置
    /// 
    /// 文件位置：$APPDATA/morningstart.corelia/config.user.json
    pub fn load_user(app: &tauri::AppHandle) -> Result<Value, String> {
        let store = app.store("morningstart.corelia/config.user.json")
            .map_err(|e| format!("无法打开用户配置文件：{}", e))?;
        
        match store.get("config") {
            Some(v) => Ok(v.clone()),
            None => {
                // 首次启动，创建默认配置
                let default_config = Self::default_user_config();
                Self::save_user(app, default_config.clone())?;
                Ok(default_config)
            }
        }
    }
    
    /// 保存用户级配置
    pub fn save_user(app: &tauri::AppHandle, config: Value) -> Result<(), String> {
        let store = app.store("morningstart.corelia/config.user.json")
            .map_err(|e| format!("无法保存用户配置：{}", e))?;
        store.set("config", config);
        store.save().map_err(|e| format!("保存用户配置失败：{}", e))
    }
    
    /// 重置用户级配置为默认值
    pub fn reset_user(app: &tauri::AppHandle) -> Result<(), String> {
        let store = app.store("morningstart.corelia/config.user.json")
            .map_err(|e| format!("无法重置用户配置：{}", e))?;
        store.set("config", Self::default_user_config());
        store.save().map_err(|e| format!("重置用户配置失败：{}", e))
    }
    
    fn default_user_config() -> Value {
        serde_json::json!({
            "theme": "dark",
            "behavior": { "autoHide": true, "autoHideDelay": 3000 },
            "window": { "width": 600, "height": 400 },
            "search": { "defaultCategory": "all", "maxResults": 20 }
        })
    }
    
    // ==================== 应用级配置 ====================
    
    /// 加载应用级配置
    /// 
    /// 文件位置：$APPDATA/morningstart.corelia/cache/app.json
    pub fn load_app(app: &tauri::AppHandle) -> Result<Value, String> {
        let store = app.store("morningstart.corelia/cache/app.json")
            .map_err(|e| format!("无法打开应用缓存：{}", e))?;
        
        match store.get("config") {
            Some(v) => Ok(v.clone()),
            None => {
                let default_config = Self::default_app_config();
                Self::save_app(app, default_config.clone())?;
                Ok(default_config)
            }
        }
    }
    
    /// 保存应用级配置
    pub fn save_app(app: &tauri::AppHandle, config: Value) -> Result<(), String> {
        let store = app.store("morningstart.corelia/cache/app.json")
            .map_err(|e| format!("无法保存应用缓存：{}", e))?;
        store.set("config", config);
        store.save().map_err(|e| format!("保存应用缓存失败：{}", e))
    }
    
    /// 清理应用级配置
    pub fn clear_app(app: &tauri::AppHandle) -> Result<(), String> {
        let store = app.store("morningstart.corelia/cache/app.json")
            .map_err(|e| format!("无法清理应用缓存：{}", e))?;
        store.set("config", Self::default_app_config());
        store.save().map_err(|e| format!("清理应用缓存失败：{}", e))
    }
    
    fn default_app_config() -> Value {
        serde_json::json!({
            "searchHistory": [],
            "plugins": { "cache": {}, "enabled": [] },
            "runtime": { "lastState": {}, "usageStats": { "launchCount": 0, "totalUsageTime": 0 } }
        })
    }
}

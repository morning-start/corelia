//! 数据存储服务模块
//! 
//! 封装 Store 操作（基于 tauri-plugin-store）

use serde_json::Value;
use tauri_plugin_store::StoreExt;

/// 存储服务（无状态，通过 AppHandle 操作）
pub struct StoreService;

impl StoreService {
    /// 保存数据到 Store
    pub fn save(app: &tauri::AppHandle, key: &str, value: Value) -> Result<(), String> {
        let store = app.store("corelia.json").map_err(|e| e.to_string())?;
        store.set(key, value);
        store.save().map_err(|e| e.to_string())
    }
    
    /// 从 Store 加载数据
    pub fn load(app: &tauri::AppHandle, key: &str) -> Result<Value, String> {
        let store = app.store("corelia.json").map_err(|e| e.to_string())?;
        match store.get(key) {
            Some(v) => Ok(v.clone()),
            None => Ok(Value::Null),
        }
    }
    
    /// 从 Store 删除数据
    pub fn delete(app: &tauri::AppHandle, key: &str) -> Result<(), String> {
        let store = app.store("corelia.json").map_err(|e| e.to_string())?;
        let _ = store.delete(key);
        store.save().map_err(|e| e.to_string())
    }
    
    // ==================== 已弃用的 API ====================
    // 注意：以下函数已被新的分层配置系统替代
    // 为保持向后兼容性暂时保留，未来版本可能删除
    
    /// 加载设置（带默认值）
    /// @deprecated 使用 ConfigService::load_system / load_user 替代
    #[allow(dead_code)]
    pub fn load_settings(app: &tauri::AppHandle) -> Result<Value, String> {
        let store = app.store("corelia.json").map_err(|e| e.to_string())?;
        match store.get("settings") {
            Some(v) => Ok(v.clone()),
            None => Ok(serde_json::json!({
                "theme": "dark",
                "shortcut": { "summon": "Alt+Space" },
                "behavior": { "autoHide": true, "autoHideDelay": 3000 },
                "startup": { "enabled": false, "minimizeToTray": true }
            })),
        }
    }
    
    /// 保存设置
    /// @deprecated 使用 ConfigService::save_system / save_user 替代
    #[allow(dead_code)]
    pub fn save_settings(app: &tauri::AppHandle, settings: Value) -> Result<(), String> {
        let store = app.store("corelia.json").map_err(|e| e.to_string())?;
        store.set("settings", settings);
        store.save().map_err(|e| e.to_string())
    }
}

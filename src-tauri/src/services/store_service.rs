//! 数据存储服务模块
//!
//! 封装 Store 操作（基于 tauri-plugin-store）
//! 存储路径: $APPDATA/morningstart.corelia/store.json

use serde_json::Value;
use tauri_plugin_store::StoreExt;

/// 存储服务（无状态，通过 AppHandle 操作）
pub struct StoreService;

/// 存储文件路径
const STORE_FILE: &str = "morningstart.corelia/store.json";

impl StoreService {
    /// 保存数据到 Store
    pub fn save(app: &tauri::AppHandle, key: &str, value: Value) -> Result<(), String> {
        let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
        store.set(key, value);
        store.save().map_err(|e| e.to_string())
    }

    /// 从 Store 加载数据
    pub fn load(app: &tauri::AppHandle, key: &str) -> Result<Value, String> {
        let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
        match store.get(key) {
            Some(v) => Ok(v.clone()),
            None => Ok(Value::Null),
        }
    }

    /// 从 Store 删除数据
    pub fn delete(app: &tauri::AppHandle, key: &str) -> Result<(), String> {
        let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
        let _ = store.delete(key);
        store.save().map_err(|e| e.to_string())
    }
}

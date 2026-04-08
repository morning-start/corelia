//! 数据存储命令
//!
//! 提供 Tauri Command 接口，调用 StoreService 执行操作

use serde_json::Value;
use tauri::AppHandle;
use crate::services::StoreService;

/// 保存数据到 Store
#[tauri::command]
pub async fn save_to_store(app: AppHandle, key: String, value: Value) -> Result<(), String> {
    StoreService::save(&app, &key, value)
}

/// 从 Store 加载数据
#[tauri::command]
pub async fn load_from_store(app: AppHandle, key: String) -> Result<Value, String> {
    StoreService::load(&app, &key)
}

/// 从 Store 删除数据
#[tauri::command]
pub async fn delete_from_store(app: AppHandle, key: String) -> Result<(), String> {
    StoreService::delete(&app, &key)
}

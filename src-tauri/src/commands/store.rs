//! 数据存储命令

use serde_json::Value;
use tauri::AppHandle;
use crate::services::StoreService;

#[tauri::command]
pub async fn save_to_store(app: AppHandle, key: String, value: Value) -> Result<(), String> {
    StoreService::save(&app, &key, value)
}

#[tauri::command]
pub async fn load_from_store(app: AppHandle, key: String) -> Result<Value, String> {
    StoreService::load(&app, &key)
}

#[tauri::command]
pub async fn delete_from_store(app: AppHandle, key: String) -> Result<(), String> {
    StoreService::delete(&app, &key)
}

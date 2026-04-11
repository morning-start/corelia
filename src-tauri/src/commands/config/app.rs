//! 应用级配置命令

use crate::services::ConfigService;

#[tauri::command]
pub async fn load_app_config(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    ConfigService::load_app(&app)
}

#[tauri::command]
pub async fn save_app_config(app: tauri::AppHandle, config: serde_json::Value) -> Result<(), String> {
    ConfigService::save_app(&app, config)
}

#[tauri::command]
pub async fn clear_app_config(app: tauri::AppHandle) -> Result<(), String> {
    ConfigService::clear_app(&app)
}

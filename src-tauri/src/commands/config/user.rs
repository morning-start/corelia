//! 用户级配置命令

use crate::services::ConfigService;

#[tauri::command]
pub async fn load_user_config(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    ConfigService::load_user(&app)
}

#[tauri::command]
pub async fn save_user_config(app: tauri::AppHandle, config: serde_json::Value) -> Result<(), String> {
    ConfigService::save_user(&app, config)
}

#[tauri::command]
pub async fn reset_user_config(app: tauri::AppHandle) -> Result<(), String> {
    ConfigService::reset_user(&app)
}

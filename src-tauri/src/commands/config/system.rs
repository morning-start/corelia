//! 系统级配置命令

use crate::services::ConfigService;

#[tauri::command]
pub async fn load_system_config(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    ConfigService::load_system(&app)
}

#[tauri::command]
pub async fn save_system_config(app: tauri::AppHandle, config: serde_json::Value) -> Result<(), String> {
    ConfigService::save_system(&app, config)
}

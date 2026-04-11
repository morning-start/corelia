//! 自启动命令

use tauri::AppHandle;
use crate::services::AutostartService;

#[tauri::command]
pub async fn enable_autostart(app: AppHandle) -> Result<(), String> {
    AutostartService::enable(&app)
}

#[tauri::command]
pub async fn disable_autostart(app: AppHandle) -> Result<(), String> {
    AutostartService::disable(&app)
}

#[tauri::command]
pub async fn is_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    AutostartService::is_enabled(&app)
}

//! 自启动命令
//! 
//! 提供 Tauri Command 接口，调用 AutostartService 执行操作

use tauri::AppHandle;
use crate::services::AutostartService;

/// 启用自启动
#[tauri::command]
pub async fn enable_autostart(app: AppHandle) -> Result<(), String> {
    AutostartService::enable(&app)
}

/// 禁用自启动
#[tauri::command]
pub async fn disable_autostart(app: AppHandle) -> Result<(), String> {
    AutostartService::disable(&app)
}

/// 检查自启动是否启用
#[tauri::command]
pub async fn is_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    AutostartService::is_enabled(&app)
}

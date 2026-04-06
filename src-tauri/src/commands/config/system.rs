//! 系统级配置命令
//! 
//! 管理系统级配置 (快捷键、开机自启动等)

use serde_json::Value;
use tauri::AppHandle;
use crate::services::ConfigService;

/// 加载系统级配置
#[tauri::command]
pub async fn load_system_config(app: AppHandle) -> Result<Value, String> {
    ConfigService::load_system(&app)
}

/// 保存系统级配置
#[tauri::command]
pub async fn save_system_config(app: AppHandle, config: Value) -> Result<(), String> {
    ConfigService::save_system(&app, config)
}

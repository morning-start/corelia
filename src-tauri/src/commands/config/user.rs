//! 用户级配置命令
//! 
//! 管理用户级配置 (主题、行为偏好等)

use serde_json::Value;
use tauri::AppHandle;
use crate::services::ConfigService;

/// 加载用户级配置
#[tauri::command]
pub async fn load_user_config(app: AppHandle) -> Result<Value, String> {
    ConfigService::load_user(&app)
}

/// 保存用户级配置
#[tauri::command]
pub async fn save_user_config(app: AppHandle, config: Value) -> Result<(), String> {
    ConfigService::save_user(&app, config)
}

/// 重置用户级配置为默认值
#[tauri::command]
pub async fn reset_user_config(app: AppHandle) -> Result<(), String> {
    ConfigService::reset_user(&app)
}

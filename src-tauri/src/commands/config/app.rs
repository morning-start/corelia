//! 应用级配置命令
//! 
//! 管理应用级配置 (搜索历史、插件缓存等)

use serde_json::Value;
use tauri::AppHandle;
use crate::services::ConfigService;

/// 加载应用级配置
#[tauri::command]
pub async fn load_app_config(app: AppHandle) -> Result<Value, String> {
    ConfigService::load_app(&app)
}

/// 保存应用级配置
#[tauri::command]
pub async fn save_app_config(app: AppHandle, config: Value) -> Result<(), String> {
    ConfigService::save_app(&app, config)
}

/// 清理应用级配置
#[tauri::command]
pub async fn clear_app_config(app: AppHandle) -> Result<(), String> {
    ConfigService::clear_app(&app)
}

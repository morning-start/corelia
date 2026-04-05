//! 自启动服务模块
//! 
//! 封装应用自启动功能

use tauri_plugin_autostart::ManagerExt;

/// 自启动服务（无状态，通过 AppHandle 操作）
pub struct AutostartService;

impl AutostartService {
    /// 启用自启动
    pub fn enable(app: &tauri::AppHandle) -> Result<(), String> {
        let autostart = app.autolaunch();
        autostart.enable().map_err(|e| format!("{:?}", e))
    }
    
    /// 禁用自启动
    pub fn disable(app: &tauri::AppHandle) -> Result<(), String> {
        let autostart = app.autolaunch();
        autostart.disable().map_err(|e| format!("{:?}", e))
    }
    
    /// 检查自启动是否启用
    pub fn is_enabled(app: &tauri::AppHandle) -> Result<bool, String> {
        let autostart = app.autolaunch();
        autostart.is_enabled().map_err(|e| format!("{:?}", e))
    }
}

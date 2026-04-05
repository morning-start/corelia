//! Shell 操作命令
//! 
//! 提供 Tauri Command 接口，调用 ShellService 执行操作

use crate::services::ShellService;

/// 打开 URL（默认浏览器）
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    ShellService::open_url(url)
}

/// 打开文件路径（资源管理器）
#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    ShellService::open_path(path)
}

/// 打开应用程序
#[tauri::command]
pub fn open_app(app: String) -> Result<(), String> {
    ShellService::open_app(app)
}

//! Shell 操作命令

use crate::services::ShellService;

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    ShellService::open_url(url)
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    ShellService::open_path(path)
}

#[tauri::command]
pub fn open_app(app: String) -> Result<(), String> {
    ShellService::open_app(app)
}

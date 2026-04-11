//! 剪贴板命令

use crate::services::ClipboardService;

#[tauri::command]
pub fn read_clipboard() -> Result<String, String> {
    ClipboardService::read()
}

#[tauri::command]
pub fn write_clipboard(text: String) -> Result<(), String> {
    ClipboardService::write(text)
}

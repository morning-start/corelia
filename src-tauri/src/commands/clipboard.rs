//! 剪贴板命令
//! 
//! 提供 Tauri Command 接口，调用 ClipboardService 执行操作

use crate::services::ClipboardService;

/// 读取剪贴板文本
#[tauri::command]
pub fn read_clipboard() -> Result<String, String> {
    ClipboardService::read()
}

/// 写入剪贴板文本
#[tauri::command]
pub fn write_clipboard(text: String) -> Result<(), String> {
    ClipboardService::write(text)
}

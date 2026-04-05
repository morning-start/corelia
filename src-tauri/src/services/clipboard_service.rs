//! 剪贴板服务模块
//! 
//! 封装剪贴板操作

use arboard::Clipboard;

/// 剪贴板服务（无状态）
pub struct ClipboardService;

impl ClipboardService {
    /// 读取剪贴板文本
    pub fn read() -> Result<String, String> {
        let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
        clipboard.get_text().map_err(|e| e.to_string())
    }
    
    /// 写入剪贴板文本
    pub fn write(text: String) -> Result<(), String> {
        let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
        clipboard.set_text(text).map_err(|e| e.to_string())
    }
}

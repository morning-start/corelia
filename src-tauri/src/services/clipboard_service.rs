//! 剪贴板服务模块
//!
//! 封装剪贴板操作

use arboard::Clipboard;
use std::sync::Mutex;

static CLIPBOARD_INSTANCE: std::sync::OnceLock<Mutex<Clipboard>> = std::sync::OnceLock::new();

fn get_clipboard() -> Result<std::sync::MutexGuard<'static, Clipboard>, String> {
    let instance = CLIPBOARD_INSTANCE.get_or_init(|| {
        Mutex::new(Clipboard::new().expect("Failed to initialize clipboard"))
    });
    instance.lock().map_err(|e| e.to_string())
}

pub struct ClipboardService;

impl ClipboardService {
    pub fn read() -> Result<String, String> {
        let mut clipboard = get_clipboard()?;
        clipboard.get_text().map_err(|e| e.to_string())
    }

    pub fn write(text: String) -> Result<(), String> {
        let mut clipboard = get_clipboard()?;
        clipboard.set_text(text).map_err(|e| e.to_string())
    }
}

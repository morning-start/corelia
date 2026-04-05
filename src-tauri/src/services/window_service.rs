//! 窗口操作服务模块
//! 
//! 封装窗口相关的业务逻辑和状态管理

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};

/// 延迟取消置顶的时间（毫秒）
const DELAY_UNPIN_MS: u64 = 100;

/// 窗口可见状态（全局唯一）
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);

/// 窗口服务（无状态，所有操作通过 AppHandle）
pub struct WindowService;

impl WindowService {
    /// 获取窗口可见状态
    #[inline]
    fn is_window_visible() -> bool {
        WINDOW_VISIBLE.load(Ordering::SeqCst)
    }

    /// 设置窗口可见状态
    #[inline]
    fn set_window_visible(visible: bool) {
        WINDOW_VISIBLE.store(visible, Ordering::SeqCst);
    }

    /// 显示窗口并置顶
    pub fn show(app: &AppHandle) -> Result<(), String> {
        let window = app.get_webview_window("main").ok_or("Window not found")?;
        
        window.set_always_on_top(true)
            .map_err(|e| e.to_string())?;
        window.show()
            .map_err(|e| e.to_string())?;
        window.set_focus()
            .map_err(|e| e.to_string())?;
        
        // 更新全局状态
        Self::set_window_visible(true);
        
        // 延迟取消置顶
        Self::schedule_unpin(window.clone());
        
        Ok(())
    }
    
    /// 隐藏窗口
    pub fn hide(app: &AppHandle) -> Result<(), String> {
        let window = app.get_webview_window("main").ok_or("Window not found")?;
        window.hide().map_err(|e| e.to_string())?;
        
        // 更新全局状态
        Self::set_window_visible(false);
        
        Ok(())
    }
    
    /// 切换窗口显示/隐藏（基于全局状态取反）
    pub fn toggle(app: &AppHandle) -> Result<bool, String> {
        // 从全局状态读取并取反，确保状态同步
        let visible = Self::is_window_visible();
        let new_visible = !visible;
        
        if visible {
            Self::hide(app)?;
        } else {
            Self::show(app)?;
        }
        
        Ok(new_visible)
    }
    
    /// 设置窗口置顶状态
    pub fn set_always_on_top(app: &AppHandle, on_top: bool) -> Result<(), String> {
        let window = app.get_webview_window("main").ok_or("Window not found")?;
        window.set_always_on_top(on_top).map_err(|e| e.to_string())
    }
    
    /// 检查窗口是否可见（返回全局状态，不是实际窗口状态）
    pub fn is_visible() -> bool {
        Self::is_window_visible()
    }
    
    /// 初始化窗口状态（应用启动时调用）
    pub fn init_state(app: &AppHandle) -> Result<(), String> {
        let window = app.get_webview_window("main").ok_or("Window not found")?;
        let is_visible = window.is_visible().unwrap_or(false);
        Self::set_window_visible(is_visible);
        Ok(())
    }
    
    /// 调度延迟取消置顶
    fn schedule_unpin(window: tauri::WebviewWindow) {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(DELAY_UNPIN_MS));
            let _ = window.set_always_on_top(false);
        });
    }
}

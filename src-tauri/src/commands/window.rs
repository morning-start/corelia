//! 窗口管理命令
//! 
//! 提供 Tauri Command 接口，调用 WindowService 执行业务逻辑

use tauri::AppHandle;
use crate::services::WindowService;

/// 切换窗口显示/隐藏状态
#[tauri::command]
pub fn toggle_window(app: AppHandle) -> Result<(), String> {
    let new_visible = WindowService::toggle(&app)?;
    eprintln!("窗口状态切换 - 新状态：{}", new_visible);
    Ok(())
}

/// 显示窗口
#[tauri::command]
pub fn show_window(app: AppHandle) -> Result<(), String> {
    WindowService::show(&app)
}

/// 隐藏窗口
#[tauri::command]
pub fn hide_window(app: AppHandle) -> Result<(), String> {
    WindowService::hide(&app)
}

/// 设置窗口置顶
#[tauri::command]
pub fn set_always_on_top(app: AppHandle, on_top: bool) -> Result<(), String> {
    WindowService::set_always_on_top(&app, on_top)
}

/// 检查窗口可见性（返回全局状态）
#[tauri::command]
pub fn check_window_visible() -> Result<bool, String> {
    Ok(WindowService::is_visible())
}

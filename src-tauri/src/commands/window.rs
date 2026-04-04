use tauri::{AppHandle, Manager};

/// 切换窗口显示/隐藏状态（使用全局状态）
#[tauri::command]
pub fn toggle_window(app: AppHandle) -> Result<(), String> {
    // 从全局状态读取（与 lib.rs 中的 WINDOW_VISIBLE 同步）
    // 注意：这里需要通过参数传递状态，或者使用共享的全局状态
    let window = app.get_webview_window("main").ok_or("Window not found")?;
    
    // 简化版本：直接切换，不依赖全局状态（由调用方管理）
    if window.is_visible().unwrap_or(false) {
        window.hide().map_err(|e| e.to_string())
    } else {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn show_window(app: AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("Window not found")?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn hide_window(app: AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("Window not found")?;
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_always_on_top(app: AppHandle, on_top: bool) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("Window not found")?;
    window.set_always_on_top(on_top).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_window_visible(app: AppHandle) -> Result<bool, String> {
    let window = app.get_webview_window("main").ok_or("Window not found")?;
    window.is_visible().map_err(|e| e.to_string())
}

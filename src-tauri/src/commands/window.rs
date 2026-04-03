use tauri::{AppHandle, Manager};

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
pub fn toggle_window(app: AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("Window not found")?;
    if window.is_visible().unwrap_or(false) {
        window.hide().map_err(|e| e.to_string())
    } else {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())
    }
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

use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[tauri::command]
pub fn register_shortcut_cmd(app: AppHandle) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
    let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
    app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            let window = app.get_webview_window("main").unwrap();
            if window.is_visible().unwrap() {
                let _ = window.hide();
            } else {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn unregister_all_shortcuts(app: AppHandle) -> Result<(), String> {
    app.global_shortcut().unregister_all().map_err(|e| e.to_string())
}

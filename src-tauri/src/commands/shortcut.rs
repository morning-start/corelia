//! 全局快捷键管理命令

use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use crate::services::WindowService;

static CURRENT_SHORTCUT: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static KEY_CODE_MAP: Lazy<Mutex<HashMap<String, Code>>> = Lazy::new(|| {
    Mutex::new({
        let mut m = HashMap::new();
        m.insert("SPACE".into(), Code::Space);
        m.insert("A".into(), Code::KeyA); m.insert("B".into(), Code::KeyB); m.insert("C".into(), Code::KeyC);
        m.insert("D".into(), Code::KeyD); m.insert("E".into(), Code::KeyE); m.insert("F".into(), Code::KeyF);
        m.insert("G".into(), Code::KeyG); m.insert("H".into(), Code::KeyH); m.insert("I".into(), Code::KeyI);
        m.insert("J".into(), Code::KeyJ); m.insert("K".into(), Code::KeyK); m.insert("L".into(), Code::KeyL);
        m.insert("M".into(), Code::KeyM); m.insert("N".into(), Code::KeyN); m.insert("O".into(), Code::KeyO);
        m.insert("P".into(), Code::KeyP); m.insert("Q".into(), Code::KeyQ); m.insert("R".into(), Code::KeyR);
        m.insert("S".into(), Code::KeyS); m.insert("T".into(), Code::KeyT); m.insert("U".into(), Code::KeyU);
        m.insert("V".into(), Code::KeyV); m.insert("W".into(), Code::KeyW); m.insert("X".into(), Code::KeyX);
        m.insert("Y".into(), Code::KeyY); m.insert("Z".into(), Code::KeyZ);
        m.insert("0".into(), Code::Digit0); m.insert("1".into(), Code::Digit1); m.insert("2".into(), Code::Digit2);
        m.insert("3".into(), Code::Digit3); m.insert("4".into(), Code::Digit4); m.insert("5".into(), Code::Digit5);
        m.insert("6".into(), Code::Digit6); m.insert("7".into(), Code::Digit7); m.insert("8".into(), Code::Digit8);
        m.insert("9".into(), Code::Digit9);
        m.insert("F1".into(), Code::F1); m.insert("F2".into(), Code::F2); m.insert("F3".into(), Code::F3);
        m.insert("F4".into(), Code::F4); m.insert("F5".into(), Code::F5); m.insert("F6".into(), Code::F6);
        m.insert("F7".into(), Code::F7); m.insert("F8".into(), Code::F8); m.insert("F9".into(), Code::F9);
        m.insert("F10".into(), Code::F10); m.insert("F11".into(), Code::F11); m.insert("F12".into(), Code::F12);
        m
    })
});

fn parse_shortcut(shortcut_str: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = shortcut_str.split('+').collect();
    if parts.is_empty() {
        return Err("Invalid shortcut format".to_string());
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;
    let map = KEY_CODE_MAP.lock().map_err(|e| e.to_string())?;

    for part in parts.iter() {
        match part.to_lowercase().as_str() {
            "ctrl" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "meta" => modifiers |= Modifiers::SUPER,
            key => {
                let upper = key.to_uppercase();
                key_code = map.get(&upper).copied()
                    .ok_or_else(|| format!("Unsupported key: {}", key))
                    .ok();
            }
        }
    }

    key_code.map(|code| Shortcut::new(Some(modifiers), code))
        .ok_or_else(|| "No key specified in shortcut".to_string())
}

#[tauri::command]
pub fn register_shortcut_cmd(app: AppHandle) -> Result<(), String> {
    let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
        if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
            let _ = WindowService::toggle(&app_handle);
        }
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn register_custom_shortcut(app: AppHandle, shortcut: String) -> Result<(), String> {
    app.global_shortcut().unregister_all().map_err(|e| e.to_string())?;
    let parsed = parse_shortcut(&shortcut)?;
    {
        let mut current = CURRENT_SHORTCUT.lock().map_err(|e| e.to_string())?;
        *current = Some(shortcut);
    }
    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(parsed, move |_app, _shortcut, event| {
        if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
            let _ = WindowService::toggle(&app_handle);
        }
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unregister_all_shortcuts(app: AppHandle) -> Result<(), String> {
    app.global_shortcut().unregister_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_current_shortcut() -> Result<Option<String>, String> {
    let current = CURRENT_SHORTCUT.lock().map_err(|e| e.to_string())?;
    Ok(current.clone())
}

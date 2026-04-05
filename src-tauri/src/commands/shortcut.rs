//! 全局快捷键管理命令
//! 
//! 提供全局快捷键的注册和管理功能

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::services::WindowService;

/// 当前快捷键（全局状态）
static CURRENT_SHORTCUT: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

/// 解析快捷键字符串为 Shortcut 对象
fn parse_shortcut(shortcut_str: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = shortcut_str.split('+').collect();
    if parts.is_empty() {
        return Err("Invalid shortcut format".to_string());
    }

    let mut modifiers = None;
    let mut key_code = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" => {
                use tauri_plugin_global_shortcut::Modifiers;
                modifiers = Some(modifiers.unwrap_or(Modifiers::empty()) | Modifiers::CONTROL);
            }
            "alt" => {
                use tauri_plugin_global_shortcut::Modifiers;
                modifiers = Some(modifiers.unwrap_or(Modifiers::empty()) | Modifiers::ALT);
            }
            "shift" => {
                use tauri_plugin_global_shortcut::Modifiers;
                modifiers = Some(modifiers.unwrap_or(Modifiers::empty()) | Modifiers::SHIFT);
            }
            "meta" => {
                use tauri_plugin_global_shortcut::Modifiers;
                modifiers = Some(modifiers.unwrap_or(Modifiers::empty()) | Modifiers::SUPER);
            }
            _ => {
                let code = match part.to_uppercase().as_str() {
                    "SPACE" => tauri_plugin_global_shortcut::Code::Space,
                    "A" => tauri_plugin_global_shortcut::Code::KeyA,
                    "B" => tauri_plugin_global_shortcut::Code::KeyB,
                    "C" => tauri_plugin_global_shortcut::Code::KeyC,
                    "D" => tauri_plugin_global_shortcut::Code::KeyD,
                    "E" => tauri_plugin_global_shortcut::Code::KeyE,
                    "F" => tauri_plugin_global_shortcut::Code::KeyF,
                    "G" => tauri_plugin_global_shortcut::Code::KeyG,
                    "H" => tauri_plugin_global_shortcut::Code::KeyH,
                    "I" => tauri_plugin_global_shortcut::Code::KeyI,
                    "J" => tauri_plugin_global_shortcut::Code::KeyJ,
                    "K" => tauri_plugin_global_shortcut::Code::KeyK,
                    "L" => tauri_plugin_global_shortcut::Code::KeyL,
                    "M" => tauri_plugin_global_shortcut::Code::KeyM,
                    "N" => tauri_plugin_global_shortcut::Code::KeyN,
                    "O" => tauri_plugin_global_shortcut::Code::KeyO,
                    "P" => tauri_plugin_global_shortcut::Code::KeyP,
                    "Q" => tauri_plugin_global_shortcut::Code::KeyQ,
                    "R" => tauri_plugin_global_shortcut::Code::KeyR,
                    "S" => tauri_plugin_global_shortcut::Code::KeyS,
                    "T" => tauri_plugin_global_shortcut::Code::KeyT,
                    "U" => tauri_plugin_global_shortcut::Code::KeyU,
                    "V" => tauri_plugin_global_shortcut::Code::KeyV,
                    "W" => tauri_plugin_global_shortcut::Code::KeyW,
                    "X" => tauri_plugin_global_shortcut::Code::KeyX,
                    "Y" => tauri_plugin_global_shortcut::Code::KeyY,
                    "Z" => tauri_plugin_global_shortcut::Code::KeyZ,
                    "0" => tauri_plugin_global_shortcut::Code::Digit0,
                    "1" => tauri_plugin_global_shortcut::Code::Digit1,
                    "2" => tauri_plugin_global_shortcut::Code::Digit2,
                    "3" => tauri_plugin_global_shortcut::Code::Digit3,
                    "4" => tauri_plugin_global_shortcut::Code::Digit4,
                    "5" => tauri_plugin_global_shortcut::Code::Digit5,
                    "6" => tauri_plugin_global_shortcut::Code::Digit6,
                    "7" => tauri_plugin_global_shortcut::Code::Digit7,
                    "8" => tauri_plugin_global_shortcut::Code::Digit8,
                    "9" => tauri_plugin_global_shortcut::Code::Digit9,
                    "F1" => tauri_plugin_global_shortcut::Code::F1,
                    "F2" => tauri_plugin_global_shortcut::Code::F2,
                    "F3" => tauri_plugin_global_shortcut::Code::F3,
                    "F4" => tauri_plugin_global_shortcut::Code::F4,
                    "F5" => tauri_plugin_global_shortcut::Code::F5,
                    "F6" => tauri_plugin_global_shortcut::Code::F6,
                    "F7" => tauri_plugin_global_shortcut::Code::F7,
                    "F8" => tauri_plugin_global_shortcut::Code::F8,
                    "F9" => tauri_plugin_global_shortcut::Code::F9,
                    "F10" => tauri_plugin_global_shortcut::Code::F10,
                    "F11" => tauri_plugin_global_shortcut::Code::F11,
                    "F12" => tauri_plugin_global_shortcut::Code::F12,
                    _ => return Err(format!("Unsupported key: {}", part)),
                };
                key_code = Some(code);
            }
        }
    }

    if let Some(code) = key_code {
        Ok(Shortcut::new(modifiers, code))
    } else {
        Err("No key specified in shortcut".to_string())
    }
}

/// 注册默认快捷键（Alt+Space）
#[tauri::command]
pub fn register_shortcut_cmd(app: AppHandle) -> Result<(), String> {
    let default_shortcut = Shortcut::new(
        Some(tauri_plugin_global_shortcut::Modifiers::ALT),
        tauri_plugin_global_shortcut::Code::Space,
    );

    app.global_shortcut().on_shortcut(default_shortcut, move |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            let _ = WindowService::toggle(app);
        }
    }).map_err(|e| e.to_string())?;

    Ok(())
}

/// 注册自定义快捷键
#[tauri::command]
pub fn register_custom_shortcut(app: AppHandle, shortcut: String) -> Result<(), String> {
    // 注销所有快捷键
    let _ = app.global_shortcut().unregister_all();

    // 解析并注册新快捷键
    let parsed_shortcut = parse_shortcut(&shortcut)?;

    {
        let mut current = CURRENT_SHORTCUT.lock().map_err(|e| e.to_string())?;
        *current = Some(shortcut.clone());
    }

    app.global_shortcut().on_shortcut(parsed_shortcut, move |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            let _ = WindowService::toggle(app);
        }
    }).map_err(|e| e.to_string())?;

    Ok(())
}

/// 注销所有快捷键
#[tauri::command]
pub fn unregister_all_shortcuts(app: AppHandle) -> Result<(), String> {
    app.global_shortcut().unregister_all().map_err(|e| e.to_string())
}

/// 获取当前快捷键
#[tauri::command]
pub fn get_current_shortcut() -> Result<Option<String>, String> {
    let current = CURRENT_SHORTCUT.lock().map_err(|e| e.to_string())?;
    Ok(current.clone())
}

#![allow(static_mut_refs)]

mod commands;
mod error;

use rquickjs::{Context, Runtime};
use tauri_plugin_store::StoreExt;
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri::{
    menu::{Menu, MenuItem},
    image::Image,
    tray::TrayIconBuilder,
    Manager,
};
use std::sync::atomic::{AtomicBool, Ordering};

static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);

static mut JS_RUNTIME: Option<Runtime> = None;
static mut JS_CONTEXT: Option<Context> = None;

/// 切换窗口显示/隐藏状态（使用全局状态）
fn toggle_window_visibility(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        // 从全局状态读取当前可见性
        let is_visible = WINDOW_VISIBLE.load(Ordering::SeqCst);
        
        eprintln!("切换窗口状态 - 当前可见：{}", is_visible);
        
        if is_visible {
            // 隐藏窗口
            eprintln!("隐藏窗口");
            let _ = window.hide();
            WINDOW_VISIBLE.store(false, Ordering::SeqCst);
        } else {
            // 显示窗口并置顶
            eprintln!("显示窗口");
            let _ = window.set_always_on_top(true);
            let _ = window.show();
            let _ = window.set_focus();
            WINDOW_VISIBLE.store(true, Ordering::SeqCst);
            
            // 延迟取消置顶
            let window_clone = window.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let _ = window_clone.set_always_on_top(false);
            });
        }
    }
}

fn get_js_context() -> Result<&'static Context, String> {
    unsafe {
        if JS_RUNTIME.is_none() {
            let runtime = Runtime::new().map_err(|e| e.to_string())?;
            let context = Context::full(&runtime).map_err(|e| e.to_string())?;
            JS_RUNTIME = Some(runtime);
            JS_CONTEXT = Some(context);
        }
        Ok(JS_CONTEXT.as_ref().unwrap())
    }
}

#[tauri::command]
fn quickjs_execute(code: String) -> Result<String, String> {
    let ctx = get_js_context()?;
    let result = ctx.with(|ctx| {
        ctx.eval::<String, _>(code.as_str())
    });
    match result {
        Ok(value) => Ok(value),
        Err(e) => Err(format!("JS Error: {:?}", e)),
    }
}

#[tauri::command]
fn quickjs_init() -> Result<String, String> {
    get_js_context()?;
    Ok("QuickJS (rquickjs) initialized".to_string())
}

#[tauri::command]
async fn save_to_store(app: tauri::AppHandle, key: String, value: serde_json::Value) -> Result<(), String> {
    let store = app.store("corelia.json").map_err(|e| e.to_string())?;
    store.set(&key, value);
    store.save().map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_from_store(app: tauri::AppHandle, key: String) -> Result<serde_json::Value, String> {
    let store = app.store("corelia.json").map_err(|e| e.to_string())?;
    match store.get(&key) {
        Some(v) => Ok(v.clone()),
        None => Ok(serde_json::Value::Null),
    }
}

#[tauri::command]
async fn delete_from_store(app: tauri::AppHandle, key: String) -> Result<(), String> {
    let store = app.store("corelia.json").map_err(|e| e.to_string())?;
    let _ = store.delete(&key);
    store.save().map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_settings(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let store = app.store("corelia.json").map_err(|e| e.to_string())?;
    match store.get("settings") {
        Some(v) => Ok(v.clone()),
        None => Ok(serde_json::json!({
            "theme": "dark",
            "shortcut": { "summon": "Alt+Space" },
            "behavior": { "autoHide": true, "autoHideDelay": 3000 },
            "startup": { "enabled": false, "minimizeToTray": true }
        })),
    }
}

#[tauri::command]
async fn save_settings(app: tauri::AppHandle, settings: serde_json::Value) -> Result<(), String> {
    let store = app.store("corelia.json").map_err(|e| e.to_string())?;
    store.set("settings", settings);
    store.save().map_err(|e| e.to_string())
}

#[tauri::command]
async fn enable_autostart(app: tauri::AppHandle) -> Result<(), String> {
    let autostart = app.autolaunch();
    autostart.enable().map_err(|e| format!("{:?}", e))
}

#[tauri::command]
async fn disable_autostart(app: tauri::AppHandle) -> Result<(), String> {
    let autostart = app.autolaunch();
    autostart.disable().map_err(|e| format!("{:?}", e))
}

#[tauri::command]
async fn is_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    let autostart = app.autolaunch();
    autostart.is_enabled().map_err(|e| format!("{:?}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
        .setup(|app| {
            let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            // 使用 image crate 解码 PNG 图标
            let icon_bytes = include_bytes!("../icons/32x32.png");
            let img = image::load_from_memory(icon_bytes)
                .map_err(|e| format!("Failed to load icon: {}", e))?
                .into_rgba8();
            
            let (width, height) = img.dimensions();
            let rgba = img.into_raw();
            let icon_image = Image::new(rgba.as_slice(), width, height);

            // 初始化窗口状态
            if let Some(window) = app.get_webview_window("main") {
                let is_visible = window.is_visible().unwrap_or(false);
                WINDOW_VISIBLE.store(is_visible, Ordering::SeqCst);
            }

            let _tray = TrayIconBuilder::new()
                .icon(icon_image)
                .menu(&menu)
                .on_menu_event(|app: &tauri::AppHandle, event: tauri::menu::MenuEvent| match event.id.as_ref() {
                    "show" => {
                        toggle_window_visibility(app);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {},
                })
                .on_tray_icon_event(|tray: &tauri::tray::TrayIcon, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        toggle_window_visibility(&app);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            quickjs_execute,
            quickjs_init,
            commands::window::show_window,
            commands::window::hide_window,
            commands::window::toggle_window,
            commands::window::set_always_on_top,
            commands::window::is_window_visible,
            commands::clipboard::read_clipboard,
            commands::clipboard::write_clipboard,
            commands::shell::open_url,
            commands::shell::open_path,
            commands::shell::open_app,
            save_to_store,
            load_from_store,
            delete_from_store,
            load_settings,
            save_settings,
            enable_autostart,
            disable_autostart,
            is_autostart_enabled,
            commands::shortcut::register_shortcut_cmd,
            commands::shortcut::register_custom_shortcut,
            commands::shortcut::unregister_all_shortcuts,
            commands::shortcut::get_current_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
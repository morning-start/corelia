#![allow(static_mut_refs)]

mod commands;
mod error;
mod services;

// 导入配置命令到当前作用域
use commands::config::{
    load_system_config, save_system_config,
    load_user_config, save_user_config, reset_user_config,
    load_app_config, save_app_config, clear_app_config,
};

use rquickjs::{Context, Runtime};
use tauri_plugin_autostart::MacosLauncher;
use tauri::{
    menu::{Menu, MenuItem},
    image::Image,
    tray::TrayIconBuilder,
};
use services::WindowService;

static mut JS_RUNTIME: Option<Runtime> = None;
static mut JS_CONTEXT: Option<Context> = None;

/// 获取 JS 上下文（懒加载）
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

/// QuickJS 执行命令
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

/// QuickJS 初始化命令
#[tauri::command]
fn quickjs_init() -> Result<String, String> {
    get_js_context()?;
    Ok("QuickJS (rquickjs) initialized".to_string())
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

            // 解码图标
            let icon_bytes = include_bytes!("../icons/32x32.png");
            let img = image::load_from_memory(icon_bytes)
                .map_err(|e| format!("Failed to load icon: {}", e))?
                .into_rgba8();
            
            let (width, height) = img.dimensions();
            let rgba = img.into_raw();
            let icon_image = Image::new(rgba.as_slice(), width, height);

            // 初始化配置目录 (首次启动时创建配置文件)
            match services::ConfigService::init_config_directory(&app.handle()) {
                Ok(config_dir) => {
                    println!("配置目录：{:?}", config_dir);
                }
                Err(e) => {
                    eprintln!("初始化配置目录失败：{}", e);
                }
            }

            // 初始化窗口状态
            WindowService::init_state(&app.handle())?;

            // 创建托盘图标
            let _tray = TrayIconBuilder::new()
                .icon(icon_image)
                .menu(&menu)
                .on_menu_event(|app: &tauri::AppHandle, event: tauri::menu::MenuEvent| match event.id.as_ref() {
                    "show" => {
                        let _ = WindowService::toggle(app);
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
                        let _ = WindowService::toggle(&app);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // QuickJS
            quickjs_execute,
            quickjs_init,
            // 窗口管理
            commands::window::show_window,
            commands::window::hide_window,
            commands::window::toggle_window,
            commands::window::set_always_on_top,
            commands::window::check_window_visible,
            // 剪贴板
            commands::clipboard::read_clipboard,
            commands::clipboard::write_clipboard,
            // Shell 操作
            commands::shell::open_url,
            commands::shell::open_path,
            commands::shell::open_app,
            // 配置管理 (分层配置)
            load_system_config,
            save_system_config,
            load_user_config,
            save_user_config,
            reset_user_config,
            load_app_config,
            save_app_config,
            clear_app_config,
            // 数据存储 (兼容旧 API)
            commands::store::save_to_store,
            commands::store::load_from_store,
            commands::store::delete_from_store,
            // 自启动
            commands::autostart::enable_autostart,
            commands::autostart::disable_autostart,
            commands::autostart::is_autostart_enabled,
            // 全局快捷键
            commands::shortcut::register_shortcut_cmd,
            commands::shortcut::register_custom_shortcut,
            commands::shortcut::unregister_all_shortcuts,
            commands::shortcut::get_current_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

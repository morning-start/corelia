#![allow(static_mut_refs)]

mod commands;
mod error;
mod plugins;
mod services;

use tauri_plugin_autostart::MacosLauncher;
use tauri::{
    menu::{Menu, MenuItem},
    image::Image,
    tray::TrayIconBuilder,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use services::WindowService;

use plugins::quickjs_runtime::QuickJSRuntime;
use plugins::loader::PluginLoader;
use plugins::registry::PluginRegistry;
use plugins::wasm_bridge::WasmBridge;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
        .manage(QuickJSRuntime::new())  // 注册 QuickJS 运行时管理器（单例，供 Commands 直接使用）
        .manage({
            // 创建共享的 Arc<QuickJSRuntime> 实例
            let shared_runtime = Arc::new(QuickJSRuntime::new());
            Mutex::new(PluginLoader::new(
                PathBuf::from("plugins"),
                shared_runtime,  // Loader 与 State 共享同一 Runtime 实例
            ))
        })  // 注册插件加载器
        .manage(RwLock::new(PluginRegistry::new()))  // 注册插件注册表
        .manage(Mutex::new(WasmBridge::new()))  // 注册 WASM 桥接
        .setup(|app| {
            // 初始化 API Bridge 的 AppHandle
            plugins::api_bridge::set_app_handle(app.handle().clone());

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
            match services::ConfigService::init_config_directory(app.handle()) {
                Ok(config_dir) => {
                    println!("配置目录：{:?}", config_dir);
                }
                Err(e) => {
                    eprintln!("初始化配置目录失败：{}", e);
                }
            }

            WindowService::init_state(app.handle())?;

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
                        let _ = WindowService::toggle(app);
                    }
                })
                .build(app)?;

            Ok(())
        })
.invoke_handler(tauri::generate_handler![
            // 窗口
            commands::window::toggle_window,
            commands::window::show_window,
            commands::window::hide_window,
            commands::window::set_always_on_top,
            commands::window::check_window_visible,
            // Shell
            commands::shell::open_url,
            commands::shell::open_path,
            commands::shell::open_app,
            // 剪贴板
            commands::clipboard::read_clipboard,
            commands::clipboard::write_clipboard,
            // 快捷键
            commands::shortcut::register_shortcut_cmd,
            commands::shortcut::register_custom_shortcut,
            commands::shortcut::unregister_all_shortcuts,
            commands::shortcut::get_current_shortcut,
            // 存储
            commands::store::save_to_store,
            commands::store::load_from_store,
            commands::store::delete_from_store,
            // 自启动
            commands::autostart::enable_autostart,
            commands::autostart::disable_autostart,
            commands::autostart::is_autostart_enabled,
            // 配置（使用三级路径：子模块定义位置）
            commands::config::system::load_system_config,
            commands::config::system::save_system_config,
            commands::config::user::load_user_config,
            commands::config::user::save_user_config,
            commands::config::user::reset_user_config,
            commands::config::app::load_app_config,
            commands::config::app::save_app_config,
            commands::config::app::clear_app_config,
            // 插件数据存储
            commands::plugin::get_plugin_data_path,
            commands::plugin::read_plugin_data,
            commands::plugin::write_plugin_data,
            commands::plugin::delete_plugin_data,
            commands::plugin::clear_plugin_data,
            commands::plugin::get_plugin_data_size,
            // 插件生命周期管理（定义在 plugins::loader::commands）
            plugins::loader::commands::scan_plugins,
            plugins::loader::commands::get_plugin_list,
            plugins::loader::commands::load_plugin,
            plugins::loader::commands::unload_plugin,
            plugins::loader::commands::find_plugins_by_prefix,
            plugins::loader::commands::cleanup_idle_plugins,
            plugins::loader::commands::get_plugin_health,
            plugins::loader::commands::plugin_execute,
            // 插件注册表
            plugins::registry::search_plugins_by_prefix,
            plugins::registry::get_active_plugins,
            plugins::registry::get_plugin_state,
            // WASM 桥接
            plugins::wasm_bridge::wasm_register_functions,
            plugins::wasm_bridge::wasm_unregister_patch,
            plugins::wasm_bridge::wasm_list_functions,
            plugins::wasm_bridge::wasm_is_patch_loaded,
            plugins::wasm_bridge::wasm_call_function,
            plugins::wasm_bridge::wasm_store_call_result,
            plugins::wasm_bridge::wasm_get_call_result,
            // QuickJS 运行时
            plugins::quickjs_runtime::quickjs_create_vm,
            plugins::quickjs_runtime::quickjs_destroy_vm,
            plugins::quickjs_runtime::quickjs_execute,
            plugins::quickjs_runtime::quickjs_active_count,
            plugins::quickjs_runtime::quickjs_cleanup,
            plugins::quickjs_runtime::quickjs_cleanup_all,
            plugins::quickjs_runtime::quickjs_vm_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

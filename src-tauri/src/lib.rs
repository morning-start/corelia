#![allow(static_mut_refs)]

mod commands;
mod error;
mod plugins;
mod services;

// 导入配置命令到当前作用域
use commands::config::{
    load_system_config, save_system_config,
    load_user_config, save_user_config, reset_user_config,
    load_app_config, save_app_config, clear_app_config,
};

// 导入插件命令（重构后的版本）
use commands::plugin::{
    scan_plugins, get_plugin_list, load_plugin, unload_plugin,
    find_plugins_by_prefix, cleanup_idle_plugins, get_plugin_health,
    plugin_execute,
};

use tauri_plugin_autostart::MacosLauncher;
use tauri::{
    menu::{Menu, MenuItem},
    image::Image,
    tray::TrayIconBuilder,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use services::{WindowService, PluginService};

// 导入 QuickJS 运行时管理器
use plugins::quickjs_runtime::{
    QuickJSRuntime, quickjs_create_vm, quickjs_destroy_vm, quickjs_execute,
    quickjs_active_count, quickjs_cleanup, quickjs_cleanup_all, quickjs_vm_stats,
};

// 导入 API 桥接层
use plugins::api_bridge::inject_apis_to_vm;

// 导入插件加载器
use plugins::loader::PluginLoader;

// 导入插件注册表
use plugins::registry::{
    PluginRegistry, search_plugins_by_prefix,
    get_active_plugins, get_plugin_state,
};

// 导入 WASM 桥接层
use plugins::wasm_bridge::{
    WasmBridge, wasm_register_functions, wasm_unregister_patch,
    wasm_list_functions, wasm_is_patch_loaded, wasm_call_function,
    wasm_store_call_result, wasm_get_call_result,
};


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 创建共享的 QuickJSRuntime 实例
    let shared_runtime = Arc::new(QuickJSRuntime::new());
    // 创建共享的 PluginLoader 实例
    let shared_loader = Arc::new(Mutex::new(PluginLoader::new(
        PathBuf::from("plugins"),
        shared_runtime.clone(),
    )));
    // 创建 PluginService 实例
    let plugin_service = PluginService::new(shared_loader.clone(), shared_runtime.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
        .plugin(tauri_plugin_notification::init())
        .manage(shared_runtime)  // 注册 QuickJS 运行时管理器
        .manage(shared_loader)   // 注册插件加载器
        .manage(plugin_service)  // 注册插件服务（职责划分新增）
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
            // QuickJS 运行时管理（VM 池化）
            quickjs_create_vm,
            quickjs_destroy_vm,
            quickjs_execute,
            quickjs_active_count,
            quickjs_cleanup,
            quickjs_cleanup_all,
            quickjs_vm_stats,
            // API 注入
            inject_apis_to_vm,
            // WASM 桥接
            wasm_register_functions,
            wasm_unregister_patch,
            wasm_list_functions,
            wasm_is_patch_loaded,
            wasm_call_function,
            wasm_store_call_result,
            wasm_get_call_result,
            // 插件加载器管理（使用重构后的 Commands）
            scan_plugins,
            get_plugin_list,
            load_plugin,
            unload_plugin,
            find_plugins_by_prefix,
            cleanup_idle_plugins,
            get_plugin_health,
            plugin_execute,
            // 插件注册表查询
            search_plugins_by_prefix,
            get_active_plugins,
            get_plugin_state,
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
            // 插件数据隔离存储（注意：这个和 plugin_service 无关，属于数据存储）
            commands::plugin::get_plugin_data_path,
            commands::plugin::read_plugin_data,
            commands::plugin::write_plugin_data,
            commands::plugin::delete_plugin_data,
            commands::plugin::clear_plugin_data,
            commands::plugin::get_plugin_data_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

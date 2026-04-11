// API 桥接层模块
// 负责：暴露 Rust API 给 QuickJS 插件调用，实现 window.utools 兼容层
//
// ## 功能概述
// 将 uTools 插件常用的 API 注入到 QuickJS 全局上下文，
// 使 uTools 插件能在 Corelia 中运行。
//
// ## 支持的 API 子集 (MVP)
// - utools.dbStorage: 数据持久化（getItem/setItem/removeItem/getAll）
// - utools.clipboard: 剪贴板操作（readText/writeText）
// - utools.shell: Shell 执行（openPath/openExternal/showItemInFolder）
// - utools.hideMainWindow/showMainWindow: 窗口控制
//
// ## 技术要点
// - 同步 API 设计（QuickJS 单线程限制）
// - 所有操作都有日志输出便于调试
// - TODO 标记标识需要实际系统调用的位置

#![allow(dead_code)]

use rquickjs::{Ctx, Object, Function};
use std::sync::{Mutex, OnceLock};
use crate::plugins::quickjs_runtime::QuickJSRuntime;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set_app_handle(app: AppHandle) {
    let _ = APP_HANDLE.set(app);
}

pub(crate) fn get_app_handle() -> Option<AppHandle> {
    APP_HANDLE.get().cloned()
}

/// 获取全局 AppHandle，若未初始化则返回 rquickjs::Error
///
/// 用于 ApiBridge 注入的 JS 回调函数中，消除重复的 match 模式。
/// # Usage
/// ```ignore
/// let app = require_app!("myFunctionName");
/// ```
macro_rules! require_app {
    ($fn_name:expr) => {
        match get_app_handle() {
            Some(h) => h,
            None => return Err(rquickjs::Error::new_from_js_message(
                $fn_name, "Error", "AppHandle not initialized"
            )),
        }
    };
}

/// API 桥接层：将 Rust 函数注入到 QuickJS 全局上下文
///
/// 所有 API 注入通过静态方法完成，AppHandle 通过全局单例获取。
pub struct ApiBridge;

impl ApiBridge {
    /// 注入完整的 utools 对象到 QuickJS 全局上下文
    /// instance_id: 当前注入的插件实例 ID，用于事件通知
    pub fn inject_utools(ctx: &Ctx, instance_id: &str) -> Result<(), String> {
        println!("[ApiBridge] 开始注入 window.utools API (插件: {})...", instance_id);

        let globals = ctx.globals();
        let utools_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 utools 对象失败: {}", e))?;

        inject_db_storage(ctx, &utools_obj, instance_id.to_string())?;
        inject_clipboard(ctx, &utools_obj)?;
        inject_shell(ctx, &utools_obj)?;
        inject_window_functions(ctx, &utools_obj)?;
        inject_path_functions(ctx, &utools_obj)?;
        inject_notification_functions(ctx, &utools_obj)?;
        inject_enhanced_clipboard(ctx, &utools_obj)?;
        inject_file_functions(ctx, &utools_obj)?;
        inject_plugin_callbacks(ctx, &utools_obj, instance_id.to_string())?;

        globals.set("utools", utools_obj).map_err(|e| format!("设置全局变量失败: {}", e))?;

        println!("[ApiBridge] window.utools API 注入成功 ✓");
        Ok(())
    }
}

fn inject_db_storage<'js>(ctx: &Ctx<'js>, parent: &Object<'js>, plugin_id: String) -> Result<(), String> {
    let db_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 dbStorage 对象失败: {}", e))?;
    let storage_path = format!("plugins/{}/storage.json", plugin_id);

    let plugin_id_get = plugin_id.clone();
    let storage_path_get = storage_path.clone();
    let get_item_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>, key: String| -> Result<Option<String>, rquickjs::Error> {
            println!("[utools.dbStorage] getItem: {} (plugin: {})", key, plugin_id_get);
            let app = require_app!("getItem");
            match app.store(&storage_path_get) {
                Ok(store) => {
                    match store.get(&key) {
                        Some(serde_json::Value::String(s)) => Ok(Some(s)),
                        Some(_) => Ok(Some("".to_string())),
                        None => Ok(None),
                    }
                }
                Err(e) => {
                    eprintln!("[utools.dbStorage] getItem 失败: {}", e);
                    Err(rquickjs::Error::new_from_js_message("getItem", "Error", format!("获取存储失败: {}", e)))
                }
            }
        },
    ).map_err(|e| format!("创建 getItem 函数失败: {}", e))?;
    db_obj.set("getItem", get_item_fn).map_err(|e| format!("设置 getItem 失败: {}", e))?;

    let plugin_id_set = plugin_id.clone();
    let storage_path_set = storage_path.clone();
    let set_item_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>, key: String, value: String| -> Result<(), rquickjs::Error> {
            println!("[utools.dbStorage] setItem: {} = {:?} (plugin: {})", key, value, plugin_id_set);
            let app = require_app!("setItem");
            match app.store(&storage_path_set) {
                Ok(store) => {
                    store.set(&key, serde_json::Value::String(value));
                    if let Err(e) = store.save() {
                        eprintln!("[utools.dbStorage] setItem 保存失败: {}", e);
                        return Err(rquickjs::Error::new_from_js_message("setItem", "Error", format!("保存失败: {}", e)));
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("[utools.dbStorage] setItem 失败: {}", e);
                    Err(rquickjs::Error::new_from_js_message("setItem", "Error", format!("获取存储失败: {}", e)))
                }
            }
        },
    ).map_err(|e| format!("创建 setItem 函数失败: {}", e))?;
    db_obj.set("setItem", set_item_fn).map_err(|e| format!("设置 setItem 失败: {}", e))?;

    let plugin_id_remove = plugin_id.clone();
    let storage_path_remove = storage_path.clone();
    let remove_item_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>, key: String| -> Result<(), rquickjs::Error> {
            println!("[utools.dbStorage] removeItem: {} (plugin: {})", key, plugin_id_remove);
            let app = require_app!("removeItem");
            match app.store(&storage_path_remove) {
                Ok(store) => {
                    store.delete(&key);
                    if let Err(e) = store.save() {
                        eprintln!("[utools.dbStorage] removeItem 保存失败: {}", e);
                        return Err(rquickjs::Error::new_from_js_message("removeItem", "Error", format!("保存失败: {}", e)));
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("[utools.dbStorage] removeItem 失败: {}", e);
                    Err(rquickjs::Error::new_from_js_message("removeItem", "Error", format!("获取存储失败: {}", e)))
                }
            }
        },
    ).map_err(|e| format!("创建 removeItem 函数失败: {}", e))?;
    db_obj.set("removeItem", remove_item_fn).map_err(|e| format!("设置 removeItem 失败: {}", e))?;

    let plugin_id_all = plugin_id.clone();
    let storage_path_all = storage_path.clone();
    let get_all_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>, ()| -> Result<std::collections::HashMap<String, String>, rquickjs::Error> {
            println!("[utools.dbStorage] getAll (plugin: {})", plugin_id_all);
            let app = require_app!("getAll");
            let store = match app.store(&storage_path_all) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[utools.dbStorage] getAll 获取存储失败: {}", e);
                    return Err(rquickjs::Error::new_from_js_message("getAll", "Error", format!("获取存储失败: {}", e)));
                }
            };
            let mut result = std::collections::HashMap::new();
            for key in store.keys() {
                if let Some(serde_json::Value::String(v)) = store.get(&key) {
                    result.insert(key, v);
                }
            }
            Ok(result)
        },
    ).map_err(|e| format!("创建 getAll 函数失败: {}", e))?;
    db_obj.set("getAll", get_all_fn).map_err(|e| format!("设置 getAll 失败: {}", e))?;

    parent.set("dbStorage", db_obj).map_err(|e| format!("设置 dbStorage 失败: {}", e))?;
    println!("[ApiBridge]   ✓ dbStorage 模块注入成功 (隔离存储: {})", storage_path);
    Ok(())
}

fn inject_clipboard<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let clip_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 clipboard 对象失败: {}", e))?;

    #[allow(unused_variables)]
    let read_text_fn = Function::new(
        ctx.clone(),
        |ctx: Ctx<'_>, ()| -> Result<String, rquickjs::Error> {
            println!("[utools.clipboard] readText");
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => match clipboard.get_text() {
                    Ok(text) => {
                        println!("[utools.clipboard] readText 成功: {} 字符", text.len());
                        Ok(text)
                    }
                    Err(e) => {
                        eprintln!("[utools.clipboard] readText 失败: {}", e);
                        Err(rquickjs::Error::new_from_js_message("readText", "Error", format!("读取剪贴板失败: {}", e)))
                    }
                },
                Err(e) => {
                    eprintln!("[utools.clipboard] 无法访问剪贴板: {}", e);
                    Err(rquickjs::Error::new_from_js_message("readText", "Error", format!("无法访问剪贴板: {}", e)))
                }
            }
        },
    ).map_err(|e| format!("创建 readText 函数失败: {}", e))?;
    clip_obj.set("readText", read_text_fn).map_err(|e| format!("设置 readText 失败: {}", e))?;

    #[allow(unused_variables)]
    let write_text_fn = Function::new(
        ctx.clone(),
        |ctx: Ctx<'_>, text: String| -> Result<(), rquickjs::Error> {
            println!("[utools.clipboard] writeText: {} 字符", text.len());
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => match clipboard.set_text(&text) {
                    Ok(_) => {
                        println!("[utools.clipboard] writeText 成功");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("[utools.clipboard] writeText 失败: {}", e);
                        Err(rquickjs::Error::new_from_js_message("writeText", "Error", format!("写入剪贴板失败: {}", e)))
                    }
                },
                Err(e) => {
                    eprintln!("[utools.clipboard] 无法访问剪贴板: {}", e);
                    Err(rquickjs::Error::new_from_js_message("writeText", "Error", format!("无法访问剪贴板: {}", e)))
                }
            }
        },
    ).map_err(|e| format!("创建 writeText 函数失败: {}", e))?;
    clip_obj.set("writeText", write_text_fn).map_err(|e| format!("设置 writeText 失败: {}", e))?;

    parent.set("clipboard", clip_obj).map_err(|e| format!("设置 clipboard 失败: {}", e))?;
    println!("[ApiBridge]   ✓ clipboard 模块注入成功");
    Ok(())
}

fn inject_shell<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let shell_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 shell 对象失败: {}", e))?;

    #[allow(unused_variables)]
    let open_path_fn = Function::new(
        ctx.clone(),
        |ctx: Ctx<'_>, path: String| -> Result<(), rquickjs::Error> {
            println!("[utools.shell] openPath: {}", path);
            match open::that(&path) {
                Ok(_) => {
                    println!("[utools.shell] openPath 成功");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("[utools.shell] openPath 失败: {}", e);
                    Err(rquickjs::Error::new_from_js_message("openPath", "Error", format!("打开路径失败: {}", e)))
                }
            }
        },
    ).map_err(|e| format!("创建 openPath 函数失败: {}", e))?;
    shell_obj.set("openPath", open_path_fn).map_err(|e| format!("设置 openPath 失败: {}", e))?;

    #[allow(unused_variables)]
    let open_external_fn = Function::new(
        ctx.clone(),
        |ctx: Ctx<'_>, url: String| -> Result<(), rquickjs::Error> {
            println!("[utools.shell] openExternal: {}", url);
            match open::that(&url) {
                Ok(_) => {
                    println!("[utools.shell] openExternal 成功");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("[utools.shell] openExternal 失败: {}", e);
                    Err(rquickjs::Error::new_from_js_message("openExternal", "Error", format!("打开外部链接失败: {}", e)))
                }
            }
        },
    ).map_err(|e| format!("创建 openExternal 函数失败: {}", e))?;
    shell_obj.set("openExternal", open_external_fn).map_err(|e| format!("设置 openExternal 失败: {}", e))?;

    #[allow(unused_variables)]
    let show_in_folder_fn = Function::new(
        ctx.clone(),
        |ctx: Ctx<'_>, path: String| -> Result<(), rquickjs::Error> {
            println!("[utools.shell] showItemInFolder: {}", path);

            #[cfg(target_os = "windows")]
            let result = open::that(format!("explorer /select,{}", path));

            #[cfg(target_os = "macos")]
            let result = open::that(format!("open -R {}", path));

            #[cfg(target_os = "linux")]
            let result = open::that(&path);

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            let result = open::that(&path);

            match result {
                Ok(_) => {
                    println!("[utools.shell] showItemInFolder 成功");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("[utools.shell] showItemInFolder 失败: {}", e);
                    Err(rquickjs::Error::new_from_js_message("showItemInFolder", "Error", format!("在文件管理器中显示失败: {}", e)))
                }
            }
        },
    ).map_err(|e| format!("创建 showItemInFolder 函数失败: {}", e))?;
    shell_obj.set("showItemInFolder", show_in_folder_fn).map_err(|e| format!("设置 showItemInFolder 失败: {}", e))?;

    parent.set("shell", shell_obj).map_err(|e| format!("设置 shell 失败: {}", e))?;
    println!("[ApiBridge]   ✓ shell 模块注入成功");
    Ok(())
}

fn inject_window_functions<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    #[allow(unused_variables)]
    let hide_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, ()| -> Result<(), rquickjs::Error> {
            println!("[utools] hideMainWindow 被调用");
            let app = require_app!("hideMainWindow");
            crate::services::WindowService::hide(&app)
                .map_err(|e| rquickjs::Error::new_from_js_message("hideMainWindow", "Error", e))
        },
    ).map_err(|e| format!("创建 hideMainWindow 函数失败: {}", e))?;
    parent.set("hideMainWindow", hide_fn).map_err(|e| format!("设置 hideMainWindow 失败: {}", e))?;

    #[allow(unused_variables)]
    let show_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, ()| -> Result<(), rquickjs::Error> {
            println!("[utools] showMainWindow 被调用");
            let app = require_app!("showMainWindow");
            crate::services::WindowService::show(&app)
                .map_err(|e| rquickjs::Error::new_from_js_message("showMainWindow", "Error", e))
        },
    ).map_err(|e| format!("创建 showMainWindow 函数失败: {}", e))?;
    parent.set("showMainWindow", show_fn).map_err(|e| format!("设置 showMainWindow 失败: {}", e))?;

    println!("[ApiBridge]   ✓ 窗口控制函数注入成功");
    Ok(())
}

fn inject_path_functions<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    #[allow(unused_variables)]
    let get_path_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, name: String| -> Result<String, rquickjs::Error> {
            println!("[utools] getPath: {}", name);
            let app = require_app!("getPath");
            let path = app.path();

            let result_path: Option<String> = if let Some(p) = match name.as_str() {
                "home" | "HOME" | "~" => path.home_dir().ok(),
                "desktop" | "DESKTOP" => path.desktop_dir().ok(),
                "document" | "DOCUMENT" | "documents" => path.document_dir().ok(),
                "download" | "DOWNLOAD" | "downloads" => path.download_dir().ok(),
                "music" | "MUSIC" => path.audio_dir().ok(),
                "picture" | "PICTURE" | "pictures" | "photo" | "PHOTO" => path.picture_dir().ok(),
                "video" | "VIDEO" | "videos" => path.video_dir().ok(),
                "temp" | "TEMP" | "tmp" => path.temp_dir().ok(),
                "appdata" | "APPDATA" => path.app_data_dir().ok(),
                "localappdata" | "LOCALAPPDATA" => path.app_local_data_dir().ok(),
                _ => return Err(rquickjs::Error::new_from_js_message("getPath", "Error", format!("未知的路径名称: {}", name))),
            } {
                Some(p.to_string_lossy().to_string())
            } else {
                None
            };

            match result_path {
                Some(p) => Ok(p),
                None => Err(rquickjs::Error::new_from_js_message("getPath", "Error", format!("无法获取路径: {}", name))),
            }
        },
    ).map_err(|e| format!("创建 getPath 函数失败: {}", e))?;
    parent.set("getPath", get_path_fn).map_err(|e| format!("设置 getPath 失败: {}", e))?;

    println!("[ApiBridge]   ✓ getPath 系统路径模块注入成功");
    Ok(())
}

fn inject_notification_functions<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    #[allow(unused_variables)]
    let show_notification_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, title: String, body: String| -> Result<(), rquickjs::Error> {
            println!("[utools] showNotification: {} - {}", title, body);
            let app = require_app!("showNotification");

            #[cfg(target_os = "windows")]
            {
                use std::process::Command;
                // 通过环境变量安全传递参数，避免 shell 注入
                let _ = Command::new("powershell")
                    .args([
                        "-NoProfile", "-Command",
                        "Add-Type -AssemblyName System.Windows.Forms; \
                         [System.Windows.Forms.MessageBox]::Show(\
                           [System.Environment]::GetEnvironmentVariable('NOTIFY_BODY'), \
                           [System.Environment]::GetEnvironmentVariable('NOTIFY_TITLE'), \
                           'OK', 'Information')"
                    ])
                    .env("NOTIFY_TITLE", &title)
                    .env("NOTIFY_BODY", &body)
                    .spawn();
            }

            #[cfg(target_os = "macos")]
            {
                use std::process::Command;
                // 通过环境变量安全传递参数，避免 shell 注入
                let _ = Command::new("osascript")
                    .args(["-e", "display notification (system attribute \"NOTIFY_BODY\" of environment) with title (system attribute \"NOTIFY_TITLE\" of environment)"])
                    .env("NOTIFY_TITLE", &title)
                    .env("NOTIFY_BODY", &body)
                    .spawn();
            }

            #[cfg(target_os = "linux")]
            {
                use std::process::Command;
                let _ = Command::new("notify-send")
                    .args([&title, &body])
                    .spawn();
            }

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            {
                eprintln!("[utools] showNotification: 不支持当前平台");
            }

            Ok(())
        },
    ).map_err(|e| format!("创建 showNotification 函数失败: {}", e))?;
    parent.set("showNotification", show_notification_fn).map_err(|e| format!("设置 showNotification 失败: {}", e))?;

    println!("[ApiBridge]   ✓ showNotification 通知模块注入成功");
    Ok(())
}

fn inject_enhanced_clipboard<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    #[allow(unused_variables)]
    let get_clipboard_image_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, ()| -> Result<Option<String>, rquickjs::Error> {
            println!("[utools] getClipboardImage");
            Ok(None)
        },
    ).map_err(|e| format!("创建 getClipboardImage 函数失败: {}", e))?;
    parent.set("getClipboardImage", get_clipboard_image_fn).map_err(|e| format!("设置 getClipboardImage 失败: {}", e))?;

    #[allow(unused_variables)]
    let set_clipboard_image_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, _base64: String| -> Result<(), rquickjs::Error> {
            println!("[utools] setClipboardImage");
            Ok(())
        },
    ).map_err(|e| format!("创建 setClipboardImage 函数失败: {}", e))?;
    parent.set("setClipboardImage", set_clipboard_image_fn).map_err(|e| format!("设置 setClipboardImage 失败: {}", e))?;

    #[allow(unused_variables)]
    let get_image_path_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, _base64: String, _name: String| -> Result<String, rquickjs::Error> {
            println!("[utools] getImagePath");
            Err(rquickjs::Error::new_from_js_message("getImagePath", "Error", "未实现"))
        },
    ).map_err(|e| format!("创建 getImagePath 函数失败: {}", e))?;
    parent.set("getImagePath", get_image_path_fn).map_err(|e| format!("设置 getImagePath 失败: {}", e))?;

    println!("[ApiBridge]   ✓ 增强版剪贴板模块注入成功");
    Ok(())
}

fn inject_file_functions<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let fs_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 fs 对象失败: {}", e))?;

    #[allow(unused_variables)]
    let read_text_file_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, path: String| -> Result<String, rquickjs::Error> {
            println!("[utools.fs] readTextFile: {}", path);
            std::fs::read_to_string(&path)
                .map_err(|e| rquickjs::Error::new_from_js_message("readTextFile", "Error", format!("读取文件失败: {}", e)))
        },
    ).map_err(|e| format!("创建 readTextFile 函数失败: {}", e))?;
    fs_obj.set("readTextFile", read_text_file_fn).map_err(|e| format!("设置 readTextFile 失败: {}", e))?;

    #[allow(unused_variables)]
    let write_text_file_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, path: String, content: String| -> Result<(), rquickjs::Error> {
            println!("[utools.fs] writeTextFile: {} ({} bytes)", path, content.len());
            std::fs::write(&path, &content)
                .map_err(|e| rquickjs::Error::new_from_js_message("writeTextFile", "Error", format!("写入文件失败: {}", e)))
        },
    ).map_err(|e| format!("创建 writeTextFile 函数失败: {}", e))?;
    fs_obj.set("writeTextFile", write_text_file_fn).map_err(|e| format!("设置 writeTextFile 失败: {}", e))?;

    #[allow(unused_variables)]
    let exists_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, path: String| -> bool {
            println!("[utools.fs] exists: {}", path);
            std::path::Path::new(&path).exists()
        },
    ).map_err(|e| format!("创建 exists 函数失败: {}", e))?;
    fs_obj.set("exists", exists_fn).map_err(|e| format!("设置 exists 失败: {}", e))?;

    #[allow(unused_variables)]
    let is_dir_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, path: String| -> bool {
            println!("[utools.fs] isDir: {}", path);
            std::path::Path::new(&path).is_dir()
        },
    ).map_err(|e| format!("创建 isDir 函数失败: {}", e))?;
    fs_obj.set("isDir", is_dir_fn).map_err(|e| format!("设置 isDir 失败: {}", e))?;

    parent.set("fs", fs_obj).map_err(|e| format!("设置 fs 失败: {}", e))?;
    println!("[ApiBridge]   ✓ fs 文件操作模块注入成功");
    Ok(())
}

fn inject_plugin_callbacks<'js>(ctx: &Ctx<'js>, parent: &Object<'js>, instance_id: String) -> Result<(), String> {
    let instance_id_clone = instance_id.clone();
    let on_ready_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>| {
            println!("[utools] onPluginReady 调用，插件: {}", instance_id_clone);
            if let Some(app) = get_app_handle() {
                let _ = app.emit("plugin-ready", instance_id_clone.clone());
            }
            Ok::<(), rquickjs::Error>(())
        },
    ).map_err(|e| format!("创建 onPluginReady 函数失败: {}", e))?;
    parent.set("onPluginReady", on_ready_fn).map_err(|e| format!("设置 onPluginReady 失败: {}", e))?;

    let instance_id_clone2 = instance_id.clone();
    let on_out_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>| {
            println!("[utools] onPluginOut 调用，插件: {}", instance_id_clone2);
            if let Some(app) = get_app_handle() {
                let _ = app.emit("plugin-out", instance_id_clone2.clone());
            }
            Ok::<(), rquickjs::Error>(())
        },
    ).map_err(|e| format!("创建 onPluginOut 函数失败: {}", e))?;
    parent.set("onPluginOut", on_out_fn).map_err(|e| format!("设置 onPluginOut 失败: {}", e))?;

    let instance_id_clone3 = instance_id.clone();
    let register_feature_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>, _feature: rquickjs::Value| -> Result<(), rquickjs::Error> {
            println!("[utools] registerPluginFeature 调用，插件: {}", instance_id_clone3);
            if let Some(app) = get_app_handle() {
                let _ = app.emit("plugin-feature", instance_id_clone3.clone());
            }
            Ok(())
        },
    ).map_err(|e| format!("创建 registerPluginFeature 函数失败: {}", e))?;
    parent.set("registerPluginFeature", register_feature_fn).map_err(|e| format!("设置 registerPluginFeature 失败: {}", e))?;

    println!("[ApiBridge]   ✓ 插件生命周期回调注入成功");
    Ok(())
}

#[tauri::command]
pub async fn inject_apis_to_vm(
    runtime: tauri::State<'_, Mutex<QuickJSRuntime>>,
    vm_id: String,
    plugin_id: String,
) -> Result<(), String> {
    println!("[Command] inject_apis_to_vm: 注入 API 到 VM {} (插件: {})", vm_id, plugin_id);

    let rt = runtime.lock().map_err(|e| format!("获取运行时锁失败: {}", e))?;

    rt.with_context(&vm_id, |ctx| ApiBridge::inject_utools(&ctx, &plugin_id))
}

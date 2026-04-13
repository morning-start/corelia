// API 桥接层模块
// 负责：暴露 Rust API 给 QuickJS 插件调用，实现 window.utools 兼容层

#![allow(dead_code)]

use rquickjs::{Ctx, Object, Function, Value};
use std::sync::{Mutex, OnceLock};
use crate::plugins::quickjs_runtime::QuickJSRuntime;
use crate::plugins::wasm_bridge::WasmBridge;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set_app_handle(app: AppHandle) {
    let _ = APP_HANDLE.set(app);
}

pub(crate) fn get_app_handle() -> Option<AppHandle> {
    APP_HANDLE.get().cloned()
}

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

pub struct ApiBridge;

impl ApiBridge {
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
        inject_file_functions(ctx, &utools_obj)?;
        inject_plugin_callbacks(ctx, &utools_obj, instance_id.to_string())?;
        inject_fetch_api(ctx, &utools_obj)?;
        inject_dialog_api(ctx, &utools_obj)?;
        inject_process_api(ctx, &utools_obj)?;
        inject_context_api(ctx, &utools_obj)?;
        inject_wasm_api(ctx, &utools_obj)?;

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
                Err(e) => Err(rquickjs::Error::new_from_js_message("getItem", "Error", format!("获取存储失败: {}", e)))
            }
        },
    ).map_err(|e| format!("创建 getItem 函数失败: {}", e))?;
    db_obj.set("getItem", get_item_fn).map_err(|e| format!("设置 getItem 失败: {}", e))?;

    let plugin_id_set = plugin_id.clone();
    let storage_path_set = storage_path.clone();
    let set_item_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>, key: String, value: String| -> Result<(), rquickjs::Error> {
            println!("[utools.dbStorage] setItem: {} (plugin: {})", key, plugin_id_set);
            let app = require_app!("setItem");
            match app.store(&storage_path_set) {
                Ok(store) => {
                    store.set(&key, serde_json::Value::String(value));
                    if let Err(e) = store.save() {
                        return Err(rquickjs::Error::new_from_js_message("setItem", "Error", format!("保存失败: {}", e)));
                    }
                    Ok(())
                }
                Err(e) => Err(rquickjs::Error::new_from_js_message("setItem", "Error", format!("获取存储失败: {}", e)))
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
                        return Err(rquickjs::Error::new_from_js_message("removeItem", "Error", format!("保存失败: {}", e)));
                    }
                    Ok(())
                }
                Err(e) => Err(rquickjs::Error::new_from_js_message("removeItem", "Error", format!("获取存储失败: {}", e)))
            }
        },
    ).map_err(|e| format!("创建 removeItem 函数失败: {}", e))?;
    db_obj.set("removeItem", remove_item_fn).map_err(|e| format!("设置 removeItem 失败: {}", e))?;

    let storage_path_all = storage_path;
    let plugin_id_all = plugin_id;
    let get_all_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>, ()| -> Result<std::collections::HashMap<String, String>, rquickjs::Error> {
            println!("[utools.dbStorage] getAll (plugin: {})", plugin_id_all);
            let app = require_app!("getAll");
            let store = match app.store(&storage_path_all) {
                Ok(s) => s,
                Err(e) => return Err(rquickjs::Error::new_from_js_message("getAll", "Error", format!("获取存储失败: {}", e)))
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
    println!("[ApiBridge]   ✓ dbStorage 模块注入成功");
    Ok(())
}

fn inject_clipboard<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let clip_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 clipboard 对象失败: {}", e))?;

    let read_text_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, ()| -> Result<String, rquickjs::Error> {
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => clipboard.get_text()
                    .map_err(|e| rquickjs::Error::new_from_js_message("readText", "Error", format!("读取剪贴板失败: {}", e))),
                Err(e) => Err(rquickjs::Error::new_from_js_message("readText", "Error", format!("无法访问剪贴板: {}", e)))
            }
        },
    ).map_err(|e| format!("创建 readText 函数失败: {}", e))?;
    clip_obj.set("readText", read_text_fn).map_err(|e| format!("设置 readText 失败: {}", e))?;

    let write_text_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, text: String| -> Result<(), rquickjs::Error> {
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => clipboard.set_text(&text)
                    .map_err(|e| rquickjs::Error::new_from_js_message("writeText", "Error", format!("写入剪贴板失败: {}", e))),
                Err(e) => Err(rquickjs::Error::new_from_js_message("writeText", "Error", format!("无法访问剪贴板: {}", e)))
            }
        },
    ).map_err(|e| format!("创建 writeText 函数失败: {}", e))?;
    clip_obj.set("writeText", write_text_fn).map_err(|e| format!("设置 writeText 失败: {}", e))?;

    let copy_text_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, text: String| -> Result<(), rquickjs::Error> {
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => clipboard.set_text(&text)
                    .map_err(|e| rquickjs::Error::new_from_js_message("copyText", "Error", format!("复制失败: {}", e))),
                Err(e) => Err(rquickjs::Error::new_from_js_message("copyText", "Error", format!("访问剪贴板失败: {}", e)))
            }
        },
    ).map_err(|e| format!("创建 copyText 函数失败: {}", e))?;
    clip_obj.set("copyText", copy_text_fn).map_err(|e| format!("设置 copyText 失败: {}", e))?;

    parent.set("clipboard", clip_obj).map_err(|e| format!("设置 clipboard 失败: {}", e))?;
    println!("[ApiBridge]   ✓ clipboard 模块注入成功");
    Ok(())
}

fn inject_shell<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let shell_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 shell 对象失败: {}", e))?;

    let open_path_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, path: String| -> Result<(), rquickjs::Error> {
            open::that(&path).map_err(|e| rquickjs::Error::new_from_js_message("openPath", "Error", format!("打开路径失败: {}", e)))
        },
    ).map_err(|e| format!("创建 openPath 函数失败: {}", e))?;
    shell_obj.set("openPath", open_path_fn).map_err(|e| format!("设置 openPath 失败: {}", e))?;

    let open_external_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, url: String| -> Result<(), rquickjs::Error> {
            open::that(&url).map_err(|e| rquickjs::Error::new_from_js_message("openExternal", "Error", format!("打开外部链接失败: {}", e)))
        },
    ).map_err(|e| format!("创建 openExternal 函数失败: {}", e))?;
    shell_obj.set("openExternal", open_external_fn).map_err(|e| format!("设置 openExternal 失败: {}", e))?;

    let show_in_folder_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, path: String| -> Result<(), rquickjs::Error> {
            #[cfg(target_os = "windows")]
            let result = open::that(format!("explorer /select,{}", path));
            #[cfg(not(target_os = "windows"))]
            let result = open::that(&path);
            result.map_err(|e| rquickjs::Error::new_from_js_message("showItemInFolder", "Error", format!("显示失败: {}", e)))
        },
    ).map_err(|e| format!("创建 showItemInFolder 函数失败: {}", e))?;
    shell_obj.set("showItemInFolder", show_in_folder_fn).map_err(|e| format!("设置 showItemInFolder 失败: {}", e))?;

    let beep_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, ()| -> Result<(), rquickjs::Error> {
            #[cfg(target_os = "windows")]
            { let _ = std::process::Command::new("powershell").args(["-NoProfile", "-Command", "[Console]::Beep(800,200)"]).spawn(); }
            Ok(())
        },
    ).map_err(|e| format!("创建 beep 函数失败: {}", e))?;
    shell_obj.set("beep", beep_fn).map_err(|e| format!("设置 beep 失败: {}", e))?;

    parent.set("shell", shell_obj).map_err(|e| format!("设置 shell 失败: {}", e))?;
    println!("[ApiBridge]   ✓ shell 模块注入成功");
    Ok(())
}

fn inject_window_functions<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let hide_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, ()| -> Result<(), rquickjs::Error> {
            let app = require_app!("hideMainWindow");
            crate::services::WindowService::hide(&app).map_err(|e| rquickjs::Error::new_from_js_message("hideMainWindow", "Error", e))
        },
    ).map_err(|e| format!("创建 hideMainWindow 函数失败: {}", e))?;
    parent.set("hideMainWindow", hide_fn).map_err(|e| format!("设置 hideMainWindow 失败: {}", e))?;

    let show_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, ()| -> Result<(), rquickjs::Error> {
            let app = require_app!("showMainWindow");
            crate::services::WindowService::show(&app).map_err(|e| rquickjs::Error::new_from_js_message("showMainWindow", "Error", e))
        },
    ).map_err(|e| format!("创建 showMainWindow 函数失败: {}", e))?;
    parent.set("showMainWindow", show_fn).map_err(|e| format!("设置 showMainWindow 失败: {}", e))?;

    println!("[ApiBridge]   ✓ 窗口控制函数注入成功");
    Ok(())
}

fn inject_path_functions<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let get_path_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, name: String| -> Result<String, rquickjs::Error> {
            let app = require_app!("getPath");
            let path = app.path();

            let result_path = match name.to_lowercase().as_str() {
                "home" | "~" => path.home_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "desktop" => path.desktop_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "document" | "documents" => path.document_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "download" | "downloads" => path.download_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "music" => path.audio_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "picture" | "pictures" | "photo" | "photos" => path.picture_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "video" | "videos" => path.video_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "temp" | "tmp" => path.temp_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "appdata" => path.app_data_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "localappdata" | "appcache" => path.app_cache_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "userdata" => path.app_data_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "config" => path.app_config_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "log" | "logs" => path.app_log_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "resource" | "resources" => std::env::current_exe().ok().map(|p| p.parent().unwrap_or(&p).join("resources").to_string_lossy().to_string()),
                "exe" | "exepath" => std::env::current_exe().ok().map(|p| p.to_string_lossy().to_string()),
                "plugin" | "pluginpath" => std::env::current_exe().ok().map(|p| p.parent().unwrap_or(&p).join("plugins").to_string_lossy().to_string()),
                "root" => Some("/".to_string()),
                "cwd" | "currentdir" => std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
                _ => None,
            };

            match result_path {
                Some(p) => Ok(p),
                None => Err(rquickjs::Error::new_from_js_message("getPath", "Error", format!("无法获取路径: {}", name)))
            }
        },
    ).map_err(|e| format!("创建 getPath 函数失败: {}", e))?;
    parent.set("getPath", get_path_fn).map_err(|e| format!("设置 getPath 失败: {}", e))?;
    println!("[ApiBridge]   ✓ getPath 模块注入成功");
    Ok(())
}

fn inject_notification_functions<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let show_notification_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, title: String, body: String| -> Result<(), rquickjs::Error> {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", &format!("[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null; $xml = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); $xml.GetElementsByTagName('text')[0].AppendChild($xml.CreateTextNode('{}')) | Out-Null; $xml.GetElementsByTagName('text')[1].AppendChild($xml.CreateTextNode('{}')) | Out-Null; $toast = [Windows.UI.Notifications.ToastNotification]::new($xml); [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Corelia').Show($toast)", title.replace('\'', "''"), body.replace('\'', "''"))])
                    .spawn();
            }
            Ok(())
        },
    ).map_err(|e| format!("创建 showNotification 函数失败: {}", e))?;
    parent.set("showNotification", show_notification_fn).map_err(|e| format!("设置 showNotification 失败: {}", e))?;
    println!("[ApiBridge]   ✓ showNotification 模块注入成功");
    Ok(())
}

fn inject_file_functions<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let fs_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 fs 对象失败: {}", e))?;

    let read_text_file_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, path: String| -> Result<String, rquickjs::Error> {
            std::fs::read_to_string(&path).map_err(|e| rquickjs::Error::new_from_js_message("readTextFile", "Error", format!("读取失败: {}", e)))
        },
    ).map_err(|e| format!("创建 readTextFile 函数失败: {}", e))?;
    fs_obj.set("readTextFile", read_text_file_fn).map_err(|e| format!("设置 readTextFile 失败: {}", e))?;

    let write_text_file_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, path: String, content: String| -> Result<(), rquickjs::Error> {
            std::fs::write(&path, &content).map_err(|e| rquickjs::Error::new_from_js_message("writeTextFile", "Error", format!("写入失败: {}", e)))
        },
    ).map_err(|e| format!("创建 writeTextFile 函数失败: {}", e))?;
    fs_obj.set("writeTextFile", write_text_file_fn).map_err(|e| format!("设置 writeTextFile 失败: {}", e))?;

    let exists_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, path: String| -> bool {
            std::path::Path::new(&path).exists()
        },
    ).map_err(|e| format!("创建 exists 函数失败: {}", e))?;
    fs_obj.set("exists", exists_fn).map_err(|e| format!("设置 exists 失败: {}", e))?;

    let is_dir_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, path: String| -> bool {
            std::path::Path::new(&path).is_dir()
        },
    ).map_err(|e| format!("创建 isDir 函数失败: {}", e))?;
    fs_obj.set("isDir", is_dir_fn).map_err(|e| format!("设置 isDir 失败: {}", e))?;

    parent.set("fs", fs_obj).map_err(|e| format!("设置 fs 失败: {}", e))?;
    println!("[ApiBridge]   ✓ fs 模块注入成功");
    Ok(())
}

fn inject_plugin_callbacks<'js>(ctx: &Ctx<'js>, parent: &Object<'js>, instance_id: String) -> Result<(), String> {
    let instance_id_clone = instance_id.clone();
    let on_ready_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>| {
            if let Some(app) = get_app_handle() { let _ = app.emit("plugin-ready", instance_id_clone.clone()); }
            Ok::<(), rquickjs::Error>(())
        },
    ).map_err(|e| format!("创建 onPluginReady 函数失败: {}", e))?;
    parent.set("onPluginReady", on_ready_fn).map_err(|e| format!("设置 onPluginReady 失败: {}", e))?;

    let instance_id_clone2 = instance_id.clone();
    let on_out_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>| {
            if let Some(app) = get_app_handle() { let _ = app.emit("plugin-out", instance_id_clone2.clone()); }
            Ok::<(), rquickjs::Error>(())
        },
    ).map_err(|e| format!("创建 onPluginOut 函数失败: {}", e))?;
    parent.set("onPluginOut", on_out_fn).map_err(|e| format!("设置 onPluginOut 失败: {}", e))?;

    let instance_id_clone3 = instance_id;
    let register_feature_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>, _feature: Value| -> Result<(), rquickjs::Error> {
            if let Some(app) = get_app_handle() { let _ = app.emit("plugin-feature", instance_id_clone3.clone()); }
            Ok(())
        },
    ).map_err(|e| format!("创建 registerPluginFeature 函数失败: {}", e))?;
    parent.set("registerPluginFeature", register_feature_fn).map_err(|e| format!("设置 registerPluginFeature 失败: {}", e))?;

    println!("[ApiBridge]   ✓ 插件生命周期回调注入成功");
    Ok(())
}

fn inject_fetch_api<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    println!("[ApiBridge]   📡 正在注入 fetch API...");

    let fetch_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'js>, url: String, _options: Value| -> Result<Value<'js>, rquickjs::Error> {
            let url_clone = url.clone();
            let ctx_clone = _ctx.clone();

            let result = std::thread::Builder::new()
                .name("fetch-thread".to_string())
                .spawn(move || -> Result<(u16, String, std::collections::HashMap<String, String>, Option<String>), String> {
                    let req = ureq::get(&url_clone).timeout(std::time::Duration::from_secs(30));
                    match req.call() {
                        Ok(resp) => {
                            let mut headers = std::collections::HashMap::new();
                            for key in resp.headers_names() {
                                if let Some(val) = resp.header(&key) {
                                    headers.insert(key, val.to_string());
                                }
                            }
                            let body = resp.into_string().ok();
                            Ok((200, "OK".to_string(), headers, body))
                        }
                        Err(ureq::Error::Status(code, resp)) => {
                            let body = resp.into_string().ok();
                            Ok((code, "Error".to_string(), std::collections::HashMap::new(), body))
                        }
                        Err(e) => Err(format!("请求失败: {}", e))
                    }
                })
                .map_err(|e| rquickjs::Error::new_from_js_message("fetch", "Error", format!("创建线程失败: {}", e)))?
                .join()
                .map_err(|_| rquickjs::Error::new_from_js_message("fetch", "Error", "线程 panicked"))?
                .map_err(|e| rquickjs::Error::new_from_js_message("fetch", "Error", e))?;

            let response_obj = Object::new(ctx_clone.clone())?;
            response_obj.set("status", result.0 as i32)?;
            response_obj.set("statusText", result.1)?;
            response_obj.set("ok", result.0 >= 200 && result.0 < 300)?;

            let headers_obj = Object::new(ctx_clone.clone())?;
            for (k, v) in &result.2 {
                headers_obj.set(k.as_str(), v.as_str())?;
            }
            response_obj.set("headers", headers_obj)?;

            let body_str = result.3.unwrap_or_default();
            let body_clone = body_str.clone();

            let text_fn = Function::new(ctx_clone.clone(), move |_: Ctx<'js>| Ok::<String, rquickjs::Error>(body_clone.clone()));
            response_obj.set("text", text_fn)?;

            let json_body = body_str.clone();
            let ctx_for_json = ctx_clone.clone();
            let json_fn = Function::new(ctx_for_json.clone(), move |_: Ctx<'js>| -> Result<Value<'js>, rquickjs::Error> {
                let json_ctx = ctx_for_json.clone();
                match serde_json::from_str::<serde_json::Value>(&json_body) {
                    Ok(v) => {
                        let s = serde_json::to_string(&v).map_err(|e| rquickjs::Error::new_from_js_message("json", "Error", format!("序列化失败: {}", e)))?;
                        Ok(Value::from_string(rquickjs::String::from_str(json_ctx, &s)?))
                    }
                    Err(_) => Err(rquickjs::Error::new_from_js_message("json", "Error", "无效 JSON"))
                }
            });
            response_obj.set("json", json_fn)?;

            Ok(response_obj.into())
        },
    ).map_err(|e| format!("创建 fetch 函数失败: {}", e))?;
    parent.set("fetch", fetch_fn).map_err(|e| format!("设置 fetch 失败: {}", e))?;
    println!("[ApiBridge]   ✓ fetch API 注入成功");
    Ok(())
}

fn inject_dialog_api<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    println!("[ApiBridge]   📁 正在注入 dialog API...");

    let dialog_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 dialog 对象失败: {}", e))?;

    let show_open_dialog_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'js>, _options: Value| -> Result<Value<'js>, rquickjs::Error> {
            let c = _ctx.clone();
            #[cfg(target_os = "windows")]
            {
                let ps = r#"[void][System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms'); $dlg = New-Object System.Windows.Forms.OpenFileDialog; if($dlg.ShowDialog() -eq 'OK') { $dlg.FileName } else { $null }"#;
                let output = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", ps])
                    .output().map_err(|e| rquickjs::Error::new_from_js_message("showOpenDialog", "Error", format!("执行失败: {}", e)))?;
                let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if result.is_empty() || result == "null" {
                    Ok(Value::new_null(c))
                } else {
                    Ok(Value::from_string(rquickjs::String::from_str(c, &result)?))
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                Ok(Value::new_null(c))
            }
        },
    ).map_err(|e| format!("创建 showOpenDialog 函数失败: {}", e))?;
    dialog_obj.set("showOpenDialog", show_open_dialog_fn).map_err(|e| format!("设置 showOpenDialog 失败: {}", e))?;

    let show_save_dialog_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'js>, _options: Value| -> Result<Value<'js>, rquickjs::Error> {
            let c = _ctx.clone();
            #[cfg(target_os = "windows")]
            {
                let ps = r#"[void][System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms'); $dlg = New-Object System.Windows.Forms.SaveFileDialog; if($dlg.ShowDialog() -eq 'OK') { $dlg.FileName } else { $null }"#;
                let output = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", ps])
                    .output().map_err(|e| rquickjs::Error::new_from_js_message("showSaveDialog", "Error", format!("执行失败: {}", e)))?;
                let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if result.is_empty() || result == "null" {
                    Ok(Value::new_null(c))
                } else {
                    Ok(Value::from_string(rquickjs::String::from_str(c, &result)?))
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                Ok(Value::new_null(c))
            }
        },
    ).map_err(|e| format!("创建 showSaveDialog 函数失败: {}", e))?;
    dialog_obj.set("showSaveDialog", show_save_dialog_fn).map_err(|e| format!("设置 showSaveDialog 失败: {}", e))?;

    let show_message_box_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'js>, _options: Value| -> Result<Value<'js>, rquickjs::Error> {
            Ok(Value::from_string(rquickjs::String::from_str(_ctx, "OK")?))
        },
    ).map_err(|e| format!("创建 showMessageBox 函数失败: {}", e))?;
    dialog_obj.set("showMessageBox", show_message_box_fn).map_err(|e| format!("设置 showMessageBox 失败: {}", e))?;

    parent.set("dialog", dialog_obj).map_err(|e| format!("设置 dialog 失败: {}", e))?;
    println!("[ApiBridge]   ✓ dialog API 注入成功");
    Ok(())
}

fn inject_process_api<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    println!("[ApiBridge]   ⚡ 正在注入 process API...");

    let process_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 process 对象失败: {}", e))?;

    let exec_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'js>, command: String, _options: Value| -> Result<Value<'js>, rquickjs::Error> {
            #[cfg(target_os = "windows")]
            let output = std::process::Command::new("cmd").args(["/C", &command]).output();
            #[cfg(not(target_os = "windows"))]
            let output = std::process::Command::new("sh").args(["-c", &command]).output();

            match output {
                Ok(output) => {
                    let obj = Object::new(_ctx.clone())?;
                    obj.set("stdout", String::from_utf8_lossy(&output.stdout).to_string())?;
                    obj.set("stderr", String::from_utf8_lossy(&output.stderr).to_string())?;
                    let code = output.status.code().unwrap_or(-1);
                    obj.set("exitCode", code)?;
                    obj.set("code", code)?;
                    Ok(obj.into())
                }
                Err(e) => Err(rquickjs::Error::new_from_js_message("exec", "Error", format!("执行失败: {}", e)))
            }
        },
    ).map_err(|e| format!("创建 exec 函数失败: {}", e))?;
    process_obj.set("exec", exec_fn).map_err(|e| format!("设置 exec 失败: {}", e))?;

    let get_native_id_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>| -> Result<String, rquickjs::Error> {
            Ok(format!("corelia-{}", hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_else(|_| "unknown".to_string())))
        },
    ).map_err(|e| format!("创建 getNativeId 函数失败: {}", e))?;
    process_obj.set("getNativeId", get_native_id_fn).map_err(|e| format!("设置 getNativeId 失败: {}", e))?;

    let get_app_name_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>| -> Result<String, rquickjs::Error> { Ok("Corelia".to_string()) },
    ).map_err(|e| format!("创建 getAppName 函数失败: {}", e))?;
    process_obj.set("getAppName", get_app_name_fn).map_err(|e| format!("设置 getAppName 失败: {}", e))?;

    let get_app_version_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>| -> Result<String, rquickjs::Error> { Ok(env!("CARGO_PKG_VERSION").to_string()) },
    ).map_err(|e| format!("创建 getAppVersion 函数失败: {}", e))?;
    process_obj.set("getAppVersion", get_app_version_fn).map_err(|e| format!("设置 getAppVersion 失败: {}", e))?;

    parent.set("process", process_obj).map_err(|e| format!("设置 process 失败: {}", e))?;
    println!("[ApiBridge]   ✓ process API 注入成功");
    Ok(())
}

fn inject_context_api<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    println!("[ApiBridge]   📋 正在注入 getContext API...");

    let get_context_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'js>| -> Result<Value<'js>, rquickjs::Error> {
            let ctx_clone = _ctx.clone();
            let ctx_obj = Object::new(ctx_clone.clone())?;
            ctx_obj.set("code", "")?;
            ctx_obj.set("type", "none")?;
            ctx_obj.set("payload", Value::new_null(ctx_clone))?;
            ctx_obj.set("refresh", false)?;
            Ok(ctx_obj.into())
        },
    ).map_err(|e| format!("创建 getContext 函数失败: {}", e))?;
    parent.set("getContext", get_context_fn).map_err(|e| format!("设置 getContext 失败: {}", e))?;

    let set_context_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>, _payload: Value| -> Result<(), rquickjs::Error> { Ok(()) },
    ).map_err(|e| format!("创建 setContext 函数失败: {}", e))?;
    parent.set("setContext", set_context_fn).map_err(|e| format!("设置 setContext 失败: {}", e))?;

    let set_expend_height_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>, height: i32| -> Result<(), rquickjs::Error> {
            let app = require_app!("setExpendHeight");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: 600, height: height as u32 }));
            }
            Ok(())
        },
    ).map_err(|e| format!("创建 setExpendHeight 函数失败: {}", e))?;
    parent.set("setExpendHeight", set_expend_height_fn).map_err(|e| format!("设置 setExpendHeight 失败: {}", e))?;

    let out_plugin_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>| -> Result<(), rquickjs::Error> {
            let app = require_app!("outPlugin");
            crate::services::WindowService::hide(&app).map_err(|e| rquickjs::Error::new_from_js_message("outPlugin", "Error", e))
        },
    ).map_err(|e| format!("创建 outPlugin 函数失败: {}", e))?;
    parent.set("outPlugin", out_plugin_fn).map_err(|e| format!("设置 outPlugin 失败: {}", e))?;

    println!("[ApiBridge]   ✓ getContext API 注入成功");
    Ok(())
}

fn inject_wasm_api<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    println!("[ApiBridge]   🧩 正在注入 WASM API...");

    let wasm_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 wasm 对象失败: {}", e))?;

    // __wasm_call(funcName, argsJson) — 发起 WASM 函数调用
    // 返回 requestId，插件代码需通过 __wasm_get_result(requestId) 轮询获取结果
    let call_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>, func_name: String, args_json: String| -> Result<String, rquickjs::Error> {
            let app = require_app!("__wasm_call");

            // 最小化锁范围：仅检查函数是否存在
            {
                let bridge = app.state::<Mutex<WasmBridge>>();
                let bridge_ref = bridge.lock().map_err(|e| rquickjs::Error::new_from_js_message(
                    "__wasm_call", "Error", format!("获取 WasmBridge 锁失败: {}", e)
                ))?;

                if !bridge_ref.has_function(&func_name) {
                    return Err(rquickjs::Error::new_from_js_message(
                        "__wasm_call", "Error", format!("WASM 函数未注册: {}", func_name)
                    ));
                }
            } // 锁在此释放，避免死锁

            // 生成唯一 request_id 并发送事件（不持锁）
            let request_id = crate::plugins::wasm_bridge::generate_request_id();

            let payload = serde_json::json!({
                "requestId": request_id,
                "function": func_name,
                "args": args_json,
            });

            app.emit("wasm-call", payload)
                .map_err(|e| rquickjs::Error::new_from_js_message(
                    "__wasm_call", "Error", format!("发送事件失败: {}", e)
                ))?;

            Ok(request_id)
        },
    ).map_err(|e| format!("创建 __wasm_call 函数失败: {}", e))?;
    wasm_obj.set("__wasm_call", call_fn).map_err(|e| format!("设置 __wasm_call 失败: {}", e))?;

    // __wasm_get_result(requestId) — 轮询获取 WASM 调用结果
    // 返回 JSON 字符串表示结果，null 表示结果尚未就绪
    let get_result_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>, request_id: String| -> Result<Option<String>, rquickjs::Error> {
            let app = require_app!("__wasm_get_result");
            let bridge = app.state::<Mutex<WasmBridge>>();
            let mut bridge_ref = bridge.lock().map_err(|e| rquickjs::Error::new_from_js_message(
                "__wasm_get_result", "Error", format!("获取 WasmBridge 锁失败: {}", e)
            ))?;

            match bridge_ref.get_call_result(&request_id) {
                Some(entry) => {
                    serde_json::to_string(&entry)
                        .map(Some)
                        .map_err(|e| rquickjs::Error::new_from_js_message(
                            "__wasm_get_result", "Error", format!("序列化结果失败: {}", e)
                        ))
                }
                None => Ok(None),
            }
        },
    ).map_err(|e| format!("创建 __wasm_get_result 函数失败: {}", e))?;
    wasm_obj.set("__wasm_get_result", get_result_fn).map_err(|e| format!("设置 __wasm_get_result 失败: {}", e))?;

    // __wasm_available() — 获取所有已注册的 WASM 函数名列表
    let available_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>| -> Result<Vec<String>, rquickjs::Error> {
            let app = require_app!("__wasm_available");
            let bridge = app.state::<Mutex<WasmBridge>>();
            let bridge_ref = bridge.lock().map_err(|e| rquickjs::Error::new_from_js_message(
                "__wasm_available", "Error", format!("获取 WasmBridge 锁失败: {}", e)
            ))?;

            Ok(bridge_ref.list_functions().into_iter().map(|f| f.name.clone()).collect())
        },
    ).map_err(|e| format!("创建 __wasm_available 函数失败: {}", e))?;
    wasm_obj.set("__wasm_available", available_fn).map_err(|e| format!("设置 __wasm_available 失败: {}", e))?;

    // __wasm_has(funcName) — 检查指定 WASM 函数是否可用
    let has_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>, func_name: String| -> Result<bool, rquickjs::Error> {
            let app = require_app!("__wasm_has");
            let bridge = app.state::<Mutex<WasmBridge>>();
            let bridge_ref = bridge.lock().map_err(|e| rquickjs::Error::new_from_js_message(
                "__wasm_has", "Error", format!("获取 WasmBridge 锁失败: {}", e)
            ))?;

            Ok(bridge_ref.has_function(&func_name))
        },
    ).map_err(|e| format!("创建 __wasm_has 函数失败: {}", e))?;
    wasm_obj.set("__wasm_has", has_fn).map_err(|e| format!("设置 __wasm_has 失败: {}", e))?;

    parent.set("wasm", wasm_obj).map_err(|e| format!("设置 wasm 失败: {}", e))?;
    println!("[ApiBridge]   ✓ wasm API 注入成功");
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

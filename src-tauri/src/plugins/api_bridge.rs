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
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set_app_handle(app: AppHandle) {
    let _ = APP_HANDLE.set(app);
}

fn get_app_handle() -> Result<AppHandle, String> {
    APP_HANDLE.get().cloned().ok_or_else(|| "AppHandle not initialized".to_string())
}

/// API 桥接层：将 Rust 函数注入到 QuickJS 上下文
pub struct ApiBridge {
    app_handle: Option<AppHandle>,
}

impl ApiBridge {
    pub fn new(app_handle: Option<AppHandle>) -> Self {
        Self { app_handle }
    }

    /// 注入完整的 utools 对象到 QuickJS 全局上下文
    pub fn inject_utools(ctx: &Ctx) -> Result<(), String> {
        println!("[ApiBridge] 开始注入 window.utools API...");

        let globals = ctx.globals();
        let utools_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 utools 对象失败: {}", e))?;

        inject_db_storage(ctx, &utools_obj)?;
        inject_clipboard(ctx, &utools_obj)?;
        inject_shell(ctx, &utools_obj)?;
        inject_window_functions(ctx, &utools_obj)?;

        globals.set("utools", utools_obj).map_err(|e| format!("设置全局变量失败: {}", e))?;

        println!("[ApiBridge] window.utools API 注入成功 ✓");
        Ok(())
    }
}

fn inject_db_storage<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let db_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 dbStorage 对象失败: {}", e))?;

    #[allow(unused_variables)]
    let get_item_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, key: String| -> Result<Option<String>, rquickjs::Error> {
            println!("[utools.dbStorage] getItem: {}", key);
            let app = match get_app_handle() {
                Ok(h) => h,
                Err(e) => return Err(rquickjs::Error::new_from_js_message("getItem", "Error", e)),
            };
            match app.store("dbStorage.json") {
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

    #[allow(unused_variables)]
    let set_item_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, key: String, value: String| -> Result<(), rquickjs::Error> {
            println!("[utools.dbStorage] setItem: {} = {:?}", key, value);
            let app = match get_app_handle() {
                Ok(h) => h,
                Err(e) => return Err(rquickjs::Error::new_from_js_message("setItem", "Error", e)),
            };
            match app.store("dbStorage.json") {
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

    #[allow(unused_variables)]
    let remove_item_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, key: String| -> Result<(), rquickjs::Error> {
            println!("[utools.dbStorage] removeItem: {}", key);
            let app = match get_app_handle() {
                Ok(h) => h,
                Err(e) => return Err(rquickjs::Error::new_from_js_message("removeItem", "Error", e)),
            };
            match app.store("dbStorage.json") {
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

    #[allow(unused_variables)]
    let get_all_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, ()| -> Result<std::collections::HashMap<String, String>, rquickjs::Error> {
            println!("[utools.dbStorage] getAll");
            let app = match get_app_handle() {
                Ok(h) => h,
                Err(e) => return Err(rquickjs::Error::new_from_js_message("getAll", "Error", e)),
            };
            let store = match app.store("dbStorage.json") {
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
    println!("[ApiBridge]   ✓ dbStorage 模块注入成功");
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
            let app = match get_app_handle() {
                Ok(h) => h,
                Err(e) => return Err(rquickjs::Error::new_from_js_message("hideMainWindow", "Error", e)),
            };
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
            let app = match get_app_handle() {
                Ok(h) => h,
                Err(e) => return Err(rquickjs::Error::new_from_js_message("showMainWindow", "Error", e)),
            };
            crate::services::WindowService::show(&app)
                .map_err(|e| rquickjs::Error::new_from_js_message("showMainWindow", "Error", e))
        },
    ).map_err(|e| format!("创建 showMainWindow 函数失败: {}", e))?;
    parent.set("showMainWindow", show_fn).map_err(|e| format!("设置 showMainWindow 失败: {}", e))?;

    println!("[ApiBridge]   ✓ 窗口控制函数注入成功");
    Ok(())
}

#[tauri::command]
pub async fn inject_apis_to_vm(
    runtime: tauri::State<'_, Mutex<QuickJSRuntime>>,
    vm_id: String,
) -> Result<(), String> {
    println!("[Command] inject_apis_to_vm: 注入 API 到 VM {}", vm_id);

    let rt = runtime.lock().map_err(|e| format!("获取运行时锁失败: {}", e))?;

    rt.with_context(&vm_id, |ctx| ApiBridge::inject_utools(&ctx))
}

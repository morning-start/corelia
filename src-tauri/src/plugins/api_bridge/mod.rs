pub mod shared;
mod storage;
mod clipboard;
mod shell;
mod window;
mod path;
mod notification;
mod fs;
mod callbacks;
mod fetch;
mod dialog;
mod process;
mod context;
mod wasm;

pub use shared::{set_app_handle, FetchResult};
pub(crate) use shared::{get_app_handle, require_app};

use rquickjs::{Ctx, Object};
use crate::plugins::quickjs_runtime::QuickJSRuntime;
use std::sync::Mutex;

pub struct ApiBridge;

impl ApiBridge {
    pub fn inject_utools(ctx: &Ctx, instance_id: &str) -> Result<(), String> {
        println!("[ApiBridge] 开始注入 window.utools API (插件: {})...", instance_id);

        let globals = ctx.globals();
        let utools_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 utools 对象失败: {}", e))?;

        storage::inject_db_storage(ctx, &utools_obj, instance_id.to_string())?;
        clipboard::inject_clipboard(ctx, &utools_obj)?;
        shell::inject_shell(ctx, &utools_obj)?;
        window::inject_window_functions(ctx, &utools_obj)?;
        path::inject_path_functions(ctx, &utools_obj)?;
        notification::inject_notification_functions(ctx, &utools_obj)?;
        fs::inject_file_functions(ctx, &utools_obj)?;
        callbacks::inject_plugin_callbacks(ctx, &utools_obj, instance_id.to_string())?;
        fetch::inject_fetch_api(ctx, &utools_obj)?;
        dialog::inject_dialog_api(ctx, &utools_obj)?;
        process::inject_process_api(ctx, &utools_obj)?;
        context::inject_context_api(ctx, &utools_obj)?;
        wasm::inject_wasm_api(ctx, &utools_obj)?;

        globals.set("utools", utools_obj).map_err(|e| format!("设置全局变量失败: {}", e))?;

        println!("[ApiBridge] window.utools API 注入成功 ✓");
        Ok(())
    }
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
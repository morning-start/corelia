mod db_storage;
mod clipboard;
mod shell;
mod window;
mod path;
mod notification;
mod fs;
mod fetch;
mod dialog;
mod process;
mod context;
mod wasm;

use rquickjs::{Ctx, Object};
use std::sync::OnceLock;
use tauri::AppHandle;

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

pub(crate) use require_app;

pub struct ApiBridge;

impl ApiBridge {
    pub fn inject_utools(ctx: &Ctx, instance_id: &str) -> Result<(), String> {
        println!("[ApiBridge] 开始注入 window.utools API (插件: {})...", instance_id);

        let globals = ctx.globals();
        let utools_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 utools 对象失败: {}", e))?;

        db_storage::inject(ctx, &utools_obj, instance_id.to_string())?;
        clipboard::inject(ctx, &utools_obj)?;
        shell::inject(ctx, &utools_obj)?;
        window::inject(ctx, &utools_obj)?;
        path::inject(ctx, &utools_obj)?;
        notification::inject(ctx, &utools_obj)?;
        fs::inject(ctx, &utools_obj)?;
        fetch::inject(ctx, &utools_obj)?;
        dialog::inject(ctx, &utools_obj)?;
        process::inject(ctx, &utools_obj)?;
        context::inject(ctx, &utools_obj, instance_id.to_string())?;
        wasm::inject(ctx, &utools_obj)?;

        globals.set("utools", utools_obj).map_err(|e| format!("设置全局变量失败: {}", e))?;

        println!("[ApiBridge] window.utools API 注入成功 ✓");
        Ok(())
    }
}

#[tauri::command]
pub async fn inject_apis_to_vm(
    runtime: tauri::State<'_, std::sync::Mutex<crate::plugins::quickjs_runtime::QuickJSRuntime>>,
    vm_id: String,
    plugin_id: String,
) -> Result<(), String> {
    println!("[Command] inject_apis_to_vm: 注入 API 到 VM {} (插件: {})", vm_id, plugin_id);

    let rt = runtime.lock().map_err(|e| format!("获取运行时锁失败: {}", e))?;

    rt.with_context(&vm_id, |ctx| ApiBridge::inject_utools(&ctx, &plugin_id))
}

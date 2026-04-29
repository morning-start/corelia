use rquickjs::{Ctx, Object, Function, Value};
use super::require_app;

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>, instance_id: String) -> Result<(), String> {
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

    let instance_id_clone = instance_id.clone();
    let on_ready_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>| {
            if let Some(app) = super::get_app_handle() { let _ = app.emit("plugin-ready", instance_id_clone.clone()); }
            Ok::<(), rquickjs::Error>(())
        },
    ).map_err(|e| format!("创建 onPluginReady 函数失败: {}", e))?;
    parent.set("onPluginReady", on_ready_fn).map_err(|e| format!("设置 onPluginReady 失败: {}", e))?;

    let instance_id_clone2 = instance_id.clone();
    let on_out_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>| {
            if let Some(app) = super::get_app_handle() { let _ = app.emit("plugin-out", instance_id_clone2.clone()); }
            Ok::<(), rquickjs::Error>(())
        },
    ).map_err(|e| format!("创建 onPluginOut 函数失败: {}", e))?;
    parent.set("onPluginOut", on_out_fn).map_err(|e| format!("设置 onPluginOut 失败: {}", e))?;

    let instance_id_clone3 = instance_id;
    let register_feature_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>, _feature: Value| -> Result<(), rquickjs::Error> {
            if let Some(app) = super::get_app_handle() { let _ = app.emit("plugin-feature", instance_id_clone3.clone()); }
            Ok(())
        },
    ).map_err(|e| format!("创建 registerPluginFeature 函数失败: {}", e))?;
    parent.set("registerPluginFeature", register_feature_fn).map_err(|e| format!("设置 registerPluginFeature 失败: {}", e))?;

    println!("[ApiBridge]   ✓ getContext API 注入成功");
    Ok(())
}

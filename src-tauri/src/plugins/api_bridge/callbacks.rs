use rquickjs::{Ctx, Object, Function, Value};
use tauri::Emitter;
use crate::plugins::api_bridge::get_app_handle;

pub fn inject_plugin_callbacks<'js>(ctx: &Ctx<'js>, parent: &Object<'js>, instance_id: String) -> Result<(), String> {
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
use rquickjs::{Ctx, Object, Function};
use super::require_app;

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
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

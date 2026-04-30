use rquickjs::{Ctx, Object, Function};
use crate::plugins::api_bridge::get_app_handle;

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let show_notification_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, title: String, body: String| -> Result<(), rquickjs::Error> {
            let app = match get_app_handle() {
                Some(h) => h,
                None => return Err(rquickjs::Error::new_from_js_message(
                    "showNotification", "Error", "AppHandle not initialized"
                )),
            };

            tauri_plugin_notification::NotificationBuilder::new()
                .title(&title)
                .body(&body)
                .show(&app)
                .map_err(|e| rquickjs::Error::new_from_js_message(
                    "showNotification", "Error", format!("发送通知失败: {}", e)
                ))
        },
    ).map_err(|e| format!("创建 showNotification 函数失败: {}", e))?;
    parent.set("showNotification", show_notification_fn).map_err(|e| format!("设置 showNotification 失败: {}", e))?;
    println!("[ApiBridge]   ✓ showNotification 模块注入成功（tauri-plugin-notification）");
    Ok(())
}

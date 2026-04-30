use std::sync::Mutex;
use rquickjs::{Ctx, Object, Function};
use once_cell::sync::Lazy;

static CLIPBOARD: Lazy<Mutex<arboard::Clipboard>> = Lazy::new(|| {
    Mutex::new(arboard::Clipboard::new().expect("无法初始化剪贴板"))
});

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let clip_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 clipboard 对象失败: {}", e))?;

    let read_text_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, ()| -> Result<String, rquickjs::Error> {
            let mut clipboard = CLIPBOARD.lock().map_err(|e| {
                rquickjs::Error::new_from_js_message("readText", "Error", format!("剪贴板锁获取失败: {}", e))
            })?;
            clipboard.get_text()
                .map_err(|e| rquickjs::Error::new_from_js_message("readText", "Error", format!("读取剪贴板失败: {}", e)))
        },
    ).map_err(|e| format!("创建 readText 函数失败: {}", e))?;
    clip_obj.set("readText", read_text_fn).map_err(|e| format!("设置 readText 失败: {}", e))?;

    let write_text_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, text: String| -> Result<(), rquickjs::Error> {
            let mut clipboard = CLIPBOARD.lock().map_err(|e| {
                rquickjs::Error::new_from_js_message("writeText", "Error", format!("剪贴板锁获取失败: {}", e))
            })?;
            clipboard.set_text(&text)
                .map_err(|e| rquickjs::Error::new_from_js_message("writeText", "Error", format!("写入剪贴板失败: {}", e)))
        },
    ).map_err(|e| format!("创建 writeText 函数失败: {}", e))?;
    clip_obj.set("writeText", write_text_fn).map_err(|e| format!("设置 writeText 失败: {}", e))?;

    let copy_text_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, text: String| -> Result<(), rquickjs::Error> {
            let mut clipboard = CLIPBOARD.lock().map_err(|e| {
                rquickjs::Error::new_from_js_message("copyText", "Error", format!("剪贴板锁获取失败: {}", e))
            })?;
            clipboard.set_text(&text)
                .map_err(|e| rquickjs::Error::new_from_js_message("copyText", "Error", format!("复制失败: {}", e)))
        },
    ).map_err(|e| format!("创建 copyText 函数失败: {}", e))?;
    clip_obj.set("copyText", copy_text_fn).map_err(|e| format!("设置 copyText 失败: {}", e))?;

    parent.set("clipboard", clip_obj).map_err(|e| format!("设置 clipboard 失败: {}", e))?;
    println!("[ApiBridge]   ✓ clipboard 模块注入成功（全局实例复用）");
    Ok(())
}

use rquickjs::{Ctx, Object, Function};

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
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

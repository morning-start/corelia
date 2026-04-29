use rquickjs::{Ctx, Object, Function};

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
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

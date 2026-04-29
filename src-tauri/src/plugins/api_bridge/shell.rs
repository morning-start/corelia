use rquickjs::{Ctx, Object, Function};

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let shell_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 shell 对象失败: {}", e))?;

    let open_path_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, path: String| -> Result<(), rquickjs::Error> {
            open::that(&path).map_err(|e| rquickjs::Error::new_from_js_message("openPath", "Error", format!("打开路径失败: {}", e)))
        },
    ).map_err(|e| format!("创建 openPath 函数失败: {}", e))?;
    shell_obj.set("openPath", open_path_fn).map_err(|e| format!("设置 openPath 失败: {}", e))?;

    let open_external_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, url: String| -> Result<(), rquickjs::Error> {
            open::that(&url).map_err(|e| rquickjs::Error::new_from_js_message("openExternal", "Error", format!("打开外部链接失败: {}", e)))
        },
    ).map_err(|e| format!("创建 openExternal 函数失败: {}", e))?;
    shell_obj.set("openExternal", open_external_fn).map_err(|e| format!("设置 openExternal 失败: {}", e))?;

    let show_in_folder_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, path: String| -> Result<(), rquickjs::Error> {
            #[cfg(target_os = "windows")]
            let result = open::that(format!("explorer /select,{}", path));
            #[cfg(not(target_os = "windows"))]
            let result = open::that(&path);
            result.map_err(|e| rquickjs::Error::new_from_js_message("showItemInFolder", "Error", format!("显示失败: {}", e)))
        },
    ).map_err(|e| format!("创建 showItemInFolder 函数失败: {}", e))?;
    shell_obj.set("showItemInFolder", show_in_folder_fn).map_err(|e| format!("设置 showItemInFolder 失败: {}", e))?;

    let beep_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, ()| -> Result<(), rquickjs::Error> {
            #[cfg(target_os = "windows")]
            { let _ = std::process::Command::new("powershell").args(["-NoProfile", "-Command", "[Console]::Beep(800,200)"]).spawn(); }
            Ok(())
        },
    ).map_err(|e| format!("创建 beep 函数失败: {}", e))?;
    shell_obj.set("beep", beep_fn).map_err(|e| format!("设置 beep 失败: {}", e))?;

    parent.set("shell", shell_obj).map_err(|e| format!("设置 shell 失败: {}", e))?;
    println!("[ApiBridge]   ✓ shell 模块注入成功");
    Ok(())
}

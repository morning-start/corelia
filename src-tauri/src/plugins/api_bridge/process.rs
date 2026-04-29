use rquickjs::{Ctx, Object, Function, Value};

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    println!("[ApiBridge]   ⚡ 正在注入 process API...");

    let process_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 process 对象失败: {}", e))?;

    let exec_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'js>, command: String, _options: Value| -> Result<Value<'js>, rquickjs::Error> {
            #[cfg(target_os = "windows")]
            let output = std::process::Command::new("cmd").args(["/C", &command]).output();
            #[cfg(not(target_os = "windows"))]
            let output = std::process::Command::new("sh").args(["-c", &command]).output();

            match output {
                Ok(output) => {
                    let obj = Object::new(_ctx.clone())?;
                    obj.set("stdout", String::from_utf8_lossy(&output.stdout).to_string())?;
                    obj.set("stderr", String::from_utf8_lossy(&output.stderr).to_string())?;
                    let code = output.status.code().unwrap_or(-1);
                    obj.set("exitCode", code)?;
                    obj.set("code", code)?;
                    Ok(obj.into())
                }
                Err(e) => Err(rquickjs::Error::new_from_js_message("exec", "Error", format!("执行失败: {}", e)))
            }
        },
    ).map_err(|e| format!("创建 exec 函数失败: {}", e))?;
    process_obj.set("exec", exec_fn).map_err(|e| format!("设置 exec 失败: {}", e))?;

    let get_native_id_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>| -> Result<String, rquickjs::Error> {
            Ok(format!("corelia-{}", hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_else(|_| "unknown".to_string())))
        },
    ).map_err(|e| format!("创建 getNativeId 函数失败: {}", e))?;
    process_obj.set("getNativeId", get_native_id_fn).map_err(|e| format!("设置 getNativeId 失败: {}", e))?;

    let get_app_name_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>| -> Result<String, rquickjs::Error> { Ok("Corelia".to_string()) },
    ).map_err(|e| format!("创建 getAppName 函数失败: {}", e))?;
    process_obj.set("getAppName", get_app_name_fn).map_err(|e| format!("设置 getAppName 失败: {}", e))?;

    let get_app_version_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>| -> Result<String, rquickjs::Error> { Ok(env!("CARGO_PKG_VERSION").to_string()) },
    ).map_err(|e| format!("创建 getAppVersion 函数失败: {}", e))?;
    process_obj.set("getAppVersion", get_app_version_fn).map_err(|e| format!("设置 getAppVersion 失败: {}", e))?;

    parent.set("process", process_obj).map_err(|e| format!("设置 process 失败: {}", e))?;
    println!("[ApiBridge]   ✓ process API 注入成功");
    Ok(())
}

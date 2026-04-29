use rquickjs::{Ctx, Object, Function, Value};

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    println!("[ApiBridge]   📁 正在注入 dialog API...");

    let dialog_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 dialog 对象失败: {}", e))?;

    let show_open_dialog_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'js>, _options: Value| -> Result<Value<'js>, rquickjs::Error> {
            let c = _ctx.clone();
            #[cfg(target_os = "windows")]
            {
                let ps = r#"[void][System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms'); $dlg = New-Object System.Windows.Forms.OpenFileDialog; if($dlg.ShowDialog() -eq 'OK') { $dlg.FileName } else { $null }"#;
                let output = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", ps])
                    .output().map_err(|e| rquickjs::Error::new_from_js_message("showOpenDialog", "Error", format!("执行失败: {}", e)))?;
                let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if result.is_empty() || result == "null" {
                    Ok(Value::new_null(c))
                } else {
                    Ok(Value::from_string(rquickjs::String::from_str(c, &result)?))
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                Ok(Value::new_null(c))
            }
        },
    ).map_err(|e| format!("创建 showOpenDialog 函数失败: {}", e))?;
    dialog_obj.set("showOpenDialog", show_open_dialog_fn).map_err(|e| format!("设置 showOpenDialog 失败: {}", e))?;

    let show_save_dialog_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'js>, _options: Value| -> Result<Value<'js>, rquickjs::Error> {
            let c = _ctx.clone();
            #[cfg(target_os = "windows")]
            {
                let ps = r#"[void][System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms'); $dlg = New-Object System.Windows.Forms.SaveFileDialog; if($dlg.ShowDialog() -eq 'OK') { $dlg.FileName } else { $null }"#;
                let output = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", ps])
                    .output().map_err(|e| rquickjs::Error::new_from_js_message("showSaveDialog", "Error", format!("执行失败: {}", e)))?;
                let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if result.is_empty() || result == "null" {
                    Ok(Value::new_null(c))
                } else {
                    Ok(Value::from_string(rquickjs::String::from_str(c, &result)?))
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                Ok(Value::new_null(c))
            }
        },
    ).map_err(|e| format!("创建 showSaveDialog 函数失败: {}", e))?;
    dialog_obj.set("showSaveDialog", show_save_dialog_fn).map_err(|e| format!("设置 showSaveDialog 失败: {}", e))?;

    let show_message_box_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'js>, _options: Value| -> Result<Value<'js>, rquickjs::Error> {
            Ok(Value::from_string(rquickjs::String::from_str(_ctx, "OK")?))
        },
    ).map_err(|e| format!("创建 showMessageBox 函数失败: {}", e))?;
    dialog_obj.set("showMessageBox", show_message_box_fn).map_err(|e| format!("设置 showMessageBox 失败: {}", e))?;

    parent.set("dialog", dialog_obj).map_err(|e| format!("设置 dialog 失败: {}", e))?;
    println!("[ApiBridge]   ✓ dialog API 注入成功");
    Ok(())
}

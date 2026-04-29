use rquickjs::{Ctx, Object, Function};

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let show_notification_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, title: String, body: String| -> Result<(), rquickjs::Error> {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", &format!("[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null; $xml = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); $xml.GetElementsByTagName('text')[0].AppendChild($xml.CreateTextNode('{}')) | Out-Null; $xml.GetElementsByTagName('text')[1].AppendChild($xml.CreateTextNode('{}')) | Out-Null; $toast = [Windows.UI.Notifications.ToastNotification]::new($xml); [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Corelia').Show($toast)", title.replace('\'', "''"), body.replace('\'', "''"))])
                    .spawn();
            }
            Ok(())
        },
    ).map_err(|e| format!("创建 showNotification 函数失败: {}", e))?;
    parent.set("showNotification", show_notification_fn).map_err(|e| format!("设置 showNotification 失败: {}", e))?;
    println!("[ApiBridge]   ✓ showNotification 模块注入成功");
    Ok(())
}

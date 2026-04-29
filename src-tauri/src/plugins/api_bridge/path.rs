use rquickjs::{Ctx, Object, Function};
use super::require_app;

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    let get_path_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'_>, name: String| -> Result<String, rquickjs::Error> {
            let app = require_app!("getPath");
            let path = app.path();

            let result_path = match name.to_lowercase().as_str() {
                "home" | "~" => path.home_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "desktop" => path.desktop_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "document" | "documents" => path.document_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "download" | "downloads" => path.download_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "music" => path.audio_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "picture" | "pictures" | "photo" | "photos" => path.picture_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "video" | "videos" => path.video_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "temp" | "tmp" => path.temp_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "appdata" => path.app_data_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "localappdata" | "appcache" => path.app_cache_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "userdata" => path.app_data_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "config" => path.app_config_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "log" | "logs" => path.app_log_dir().ok().map(|p| p.to_string_lossy().to_string()),
                "resource" | "resources" => std::env::current_exe().ok().map(|p| p.parent().unwrap_or(&p).join("resources").to_string_lossy().to_string()),
                "exe" | "exepath" => std::env::current_exe().ok().map(|p| p.to_string_lossy().to_string()),
                "plugin" | "pluginpath" => std::env::current_exe().ok().map(|p| p.parent().unwrap_or(&p).join("plugins").to_string_lossy().to_string()),
                "root" => Some("/".to_string()),
                "cwd" | "currentdir" => std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
                _ => None,
            };

            match result_path {
                Some(p) => Ok(p),
                None => Err(rquickjs::Error::new_from_js_message("getPath", "Error", format!("无法获取路径: {}", name)))
            }
        },
    ).map_err(|e| format!("创建 getPath 函数失败: {}", e))?;
    parent.set("getPath", get_path_fn).map_err(|e| format!("设置 getPath 失败: {}", e))?;
    println!("[ApiBridge]   ✓ getPath 模块注入成功");
    Ok(())
}

//! Shell 操作服务模块
//! 
//! 封装系统 Shell 操作

use std::process::Command;

/// Shell 服务（无状态）
pub struct ShellService;

impl ShellService {
    /// 打开 URL（默认浏览器）
    pub fn open_url(url: String) -> Result<(), String> {
        open::that(&url).map_err(|e| e.to_string())
    }
    
    /// 打开文件路径（资源管理器）
    pub fn open_path(path: String) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .arg(&path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(&path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(&path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    
    /// 打开应用程序
    pub fn open_app(app: String) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/c", "start", "", &app])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg("-a")
                .arg(&app)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "linux")]
        {
            Command::new(&app)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

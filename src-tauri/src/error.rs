//! 错误处理模块
//! 
//! 提供统一的错误类型和转换机制

use std::fmt;

/// 应用级错误类型
#[derive(Debug)]
pub enum AppError {
    Window(String),
    Shortcut(String),
    Store(String),
    Autostart(String),
    Js(String),
    Generic(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Window(msg) => write!(f, "窗口错误：{}", msg),
            AppError::Shortcut(msg) => write!(f, "快捷键错误：{}", msg),
            AppError::Store(msg) => write!(f, "存储错误：{}", msg),
            AppError::Autostart(msg) => write!(f, "启动项错误：{}", msg),
            AppError::Js(msg) => write!(f, "JS 错误：{}", msg),
            AppError::Generic(msg) => write!(f, "错误：{}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

impl From<tauri::Error> for AppError {
    fn from(err: tauri::Error) -> Self {
        AppError::Window(err.to_string())
    }
}

impl From<tauri_plugin_store::Error> for AppError {
    fn from(err: tauri_plugin_store::Error) -> Self {
        AppError::Store(err.to_string())
    }
}

impl From<image::ImageError> for AppError {
    fn from(err: image::ImageError) -> Self {
        AppError::Generic(format!("图片错误：{}", err))
    }
}

/// 结果类型别名
#[allow(dead_code)]
pub type AppResult<T> = Result<T, AppError>;

/// 错误处理辅助函数
pub mod helpers {
    use super::*;
    
    /// 记录错误到日志
    #[allow(dead_code)]
    pub fn log_error(error: &AppError, context: &str) {
        eprintln!("[{}] {}", context, error);
    }
    
    /// 包装错误并添加上下文
    #[allow(dead_code)]
    pub fn with_context<T, E>(result: Result<T, E>, context: &str) -> Result<T, AppError>
    where
        E: std::fmt::Display,
    {
        result.map_err(|e| AppError::Generic(format!("{}: {}", context, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = AppError::Window("窗口未找到".to_string());
        assert_eq!(format!("{}", err), "窗口错误：窗口未找到");
    }
    
    #[test]
    fn test_error_conversion() {
        let err = AppError::Store("存储失败".to_string());
        let msg: String = err.into();
        assert_eq!(msg, "存储错误：存储失败");
    }
}

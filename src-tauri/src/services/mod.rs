//! 服务模块
//! 
//! 提供应用级的业务服务

pub mod window_service;
pub mod clipboard_service;
pub mod shell_service;
pub mod store_service;
pub mod autostart_service;

pub use window_service::WindowService;
pub use clipboard_service::ClipboardService;
pub use shell_service::ShellService;
pub use store_service::StoreService;
pub use autostart_service::AutostartService;

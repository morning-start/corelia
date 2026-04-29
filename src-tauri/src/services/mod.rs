//! 服务模块
//! 
//! 提供应用级的业务服务
//! 
//! 职责划分：
//! - Commands 层：仅做参数验证和转发
//! - Services 层：包含业务逻辑
//! - 插件/工具层：底层功能实现

pub mod window_service;
pub mod clipboard_service;
pub mod shell_service;
pub mod store_service;
pub mod autostart_service;
pub mod config_service;
pub mod plugin_service;

pub use window_service::WindowService;
pub use clipboard_service::ClipboardService;
pub use shell_service::ShellService;
pub use store_service::StoreService;
pub use autostart_service::AutostartService;
pub use config_service::ConfigService;
pub use plugin_service::PluginService;

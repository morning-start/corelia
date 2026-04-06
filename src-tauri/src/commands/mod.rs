pub mod window;
pub mod shortcut;
pub mod clipboard;
pub mod shell;
pub mod store;
pub mod autostart;
pub mod config;

// 配置管理命令导出 (用于 Tauri invoke_handler)
#[allow(unused_imports)]
pub use config::{
    load_system_config, save_system_config,
    load_user_config, save_user_config, reset_user_config,
    load_app_config, save_app_config, clear_app_config,
};

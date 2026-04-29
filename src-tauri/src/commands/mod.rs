//! 命令模块
//!
//! 职责：
//! - 仅做参数验证和转发
//! - 不包含业务逻辑
//! - 业务逻辑委托给 Services 层
//!
//! 提供 Tauri Command 接口

pub mod window;
pub mod shortcut;
pub mod clipboard;
pub mod shell;
pub mod store;
pub mod autostart;
pub mod config;
pub mod plugin;

// 配置管理命令导出 (用于 Tauri invoke_handler)
#[allow(unused_imports)]
pub use config::{
    load_system_config, save_system_config,
    load_user_config, save_user_config, reset_user_config,
    load_app_config, save_app_config, clear_app_config,
};

// 插件管理命令导出
#[allow(unused_imports)]
pub use plugin::{
    scan_plugins, get_plugin_list, load_plugin, unload_plugin,
    find_plugins_by_prefix, cleanup_idle_plugins, get_plugin_health,
    plugin_execute,
};

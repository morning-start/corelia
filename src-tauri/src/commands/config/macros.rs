//! 配置命令宏

#[macro_export]
macro_rules! define_config_commands {
    ($config_type:ident, $load_fn:ident, $save_fn:ident) => {
        #[tauri::command]
        pub async fn $load_fn(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
            $config_type::$load_fn(&app)
        }

        #[tauri::command]
        pub async fn $save_fn(app: tauri::AppHandle, config: serde_json::Value) -> Result<(), String> {
            $config_type::$save_fn(&app, config)
        }
    };

    ($config_type:ident, $load_fn:ident, $save_fn:ident, $reset_fn:ident) => {
        #[tauri::command]
        pub async fn $load_fn(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
            $config_type::$load_fn(&app)
        }

        #[tauri::command]
        pub async fn $save_fn(app: tauri::AppHandle, config: serde_json::Value) -> Result<(), String> {
            $config_type::$save_fn(&app, config)
        }

        #[tauri::command]
        pub async fn $reset_fn(app: tauri::AppHandle) -> Result<(), String> {
            $config_type::$reset_fn(&app)
        }
    };
}

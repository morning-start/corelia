use serde_json::Value;

#[tauri::command]
pub fn save_settings(settings: Value) -> Result<(), String> {
    println!("Settings saved: {:?}", settings);
    Ok(())
}

#[tauri::command]
pub fn load_settings() -> Result<Value, String> {
    Ok(serde_json::json!({
        "theme": "dark",
        "shortcut": {
            "summon": "Alt+Space"
        },
        "behavior": {
            "autoHide": true,
            "autoHideDelay": 3000
        }
    }))
}

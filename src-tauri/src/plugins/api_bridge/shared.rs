use rquickjs::Value;
use std::sync::OnceLock;
use std::collections::HashMap;
use tauri::AppHandle;

pub type FetchResult = Result<(u16, String, HashMap<String, String>, Option<String>), String>;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set_app_handle(app: AppHandle) {
    let _ = APP_HANDLE.set(app);
}

pub(crate) fn get_app_handle() -> Option<AppHandle> {
    APP_HANDLE.get().cloned()
}

macro_rules! require_app {
    ($fn_name:expr) => {
        match $crate::plugins::api_bridge::get_app_handle() {
            Some(h) => h,
            None => return Err(rquickjs::Error::new_from_js_message(
                $fn_name, "Error", "AppHandle not initialized"
            )),
        }
    };
}
pub(crate) use require_app;

pub fn convert_to_serde(value: Value<'_>) -> serde_json::Value {
    if value.is_null() || value.is_undefined() { return serde_json::Value::Null; }
    if let Some(b) = value.as_bool() { return serde_json::Value::Bool(b); }
    if let Some(i) = value.as_int() { return serde_json::Value::Number(serde_json::Number::from(i)); }
    if let Some(f) = value.as_float() { return serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0))); }
    if let Some(s) = value.as_string() { return serde_json::Value::String(s.to_string().unwrap_or_default()); }
    if let Some(arr) = value.clone().into_array() {
        return serde_json::Value::Array(
            arr.iter()
                .filter_map(|r| r.ok())
                .map(|v| convert_to_serde(v))
                .collect()
        );
    }
    if let Some(obj) = value.into_object() {
        let mut map = serde_json::Map::new();
        for key in obj.keys::<String>().flatten() {
            if let Ok(val) = obj.get::<_, Value<'_>>(key.clone()) {
                map.insert(key, convert_to_serde(val));
            }
        }
        return serde_json::Value::Object(map);
    }
    serde_json::Value::Null
}
#![allow(static_mut_refs)]

mod commands;

use rquickjs::{Context, Runtime};

static mut JS_RUNTIME: Option<Runtime> = None;
static mut JS_CONTEXT: Option<Context> = None;

fn get_js_context() -> Result<&'static Context, String> {
    unsafe {
        if JS_RUNTIME.is_none() {
            let runtime = Runtime::new().map_err(|e| e.to_string())?;
            let context = Context::full(&runtime).map_err(|e| e.to_string())?;
            JS_RUNTIME = Some(runtime);
            JS_CONTEXT = Some(context);
        }
        Ok(JS_CONTEXT.as_ref().unwrap())
    }
}

#[tauri::command]
fn quickjs_execute(code: String) -> Result<String, String> {
    let ctx = get_js_context()?;
    let result = ctx.with(|ctx| {
        ctx.eval::<String, _>(code.as_str())
    });
    match result {
        Ok(value) => Ok(value),
        Err(e) => Err(format!("JS Error: {:?}", e)),
    }
}

#[tauri::command]
fn quickjs_init() -> Result<String, String> {
    get_js_context()?;
    Ok("QuickJS (rquickjs) initialized".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            quickjs_execute,
            quickjs_init,
            commands::window::show_window,
            commands::window::hide_window,
            commands::window::toggle_window,
            commands::window::set_always_on_top,
            commands::window::is_window_visible,
            commands::clipboard::read_clipboard,
            commands::clipboard::write_clipboard,
            commands::shell::open_url,
            commands::shell::open_path,
            commands::shell::open_app,
            commands::store::save_settings,
            commands::store::load_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

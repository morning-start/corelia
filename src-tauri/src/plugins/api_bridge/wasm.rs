use rquickjs::{Ctx, Object, Function};
use std::sync::Mutex;
use tauri::Emitter;
use crate::plugins::wasm_bridge::WasmBridge;
use super::require_app;

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    println!("[ApiBridge]   🧩 正在注入 WASM API...");

    let wasm_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 wasm 对象失败: {}", e))?;

    let call_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>, func_name: String, args_json: String| -> Result<String, rquickjs::Error> {
            let app = require_app!("__wasm_call");

            {
                let bridge = app.state::<Mutex<WasmBridge>>();
                let bridge_ref = bridge.lock().map_err(|e| rquickjs::Error::new_from_js_message(
                    "__wasm_call", "Error", format!("获取 WasmBridge 锁失败: {}", e)
                ))?;

                if !bridge_ref.has_function(&func_name) {
                    return Err(rquickjs::Error::new_from_js_message(
                        "__wasm_call", "Error", format!("WASM 函数未注册: {}", func_name)
                    ));
                }
            }

            let request_id = crate::plugins::wasm_bridge::generate_request_id();

            let payload = serde_json::json!({
                "requestId": request_id,
                "function": func_name,
                "args": args_json,
            });

            app.emit("wasm-call", payload)
                .map_err(|e| rquickjs::Error::new_from_js_message(
                    "__wasm_call", "Error", format!("发送事件失败: {}", e)
                ))?;

            Ok(request_id)
        },
    ).map_err(|e| format!("创建 __wasm_call 函数失败: {}", e))?;
    wasm_obj.set("__wasm_call", call_fn).map_err(|e| format!("设置 __wasm_call 失败: {}", e))?;

    let get_result_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>, request_id: String, max_wait_ms: Option<i64>| -> Result<Option<String>, rquickjs::Error> {
            let app = require_app!("__wasm_get_result");
            let max_wait = max_wait_ms.map(|v| v.max(0) as u64).unwrap_or(0);
            let start = std::time::Instant::now();

            let mut poll_interval = 10u64;
            const MAX_POLL_INTERVAL: u64 = 100;

            loop {
                {
                    let bridge = app.state::<Mutex<WasmBridge>>();
                    let mut bridge_ref = bridge.lock().map_err(|e| rquickjs::Error::new_from_js_message(
                        "__wasm_get_result", "Error", format!("获取 WasmBridge 锁失败: {}", e)
                    ))?;

                    match bridge_ref.get_call_result(&request_id) {
                        Some(entry) => {
                            return serde_json::to_string(&entry)
                                .map(Some)
                                .map_err(|e| rquickjs::Error::new_from_js_message(
                                    "__wasm_get_result", "Error", format!("序列化结果失败: {}", e)
                                ));
                        }
                        None if max_wait == 0 => return Ok(None),
                        None => {},
                    }
                }

                if max_wait > 0 && start.elapsed().as_millis() as u64 >= max_wait {
                    return Ok(None);
                }

                std::thread::sleep(std::time::Duration::from_millis(poll_interval));
                poll_interval = (poll_interval * 2).min(MAX_POLL_INTERVAL);
            }
        },
    ).map_err(|e| format!("创建 __wasm_get_result 函数失败: {}", e))?;
    wasm_obj.set("__wasm_get_result", get_result_fn).map_err(|e| format!("设置 __wasm_get_result 失败: {}", e))?;

    let available_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>| -> Result<Vec<String>, rquickjs::Error> {
            let app = require_app!("__wasm_available");
            let bridge = app.state::<Mutex<WasmBridge>>();
            let bridge_ref = bridge.lock().map_err(|e| rquickjs::Error::new_from_js_message(
                "__wasm_available", "Error", format!("获取 WasmBridge 锁失败: {}", e)
            ))?;

            Ok(bridge_ref.list_functions().into_iter().map(|f| f.name.clone()).collect())
        },
    ).map_err(|e| format!("创建 __wasm_available 函数失败: {}", e))?;
    wasm_obj.set("__wasm_available", available_fn).map_err(|e| format!("设置 __wasm_available 失败: {}", e))?;

    let has_fn = Function::new(
        ctx.clone(),
        |_ctx: Ctx<'js>, func_name: String| -> Result<bool, rquickjs::Error> {
            let app = require_app!("__wasm_has");
            let bridge = app.state::<Mutex<WasmBridge>>();
            let bridge_ref = bridge.lock().map_err(|e| rquickjs::Error::new_from_js_message(
                "__wasm_has", "Error", format!("获取 WasmBridge 锁失败: {}", e)
            ))?;

            Ok(bridge_ref.has_function(&func_name))
        },
    ).map_err(|e| format!("创建 __wasm_has 函数失败: {}", e))?;
    wasm_obj.set("__wasm_has", has_fn).map_err(|e| format!("设置 __wasm_has 失败: {}", e))?;

    parent.set("wasm", wasm_obj).map_err(|e| format!("设置 wasm 失败: {}", e))?;
    println!("[ApiBridge]   ✓ wasm API 注入成功");
    Ok(())
}

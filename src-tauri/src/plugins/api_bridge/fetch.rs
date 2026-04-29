use rquickjs::{Ctx, Object, Function, Value};
use std::collections::HashMap;
use super::require_app;

type FetchResult = Result<(u16, String, HashMap<String, String>, Option<String>), String>;

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>) -> Result<(), String> {
    println!("[ApiBridge]   📡 正在注入 fetch API...");

    let fetch_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'js>, url: String, options: Value<'js>| -> Result<Value<'js>, rquickjs::Error> {
            let url_clone = url.clone();
            let ctx_clone = _ctx.clone();

            let method = options
                .as_object()
                .and_then(|o: &Object<'js>| o.get("method").ok())
                .and_then(|v: Value<'js>| v.as_string().map(|s| s.to_string().unwrap_or_default()))
                .unwrap_or_else(|| "GET".to_string());

            let mut req_headers: Vec<(String, String)> = Vec::new();
            if let Some(headers_obj) = options
                .as_object()
                .and_then(|o: &Object<'js>| o.get("headers").ok())
                .and_then(|v: Value<'js>| v.into_object())
            {
                for key in headers_obj.keys::<String>().flatten() {
                    if let Ok(val) = headers_obj.get::<_, String>(&key) {
                        req_headers.push((key, val));
                    }
                }
            }

            let body_str: Option<String> = options
                .as_object()
                .and_then(|o: &Object<'js>| o.get("body").ok())
                .and_then(|v: Value<'js>| -> Option<String> {
                    if v.is_string() { v.as_string().map(|s| s.to_string().unwrap_or_default()) }
                    else if !v.is_null() && !v.is_undefined() { serde_json::to_string(&convert_to_serde(v)).ok() }
                    else { None }
                });

            let timeout_secs: u64 = options
                .as_object()
                .and_then(|o: &Object<'js>| o.get("timeout").ok())
                .and_then(|v: Value<'js>| v.as_int())
                .unwrap_or(30) as u64;

            let result = std::thread::Builder::new()
                .name("fetch-thread".to_string())
                .spawn(move || -> FetchResult {
                    let method_upper = method.to_uppercase();
                    let req = match method_upper.as_str() {
                        "POST" => ureq::post(&url_clone),
                        "PUT" => ureq::put(&url_clone),
                        "DELETE" => ureq::delete(&url_clone),
                        "PATCH" => ureq::request("PATCH", &url_clone),
                        "HEAD" => ureq::head(&url_clone),
                        _ => ureq::get(&url_clone),
                    };

                    let req = req.timeout(std::time::Duration::from_secs(timeout_secs));
                    let req = req_headers.into_iter().fold(req, |req, (k, v)| req.set(k.as_str(), v.as_str()));

                    let resp: Result<ureq::Response, ureq::Error> = if let Some(body) = body_str {
                        match req.send_string(&body) {
                            Ok(r) => Ok(r),
                            Err(ureq::Error::Status(code, resp)) => {
                                let body_text = resp.into_string().ok();
                                return Ok((code, "Error".to_string(), HashMap::new(), body_text));
                            }
                            Err(e) => return Err(format!("请求失败: {}", e)),
                        }
                    } else {
                        match req.call() {
                            Ok(r) => Ok(r),
                            Err(ureq::Error::Status(code, resp)) => {
                                let body_text = resp.into_string().ok();
                                return Ok((code, "Error".to_string(), HashMap::new(), body_text));
                            }
                            Err(e) => return Err(format!("请求失败: {}", e)),
                        }
                    };

                    match resp {
                        Ok(resp) => {
                            let mut headers = HashMap::new();
                            for key in resp.headers_names() {
                                if let Some(val) = resp.header(&key) {
                                    headers.insert(key, val.to_string());
                                }
                            }
                            let body = resp.into_string().ok();
                            Ok((200, "OK".to_string(), headers, body))
                        }
                        Err(e) => Err(format!("请求异常: {}", e))
                    }
                })
                .map_err(|e| rquickjs::Error::new_from_js_message("fetch", "Error", format!("创建线程失败: {}", e)))?
                .join()
                .map_err(|_| rquickjs::Error::new_from_js_message("fetch", "Error", "线程 panicked"))?
                .map_err(|e| rquickjs::Error::new_from_js_message("fetch", "Error", e))?;

            let response_obj = Object::new(ctx_clone.clone())?;
            response_obj.set("status", result.0 as i32)?;
            response_obj.set("statusText", result.1)?;
            response_obj.set("ok", result.0 >= 200 && result.0 < 300)?;

            let headers_obj = Object::new(ctx_clone.clone())?;
            for (k, v) in &result.2 {
                headers_obj.set(k.as_str(), v.as_str())?;
            }
            response_obj.set("headers", headers_obj)?;

            let body_str = result.3.unwrap_or_default();
            let body_clone = body_str.clone();

            let text_fn = Function::new(ctx_clone.clone(), move |_: Ctx<'js>| Ok::<String, rquickjs::Error>(body_clone.clone()));
            response_obj.set("text", text_fn)?;

            let json_body = body_str.clone();
            let ctx_for_json = ctx_clone.clone();
            let json_fn = Function::new(ctx_for_json.clone(), move |_: Ctx<'js>| -> Result<Value<'js>, rquickjs::Error> {
                let json_ctx = ctx_for_json.clone();
                match serde_json::from_str::<serde_json::Value>(&json_body) {
                    Ok(v) => {
                        let s = serde_json::to_string(&v).map_err(|e| rquickjs::Error::new_from_js_message("json", "Error", format!("序列化失败: {}", e)))?;
                        Ok(Value::from_string(rquickjs::String::from_str(json_ctx, &s)?))
                    }
                    Err(_) => Err(rquickjs::Error::new_from_js_message("json", "Error", "无效 JSON"))
                }
            });
            response_obj.set("json", json_fn)?;

            Ok(response_obj.into())
        },
    ).map_err(|e| format!("创建 fetch 函数失败: {}", e))?;
    parent.set("fetch", fetch_fn).map_err(|e| format!("设置 fetch 失败: {}", e))?;
    println!("[ApiBridge]   ✓ fetch API 注入成功");
    Ok(())
}

fn convert_to_serde(value: Value<'_>) -> serde_json::Value {
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

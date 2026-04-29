use rquickjs::{Ctx, Object, Function};
use std::collections::HashMap;
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use super::require_app;

pub fn inject<'js>(ctx: &Ctx<'js>, parent: &Object<'js>, plugin_id: String) -> Result<(), String> {
    let db_obj = Object::new(ctx.clone()).map_err(|e| format!("创建 dbStorage 对象失败: {}", e))?;
    let storage_path = format!("plugins/{}/storage.json", plugin_id);

    let plugin_id_get = plugin_id.clone();
    let storage_path_get = storage_path.clone();
    let get_item_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>, key: String| -> Result<Option<String>, rquickjs::Error> {
            println!("[utools.dbStorage] getItem: {} (plugin: {})", key, plugin_id_get);
            let app = require_app!("getItem");
            match app.store(&storage_path_get) {
                Ok(store) => {
                    match store.get(&key) {
                        Some(serde_json::Value::String(s)) => Ok(Some(s)),
                        Some(_) => Ok(Some("".to_string())),
                        None => Ok(None),
                    }
                }
                Err(e) => Err(rquickjs::Error::new_from_js_message("getItem", "Error", format!("获取存储失败: {}", e)))
            }
        },
    ).map_err(|e| format!("创建 getItem 函数失败: {}", e))?;
    db_obj.set("getItem", get_item_fn).map_err(|e| format!("设置 getItem 失败: {}", e))?;

    let plugin_id_set = plugin_id.clone();
    let storage_path_set = storage_path.clone();
    let set_item_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>, key: String, value: String| -> Result<(), rquickjs::Error> {
            println!("[utools.dbStorage] setItem: {} (plugin: {})", key, plugin_id_set);
            let app = require_app!("setItem");
            match app.store(&storage_path_set) {
                Ok(store) => {
                    store.set(&key, serde_json::Value::String(value));
                    if let Err(e) = store.save() {
                        return Err(rquickjs::Error::new_from_js_message("setItem", "Error", format!("保存失败: {}", e)));
                    }
                    Ok(())
                }
                Err(e) => Err(rquickjs::Error::new_from_js_message("setItem", "Error", format!("获取存储失败: {}", e)))
            }
        },
    ).map_err(|e| format!("创建 setItem 函数失败: {}", e))?;
    db_obj.set("setItem", set_item_fn).map_err(|e| format!("设置 setItem 失败: {}", e))?;

    let plugin_id_remove = plugin_id.clone();
    let storage_path_remove = storage_path.clone();
    let remove_item_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>, key: String| -> Result<(), rquickjs::Error> {
            println!("[utools.dbStorage] removeItem: {} (plugin: {})", key, plugin_id_remove);
            let app = require_app!("removeItem");
            match app.store(&storage_path_remove) {
                Ok(store) => {
                    store.delete(&key);
                    if let Err(e) = store.save() {
                        return Err(rquickjs::Error::new_from_js_message("removeItem", "Error", format!("保存失败: {}", e)));
                    }
                    Ok(())
                }
                Err(e) => Err(rquickjs::Error::new_from_js_message("removeItem", "Error", format!("获取存储失败: {}", e)))
            }
        },
    ).map_err(|e| format!("创建 removeItem 函数失败: {}", e))?;
    db_obj.set("removeItem", remove_item_fn).map_err(|e| format!("设置 removeItem 失败: {}", e))?;

    let storage_path_all = storage_path;
    let plugin_id_all = plugin_id;
    let get_all_fn = Function::new(
        ctx.clone(),
        move |_ctx: Ctx<'_>, ()| -> Result<HashMap<String, String>, rquickjs::Error> {
            println!("[utools.dbStorage] getAll (plugin: {})", plugin_id_all);
            let app = require_app!("getAll");
            let store = match app.store(&storage_path_all) {
                Ok(s) => s,
                Err(e) => return Err(rquickjs::Error::new_from_js_message("getAll", "Error", format!("获取存储失败: {}", e)))
            };
            let mut result = HashMap::new();
            for key in store.keys() {
                if let Some(serde_json::Value::String(v)) = store.get(&key) {
                    result.insert(key, v);
                }
            }
            Ok(result)
        },
    ).map_err(|e| format!("创建 getAll 函数失败: {}", e))?;
    db_obj.set("getAll", get_all_fn).map_err(|e| format!("设置 getAll 失败: {}", e))?;

    parent.set("dbStorage", db_obj).map_err(|e| format!("设置 dbStorage 失败: {}", e))?;
    println!("[ApiBridge]   ✓ dbStorage 模块注入成功");
    Ok(())
}

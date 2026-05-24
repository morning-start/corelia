use std::path::Path;
use std::time::{Instant, Duration};
use tauri::Emitter;
use super::types::PluginState;

/// 加载指定插件
pub fn load_plugin(loader: &mut super::PluginLoader, id: &str) -> Result<(PluginState, Option<String>), String> {
    // Phase 1: 检查状态 + 设置 Loading
    {
        let instance = loader.instances.get_mut(id)
            .ok_or_else(|| format!("插件不存在: {}", id))?;

        match &instance.state {
            PluginState::Loading => {
                println!("[PluginLoader] 插件 {} 正在加载中，跳过重复加载请求", id);
                return Ok((instance.state.clone(), instance.vm_id.clone()));
            }
            PluginState::Ready => {
                instance.last_used = Some(Instant::now());
                return Ok((instance.state.clone(), instance.vm_id.clone()));
            }
            PluginState::Error(ref err_msg) => {
                if instance.load_error_count >= instance.max_retries {
                    let msg = format!("插件 {} 加载失败次数已达上限 ({}): {}", id, instance.max_retries, err_msg);
                    eprintln!("[PluginLoader] {}", msg);
                    return Err(msg);
                }
                if let Some(retry_after) = instance.retry_after {
                    if Instant::now() < retry_after {
                        let remaining = retry_after.duration_since(Instant::now()).as_secs();
                        return Err(format!("插件 {} 重试冷却中，请等待 {} 秒后再试", id, remaining));
                    }
                }
                println!("[PluginLoader] 插件 {} 处于 Error 状态，尝试第 {} 次重载...", id, instance.load_error_count + 1);
            }
            PluginState::Cached => println!("[PluginLoader] 插件 {} 处于 Cached 状态，重新加载...", id),
            PluginState::MetaLoaded | PluginState::Unloaded => {}
        }

        instance.state = PluginState::Loading;
        instance.last_used = Some(Instant::now());
    }

    // Phase 2: 清理闲置 VM + 读取入口文件 + 创建 VM
    let (vm_id, code) = {
        if let Ok(cleaned) = loader.quickjs_runtime.cleanup() {
            if cleaned > 0 {
                println!("[PluginLoader] 清理了 {} 个闲置 VM", cleaned);
            }
        }

        let (main_file, plugin_dir) = {
            let instance = loader.instances.get(id).unwrap();
            (instance.manifest.main.clone().unwrap_or_else(|| "index.js".into()), instance.plugin_dir.clone())
        };
        let entry_path = plugin_dir.join(&main_file);

        let code = match std::fs::read_to_string(&entry_path) {
            Ok(c) => c,
            Err(e) => {
                let err = format!("读取入口文件失败: {}", e);
                let instance = loader.instances.get_mut(id).unwrap();
                instance.load_error_count += 1;
                instance.state = PluginState::Error(err.clone());
                eprintln!("[PluginLoader] ❌ 插件 {} 入口文件读取失败: {}", id, e);
                return Err(err);
            }
        };

        let vm_id = match loader.quickjs_runtime.create_vm() {
            Ok(v) => v,
            Err(e) => {
                let err = format!("创建 VM 失败: {}", e);
                let instance = loader.instances.get_mut(id).unwrap();
                instance.load_error_count += 1;
                instance.state = PluginState::Error(err.clone());
                eprintln!("[PluginLoader] ❌ 插件 {} 创建 VM 失败: {}", id, e);
                return Err(err);
            }
        };

        (vm_id, code)
    };

    // Phase 3: 设置 VM ID
    {
        let instance = loader.instances.get_mut(id).unwrap();
        instance.vm_id = Some(vm_id.clone());
    }

    // Phase 4: 注入 API
    if let Err(e) = loader.quickjs_runtime.with_context(&vm_id, |ctx| {
        crate::plugins::api_bridge::ApiBridge::inject_utools(&ctx, id)
    }) {
        let _ = loader.quickjs_runtime.destroy_vm(&vm_id);
        let instance = loader.instances.get_mut(id).unwrap();
        instance.vm_id = None;
        instance.state = PluginState::Error(format!("API 注入失败: {}", e));
        instance.load_error_count += 1;
        eprintln!("[PluginLoader] ❌ 插件 {} API 注入失败: {}", id, e);
        return Err(format!("API 注入失败: {}", e));
    }
    println!("[PluginLoader] API 注入成功: {}", id);

    // Phase 5: 加载 WASM patches
    {
        let instance = loader.instances.get_mut(id).unwrap();
        let patches = instance.manifest.patches.clone();
        let plugin_dir = instance.plugin_dir.clone();
        if !patches.is_empty() {
            load_plugin_patches(id, &patches, &plugin_dir);
        }
    }

    // Phase 6: 执行插件代码 + 更新状态
    {
        match loader.quickjs_runtime.execute(&vm_id, &code) {
            Ok(_) => {
                let instance = loader.instances.get_mut(id).unwrap();
                instance.state = PluginState::Ready;
                instance.loaded_at = Some(Instant::now());
                instance.last_used = Some(Instant::now());
                instance.load_error_count = 0;
                instance.last_error = None;
                instance.retry_after = None;
                instance.retry_backoff_ms = 1000;
                println!("[PluginLoader] 插件加载成功: {} (VM: {})", id, vm_id);
                Ok((instance.state.clone(), Some(vm_id.to_string())))
            }
            Err(e) => {
                let _ = loader.quickjs_runtime.destroy_vm(&vm_id);
                let instance = loader.instances.get_mut(id).unwrap();
                instance.vm_id = None;
                let error_msg = format!("执行插件代码失败: {}", e);
                instance.last_error = Some(error_msg.clone());
                instance.load_error_count += 1;
                let backoff = instance.retry_backoff_ms;
                instance.retry_backoff_ms = (backoff * 2).min(30000);
                instance.retry_after = Some(Instant::now() + Duration::from_millis(backoff));
                let retry_info = if instance.load_error_count < instance.max_retries {
                    format!(" 将在 {}ms 后允许重试 (剩余 {} 次)", backoff, instance.max_retries - instance.load_error_count)
                } else {
                    " 已达到最大重试次数，不再自动重试".to_string()
                };
                instance.state = PluginState::Error(error_msg.clone());
                eprintln!("[PluginLoader] ❌ 插件 {} 加载失败 (第 {} 次): {}{}", id, instance.load_error_count, error_msg, retry_info);
                Err(format!("{}{}", error_msg, retry_info))
            }
        }
    }
}

/// 加载 WASM patches 依赖
fn load_plugin_patches(plugin_id: &str, patches: &[String], plugin_dir: &Path) {
    for patch_name in patches {
        println!("[PluginLoader] 📦 加载 WASM patch: {} (插件: {})", patch_name, plugin_id);

        let patch_dir = plugin_dir.join("patches").join(patch_name);
        if !patch_dir.exists() {
            eprintln!("[PluginLoader] ⚠️ patch 目录不存在: {}", patch_dir.display());
            continue;
        }

        let pkg_dir = patch_dir.join("pkg");
        if !pkg_dir.exists() {
            eprintln!("[PluginLoader] ⚠️ patch pkg 目录不存在: {}", pkg_dir.display());
            continue;
        }

        let wasm_file = pkg_dir.join(format!("{}_bg.wasm", patch_name));
        let js_file = pkg_dir.join(format!("{}.js", patch_name));

        if !wasm_file.exists() {
            eprintln!("[PluginLoader] ⚠️ WASM 文件不存在: {}", wasm_file.display());
            continue;
        }
        if !js_file.exists() {
            eprintln!("[PluginLoader] ⚠️ JS 胶水文件不存在: {}", js_file.display());
            continue;
        }

        let mut exported_functions: Vec<String> = Vec::new();
        let pkg_json_path = pkg_dir.join("package.json");
        if pkg_json_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&pkg_json_path) {
                if let Ok(pkg_json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(exports) = pkg_json.get("exports") {
                        if let Some(obj) = exports.as_object() {
                            for key in obj.keys() {
                                if key != "." && !key.starts_with("./") {
                                    exported_functions.push(key.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(app) = crate::plugins::api_bridge::get_app_handle() {
            let payload = serde_json::json!({
                "pluginId": plugin_id,
                "patchName": patch_name,
                "patchDir": patch_dir.to_string_lossy().to_string(),
                "pkgDir": pkg_dir.to_string_lossy().to_string(),
                "wasmFile": wasm_file.to_string_lossy().to_string(),
                "jsFile": js_file.to_string_lossy().to_string(),
                "exportedFunctions": exported_functions,
            });

            match app.emit("wasm-load-patch", payload) {
                Ok(_) => println!("[PluginLoader] ✅ 已通知前端加载 patch: {}", patch_name),
                Err(e) => eprintln!("[PluginLoader] ❌ 通知前端加载 patch 失败: {}", e),
            }
        } else {
            eprintln!("[PluginLoader] ⚠️ AppHandle 未初始化，无法通知前端加载 patch");
        }
    }
}

impl super::PluginLoader {
    pub fn load_plugin(&mut self, id: &str) -> Result<(PluginState, Option<String>), String> {
        load_plugin(self, id)
    }

    pub fn unload_plugin(&mut self, id: &str) -> Result<(), String> {
        let instance = self.instances.get_mut(id)
            .ok_or_else(|| format!("插件不存在: {}", id))?;

        if instance.state == PluginState::Loading {
            println!("[PluginLoader] 插件 {} 正在加载中，等待完成后再卸载", id);
        }

        if let Some(app) = crate::plugins::api_bridge::get_app_handle() {
            let _ = app.emit("plugin-out", id.to_string());
        }

        if let Some(ref vm_id) = instance.vm_id {
            match self.quickjs_runtime.destroy_vm(vm_id) {
                Ok(_) => println!("[PluginLoader] VM 已销毁: {} (插件: {})", vm_id, id),
                Err(e) => eprintln!("[PluginLoader] 销毁 VM 失败: {} (插件: {})，将标记为孤儿 VM", e, id),
            }
        }

        instance.vm_id = None;
        let old_state = instance.state.clone();
        instance.state = PluginState::Unloaded;
        instance.loaded_at = None;
        instance.last_used = None;
        instance.retry_backoff_ms = 1000;
        instance.retry_after = None;

        println!("[PluginLoader] 插件已卸载: {} (旧状态: {})", id, old_state);
        Ok(())
    }

    pub fn unload_all(&mut self) -> Result<(), String> {
        let plugin_ids: Vec<String> = self.instances.keys().cloned().collect();
        let mut errors = Vec::new();

        for id in plugin_ids {
            if let Err(e) = self.unload_plugin(&id) {
                errors.push(format!("{}: {}", id, e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            eprintln!("[PluginLoader] ⚠️ 部分插件卸载失败:\n{}", errors.join("\n"));
            Ok(())
        }
    }
}
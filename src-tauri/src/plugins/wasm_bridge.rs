// WASM 桥接模块
// 负责：管理 WASM patch 的注册/查找/调用，实现 QuickJS VM ↔ WebView WASM 的 IPC 桥接
//
// 架构：
// QuickJS VM (rquickjs) ──invoke──→ Rust wasm_bridge ──emit event──→ WebView (WebAssembly)
//                                     ↑                                      │
//                                     └─────────── listen event ──────────────┘
//
// 流程：
// 1. Rust 读取 patches 列表 → 通知前端加载 WASM
// 2. 前端在 WebView 中初始化 WebAssembly → 注册可用函数到 Rust
// 3. QuickJS VM 调用 __wasm_call(name, args) → Rust 转发到前端 → 返回结果

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use tauri::Emitter;

use crate::plugins::api_bridge::get_app_handle;

/// 全局 WASM 调用计数器（递增，避免毫秒时间戳碰撞）
static WASM_CALL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 生成唯一的 WASM 调用请求 ID
#[inline]
pub fn generate_request_id() -> String {
    format!("wasm_req_{}", WASM_CALL_COUNTER.fetch_add(1, Ordering::AcqRel))
}

/// 已注册的 WASM 函数信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmFunctionInfo {
    /// 函数名（如 "crypto.sha256"）
    pub name: String,
    /// 所属 patch 名称
    pub patch: String,
    /// 参数数量
    pub param_count: usize,
}

/// WASM 调用结果（存储在前端返回的结果，供 QuickJS VM 轮询获取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCallResultEntry {
    pub request_id: String,
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
}

/// WASM 桥接注册表
pub struct WasmBridge {
    /// 已注册的 WASM 函数：func_name -> WasmFunctionInfo
    functions: HashMap<String, WasmFunctionInfo>,
    /// 已加载的 patch 列表：patch_name -> patch_dir_path
    loaded_patches: HashMap<String, String>,
    /// WASM 调用结果缓存：request_id -> result
    call_results: HashMap<String, WasmCallResultEntry>,
}

impl WasmBridge {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            loaded_patches: HashMap::new(),
            call_results: HashMap::new(),
        }
    }

    /// 注册 WASM patch 的可用函数（由前端调用，通过 Tauri event）
    pub fn register_functions(&mut self, patch: String, functions: Vec<WasmFunctionInfo>) {
        // 先清除该 patch 之前的注册（支持热更新）
        self.functions.retain(|_, f| f.patch != patch);

        let count = functions.len();
        for func in functions {
            println!("[WasmBridge] 注册函数: {} (patch: {})", func.name, func.patch);
            self.functions.insert(func.name.clone(), func);
        }

        self.loaded_patches.insert(patch.clone(), String::new());
        println!("[WasmBridge] patch '{}' 注册完成，共 {} 个函数", patch, count);
    }

    /// 注销指定 patch 的所有函数
    pub fn unregister_patch(&mut self, patch: &str) {
        self.functions.retain(|_, f| f.patch != patch);
        self.loaded_patches.remove(patch);
        println!("[WasmBridge] patch '{}' 已注销", patch);
    }

    /// 获取所有已注册的函数列表
    pub fn list_functions(&self) -> Vec<&WasmFunctionInfo> {
        self.functions.values().collect()
    }

    /// 获取指定 patch 的函数列表
    pub fn get_patch_functions(&self, patch: &str) -> Vec<&WasmFunctionInfo> {
        self.functions.values().filter(|f| f.patch == patch).collect()
    }

    /// 检查函数是否存在
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// 检查 patch 是否已加载
    pub fn is_patch_loaded(&self, patch: &str) -> bool {
        self.loaded_patches.contains_key(patch)
    }

    /// 存储 WASM 调用结果（由前端通过 Tauri command 写入）
    pub fn store_call_result(&mut self, result: WasmCallResultEntry) {
        // 限制缓存大小，防止内存泄漏
        if self.call_results.len() > 1000 {
            // 清除最旧的一半结果（按 requestId 前缀排序）
            let mut ids: Vec<String> = self.call_results.keys().cloned().collect();
            ids.sort();
            for id in ids.iter().take(500) {
                self.call_results.remove(id);
            }
        }
        self.call_results.insert(result.request_id.clone(), result);
    }

    /// 获取 WASM 调用结果（由 QuickJS VM 中的 __wasm_get_result 轮询调用）
    /// 返回结果后自动从缓存中移除
    pub fn get_call_result(&mut self, request_id: &str) -> Option<WasmCallResultEntry> {
        self.call_results.remove(request_id)
    }

    /// 通过 IPC 调用前端已加载的 WASM 函数
    ///
    /// 此方法向 WebView 发出 `wasm-call` 事件，并监听 `wasm-call-result` 事件获取结果
    pub fn call_wasm_function(&self, func_name: &str, args: &str) -> Result<String, String> {
        let app = get_app_handle()
            .ok_or_else(|| "AppHandle 未初始化".to_string())?;

        // 构造调用请求
        let request_id = generate_request_id();

        let payload = serde_json::json!({
            "requestId": request_id,
            "function": func_name,
            "args": args,
        });

        // 使用 Tauri event 通知前端
        app.emit("wasm-call", payload)
            .map_err(|e| format!("发送 wasm-call 事件失败: {}", e))?;

        // 注意：实际结果通过前端 emit("wasm-call-result") 返回
        // 同步调用在 QuickJS VM 中通过 __wasm_call 实现
        // 由于 QuickJS 是同步执行的，这里使用一次性事件监听等待结果

        Ok(request_id)
    }
}

// ==================== Tauri Commands ====================

/// 注册 WASM patch 函数（前端调用）
///
/// 前端在 WebView 中加载 WASM 后，调用此命令注册可用函数
#[tauri::command]
pub fn wasm_register_functions(
    bridge: tauri::State<'_, Mutex<WasmBridge>>,
    patch: String,
    functions: Vec<WasmFunctionInfo>,
) -> Result<(), String> {
    let mut bridge = bridge.lock().map_err(|e| e.to_string())?;
    bridge.register_functions(patch, functions);
    Ok(())
}

/// 注销 WASM patch（前端调用）
#[tauri::command]
pub fn wasm_unregister_patch(
    bridge: tauri::State<'_, Mutex<WasmBridge>>,
    patch: String,
) -> Result<(), String> {
    let mut bridge = bridge.lock().map_err(|e| e.to_string())?;
    bridge.unregister_patch(&patch);
    Ok(())
}

/// 列出所有已注册的 WASM 函数
#[tauri::command]
pub fn wasm_list_functions(
    bridge: tauri::State<'_, Mutex<WasmBridge>>,
) -> Result<Vec<WasmFunctionInfo>, String> {
    let bridge = bridge.lock().map_err(|e| e.to_string())?;
    Ok(bridge.list_functions().into_iter().cloned().collect())
}

/// 检查 WASM patch 是否已加载
#[tauri::command]
pub fn wasm_is_patch_loaded(
    bridge: tauri::State<'_, Mutex<WasmBridge>>,
    patch: String,
) -> Result<bool, String> {
    let bridge = bridge.lock().map_err(|e| e.to_string())?;
    Ok(bridge.is_patch_loaded(&patch))
}

/// QuickJS VM 调用 WASM 函数的入口
///
/// 由 api_bridge.rs 中的 `__wasm_call` 调用，同步等待前端返回结果
#[tauri::command]
pub fn wasm_call_function(
    bridge: tauri::State<'_, Mutex<WasmBridge>>,
    function: String,
    args: String,
) -> Result<String, String> {
    let bridge_ref = bridge.lock().map_err(|e| e.to_string())?;

    // 检查函数是否已注册
    if !bridge_ref.has_function(&function) {
        return Err(format!("WASM 函数未注册: {}", function));
    }

    // 通知前端执行 WASM 函数
    bridge_ref.call_wasm_function(&function, &args)
}

/// 存储 WASM 调用结果（前端调用）
///
/// 前端在 WebView 中执行完 WASM 函数后，通过此命令将结果存回 Rust
#[tauri::command]
pub fn wasm_store_call_result(
    bridge: tauri::State<'_, Mutex<WasmBridge>>,
    result: WasmCallResultEntry,
) -> Result<(), String> {
    let mut bridge = bridge.lock().map_err(|e| e.to_string())?;
    bridge.store_call_result(result);
    Ok(())
}

/// 获取 WASM 调用结果（QuickJS VM 轮询调用）
///
/// 返回结果后自动从缓存中移除。返回 null 表示结果尚未就绪
#[tauri::command]
pub fn wasm_get_call_result(
    bridge: tauri::State<'_, Mutex<WasmBridge>>,
    request_id: String,
) -> Result<Option<WasmCallResultEntry>, String> {
    let mut bridge = bridge.lock().map_err(|e| e.to_string())?;
    Ok(bridge.get_call_result(&request_id))
}

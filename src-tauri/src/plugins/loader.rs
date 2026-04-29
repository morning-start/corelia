// 插件加载器模块
// 负责：扫描、解析、加载、卸载 QuickJS 插件的全流程管理

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use tauri::Emitter;

use crate::plugins::quickjs_runtime::QuickJSRuntime;

// ==================== 数据结构定义 ====================

/// 插件元数据（来自 plugin.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub plugin_type: String,              // "quickjs"
    pub logo: Option<String>,
    pub prefix: Option<String>,          // 触发前缀
    pub main: Option<String>,             // 入口文件（默认 index.js）
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(default)]
    pub patches: Vec<String>,             // WASM 依赖
    pub features: Option<Vec<FeatureConfig>>,
}

/// 功能配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub code: String,
    pub label: String,
    #[serde(rename = "type")]
    pub feature_type: String,             // "list", "text", "cmd"
    pub items: Option<Vec<FeatureItem>>,
}

/// 功能子项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureItem {
    pub label: String,
    pub action: String,
    pub icon: Option<String>,
}

/// 动态注册的功能（运行时由插件调用 registerPluginFeature 注册）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredFeature {
    pub code: String,
    pub label: String,
    #[serde(rename = "type")]
    pub feature_type: String,
}

/// 插件运行时状态（状态机：MetaLoaded → Loading → Ready/Cached → Unloaded）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PluginState {
    /// 仅元数据已加载（scan_plugins 后的状态）
    MetaLoaded,
    /// 正在加载中（load_plugin 过程中的临时状态）
    Loading,
    /// 就绪，可以执行
    Ready,
    /// 已缓存（长时间未使用后自动缓存）
    Cached,
    /// 已卸载
    Unloaded,
    /// 错误状态
    Error(String),
}

impl std::fmt::Display for PluginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginState::MetaLoaded => write!(f, "MetaLoaded"),
            PluginState::Loading => write!(f, "Loading"),
            PluginState::Ready => write!(f, "Ready"),
            PluginState::Cached => write!(f, "Cached"),
            PluginState::Unloaded => write!(f, "Unloaded"),
            PluginState::Error(msg) => write!(f, "Error({})", msg),
        }
    }
}

/// 插件实例（运行时状态）
#[derive(Debug, Clone)]
pub struct PluginInstance {
    /// 插件唯一标识符（使用 name 字段作为 ID）
    pub id: String,
    /// 插件元数据
    pub manifest: PluginManifest,
    /// 当前状态
    pub state: PluginState,
    /// 关联的 QuickJS VM ID
    pub vm_id: Option<String>,
    /// 插件目录路径
    pub plugin_dir: PathBuf,
    /// 加载时间戳
    pub loaded_at: Option<Instant>,
    /// 最后使用时间戳
    pub last_used: Option<Instant>,
    /// 动态注册的功能列表（运行时由插件调用 registerPluginFeature 注册）
    pub registered_features: Vec<RegisteredFeature>,
    /// 插件就绪回调函数（JavaScript 函数引用）
    pub on_ready_callback: Option<String>,
    /// 插件退出回调函数（JavaScript 函数引用）
    pub on_out_callback: Option<String>,
    /// 加载失败次数（用于重试逻辑和错误隔离）
    pub load_error_count: u32,
    /// 最大重试次数
    pub max_retries: u32,
    /// 最后一次错误信息
    pub last_error: Option<String>,
    /// 加载失败后的下次允许重试时间
    pub retry_after: Option<Instant>,
    /// 加载重试的退避延迟（毫秒），每次失败翻倍
    pub retry_backoff_ms: u64,
}

// ==================== 插件加载器核心实现 ====================

/// 插件加载器
///
/// 负责管理插件的生命周期：扫描 → 解析 → 加载 → 执行 → 卸载
pub struct PluginLoader {
    /// 插件根目录路径（如 "plugins/"）
    plugins_dir: PathBuf,
    /// 所有已发现的插件实例（plugin_id -> instance）
    instances: HashMap<String, PluginInstance>,
    /// QuickJS 运行时共享引用（与 Tauri State 中的 QuickJSRuntime 为同一实例）
    quickjs_runtime: Arc<QuickJSRuntime>,
}

// 安全性说明：PluginLoader 通过 Tauri State 管理，所有操作在主线程执行，
// 不会出现多线程并发访问问题
unsafe impl Send for PluginLoader {}
unsafe impl Sync for PluginLoader {}

impl PluginLoader {
    /// 创建新的插件加载器实例
    ///
    /// # Arguments
    /// - `plugins_dir`: 插件根目录路径
    /// - `runtime`: QuickJS 运行时管理器的 Arc 共享引用
    pub fn new(plugins_dir: PathBuf, runtime: Arc<QuickJSRuntime>) -> Self {
        Self {
            plugins_dir,
            instances: HashMap::new(),
            quickjs_runtime: runtime,
        }
    }

    /// 获取内部 QuickJSRuntime 的 Arc 引用（供外部 Commands 共享使用）
    pub fn runtime(&self) -> &Arc<QuickJSRuntime> {
        &self.quickjs_runtime
    }

    /// 获取配置中的闲置超时时间
    pub fn idle_timeout_secs(&self) -> u64 {
        // QuickJSConfig 的 idle_timeout_secs 默认值为 300
        300
    }

    /// 扫描所有插件（仅加载元数据，懒加载策略）
    ///
    /// 遍历 plugins_dir 下所有子目录，查找 plugin.json 文件并解析元数据。
    /// 此方法不会加载插件的 JavaScript 代码，仅读取配置信息。
    ///
    /// # Returns
    /// - `Ok(Vec<String>)`: 成功扫描到的插件 ID 列表
    /// - `Err(String)`: 错误信息（如目录不存在、权限不足等）
    ///
    /// # Example
    /// ```rust
    /// let ids = loader.scan_plugins()?;
    /// println!("发现 {} 个插件", ids.len());
    /// ```
    pub fn scan_plugins(&mut self) -> Result<Vec<String>, String> {
        // 1. 检查 plugins_dir 是否存在
        if !self.plugins_dir.exists() {
            return Err(format!(
                "插件目录不存在: {}",
                self.plugins_dir.display()
            ));
        }

        // 2. 检查是否为目录
        if !self.plugins_dir.is_dir() {
            return Err(format!(
                "插件路径不是目录: {}",
                self.plugins_dir.display()
            ));
        }

        // 3. 读取目录下所有条目
        let entries = std::fs::read_dir(&self.plugins_dir)
            .map_err(|e| format!("无法读取插件目录: {}", e))?;

        let mut discovered_ids = Vec::new();

        // 4. 遍历每个子目录
        for entry in entries {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();

            // 只处理目录
            if !path.is_dir() {
                continue;
            }

            // 5. 查找 plugin.json 文件
            let manifest_path = path.join("plugin.json");
            if !manifest_path.exists() {
                // 没有 plugin.json 的目录跳过（可能是其他文件或无效插件）
                continue;
            }

            // 6. 解析 plugin.json
            match self.parse_plugin_json(&path) {
                Ok(manifest) => {
                    // 使用 name 作为插件 ID
                    let plugin_id = manifest.name.clone();

                    // 7. 创建 PluginInstance（初始状态为 MetaLoaded）
                    let instance = PluginInstance {
                        id: plugin_id.clone(),
                        manifest,
                        state: PluginState::MetaLoaded,
                        vm_id: None,
                        plugin_dir: path.clone(),
                        loaded_at: None,
                        last_used: None,
                        registered_features: Vec::new(),
                        on_ready_callback: None,
                        on_out_callback: None,
                        load_error_count: 0,
                        max_retries: 3,
                        last_error: None,
                        retry_after: None,
                        retry_backoff_ms: 1000,
                    };

                    // 8. 添加到 instances HashMap
                    self.instances.insert(plugin_id.clone(), instance);

                    println!("[PluginLoader] 发现插件: {}", plugin_id);
                    discovered_ids.push(plugin_id);
                }
                Err(e) => {
                    eprintln!("[PluginLoader] 解析失败 ({}): {:?}", path.display(), e);
                    // 继续扫描其他插件，不中断整个流程
                }
            }
        }

        Ok(discovered_ids)
    }

    /// 解析 plugin.json 文件
    ///
    /// # Arguments
    /// - `plugin_path`: 插件目录的路径（包含 plugin.json 的父目录）
    ///
    /// # Returns
    /// - `Ok(PluginManifest)`: 解析成功后的元数据
    /// - `Err(String)`: 错误信息（缺少必填字段、JSON 格式错误等）
    fn parse_plugin_json(&self, plugin_path: &Path) -> Result<PluginManifest, String> {
        // 1. 构建 plugin.json 完整路径
        let json_path = plugin_path.join("plugin.json");

        // 2. 读取文件内容
        let content = std::fs::read_to_string(&json_path)
            .map_err(|e| format!("无法读取 plugin.json: {}", e))?;

        // 3. 解析 JSON
        let mut manifest: PluginManifest = serde_json::from_str(&content)
            .map_err(|e| format!("JSON 解析失败: {}", e))?;

        // 4. 验证必填字段
        if manifest.name.is_empty() {
            return Err("插件名称 (name) 不能为空".to_string());
        }
        if manifest.version.is_empty() {
            return Err("插件版本 (version) 不能为空".to_string());
        }
        if manifest.plugin_type.is_empty() {
            return Err("插件类型 (type) 不能为空".to_string());
        }

        // 5. 填充可选字段默认值
        if manifest.main.is_none() {
            manifest.main = Some("index.js".to_string());
        }
        if manifest.features.is_none() {
            manifest.features = Some(Vec::new());
        }

        Ok(manifest)
    }

    /// 获取所有已注册的插件列表
    ///
    /// # Returns
    /// 所有已发现插件的不可变引用列表
    pub fn list_plugins(&self) -> Vec<&PluginInstance> {
        self.instances.values().collect()
    }

    /// 获取所有插件元数据列表（用于 Command 返回）
    ///
    /// # Returns
    /// 所有插件的 manifest 副本
    pub fn list_manifests(&self) -> Vec<PluginManifest> {
        self.instances.values().map(|p| p.manifest.clone()).collect()
    }

    /// 根据 ID 获取插件
    ///
    /// # Arguments
    /// - `id`: 插件标识符（即 manifest.name）
    ///
    /// # Returns
    /// - `Some(&PluginInstance)`: 找到的插件实例
    /// - `None`: 插件不存在
    pub fn get_plugin(&self, id: &str) -> Option<&PluginInstance> {
        self.instances.get(id)
    }

    /// 根据前缀查找匹配的插件（支持模糊匹配）
    ///
    /// 匹配规则：
    /// - 如果输入前缀是某个插件 prefix 的前缀，则匹配
    /// - 如果某个插件的 prefix 是输入前缀的前缀，也匹配
    ///
    /// # Arguments
    /// - `prefix`: 要搜索的前缀字符串
    ///
    /// # Returns
    /// 匹配到的插件列表
    ///
    /// # Example
    /// ```rust
    /// // 假设有插件 prefix 为 "hw"
    /// let matches = loader.find_by_prefix("h");      // 匹配到 hello-world
    /// let matches = loader.find_by_prefix("helloworld"); // 也匹配到 hello-world
    /// ```
    pub fn find_by_prefix(&self, prefix: &str) -> Vec<&PluginInstance> {
        let mut matches = Vec::new();
        let prefix_lower = prefix.to_lowercase();

        for instance in self.instances.values() {
            if let Some(ref p) = instance.manifest.prefix {
                let p_lower = p.to_lowercase();
                // 支持部分前缀匹配（双向匹配）
                if p_lower.starts_with(&prefix_lower) || prefix_lower.starts_with(&p_lower) {
                    matches.push(instance);
                }
            }
        }
        matches
    }

    /// 加载指定插件（包括 JS 代码执行）
    ///
    /// 此方法会：
    /// 1. 检查插件是否存在和当前状态
    /// 2. 读取入口文件（index.js 或 manifest.main 指定的文件）
    /// 3. 创建 QuickJS VM 实例
    /// 4. 在 VM 中注入 uTools API 并执行插件代码
    /// 5. 更新插件状态为 Ready
    ///
    /// # Arguments
    /// - `id`: 要加载的插件 ID
    ///
    /// # Returns
    /// - `Ok((PluginState, Option<String>))`: 加载完成后的新状态和 VM ID
    /// - `Err(String)`: 错误信息
    pub fn load_plugin(&mut self, id: &str) -> Result<(PluginState, Option<String>), String> {
        // 1. 检查插件是否存在
        let instance = self.instances.get_mut(id)
            .ok_or_else(|| format!("插件不存在: {}", id))?;

        // 2. 检查当前状态，防止重复加载和非法状态转换
        match &instance.state {
            PluginState::Loading => {
                // 正在加载中，直接返回当前状态 + 已有的 vm_id
                let vm = instance.vm_id.clone();
                println!("[PluginLoader] 插件 {} 正在加载中，跳过重复加载请求", id);
                return Ok((instance.state.clone(), vm));
            }
            PluginState::Ready => {
                // 已加载，直接返回
                let vm = instance.vm_id.clone();
                instance.last_used = Some(Instant::now());
                return Ok((instance.state.clone(), vm));
            }
            PluginState::Error(ref err_msg) => {
                // Error 状态：检查是否允许重试
                if instance.load_error_count >= instance.max_retries {
                    let msg = format!(
                        "插件 {} 加载失败次数已达上限 ({}): {}",
                        id, instance.max_retries, err_msg
                    );
                    eprintln!("[PluginLoader] {}", msg);
                    return Err(msg);
                }
                // 检查退避冷却是否结束
                if let Some(retry_after) = instance.retry_after {
                    if Instant::now() < retry_after {
                        let remaining = retry_after.duration_since(Instant::now()).as_secs();
                        let msg = format!(
                            "插件 {} 重试冷却中，请等待 {} 秒后再试",
                            id, remaining
                        );
                        println!("[PluginLoader] {}", msg);
                        return Err(msg);
                    }
                }
                println!(
                    "[PluginLoader] 插件 {} 处于 Error 状态，尝试第 {} 次重载...",
                    id, instance.load_error_count + 1
                );
            }
            PluginState::Cached => {
                // Cached 状态允许重新加载
                println!("[PluginLoader] 插件 {} 处于 Cached 状态，重新加载...", id);
            }
            PluginState::MetaLoaded | PluginState::Unloaded => {
                // 正常加载流程
            }
        }

        // 3. 清理闲置超时的 VM（释放资源）
        if let Ok(cleaned) = self.quickjs_runtime.cleanup() {
            if cleaned > 0 {
                println!("[PluginLoader] 清理了 {} 个闲置 VM", cleaned);
            }
        }

        // 4. 更新状态为 Loading（原子操作，防止并发重复加载）
        instance.state = PluginState::Loading;
        instance.last_used = Some(Instant::now());

        // 5. 确定入口文件路径
        let main_file = instance.manifest.main.as_deref().unwrap_or("index.js");
        let entry_path = instance.plugin_dir.join(main_file);

        // 6. 读取入口文件内容（带错误隔离）
        if !entry_path.exists() {
            let err = format!("入口文件不存在: {}", entry_path.display());
            instance.state = PluginState::Error(err.clone());
            instance.load_error_count += 1;
            eprintln!("[PluginLoader] ❌ 插件 {} 入口文件缺失: {}", id, entry_path.display());
            return Err(err);
        }

        let code = match std::fs::read_to_string(&entry_path) {
            Ok(content) => content,
            Err(e) => {
                let err = format!("读取入口文件失败: {}", e);
                instance.state = PluginState::Error(err.clone());
                instance.load_error_count += 1;
                eprintln!("[PluginLoader] ❌ 插件 {} 读取入口文件失败: {}", id, e);
                return Err(err);
            }
        };

        // 7. 创建 QuickJS VM（带错误隔离）
        let vm_id = match self.quickjs_runtime.create_vm() {
            Ok(id) => id,
            Err(e) => {
                let err = format!("创建 VM 失败: {}", e);
                instance.state = PluginState::Error(err.clone());
                instance.load_error_count += 1;
                eprintln!("[PluginLoader] ❌ 插件 {} 创建 VM 失败: {}", id, e);
                return Err(err);
            }
        };

        // 8. 记录 vm_id 到 instance
        instance.vm_id = Some(vm_id.clone());

        // 9. 注入 uTools API 到 VM（带错误隔离和自动清理）
        let vm_id_for_inject = vm_id.clone();
        let plugin_id = id.to_string();
        if let Err(e) = self.quickjs_runtime.with_context(&vm_id_for_inject, |ctx| {
            crate::plugins::api_bridge::ApiBridge::inject_utools(&ctx, &plugin_id)
        }) {
            // API 注入失败 —— 清理 VM 并记录错误
            let _ = self.quickjs_runtime.destroy_vm(&vm_id);
            instance.vm_id = None;
            instance.state = PluginState::Error(format!("API 注入失败: {}", e));
            instance.load_error_count += 1;
            eprintln!("[PluginLoader] ❌ 插件 {} API 注入失败: {}", id, e);
            return Err(format!("API 注入失败: {}", e));
        }
        println!("[PluginLoader] API 注入成功: {}", id);

        // 9.5. 处理 WASM patches 依赖（带降级策略）
        // 先克隆需要的数据，避免与 instance 的可变借用冲突
        let patches = instance.manifest.patches.clone();
        let plugin_dir = instance.plugin_dir.clone();
        if !patches.is_empty() {
            // patches 加载失败不应阻塞插件加载，仅记录警告
            match std::panic::catch_unwind(|| {
                Self::load_patches(id, &patches, &plugin_dir);
            }) {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("[PluginLoader] ⚠️ 插件 {} 的 WASM patches 加载出现 panic，已隔离错误", id);
                }
            }
        }

        // 10. 执行插件代码（带错误隔离）
        match self.quickjs_runtime.execute(&vm_id, &code) {
            Ok(_) => {
                // 11. 加载成功 —— 重置错误计数并更新状态为 Ready
                instance.state = PluginState::Ready;
                instance.loaded_at = Some(Instant::now());
                instance.last_used = Some(Instant::now());
                instance.load_error_count = 0;
                instance.last_error = None;
                instance.retry_after = None;
                instance.retry_backoff_ms = 1000;

                println!("[PluginLoader] 插件加载成功: {} (VM: {})", id, vm_id);
                Ok((instance.state.clone(), Some(vm_id)))
            }
            Err(e) => {
                // 执行失败 —— 清理 VM、更新错误状态、设置重试退避
                let _ = self.quickjs_runtime.destroy_vm(&vm_id);
                instance.vm_id = None;

                let error_msg = format!("执行插件代码失败: {}", e);
                instance.last_error = Some(error_msg.clone());
                instance.load_error_count += 1;

                // 计算退避延迟：每次失败翻倍（1s, 2s, 4s, 8s...），上限 30s
                let backoff = instance.retry_backoff_ms;
                instance.retry_backoff_ms = (backoff * 2).min(30000);
                instance.retry_after = Some(Instant::now() + Duration::from_millis(backoff));

                let retry_info = if instance.load_error_count < instance.max_retries {
                    format!(" 将在 {}ms 后允许重试 (剩余 {} 次)", backoff, instance.max_retries - instance.load_error_count)
                } else {
                    " 已达到最大重试次数，不再自动重试".to_string()
                };

                instance.state = PluginState::Error(error_msg.clone());

                eprintln!("[PluginLoader] ❌ 插件 {} 加载失败 (第 {} 次): {}{}",
                    id, instance.load_error_count, error_msg, retry_info);

                Err(format!("{}{}", error_msg, retry_info))
            }
        }
    }

    /// 加载插件的 WASM patches 依赖（关联函数，避免借用冲突）
    ///
    /// patches 位于插件目录的 `patches/` 子目录下，每个 patch 包含：
    /// - `pkg/` 目录：wasm-pack 构建产物（*.wasm, *.js, *.d.ts）
    /// - `src/` 目录：Rust 源码
    ///
    /// 加载流程：
    /// 1. 验证 patch 目录和 pkg/ 目录是否存在
    /// 2. 读取 patch 的 package.json 获取元数据
    /// 3. 通过 Tauri event 通知前端加载 WASM 模块
    /// 4. 前端在 WebView 中初始化 WebAssembly 并注册函数到 WasmBridge
    fn load_patches(plugin_id: &str, patches: &[String], plugin_dir: &Path) {
        for patch_name in patches {
            println!("[PluginLoader] 📦 加载 WASM patch: {} (插件: {})", patch_name, plugin_id);

            // 1. 检查 patches 目录结构
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

            // 2. 检查关键文件
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

            // 3. 读取 package.json 获取导出函数信息（如果存在）
            let pkg_json_path = pkg_dir.join("package.json");
            let mut exported_functions: Vec<String> = Vec::new();

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

            // 4. 通过 Tauri event 通知前端加载 WASM
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

    /// 卸载指定插件
    ///
    /// 此方法会：
    /// 1. 销毁关联的 QuickJS VM（释放内存资源）
    /// 2. 清理运行时状态
    /// 3. 将插件状态重置为 Unloaded
    /// 4. 保留错误计数信息，便于调试
    ///
    /// # Arguments
    /// - `id`: 要卸载的插件 ID
    ///
    /// # Returns
    /// - `Ok(())`: 卸载成功
    /// - `Err(String)`: 错误信息（如插件不存在）
    pub fn unload_plugin(&mut self, id: &str) -> Result<(), String> {
        // 1. 检查插件是否存在
        let instance = self.instances.get_mut(id)
            .ok_or_else(|| format!("插件不存在: {}", id))?;

        // 防止在 Loading 状态时卸载（等待加载完成或超时）
        if instance.state == PluginState::Loading {
            println!("[PluginLoader] 插件 {} 正在加载中，等待完成后再卸载", id);
            // 这里选择继续卸载，但标记为需要清理
        }

        // 触发 onPluginOut 事件
        if let Some(app) = crate::plugins::api_bridge::get_app_handle() {
            let _ = app.emit("plugin-out", id.to_string());
        }

        // 2. 如果有关联的 VM，销毁它（错误隔离：即使销毁失败也不影响主程序）
        if let Some(ref vm_id) = instance.vm_id {
            match self.quickjs_runtime.destroy_vm(vm_id) {
                Ok(_) => {
                    println!("[PluginLoader] VM 已销毁: {} (插件: {})", vm_id, id);
                }
                Err(e) => {
                    eprintln!("[PluginLoader] 销毁 VM 失败: {} (插件: {})，将标记为孤儿 VM", e, id);
                    // 记录错误但不中断卸载流程
                }
            }
        }

        // 3. 清除 vm_id
        instance.vm_id = None;

        // 4. 更新状态为 Unloaded
        let old_state = instance.state.clone();
        instance.state = PluginState::Unloaded;

        // 5. 清理时间戳
        instance.loaded_at = None;
        instance.last_used = None;

        // 6. 保留错误统计，但重置重试退避（允许重新加载时从头开始）
        instance.retry_backoff_ms = 1000;
        instance.retry_after = None;

        println!("[PluginLoader] 插件已卸载: {} (旧状态: {})", id, old_state);
        Ok(())
    }

    /// 卸载所有插件
    ///
    /// 遍历所有已加载的插件，逐个调用 unload_plugin()
    ///
    /// # Returns
    /// - `Ok(())`: 全部卸载成功
    /// - `Err(String)`: 部分插件卸载失败（但会继续尝试其他插件）
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
            // 错误隔离：即使有插件卸载失败，也不影响其他插件，仅记录日志
            eprintln!("[PluginLoader] ⚠️ 部分插件卸载失败（但其他插件已成功卸载）:\n{}", errors.join("\n"));
            Ok(())
        }
    }

    /// 获取已加载插件的数量
    pub fn loaded_count(&self) -> usize {
        self.instances.values()
            .filter(|p| matches!(p.state, PluginState::Ready | PluginState::Cached))
            .count()
    }

    /// 获取总插件数量（包括仅元数据加载的）
    pub fn total_count(&self) -> usize {
        self.instances.len()
    }

    /// 清理所有闲置插件（将其 VM 销毁并状态转为 Cached/Unloaded）
    ///
    /// 遍历所有 Ready 状态的插件，如果超过 idle_timeout_secs 未使用，
    /// 则销毁 VM 并将状态设为 Cached，以释放内存资源。
    ///
    /// # Returns
    /// 被清理的插件数量
    pub fn cleanup_idle_plugins(&mut self, idle_timeout_secs: u64) -> usize {
        let mut cleaned = 0;
        let ids: Vec<String> = self.instances.keys().cloned().collect();

        for id in ids {
            if let Some(instance) = self.instances.get_mut(&id) {
                if instance.state != PluginState::Ready {
                    continue;
                }

                let should_cleanup = match instance.last_used {
                    Some(last) => last.elapsed().as_secs() >= idle_timeout_secs,
                    None => true, // 没有使用记录，视为闲置
                };

                if should_cleanup {
                    println!("[PluginLoader] 插件 {} 闲置超过 {}s，执行缓存清理", id, idle_timeout_secs);

                    // 销毁关联的 VM
                    if let Some(ref vm_id) = instance.vm_id {
                        if let Err(e) = self.quickjs_runtime.destroy_vm(vm_id) {
                            eprintln!("[PluginLoader] 清理 VM 失败 ({}): {}", id, e);
                        }
                    }

                    instance.vm_id = None;
                    instance.state = PluginState::Cached;
                    instance.loaded_at = None;
                    cleaned += 1;
                }
            }
        }

        if cleaned > 0 {
            println!("[PluginLoader] 共清理 {} 个闲置插件", cleaned);
        }
        cleaned
    }

    /// 扫描并上报所有插件的 VM 健康状态
    ///
    /// 返回每个插件的状态摘要，用于前端监控面板展示
    pub fn get_plugin_health(&self) -> Vec<PluginHealth> {
        self.instances.values()
            .map(|inst| PluginHealth {
                id: inst.id.clone(),
                state: format!("{}", inst.state),
                vm_id: inst.vm_id.clone(),
                loaded_at: inst.loaded_at.map(|t| t.elapsed().as_secs()),
                last_used: inst.last_used.map(|t| t.elapsed().as_secs()),
                error_count: inst.load_error_count,
                last_error: inst.last_error.clone(),
            })
            .collect()
    }
}

/// 插件健康状态摘要（用于监控面板）
#[derive(Debug, Clone, serde::Serialize)]
pub struct PluginHealth {
    pub id: String,
    pub state: String,
    pub vm_id: Option<String>,
    pub loaded_at: Option<u64>,
    pub last_used: Option<u64>,
    pub error_count: u32,
    pub last_error: Option<String>,
}

// ==================== Tauri Commands ====================

/// 扫描插件目录
///
/// 前端调用示例：
/// ```typescript
/// const plugins = await invoke('scan_plugins');
/// console.log(`发现 ${plugins.length} 个插件`);
/// ```
#[tauri::command]
pub fn scan_plugins(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
) -> Result<Vec<PluginManifest>, String> {
    let mut loader = loader.lock().map_err(|e| e.to_string())?;
    let _ids = loader.scan_plugins()?;

    Ok(loader.list_manifests())
}

/// 获取插件列表
///
/// 前端调用示例：
/// ```typescript
/// const plugins = await invoke('get_plugin_list');
/// plugins.forEach(p => console.log(p.name));
/// ```
#[tauri::command]
pub fn get_plugin_list(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
) -> Result<Vec<PluginManifest>, String> {
    let loader = loader.lock().map_err(|e| e.to_string())?;
    Ok(loader.list_manifests())
}

/// 加载指定插件
///
/// 前端调用示例：
/// ```typescript
/// const result = await invoke('load_plugin', { id: 'hello-world' });
/// console.log(result.state); // "Ready"
/// console.log(result.vmId);  // "vm_1744356789012_xxx"
/// ```
#[tauri::command]
pub fn load_plugin(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
    id: String,
) -> Result<LoadResult, String> {
    let mut loader = loader.lock().map_err(|e| e.to_string())?;
    let (state, vm_id) = loader.load_plugin(&id)?;
    Ok(LoadResult { state: format!("{}", state), vm_id })
}

/// 加载插件命令的返回值
#[derive(serde::Serialize)]
pub struct LoadResult {
    pub state: String,
    pub vm_id: Option<String>,
}

/// 卸载指定插件
///
/// 前端调用示例：
/// ```typescript
/// await invoke('unload_plugin', { id: 'hello-world' });
/// console.log('插件已卸载');
/// ```
#[tauri::command]
pub fn unload_plugin(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
    id: String,
) -> Result<(), String> {
    let mut loader = loader.lock().map_err(|e| e.to_string())?;
    loader.unload_plugin(&id)
}

/// 根据前缀搜索插件
///
/// 前端调用示例：
/// ```typescript
/// const results = await invoke('find_plugins_by_prefix', { prefix: 'hw' });
/// results.forEach(p => console.log(p.name));
/// ```
#[tauri::command]
pub fn find_plugins_by_prefix(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
    prefix: String,
) -> Result<Vec<PluginManifest>, String> {
    let loader = loader.lock().map_err(|e| e.to_string())?;
    Ok(loader.find_by_prefix(&prefix)
        .into_iter()
        .map(|p| p.manifest.clone())
        .collect())
}

/// 清理闲置插件（手动触发或定时任务调用）
///
/// 前端调用示例：
/// ```typescript
/// const cleaned = await invoke('cleanup_idle_plugins');
/// console.log(`清理了 ${cleaned} 个闲置插件`);
/// ```
#[tauri::command]
pub fn cleanup_idle_plugins(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
) -> Result<usize, String> {
    let mut loader = loader.lock().map_err(|e| e.to_string())?;
    // 使用 QuickJS 默认的闲置超时配置（300秒）
    let timeout = loader.idle_timeout_secs();
    Ok(loader.cleanup_idle_plugins(timeout))
}

/// 获取插件健康状态（用于监控面板）
///
/// 前端调用示例：
/// ```typescript
/// const health = await invoke('get_plugin_health');
/// health.forEach(h => console.log(`${h.id}: ${h.state}`));
/// ```
#[tauri::command]
pub fn get_plugin_health(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
) -> Result<Vec<PluginHealth>, String> {
    let loader = loader.lock().map_err(|e| e.to_string())?;
    Ok(loader.get_plugin_health())
}

/// 在指定插件的 VM 中执行 JS 代码
///
/// 前端无需管理 VM ID，只需传入 plugin_id，后端会自动找到对应的 VM 并执行。
/// 这是前端移除 VM 缓存后的主要执行接口。
///
/// 前端调用示例：
/// ```typescript
/// const result = await invoke('plugin_execute', {
///     pluginId: 'hello-world',
///     code: 'onSearch("hello")'
/// });
/// ```
#[tauri::command]
pub fn plugin_execute(
    loader: tauri::State<'_, Mutex<PluginLoader>>,
    plugin_id: String,
    code: String,
) -> Result<serde_json::Value, String> {
    let loader = loader.lock().map_err(|e| e.to_string())?;

    // 1. 查找插件
    let instance = loader.get_plugin(&plugin_id)
        .ok_or_else(|| format!("插件不存在: {}", plugin_id))?;

    // 2. 获取插件的 vm_id
    let vm_id = instance.vm_id.as_ref()
        .ok_or_else(|| format!("插件 {} 未加载或 VM 未创建", plugin_id))?;

    // 3. 检查插件状态
    if !matches!(instance.state, PluginState::Ready | PluginState::Cached) {
        return Err(format!("插件 {} 当前状态不可用: {:?}", plugin_id, instance.state));
    }

    // 4. 执行代码（通过 QuickJSRuntime）
    let result = loader.runtime().execute(vm_id, &code)
        .map_err(|e| format!("执行失败: {}", e))?;

    Ok(result)
}

// 插件加载器模块
// 负责：扫描、解析、加载、卸载 QuickJS 插件的全流程管理

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;
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
    /// QuickJS 运行时引用（用于创建/销毁 VM）
    quickjs_runtime: QuickJSRuntime,
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
    /// - `runtime`: QuickJS 运行时管理器引用
    ///
    /// # Example
    /// ```rust
    /// let runtime = QuickJSRuntime::new();
    /// let loader = PluginLoader::new(PathBuf::from("plugins"), runtime);
    /// ```
    pub fn new(plugins_dir: PathBuf, runtime: QuickJSRuntime) -> Self {
        Self {
            plugins_dir,
            instances: HashMap::new(),
            quickjs_runtime: runtime,
        }
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
    fn parse_plugin_json(&self, plugin_path: &PathBuf) -> Result<PluginManifest, String> {
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

        // 2. 检查当前状态（仅允许从 MetaLoaded 或 Unloaded 状态加载）
        match &instance.state {
            PluginState::Ready | PluginState::Loading => {
                // 已加载或正在加载中，直接返回当前状态 + 已有的 vm_id
                let vm = instance.vm_id.clone();
                return Ok((instance.state.clone(), vm));
            }
            PluginState::MetaLoaded | PluginState::Unloaded | PluginState::Cached | PluginState::Error(_) => {
                // 允许从这些状态重新加载
            }
        }

        // 3. 清理闲置超时的 VM（释放资源）
        if let Ok(cleaned) = self.quickjs_runtime.cleanup() {
            if cleaned > 0 {
                println!("[PluginLoader] 清理了 {} 个闲置 VM", cleaned);
            }
        }

        // 4. 更新状态为 Loading
        instance.state = PluginState::Loading;

        // 5. 确定入口文件路径
        let main_file = instance.manifest.main.as_deref().unwrap_or("index.js");
        let entry_path = instance.plugin_dir.join(main_file);

        // 6. 读取入口文件内容
        if !entry_path.exists() {
            instance.state = PluginState::Error(format!("入口文件不存在: {}", entry_path.display()));
            return Err(format!("入口文件不存在: {}", entry_path.display()));
        }

        let code = std::fs::read_to_string(&entry_path)
            .map_err(|e| {
                instance.state = PluginState::Error(format!("读取入口文件失败: {}", e));
                format!("读取入口文件失败: {}", e)
            })?;

        // 7. 创建 QuickJS VM
        let vm_id = self.quickjs_runtime.create_vm()
            .map_err(|e| {
                instance.state = PluginState::Error(format!("创建 VM 失败: {}", e));
                format!("创建 VM 失败: {}", e)
            })?;

        // 8. 记录 vm_id 到 instance
        instance.vm_id = Some(vm_id.clone());

        // 9. 注入 uTools API 到 VM
        let vm_id_for_inject = vm_id.clone();
        let plugin_id = id.to_string();
        if let Err(e) = self.quickjs_runtime.with_context(&vm_id_for_inject, |ctx| {
            crate::plugins::api_bridge::ApiBridge::inject_utools(&ctx, &plugin_id)
        }) {
            let _ = self.quickjs_runtime.destroy_vm(&vm_id);
            instance.vm_id = None;
            instance.state = PluginState::Error(format!("API 注入失败: {}", e));
            return Err(format!("API 注入失败: {}", e));
        }
        println!("[PluginLoader] API 注入成功: {}", id);

        // 9.5. 处理 WASM patches 依赖
        // 先克隆需要的数据，避免与 instance 的可变借用冲突
        let patches = instance.manifest.patches.clone();
        let plugin_dir = instance.plugin_dir.clone();
        if !patches.is_empty() {
            Self::load_patches(id, &patches, &plugin_dir);
        }

        // 10. 执行插件代码
        match self.quickjs_runtime.execute(&vm_id, &code) {
            Ok(_) => {
                // 11. 更新状态为 Ready
                instance.state = PluginState::Ready;
                instance.loaded_at = Some(Instant::now());
                instance.last_used = Some(Instant::now());

                println!("[PluginLoader] 插件加载成功: {} (VM: {})", id, vm_id);
                Ok((instance.state.clone(), Some(vm_id)))
            }
            Err(e) => {
                // 执行失败，清理 VM 并更新状态
                let _ = self.quickjs_runtime.destroy_vm(&vm_id);
                instance.vm_id = None;
                instance.state = PluginState::Error(format!("执行插件代码失败: {}", e));

                Err(format!("执行插件代码失败: {}", e))
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
    fn load_patches(plugin_id: &str, patches: &[String], plugin_dir: &PathBuf) {
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

        // 触发 onPluginOut 事件
        if let Some(app) = crate::plugins::api_bridge::get_app_handle() {
            let _ = app.emit("plugin-out", id.to_string());
        }

        // 2. 如果有关联的 VM，销毁它
        if let Some(ref vm_id) = instance.vm_id {
            match self.quickjs_runtime.destroy_vm(vm_id) {
                Ok(_) => {
                    println!("[PluginLoader] VM 已销毁: {} (插件: {})", vm_id, id);
                }
                Err(e) => {
                    eprintln!("[PluginLoader] 销毁 VM 失败: {} ({})", e, id);
                }
            }
        }

        // 3. 清除 vm_id
        instance.vm_id = None;

        // 4. 更新状态为 Unloaded
        instance.state = PluginState::Unloaded;

        // 5. 清理时间戳
        instance.loaded_at = None;
        instance.last_used = None;

        println!("[PluginLoader] 插件已卸载: {}", id);
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
            Err(format!("部分插件卸载失败:\n{}", errors.join("\n")))
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

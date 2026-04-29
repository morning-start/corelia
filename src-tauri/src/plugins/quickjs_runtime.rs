// QuickJS 运行时管理模块
// 负责：创建和管理插件 VM 实例，提供 VM 池化管理
//
// ## 线程安全说明
// 由于 rquickjs 的 Runtime/Context 不是线程安全的（!Send + !Sync），
// 但 Tauri State 要求 Send + Sync，因此我们需要手动实现这些 trait。
//
// **安全性保证**：
// - 所有 Tauri Commands 都在主线程中同步执行
// - 内部使用 RefCell 保证运行时借用检查
// - VM 池的操作都是非阻塞的快速操作
// - 实际使用中不会出现多线程并发访问

#![allow(dead_code)]

use std::cell::RefCell;
use rquickjs::{Context, Runtime, Value};
use std::time::{Instant, SystemTime};

/// QuickJS 运行时配置
#[derive(Debug, Clone)]
pub struct QuickJSConfig {
    /// 最大内存限制（字节），默认 50MB
    pub max_memory_bytes: usize,
    /// 最大执行时间（毫秒），默认 5秒
    pub max_execution_time_ms: u64,
    /// 最大并发 VM 数，默认 10
    pub max_vm_count: usize,
    /// 闲置超时自动销毁时间（秒），默认 300秒（5分钟）
    pub idle_timeout_secs: u64,
}

impl Default for QuickJSConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 50 * 1024 * 1024,
            max_execution_time_ms: 5000,
            max_vm_count: 10,
            idle_timeout_secs: 300,
        }
    }
}

/// VM 实例信息
pub struct VmInstance {
    /// VM 唯一标识符
    pub id: String,
    /// QuickJS Runtime 实例
    runtime: Runtime,
    /// QuickJS Context 实例
    context: Context,
    /// 创建时间
    created_at: Instant,
    /// 最后使用时间
    last_used_at: Instant,
}

// 安全性说明：VmInstance 只在主线程中通过 Tauri State 访问
unsafe impl Send for VmInstance {}
unsafe impl Sync for VmInstance {}

impl VmInstance {
    /// 创建新的 VM 实例
    fn new(id: String) -> Result<Self, String> {
        let runtime = Runtime::new()
            .map_err(|e| format!("Failed to create QuickJS Runtime: {}", e))?;
        let context = Context::full(&runtime)
            .map_err(|e| format!("Failed to create QuickJS Context: {}", e))?;
        
        let now = Instant::now();
        Ok(Self {
            id,
            runtime,
            context,
            created_at: now,
            last_used_at: now,
        })
    }
    
    /// 更新最后使用时间
    fn touch(&mut self) {
        self.last_used_at = Instant::now();
    }
    
    /// 检查是否闲置超时
    fn is_idle_timeout(&self, timeout_secs: u64) -> bool {
        self.last_used_at.elapsed().as_secs() >= timeout_secs
    }
}

/// QuickJS 运行时管理器
/// 
/// 通过 Tauri State 管理，保证在主线程中运行
pub struct QuickJSRuntime {
    /// 运行时配置
    config: QuickJSConfig,
    /// VM 池（使用 RefCell 因为 Runtime/Context 不是 Send 的）
    vm_pool: RefCell<Vec<VmInstance>>,
}

// 安全性说明：QuickJSRuntime 只在主线程中通过 Tauri State 访问，
// 所有操作都在 Tauri Commands 中同步执行，不会出现数据竞争
unsafe impl Send for QuickJSRuntime {}
unsafe impl Sync for QuickJSRuntime {}

impl QuickJSRuntime {
    /// 创建新的 QuickJSRuntime 实例
    pub fn new() -> Self {
        Self {
            config: QuickJSConfig::default(),
            vm_pool: RefCell::new(Vec::new()),
        }
    }
    
    /// 带自定义配置创建实例
    pub fn with_config(config: QuickJSConfig) -> Self {
        Self {
            config,
            vm_pool: RefCell::new(Vec::new()),
        }
    }
    
    /// 创建新的 VM 实例
    ///
    /// # Returns
    /// - `Ok(String)`: 新创建的 VM ID，格式为 "vm_{timestamp}_{random}"
    /// - `Err(String)`: 错误信息（如 VM 池已满）
    pub fn create_vm(&self) -> Result<String, String> {
        let mut pool = self.vm_pool.borrow_mut();
        
        // 检查 VM 池是否已满
        if pool.len() >= self.config.max_vm_count {
            return Err(format!(
                "VM pool is full (max: {})",
                self.config.max_vm_count
            ));
        }
        
        // 生成唯一 ID
        let vm_id = generate_vm_id();
        
        // 创建 VM 实例
        let vm = VmInstance::new(vm_id.clone())?;
        
        // 加入池
        pool.push(vm);
        
        Ok(vm_id)
    }
    
    /// 销毁指定 VM 实例
    ///
    /// # Arguments
    /// - `vm_id`: 要销毁的 VM 标识符
    ///
    /// # Returns
    /// - `Ok(())`: 成功销毁
    /// - `Err(String)`: 错误信息（如找不到指定 VM）
    pub fn destroy_vm(&self, vm_id: &str) -> Result<(), String> {
        let mut pool = self.vm_pool.borrow_mut();
        
        // 查找并移除指定 VM
        let original_len = pool.len();
        pool.retain(|vm| vm.id != vm_id);
        
        if pool.len() == original_len {
            return Err(format!("VM not found: {}", vm_id));
        }
        
        // VM 被 drop 后自动释放资源
        Ok(())
    }
    
    /// 在指定 VM 中执行 JavaScript 代码
    ///
    /// # Arguments
    /// - `vm_id`: 目标 VM 的标识符
    /// - `code`: 要执行的 JavaScript 代码
    ///
    /// # Returns
    /// - `Ok(serde_json::Value)`: 执行结果（已转换为 JSON）
    /// - `Err(String)`: 错误信息（包含堆栈跟踪）
    pub fn execute(&self, vm_id: &str, code: &str) -> Result<serde_json::Value, String> {
        let mut pool = self.vm_pool.borrow_mut();
        
        // 查找 VM
        let vm = pool.iter_mut()
            .find(|v| v.id == vm_id)
            .ok_or_else(|| format!("VM not found: {}", vm_id))?;
        
        // 更新最后使用时间
        vm.touch();
        
        // 执行代码并转换为 JSON（在 context 作用域内完成转换）
        vm.context.with(|ctx| {
            match ctx.eval::<Value, _>(code) {
                Ok(value) => Ok(convert_value_to_json(value)),
                Err(e) => Err(format!("JavaScript Error: {}", e)),
            }
        })
    }
    
    /// 获取当前活跃 VM 数量
    pub fn active_vm_count(&self) -> usize {
        let pool = self.vm_pool.borrow();
        pool.len()
    }
    
    /// 清理闲置超时的 VM
    ///
    /// # Returns
    /// - `Ok(usize)`: 清理的 VM 数量
    /// - `Err(String)`: 错误信息
    pub fn cleanup(&self) -> Result<usize, String> {
        let mut pool = self.vm_pool.borrow_mut();
        let original_len = pool.len();
        
        // 移除闲置超时的 VM
        pool.retain(|vm| !vm.is_idle_timeout(self.config.idle_timeout_secs));
        
        let removed_count = original_len - pool.len();
        Ok(removed_count)
    }
    
    /// 检查 VM 是否存在
    pub fn vm_exists(&self, vm_id: &str) -> bool {
        let pool = self.vm_pool.borrow();
        pool.iter().any(|vm| vm.id == vm_id)
    }

    /// 在指定 VM 的 Context 中执行操作（用于 API 注入等场景）
    ///
    /// # Arguments
    /// - `vm_id`: 目标 VM 的标识符
    /// - `operation`: 在 Context 中执行的闭包操作
    ///
    /// # Returns
    /// - `Ok(T)`: 操作成功返回的结果
    /// - `Err(String)`: 错误信息（如找不到指定 VM 或操作失败）
    ///
    /// # Example
    /// ```ignore
    /// runtime.with_context(&vm_id, |ctx| {
    ///     // 在 ctx 中注入 API
    ///     ApiBridge::inject_utools(&ctx)
    /// })?;
    /// ```
    pub fn with_context<T, F>(&self, vm_id: &str, operation: F) -> Result<T, String>
    where
        F: FnOnce(rquickjs::Ctx<'_>) -> Result<T, String>,
    {
        let pool = self.vm_pool.borrow();

        // 查找 VM
        let vm = pool
            .iter()
            .find(|v| v.id == vm_id)
            .ok_or_else(|| format!("VM not found: {}", vm_id))?;

        // 在 context 作用域内执行操作
        vm.context.with(operation)
    }
}

/// 生成唯一的 VM ID
///
/// 格式：`vm_{timestamp}_{random}`
fn generate_vm_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let random: u32 = rand::random();
    format!("vm_{}_{}", timestamp, random)
}

// ==================== Tauri Commands ====================

/// 创建新的 QuickJS VM
/// 
/// # Example
/// ```typescript
/// const vmId = await invoke('quickjs_create_vm');
/// console.log(vmId); // "vm_1744356789012_123456789"
/// ```
#[tauri::command]
pub async fn quickjs_create_vm(
    runtime: tauri::State<'_, QuickJSRuntime>,
) -> Result<String, String> {
    runtime.create_vm()
}

/// 销毁指定的 QuickJS VM
/// 
/// # Example
/// ```typescript
/// await invoke('quickjs_destroy_vm', { vmId: 'vm_xxx' });
/// ```
#[tauri::command]
pub async fn quickjs_destroy_vm(
    vm_id: String,
    runtime: tauri::State<'_, QuickJSRuntime>,
) -> Result<(), String> {
    runtime.destroy_vm(&vm_id)
}

/// 在指定 VM 中执行 JS 代码
/// 
/// # Example
/// ```typescript
/// const result = await invoke('quickjs_execute', { 
///     vmId: 'vm_xxx',
///     code: '1 + 1' 
/// });
/// console.log(result); // 2
/// ```
#[tauri::command]
pub async fn quickjs_execute(
    vm_id: String,
    code: String,
    runtime: tauri::State<'_, QuickJSRuntime>,
) -> Result<serde_json::Value, String> {
    runtime.execute(&vm_id, &code)
}

/// 获取活跃 VM 数量
/// 
/// # Example
/// ```typescript
/// const count = await invoke('quickjs_active_count');
/// console.log(count); // 3
/// ```
#[tauri::command]
pub async fn quickjs_active_count(
    runtime: tauri::State<'_, QuickJSRuntime>,
) -> Result<usize, String> {
    Ok(runtime.active_vm_count())
}

/// 清理闲置 VM
/// 
/// # Example
/// ```typescript
/// const removed = await invoke('quickjs_cleanup');
/// console.log(`Cleaned up ${removed} VMs`);
/// ```
#[tauri::command]
pub async fn quickjs_cleanup(
    runtime: tauri::State<'_, QuickJSRuntime>,
) -> Result<usize, String> {
    runtime.cleanup()
}

/// 将 rquickjs::Value 转换为 serde_json::Value（crate 内部共享）
pub(crate) fn convert_value_to_json(value: Value) -> serde_json::Value {
    // 尝试按类型转换
    if value.is_null() || value.is_undefined() {
        return serde_json::Value::Null;
    }

    if let Some(b) = value.as_bool() {
        return serde_json::Value::Bool(b);
    }

    if let Some(i) = value.as_int() {
        return serde_json::Value::Number(serde_json::Number::from(i));
    }

    if let Some(f) = value.as_float() {
        return serde_json::Value::Number(
            serde_json::Number::from_f64(f)
                .unwrap_or_else(|| serde_json::Number::from(0))
        );
    }

    // 处理字符串类型
    if let Some(s) = value.as_string() {
        return serde_json::Value::String(s.to_string().unwrap_or_default());
    }

    // 先检查是否为数组（必须在 object 之前，因为数组也是对象）
    if value.is_array() {
        if let Some(arr) = value.clone().into_array() {
            return convert_array_to_json(arr);
        }
    }

    // 再尝试转换为普通对象
    if value.is_object() {
        if let Some(obj) = value.into_object() {
            return convert_object_to_json(obj);
        }
    }

    // 兜底
    serde_json::Value::Null
}

/// 将数组转换为 JSON
fn convert_array_to_json(arr: rquickjs::Array) -> serde_json::Value {
    let mut result = Vec::new();
    
    // 遍历数组元素
    for i in 0.. {
        match arr.get::<Value>(i) {
            Ok(val) => result.push(convert_value_to_json(val)),
            Err(_) => break,
        }
    }
    
    serde_json::Value::Array(result)
}

/// 将对象转换为 JSON
fn convert_object_to_json(obj: rquickjs::Object) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    
    // 遍历对象属性
    let keys_iter = obj.keys::<String>();
    for key_result in keys_iter {
        match key_result {
            Ok(key) => {
                // 直接使用 String 类型的 key（实现 IntoAtom）
                match obj.get(key.as_str()) {
                    Ok(val) => {
                        map.insert(key, convert_value_to_json(val));
                    }
                    Err(_) => continue,
                }
            }
            Err(_) => continue,
        }
    }
    
    serde_json::Value::Object(map)
}

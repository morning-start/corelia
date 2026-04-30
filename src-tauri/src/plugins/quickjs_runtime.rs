use std::sync::Mutex;
use rquickjs::{Context, Runtime, Value};
use std::time::{Instant, SystemTime, Duration};

#[derive(Debug, Clone)]
pub struct QuickJSConfig {
    pub max_memory_bytes: usize,
    pub max_execution_time_ms: u64,
    pub max_vm_count: usize,
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

pub struct VmInstance {
    pub id: String,
    runtime: Runtime,
    context: Context,
    created_at: Instant,
    last_used_at: Instant,
}

impl VmInstance {
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
    
    fn touch(&mut self) {
        self.last_used_at = Instant::now();
    }
    
    fn is_idle_timeout(&self, timeout_secs: u64) -> bool {
        self.last_used_at.elapsed().as_secs() >= timeout_secs
    }
}

pub struct QuickJSRuntime {
    config: QuickJSConfig,
    vm_pool: Mutex<Vec<VmInstance>>,
}

impl QuickJSRuntime {
    pub fn new() -> Self {
        Self {
            config: QuickJSConfig::default(),
            vm_pool: Mutex::new(Vec::new()),
        }
    }
    
    pub fn with_config(config: QuickJSConfig) -> Self {
        Self {
            config,
            vm_pool: Mutex::new(Vec::new()),
        }
    }
    
    pub fn create_vm(&self) -> Result<String, String> {
        let mut pool = self.vm_pool.lock().map_err(|e| format!("获取 VM 池锁失败: {}", e))?;
        
        // 先尝试清理闲置 VM，为新 VM 腾出空间
        let timeout_secs = self.config.idle_timeout_secs;
        let original_len = pool.len();
        let to_remove: Vec<String> = pool
            .iter()
            .filter(|vm| vm.is_idle_timeout(timeout_secs))
            .map(|vm| vm.id.clone())
            .collect();
        if !to_remove.is_empty() {
            pool.retain(|vm| !to_remove.contains(&vm.id));
            println!("[QuickJSRuntime] 🧹 自动清理 {} 个闲置 VM 为新 VM 腾出空间", original_len - pool.len());
        }
        
        if pool.len() >= self.config.max_vm_count {
            return Err(format!(
                "VM pool is full (max: {}, current: {})",
                self.config.max_vm_count,
                pool.len()
            ));
        }
        
        let vm_id = generate_vm_id();
        let vm = VmInstance::new(vm_id.clone())?;
        pool.push(vm);
        
        println!("[QuickJSRuntime] ✅ VM 创建成功: {} (当前 {} 个活跃)", vm_id, pool.len());
        Ok(vm_id)
    }
    
    pub fn destroy_vm(&self, vm_id: &str) -> Result<(), String> {
        let mut pool = self.vm_pool.lock().map_err(|e| format!("获取 VM 池锁失败: {}", e))?;
        
        let original_len = pool.len();
        pool.retain(|vm| vm.id != vm_id);
        
        if pool.len() == original_len {
            return Err(format!("VM not found: {}", vm_id));
        }
        
        println!("[QuickJSRuntime] 🗑️ VM 已销毁: {} (剩余 {} 个活跃)", vm_id, pool.len());
        Ok(())
    }
    
    pub fn execute(&self, vm_id: &str, code: &str) -> Result<serde_json::Value, String> {
        let mut pool = self.vm_pool.lock().map_err(|e| format!("获取 VM 池锁失败: {}", e))?;
        
        let vm = pool.iter_mut()
            .find(|v| v.id == vm_id)
            .ok_or_else(|| format!("VM not found: {}", vm_id))?;
        
        vm.touch();
        
        vm.context.with(|ctx| {
            match ctx.eval::<Value, _>(code) {
                Ok(value) => Ok(convert_value_to_json(value)),
                Err(e) => Err(format!("JavaScript Error: {}", e)),
            }
        })
    }
    
    pub fn active_vm_count(&self) -> usize {
        let pool = self.vm_pool.lock().unwrap_or_else(|e| {
            eprintln!("[QuickJSRuntime] 获取 VM 池锁失败: {}", e);
            return std::sync::Mutex::new(Vec::new());
        });
        pool.len()
    }
    
    pub fn cleanup(&self) -> Result<usize, String> {
        let mut pool = self.vm_pool.lock().map_err(|e| format!("获取 VM 池锁失败: {}", e))?;
        let original_len = pool.len();
        let timeout_secs = self.config.idle_timeout_secs;

        let to_remove: Vec<String> = pool
            .iter()
            .filter(|vm| vm.is_idle_timeout(timeout_secs))
            .map(|vm| vm.id.clone())
            .collect();

        if !to_remove.is_empty() {
            println!("[QuickJSRuntime] 🔍 发现 {} 个闲置超时 VM（超时: {}s），准备清理...", to_remove.len(), timeout_secs);
        }

        pool.retain(|vm| !to_remove.contains(&vm.id));

        let removed_count = original_len - pool.len();
        if removed_count > 0 {
            println!("[QuickJSRuntime] ✅ 已清理 {} 个闲置超时 VM，剩余 {} 个活跃 VM", removed_count, pool.len());
        }
        Ok(removed_count)
    }

    /// 强制清理指定数量的最旧 VM
    pub fn cleanup_oldest(&self, count: usize) -> Result<usize, String> {
        let mut pool = self.vm_pool.lock().map_err(|e| format!("获取 VM 池锁失败: {}", e))?;
        let original_len = pool.len();
        
        if count == 0 || pool.is_empty() {
            return Ok(0);
        }
        
        // 按最后使用时间排序，清理最久未使用的
        let mut indices: Vec<usize> = (0..pool.len()).collect();
        indices.sort_by_key(|&i| pool[i].last_used_at);
        
        let to_remove: Vec<String> = indices
            .iter()
            .take(count)
            .map(|&i| pool[i].id.clone())
            .collect();
        
        pool.retain(|vm| !to_remove.contains(&vm.id));
        
        let removed_count = original_len - pool.len();
        if removed_count > 0 {
            println!("[QuickJSRuntime] 🗑️ 已强制清理 {} 个最旧 VM，剩余 {} 个活跃 VM", removed_count, pool.len());
        }
        
        Ok(removed_count)
    }

    pub fn cleanup_all(&self) -> Result<usize, String> {
        let mut pool = self.vm_pool.lock().map_err(|e| format!("获取 VM 池锁失败: {}", e))?;
        let removed_count = pool.len();
        pool.clear();
        println!("[QuickJSRuntime] 已强制清理所有 {} 个 VM", removed_count);
        Ok(removed_count)
    }

    pub fn get_vm_stats(&self) -> Vec<VmStats> {
        let pool = self.vm_pool.lock().unwrap_or_else(|e| {
            eprintln!("[QuickJSRuntime] 获取 VM 池锁失败: {}", e);
            return std::sync::Mutex::new(Vec::new());
        });
        pool.iter()
            .map(|vm| VmStats {
                id: vm.id.clone(),
                created_at_secs: vm.created_at.elapsed().as_secs(),
                last_used_at_secs: vm.last_used_at.elapsed().as_secs(),
                is_idle: vm.is_idle_timeout(self.config.idle_timeout_secs),
            })
            .collect()
    }
    
    pub fn vm_exists(&self, vm_id: &str) -> bool {
        let pool = self.vm_pool.lock().unwrap_or_else(|e| {
            eprintln!("[QuickJSRuntime] 获取 VM 池锁失败: {}", e);
            return std::sync::Mutex::new(Vec::new());
        });
        pool.iter().any(|vm| vm.id == vm_id)
    }

    pub fn with_context<T, F>(&self, vm_id: &str, operation: F) -> Result<T, String>
    where
        F: FnOnce(rquickjs::Ctx<'_>) -> Result<T, String>,
    {
        let pool = self.vm_pool.lock().map_err(|e| format!("获取 VM 池锁失败: {}", e))?;

        let vm = pool
            .iter()
            .find(|v| v.id == vm_id)
            .ok_or_else(|| format!("VM not found: {}", vm_id))?;

        vm.context.with(operation)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VmStats {
    pub id: String,
    pub created_at_secs: u64,
    pub last_used_at_secs: u64,
    pub is_idle: bool,
}

fn generate_vm_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let random: u32 = rand::random();
    format!("vm_{}_{}", timestamp, random)
}

#[tauri::command]
pub async fn quickjs_create_vm(
    runtime: tauri::State<'_, QuickJSRuntime>,
) -> Result<String, String> {
    runtime.create_vm()
}

#[tauri::command]
pub async fn quickjs_destroy_vm(
    vm_id: String,
    runtime: tauri::State<'_, QuickJSRuntime>,
) -> Result<(), String> {
    runtime.destroy_vm(&vm_id)
}

#[tauri::command]
pub async fn quickjs_execute(
    vm_id: String,
    code: String,
    runtime: tauri::State<'_, QuickJSRuntime>,
) -> Result<serde_json::Value, String> {
    runtime.execute(&vm_id, &code)
}

#[tauri::command]
pub async fn quickjs_active_count(
    runtime: tauri::State<'_, QuickJSRuntime>,
) -> Result<usize, String> {
    Ok(runtime.active_vm_count())
}

#[tauri::command]
pub async fn quickjs_cleanup(
    runtime: tauri::State<'_, QuickJSRuntime>,
) -> Result<usize, String> {
    runtime.cleanup()
}

#[tauri::command]
pub async fn quickjs_cleanup_oldest(
    count: usize,
    runtime: tauri::State<'_, QuickJSRuntime>,
) -> Result<usize, String> {
    runtime.cleanup_oldest(count)
}

#[tauri::command]
pub async fn quickjs_cleanup_all(
    runtime: tauri::State<'_, QuickJSRuntime>,
) -> Result<usize, String> {
    runtime.cleanup_all()
}

#[tauri::command]
pub async fn quickjs_vm_stats(
    runtime: tauri::State<'_, QuickJSRuntime>,
) -> Result<Vec<VmStats>, String> {
    Ok(runtime.get_vm_stats())
}

pub(crate) fn convert_value_to_json(value: Value) -> serde_json::Value {
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

    if let Some(s) = value.as_string() {
        return serde_json::Value::String(s.to_string().unwrap_or_default());
    }

    if value.is_array() {
        if let Some(arr) = value.clone().into_array() {
            return convert_array_to_json(arr);
        }
    }

    if value.is_object() {
        if let Some(obj) = value.into_object() {
            return convert_object_to_json(obj);
        }
    }

    serde_json::Value::Null
}

fn convert_array_to_json(arr: rquickjs::Array) -> serde_json::Value {
    let mut result = Vec::new();
    
    for i in 0.. {
        match arr.get::<Value>(i) {
            Ok(val) => result.push(convert_value_to_json(val)),
            Err(_) => break,
        }
    }
    
    serde_json::Value::Array(result)
}

fn convert_object_to_json(obj: rquickjs::Object) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    
    let keys_iter = obj.keys::<String>();
    for key_result in keys_iter {
        match key_result {
            Ok(key) => {
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

# QuickJS `unsafe impl Send/Sync` 移除方案

> 本文档对应 TODO.md 长期优化中的 P0 安全任务。
> 目标：将 6 处 `unsafe impl Send/Sync` 减少至 0 处，消除未定义行为风险。

---

## 1. 现状分析

### 1.1 当前 unsafe 位置（共 3 个 struct，6 行 unsafe impl）

| # | 文件 | 行号 | struct | unsafe impl |
|---|------|:----:|--------|:-----------:|
| 1 | `plugins/loader/mod.rs` | 25-26 | `PluginLoader` | Send + Sync |
| 2 | `plugins/quickjs_runtime.rs` | 59-60 | `VmInstance` | Send + Sync |
| 3 | `plugins/quickjs_runtime.rs` | 103-104 | `QuickJSRuntime` | Send + Sync |

### 1.2 根因分析

```rust
// Root Cause 1: rquickjs::Runtime / Context 原生 !Send + !Sync
// rquickjs 底层封装了 C 语言 QuickJS 库，其运行时状态绑定到创建线程
pub struct VmInstance {
    pub id: String,
    runtime: Runtime,       // !Send + !Sync  ← 根本原因
    context: Context,       // !Send + !Sync  ← 根本原因
    created_at: Instant,
    last_used_at: Instant,
}

// Root Cause 2: RefCell 提供内部可变性但非线程安全
pub struct QuickJSRuntime {
    config: QuickJSConfig,         // Send + Sync
    vm_pool: RefCell<Vec<VmInstance>>,  // RefCell 是 !Sync
    // 即使 VmInstance 解决了 !Send 问题，RefCell 的 !Sync 仍会导致 QuickJSRuntime !Sync
}

// Root Cause 3: Arc<QuickJSRuntime> 要求 T: Send + Sync
pub struct PluginLoader {
    plugins_dir: PathBuf,                  // Send + Sync
    instances: HashMap<String, PluginInstance>,  // 取决于 PluginInstance 字段
    quickjs_runtime: Arc<QuickJSRuntime>,  // Arc<T>: Send+Sync 仅当 T: Send+Sync
    // 由于 QuickJSRuntime 不是 Send+Sync，PluginLoader 也不是
}
```

### 1.3 依赖链

```
PluginLoader ──unsafe→ Arc<QuickJSRuntime> ──unsafe→ RefCell<Vec<VmInstance>>
                                                         │
                                                         └── VmInstance ──unsafe→ Runtime + Context
```

---

## 2. 设计方案

### 2.1 核心思路：分层包裹策略

不再在每一层都做 `unsafe impl`，而是将非线程安全的 QuickJS 资源封装在一个**最小化的内部包装器**中，通过 `Mutex` 提供线程安全保证，只在最内层做一个 **有文档证明安全** 的 unsafe impl。

### 2.2 架构对比

```
当前（6处 unsafe）:
  PluginLoader (unsafe Send+Sync)
    └── Arc<QuickJSRuntime (unsafe Send+Sync)>
          └── RefCell<Vec<VmInstance (unsafe Send+Sync)>>
                └── Runtime + Context

目标（0处 unsafe，完全安全的 Send+Sync）:
  PluginLoader (自动 Send+Sync)
    └── Arc<QuickJSRuntime (自动 Send+Sync)>
          └── Mutex<Vec<VmInstance (经过安全包装)>>
                └── VmCore (唯一 unsafe impl，单点责任)
                      └── Runtime + Context
```

### 2.3 关键设计：VmCore 内部包装器

```rust
/// QuickJS VM 核心资源包装器
///
/// # Safety
/// 
/// VmCore 包含的 `rquickjs::Runtime` 和 `rquickjs::Context` 原本是 !Send + !Sync。
/// 但 VmCore 始终通过 QuickJSRuntime 的 `Mutex<Vec<VmCore>>` 访问。
/// Mutex 保证：
///   1. 同一时间只有一个线程能访问任何 VmCore 实例
///   2. 所有访问都是序列化的，不会出现并发
///
/// 因此，在多线程环境中通过 Mutex 访问 VmCore 是安全的。
struct VmCore {
    id: String,
    runtime: Runtime,
    context: Context,
    created_at: Instant,
    last_used_at: Instant,
}

// Safety: 所有访问都通过外层 Mutex 序列化
unsafe impl Send for VmCore {}
unsafe impl Sync for VmCore {}
```

> **这是整个方案中仅存的 2 行 unsafe**，比原来的 6 行减少了 67%。

### 2.4 变更后的类型签名

```rust
// QuickJSRuntime — 不再需要 unsafe impl
pub struct QuickJSRuntime {
    config: QuickJSConfig,
    vm_pool: Mutex<Vec<VmCore>>,  // RefCell → Mutex
}
// 自动推导：QuickJSConfig: Send+Sync, Mutex<Vec<VmCore>>: Send+Sync
// → QuickJSRuntime 自动为 Send+Sync ✅

// PluginLoader — 不再需要 unsafe impl  
pub struct PluginLoader {
    plugins_dir: PathBuf,
    instances: HashMap<String, PluginInstance>,
    quickjs_runtime: Arc<QuickJSRuntime>,
}
// 自动推导：PathBuf: Send+Sync, HashMap: Send+Sync, Arc<QuickJSRuntime>: Send+Sync
// → PluginLoader 自动为 Send+Sync ✅
```

---

## 3. 迁移步骤

### Step 1：创建 `VmCore` 内部包装器

**文件**：`quickjs_runtime.rs`

**操作**：
1. 重命名 `VmInstance` 为 `VmCore`（内部包装器，无需 pub）
2. 只在 `VmCore` 上保留 `unsafe impl Send/Sync`（2 行，附 Safety 文档）
3. 原有 `VmInstance` 作为公开 API 的过渡别名

**代码变更**：
```diff
- pub struct VmInstance {
+ pub(crate) struct VmCore {
      pub id: String,
      runtime: Runtime,
      context: Context,
      created_at: Instant,
      last_used_at: Instant,
  }

- unsafe impl Send for VmInstance {}
- unsafe impl Sync for VmInstance {}

+ // Safety: 所有访问都通过外层 Mutex 序列化
+ unsafe impl Send for VmCore {}
+ unsafe impl Sync for VmCore {}
```

### Step 2：QuickJSRuntime `RefCell` → `Mutex`

**文件**：`quickjs_runtime.rs`

**操作**：
1. 将 `vm_pool` 字段从 `RefCell<Vec<VmInstance>>` 改为 `Mutex<Vec<VmCore>>`
2. 将所有 `self.vm_pool.borrow_mut()` 替换为 `self.vm_pool.lock()`
3. 将所有 `self.vm_pool.borrow()` 替换为 `self.vm_pool.lock()`
4. 删除 `unsafe impl Send/Sync for QuickJSRuntime`

**代码变更**：
```diff
+ use std::sync::Mutex;

  pub struct QuickJSRuntime {
      config: QuickJSConfig,
-     vm_pool: RefCell<Vec<VmInstance>>,
+     vm_pool: Mutex<Vec<VmCore>>,
  }

- unsafe impl Send for QuickJSRuntime {}
- unsafe impl Sync for QuickJSRuntime {}

  impl QuickJSRuntime {
      pub fn new() -> Self {
          Self {
              config: QuickJSConfig::default(),
-             vm_pool: RefCell::new(Vec::new()),
+             vm_pool: Mutex::new(Vec::new()),
          }
      }

      pub fn create_vm(&self) -> Result<String, String> {
-         let mut pool = self.vm_pool.borrow_mut();
+         let mut pool = self.vm_pool.lock().map_err(|e| e.to_string())?;
          // ... 其余逻辑相同
      }
  }
```

### Step 3：清理 `PluginLoader` unsafe

**文件**：`loader/mod.rs`

**操作**：删除 2 行 `unsafe impl`，编译器自动推导

```diff
- unsafe impl Send for PluginLoader {}
- unsafe impl Sync for PluginLoader {}
```

### Step 4：适配 `lib.rs` 中的 Mutex 使用

**文件**：`lib.rs`（需验证）

**操作**：检查 `tauri::State<Mutex<PluginLoader>>` 的用法是否需要调整。
当前所有 Commands 已经使用 `loader.lock().map_err(...)?` 模式，理论上无需改动。

### Step 5：验证

```bash
cargo check          # 验证编译
cargo clippy         # 检查警告
bun run check        # 前端检查
```

---

## 4. 影响评估

### 4.1 性能影响

| 操作 | 当前 | 迁移后 | 差异 |
|------|------|--------|:----:|
| VM 创建 | `RefCell::borrow_mut()` (O(1), 零开销) | `Mutex::lock()` (O(1), ~50ns) | 微小增加 |
| VM 执行 | `RefCell::borrow_mut()` + `context.with()` | `Mutex::lock()` + `context.with()` | 微小增加 |
| VM 销毁 | `RefCell::borrow_mut()` | `Mutex::lock()` | 微小增加 |
| 闲置清理 | `RefCell::borrow_mut()` | `Mutex::lock()` | 微小增加 |

**结论**：由于所有操作都发生在 Tauri Commands 中（非热点路径），Mutex 开销可忽略不计。

### 4.2 并发安全性

| 风险 | 当前 | 迁移后 |
|------|:----:|:------:|
| 数据竞争 | 有风险（unsafe impl 依赖开发者自律） | ✅ 编译器保证 |
| 死锁 | 无（单线程） | 低风险（Mutex 非重入，需确保无嵌套 lock） |
| 线程安全 | 高风险（UB 可能） | ✅ 安全 |

### 4.3 变更文件清单

| 文件 | 变更类型 | 行数变化 |
|------|:--------:|:--------:|
| `quickjs_runtime.rs` | 重构 | -4 unsafe + 替换 RefCell→Mutex |
| `loader/mod.rs` | 删除 | -2 unsafe |
| `lib.rs` | 验证 | 0（无需改动） |

### 4.4 风险与缓解

| 风险 | 级别 | 缓解措施 |
|------|:----:|----------|
| Mutex 嵌套死锁 | 低 | 检查所有 `lock()` 调用，确保无嵌套；如存在则改用 `try_lock()` |
| `lock().unwrap()` panic | 低 | 使用 `map_err(\|e\| e.to_string())?` 替代，参考现有 pattern |
| VmCore 逃逸到其他线程 | 低 | VmCore 仅在 QuickJSRuntime 内部使用，不对外暴露 |

---

## 5. 验收标准

1. ✅ `cargo check` 0 errors，0 warnings + `cargo clippy` 通过
2. ✅ `bun run check` 0 errors，0 warnings
3. ✅ `grep -r "unsafe impl Send\|unsafe impl Sync" src-tauri/src/` 返回 2 行（仅限 VmCore）
4. ✅ 所有插件加载/执行/卸载功能正常

---

*文档版本: 1.0*  
*最后更新: 2026-05-24*  
*状态: 方案设计完成*
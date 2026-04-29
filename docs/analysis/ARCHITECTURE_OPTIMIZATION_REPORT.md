# Corelia 架构优化分析报告

> 版本: 1.0 | 生成日期: 2026-04-29 | 范围: MVP核心框架阶段

---

## 目录

1. [执行摘要](#1-执行摘要)
2. [当前架构概览](#2-当前架构概览)
3. [性能瓶颈分析](#3-性能瓶颈分析)
4. [内存安全问题](#4-内存安全问题)
5. [并发安全分析](#5-并发安全分析)
6. [模块化重构建议](#6-模块化重构建议)
7. [前端架构优化](#7-前端架构优化)
8. [优化实施路径](#8-优化实施路径)
9. [风险与缓解](#9-风险与缓解)

---

## 1. 执行摘要

### 当前状态

After comprehensive codebase analysis (`cargo clippy`, `bun run check`, deep code review), Corelia's plugin system backend is functionally complete with **clean compilation** (0 errors). The frontend has **0 type errors** with 22 minor warnings (a11y/CSS compatibility).

### 关键发现

| 维度 | 评级 | 说明 |
|------|:----:|------|
| 功能完整性 | A | 后端核心模块完整，前端基础组件就绪 |
| 代码质量 | B+ | Rust后端良好，前端有22个Svelte警告待修复 |
| 性能优化空间 | B | VM池化已实施，但存在多处可优化点 |
| 安全性 | B+ | 沙箱隔离到位，存在`unsafe impl`使用需谨慎 |
| 模块化程度 | B | 后端结构清晰，前端Store耦合度偏高 |

### 核心问题速览

1. **QuickJS `unsafe impl Send/Sync`** — 线程安全假设需文档化验证
2. **VM双重缓存** — 前端`service.ts`和后端`loader.rs`各自缓存VM，可能不一致
3. **WASM桥接轮询模式** — 同步轮询获取异步结果，效率低下
4. **搜索串行执行** — `performPluginSearch`串行调用多个插件的`onSearch`
5. **前端非反应式状态** — `vmCacheStatus`未使用`$state`声明

---

## 2. 当前架构概览

### 2.1 整体架构

```
┌────────────────────────────────────────────────────────┐
│  Frontend (Svelte 5 + TypeScript)                      │
│  ├─ Components: SearchBox, ResultList, SettingPanel    │
│  ├─ Stores: searchStore, theme, history, user, system  │
│  ├─ Services: executor, pluginService, patchLoader     │
│  └─ Search Engine: fuzzy.ts (with pinyin support)      │
│                      │                                  │
│              invoke / listen / emit                     │
│                      │                                  │
├──────────────────────┼──────────────────────────────────┤
│  Tauri IPC Bridge    │                                  │
├──────────────────────┼──────────────────────────────────┤
│                      │                                  │
│  Rust Backend        │                                  │
│  ├─ quickjs_runtime  │ VM pool (max 10, idle 300s)      │
│  ├─ api_bridge       │ utools API injection (11 modules)│
│  ├─ loader           │ Plugin lifecycle management      │
│  ├─ registry         │ Dual-indexed plugin registry     │
│  ├─ wasm_bridge      │ WASM IPC bridge                  │
│  ├─ window_service   │ Window show/hide/toggle          │
│  └─ commands         │ Tauri command handlers           │
└────────────────────────────────────────────────────────┘
```

### 2.2 模块依赖图

```
lib.rs
├── plugins/
│   ├── mod.rs           (exports all)
│   ├── quickjs_runtime  ←──┐
│   ├── api_bridge       ←──┼── loader depends on both
│   ├── loader           ←──┘
│   ├── registry         (standalone)
│   └── wasm_bridge      (standalone)
├── commands/ (window, clipboard, shell, config, store, autostart, shortcut, plugin)
├── services/ (window_service, config_service)
└── error.rs
```

### 2.3 前端数据流

```
User Input → SearchBox
    │
    ├──→ searchStore.setQuery() ──┐
    │                              │
    ├──→ fuzzy search (系统项)     │ (并行的两条路径)
    │                              │
    └──→ pluginService.searchByPrefix() ──→ scan_plugins()
                                              │
                                              ├──→ find_plugins_by_prefix()
                                              │       │
                                              │       └──→ 对每个匹配插件：
                                              │               executeSearch() → onSearch()
                                              │
                                              └──→ mergeResults()
                                                          │
                                                          ↓
                                              ResultList.render()
                                                          │
                                              User Select → resultExecutor.execute()
                                                              │
                                                              ├──→ 系统项: invoke Tauri command
                                                              └──→ 插件项: pluginService.executeAction()
```

---

## 3. 性能瓶颈分析

### 3.1 高优先级瓶颈

#### 3.1.1 VM双重管理与缓存不一致

**位置**: `src/lib/plugins/service.ts` + `src-tauri/src/plugins/loader.rs`

**问题描述**: 
前端`PluginService`维护独立的VM缓存池 (`vmCache: Map<string, VmCacheEntry>`)，而后端`PluginLoader`也持有`vm_id`。两者可能不同步：

```typescript
// service.ts: 前端缓存
private vmCache: Map<string, VmCacheEntry> = new Map();

// loader.rs: 后端也缓存vm_id
pub vm_id: Option<String>
```

**影响**: 前端可能尝试复用已被后端清理的VM，导致`"VM not found"`错误。

**优化建议**: 
- 移除前端VM缓存，改为纯代理模式（信任后端VM管理）
- 或使用事件机制同步VM状态变化

#### 3.1.2 搜索串行执行

**位置**: `src/lib/stores/search.ts:98-111`

**问题代码**:
```typescript
for (const plugin of matchedPlugins) {
  try {
    const results = await pluginService.executeSearch(plugin.name, query);
    // 串行执行！
  }
}
```

**影响**: 如果匹配5个插件，每个插件搜索耗时100ms，总耗时500ms。

**优化建议**: 
```typescript
// 并行执行所有插件搜索
const results = await Promise.allSettled(
  matchedPlugins.map(p => pluginService.executeSearch(p.name, query))
);
```

#### 3.1.3 WASM轮询模式低效

**位置**: `src-tauri/src/plugins/api_bridge.rs` (WASM API注入)

**问题描述**: WASM调用采用轮询模式：`__wasm_call`返回requestId，然后通过`__wasm_get_result`轮询获取结果。

**影响**: 在异步场景下（如文件加密），QuickJS需要主动轮询，浪费CPU。

**优化建议**: 
- 短期：增加退避策略（从10ms开始，指数增长到100ms上限）
- 长期：改用Promise-based API，利用rquickjs的Promise支持（如果可行）

### 3.2 中优先级瓶颈

#### 3.2.1 fetch API同步阻塞

**位置**: `src-tauri/src/plugins/api_bridge.rs:454-512`

**问题代码**:
```rust
let result = std::thread::Builder::new()
    .spawn(move || -> FetchResult { ... })
    .join()  // 阻塞等待！
```

**影响**: 每个HTTP请求创建OS线程（较重），且`.join()`阻塞QuickJS执行。

**优化建议**: 
- 使用`tokio`或`async`运行时处理异步IO
- 或限制并发线程数，使用线程池

#### 3.2.2 拼音搜索冗余计算

**位置**: `src/lib/search/fuzzy.ts:12-19`

**问题代码**:
```typescript
const queryPinyin = pinyin(query, { toneType: 'none' }).replace(/\s+/g, '');
// 每次搜索都计算拼音，即使query是英文
```

**优化建议**: 
- 缓存拼音转换结果
- 先判断query是否包含中文字符，无则跳过拼音转换

#### 3.2.3 WindowService状态单点

**位置**: `src-tauri/src/services/window_service.rs:12`

**问题代码**:
```rust
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);
```

**问题**: 原子状态与Tauri实际窗口状态可能不同步（如用户点击托盘或其他程序调用了窗口API）。

**优化建议**: 
- 每次查询时直接调用`window.is_visible()`，不依赖缓存状态
- 或使用Tauri的事件监听窗口状态变化

### 3.3 低优先级瓶颈

| 问题 | 位置 | 优化建议 |
|------|------|----------|
| 插件目录IO每次调用 | `loader.rs:scan_plugins()` | 增加文件系统监听/缓存mtime |
| clipboard每次新建实例 | `api_bridge.rs:166-172` | 使用全局Clipboard实例（但注意线程安全） |
| history每次save全量写入 | `stores/history.ts:70` | 增量写入或批量合并 |
| 通知使用powershell | `api_bridge.rs:320-323` | 使用tauri-plugin-notification |

---

## 4. 内存安全问题

### 4.1 QuickJS VM `unsafe impl Send/Sync`

**位置**: `src-tauri/src/plugins/quickjs_runtime.rs:62-65`

**问题代码**:
```rust
// 安全性说明：VmInstance 只在主线程中通过 Tauri State 访问
unsafe impl Send for VmInstance {}
unsafe impl Sync for VmInstance {}
```

**风险评估**: 
- **当前风险: 低**。Tauri Commands确实在主线程执行，且使用了`RefCell`（非线程安全）作为哨兵。
- **未来风险: 中**。如果以后引入多线程Command（Tauri支持），此假设失效。

**缓解措施**:
1. 在文档中明确标注此设计决策的约束
2. 考虑使用`std::sync::Mutex`替代`RefCell`，增加运行时检查
3. 添加`#[cfg(debug_assertions)]`断言检测跨线程访问

### 4.2 PluginLoader `unsafe impl Send/Sync`

**位置**: `src-tauri/src/plugins/loader.rs:153-154`

```rust
unsafe impl Send for PluginLoader {}
unsafe impl Sync for PluginLoader {}
```

**问题**: `PluginLoader`包含`HashMap`（非线程安全）和`Arc<QuickJSRuntime>`。

**缓解措施**: 同上，建议使用`Mutex<HashMap>`增加安全性。

### 4.3 VM生命周期与内存泄漏

**当前防护**:
- `QuickJSRuntime::cleanup()` 清理闲置VM（300秒超时）
- `PluginLoader::unload_plugin()` 销毁关联VM
- `PluginLoader::cleanup_idle_plugins()` 外部触发清理

**潜在泄漏场景**:
1. 插件加载失败（Error状态）后VM未正确销毁
2. 应用崩溃时VM未清理（`cleanup_all`仅在正常退出时调用）
3. 前端`service.ts`缓存VM但后端已销毁

**优化建议**:
- 增加`Drop`实现确保VM释放
- 添加VM健康检查定时任务
- 实现VM引用计数或弱引用

### 4.4 WASM结果缓存无限增长

**位置**: `src-tauri/src/plugins/wasm_bridge.rs:118-128`

```rust
if self.call_results.len() > 1000 {
    // 清除最旧的一半结果
}
```

**问题**: 缓存淘汰策略基于数量而非内存大小，恶意插件可填充大量结果数据。

**优化建议**: 增加基于内存大小的限制，或添加TTL过期时间。

---

## 5. 并发安全分析

### 5.1 State管理概览

| State | 类型 | 线程安全 | 风险 |
|-------|------|----------|------|
| `QuickJSRuntime` | 直接State | `RefCell` + unsafe | 低（主线程） |
| `PluginLoader` | `Mutex<>` | `unsafe impl` | 低（锁保护） |
| `PluginRegistry` | `RwLock<>` | 标准 | 低 |
| `WasmBridge` | `Mutex<>` | 标准 | 低 |

### 5.2 锁粒度分析

#### 5.2.1 PluginLoader Mutex

`Mutex<PluginLoader>`保护整个加载器。所有插件操作串行化。

**瓶颈场景**: 多个插件同时被搜索或加载时，会产生锁竞争。

**优化建议**: 
- 细粒度锁：按插件ID分片锁定
- 读多写少用`RwLock`替代`Mutex`

#### 5.2.2 ApiBridge AppHandle访问

```rust
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
```

`OnceLock`是线程安全的，但`AppHandle`的某些操作（如emit）可能有内部锁。

**问题**: `inject_utools`中每次API调用都通过`get_app_handle()`获取，如果Tauri的事件分发有锁，可能形成隐式瓶颈。

**优化建议**: 在频繁调用的场景中缓存`AppHandle`引用。

### 5.3 死锁风险

| 场景 | 风险等级 | 说明 |
|------|:--------:|------|
| 插件A调用`fetch`同步等待 → `fetch`尝试获取VM | 低 | 当前是新建线程，未复用VM |
| `unload_plugin`持有PluginLoader锁 → 尝试destroy_vm | 低 | destroy_vm是独立操作 |
| WASM call循环依赖 | 中 | WASM调用Rust API，API又触发WASM |

---

## 6. 模块化重构建议

### 6.1 后端模块拆分

#### 6.1.1 api_bridge.rs 拆分（当前42.5KB）

当前`api_bridge.rs`包含11个API模块的注入函数，文件过大。

**建议拆分**:
```
api_bridge/
├── mod.rs           (ApiBridge入口 + inject_utools)
├── db_storage.rs    (utools.dbStorage)
├── clipboard.rs     (utools.clipboard)
├── shell.rs         (utools.shell)
├── window.rs        (utools hideMainWindow/showMainWindow)
├── path.rs          (utools.getPath)
├── notification.rs  (utools.showNotification)
├── fs.rs            (utools.fs)
├── fetch.rs         (utools.fetch)
├── dialog.rs        (utools.dialog)
├── process.rs       (utools.process)
├── context.rs       (utools.getContext/setContext)
└── wasm.rs          (utools.wasm)
```

#### 6.1.2 loader.rs 拆分（当前34.8KB）

```
loader/
├── mod.rs           (PluginLoader核心)
├── manifest.rs      (PluginManifest/FeatureConfig定义)
├── state.rs         (PluginState/状态机)
├── instance.rs      (PluginInstance)
├── scanner.rs       (scan_plugins逻辑)
├── lifecycle.rs     (load_plugin/unload_plugin)
└── health.rs        (PluginHealth监控)
```

### 6.2 前端Store重构

#### 6.2.1 搜索Store解耦

当前`searchStore`耦合了系统搜索、插件搜索和结果合并。

**建议拆分**:
```
stores/
├── search/
│   ├── index.ts       (SearchOrchestrator)
│   ├── system.ts      (系统项搜索)
│   ├── plugin.ts      (插件搜索)
│   └── merger.ts      (结果合并排序)
├── theme.ts
├── history.ts
├── system.ts
├── user.ts
└── app.ts
```

#### 6.2.2 消除前端VM缓存

`service.ts`中的VM缓存与后端重复，建议移除：

```typescript
// 移除以下内容:
// private vmCache: Map<string, VmCacheEntry>
// private getOrCreateVm()
// private evictOldestVm()

// 改为直接调用后端API，由后端管理VM生命周期
```

### 6.3 服务层重构

#### 6.3.1 executor.ts 职责过重

当前`ResultExecutor`处理了：
- 系统应用启动
- URL打开
- 路径打开
- 命令执行
- 设置打开
- 插件执行
- 历史记录
- 窗口隐藏

**建议拆分**:
```
services/
├── executor/
│   ├── index.ts      (分发器)
│   ├── system.ts     (app/url/path/command)
│   ├── setting.ts    (设置项)
│   └── plugin.ts     (插件执行)
├── history.ts        (历史记录管理)
└── window.ts         (窗口管理)
```

---

## 7. 前端架构优化

### 7.1 当前警告修复

`bun run check`输出22个警告，主要集中在：

| 文件 | 警告数量 | 类型 | 修复优先级 |
|------|:--------:|------|:----------:|
| PluginManager.svelte | 18 | a11y + CSS + reactivity | P2 |
| TitleBar.svelte | 4 | CSS兼容性 | P3 |

**具体修复**:

1. **`vmCacheStatus`非反应式** (PluginManager.svelte:43)
   ```svelte
   <!-- 修复前 -->
   let vmCacheStatus = pluginService.getCacheStatus();
   
   <!-- 修复后 -->
   let vmCacheStatus = $state(pluginService.getCacheStatus());
   ```

2. **label缺少关联control** (PluginManager.svelte:436)
   ```svelte
   <label for="result">执行结果</label>
   <pre id="result">{testActionResult}</pre>
   ```

3. **CSS line-clamp兼容性** (PluginManager.svelte:633)
   ```css
   .plugin-description {
     display: -webkit-box;
     -webkit-line-clamp: 1;
     -webkit-box-orient: vertical;
     line-clamp: 1; /* 标准属性 */
     overflow: hidden;
   }
   ```

### 7.2 Svelte 5 Runes迁移

部分组件仍使用Svelte 4的写法：

- `theme.ts`使用`writable`而非`$state` — 建议迁移到Svelte 5 Runes
- `history.ts`使用`writable` — 同样可迁移

### 7.3 搜索性能优化

当前fuzzy搜索每次构建拼音索引：

```typescript
// 建议: 预构建索引
const pinyinIndex = new Map<string, string>();

function buildIndex(items: SearchItem[]) {
  items.forEach(item => {
    const namePinyin = pinyin(item.name, { toneType: 'none' }).replace(/\s+/g, '');
    pinyinIndex.set(item.id, `${item.name} ${namePinyin} ${item.description}`);
  });
}
```

---

## 8. 优化实施路径

### 8.1 短期优化（1-2周）

| # | 任务 | 影响 | 工作量 |
|---|------|:----:|:------:|
| 1 | 修复前端22个Svelte警告 | 代码质量 | 2h |
| 2 | 插件搜索并行化 | 性能↑ | 2h |
| 3 | 移除前端VM缓存 | 简化+可靠性↑ | 4h |
| 4 | WASM轮询退避策略 | 性能↑ | 2h |
| 5 | 拼音搜索短路优化 | 性能↑ | 1h |

### 8.2 中期优化（2-4周）

| # | 任务 | 影响 | 工作量 |
|---|------|:----:|:------:|
| 6 | api_bridge.rs模块化拆分 | 可维护性↑ | 8h |
| 7 | loader.rs模块化拆分 | 可维护性↑ | 6h |
| 8 | 搜索Store解耦重构 | 可测试性↑ | 6h |
| 9 | executor服务拆分 | 可维护性↑ | 4h |
| 10 | Store迁移至Svelte 5 Runes | 现代化 | 8h |

### 8.3 长期优化（1-2月）

| # | 任务 | 影响 | 工作量 |
|---|------|:----:|:------:|
| 11 | QuickJS `unsafe impl`移除 | 安全性↑↑ | 12h |
| 12 | 多线程VM支持 | 性能↑↑ | 16h |
| 13 | WASM Promise异步支持 | 性能↑↑ | 12h |
| 14 | 插件热重载 | 开发体验↑ | 8h |
| 15 | 增量搜索索引 | 性能↑ | 10h |

### 8.4 依赖关系

```
短期:
  修复警告 (1)
    → Store迁移 (10) [依赖]
  
  搜索并行化 (2)
    └── 搜索Store解耦 (8) [建议前置]

中期:
  api_bridge拆分 (6)
    └── loader拆分 (7) [独立]
    └── executor拆分 (9) [独立]

长期:
  unsafe移除 (11)
    └── 多线程VM (12) [依赖]
  
  WASM Promise (13)
    └── wasm_bridge重构 [依赖部分中期]
```

---

## 9. 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|:----:|:----:|----------|
| `unsafe impl`在多线程场景下失效 | 低 | 高 | 文档化约束，加断言检测，逐步迁移到Mutex |
| VM缓存不一致导致运行时错误 | 中 | 中 | 短期移除前端缓存，长期统一状态管理 |
| 模块化拆引入regression | 低 | 中 | 保持现有测试覆盖，分阶段迁移 |
| rquickjs异步支持不足阻碍Promise方案 | 高 | 中 | 保持轮询回退，关注上游更新 |
| Svelte 5 Runes迁移破坏现有逻辑 | 低 | 低 | 渐进式迁移，先Store后组件 |

---

## 附录A: 代码质量指标

### 后端 (Rust)

| 指标 | 数值 | 状态 |
|------|:----:|:----:|
| `cargo check` | 0 errors | ✅ |
| `cargo clippy` | 0 warnings | ✅ |
| `unsafe`使用处 | 3处 (`Send`/`Sync` impl) | ⚠️ |
| 核心模块文件大小 | 最大42.5KB | ⚠️ |
| 模块数量 | 5个核心插件模块 | ✅ |

### 前端 (TypeScript/Svelte)

| 指标 | 数值 | 状态 |
|------|:----:|:----:|
| `bun run check` | 0 errors, 22 warnings | ⚠️ |
| 组件数量 | 8个 | ✅ |
| Store数量 | 6个 | ✅ |
| 服务模块 | 5个 | ✅ |

---

*报告生成: 2026-04-29*  
*工具链: cargo clippy, svelte-check, ripgrep, manual review*  
*覆盖范围: src-tauri/src/plugins/*, src/lib/{plugins,stores,services}/**, src/routes/+page.svelte*
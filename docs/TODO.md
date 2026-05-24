# Corelia 当前阶段任务清单 (TODO)

> 本文档聚焦于 **MVP 核心框架阶段**（阶段一）的当前任务。
> 更新频率：每周评审，任务完成后归档。
>
> 另有架构优化报告见 [`analysis/ARCHITECTURE_OPTIMIZATION_REPORT.md`](analysis/ARCHITECTURE_OPTIMIZATION_REPORT.md)

---

## 📋 架构优化未完成任务（来自优化报告）

> 短期优化（1-2周）项已全部完成 ✅，以下为中期和长期待办项。

### 中期优化（2-4周）

| 任务 | 状态 | 优先级 | 影响 | 关联文件 |
|------|:----:|:------:|:----:|----------|
| `api_bridge.rs` 模块化拆分（已拆分为 15 个子模块：shared/storage/clipboard/shell/window/path/notification/fs/callbacks/fetch/dialog/process/context/wasm + mod.rs） | ✅ | P0 | 可维护性↑ | [`api_bridge/`](src-tauri/src/plugins/api_bridge/) |
| `loader.rs` 模块化拆分（已拆分为 7 个子模块：types/scanner/lifecycle/query/cleanup/commands + mod.rs） | ✅ | P1 | 可维护性↑ | [`loader/`](src-tauri/src/plugins/loader/) |
| 搜索 Store 解耦重构（拆分为system/plugin/merger独立模块） | ✅ | P1 | 可测试性↑ | [`search/`](src/lib/stores/search/) |
| `executor` 服务拆分（职责过重，拆为system/setting/plugin） | ✅ | P2 | 可维护性↑ | [`executor/`](src/lib/services/executor/) |
| Store 迁移至 Svelte 5 Runes（theme.ts/history.ts 使用 writable） | ✅ | P2 | 现代化 | [`theme.svelte.ts`](src/lib/stores/theme.svelte.ts), [`history.svelte.ts`](src/lib/stores/history.svelte.ts) |

### 长期优化（1-2月）

| 任务 | 状态 | 优先级 | 影响 | 关联文件 |
|------|:----:|:------:|:----:|----------|
| QuickJS `unsafe impl Send/Sync` 移除（已执行方案：6处→2处，仅VmCore保留2行有文档的unsafe） | ✅ | P0 | 安全性↑↑ | [`quickjs_runtime.rs`](src-tauri/src/plugins/quickjs_runtime.rs), [`loader/mod.rs`](src-tauri/src/plugins/loader/mod.rs) |
| 多线程 VM 支持 | ⬜ | P1 | 性能↑↑ | [`quickjs_runtime.rs`](src-tauri/src/plugins/quickjs_runtime.rs) |
| WASM Promise 异步支持（替代轮询方案） | ⬜ | P1 | 性能↑↑ | [`wasm_bridge.rs`](src-tauri/src/plugins/wasm_bridge.rs), [`api_bridge.rs`](src-tauri/src/plugins/api_bridge.rs) |
| 插件热重载 | ⬜ | P2 | 开发体验↑ | 新增模块 |
| 增量搜索索引（预构建拼音索引，避免每次重建） | ⬜ | P2 | 性能↑ | [`fuzzy.ts`](src/lib/search/fuzzy.ts) |

### 低优先级性能项（文档3.3节）

| 任务 | 状态 | 优先级 | 说明 |
|------|:----:|:------:|------|
| 插件目录 IO 缓存（监听文件系统 mtime） | ⬜ | P2 | `loader.rs:scan_plugins()` 每次调用都读取目录 |
| Clipboard 全局实例复用 | ⬜ | P3 | `api_bridge.rs:166` 每次新建实例 |
| History 增量写入 | ⬜ | P3 | `history.ts:70` 每次全量写入 |
| 通知改用 tauri-plugin-notification | ⬜ | P3 | 替代当前 powershell 方案 |

---

## 当前冲刺目标

**窗口**：2026-05 ~ 2026-06（代码质量即时修复已完成）  
**目标**：完成核心功能闭环后，启动架构优化与代码质量提升。

---

## 核心任务

### 1. 插件系统完善 🔧

| 任务 | 状态 | 优先级 | 说明 | 关联文件 |
|------|:----:|:------:|------|----------|
| 插件状态机鲁棒性强化 | ✅ | P0 | 已实现重试退避（指数退避1000ms→30000ms）、错误计数、Error→Ready自动恢复 | [`loader.rs`](src-tauri/src/plugins/loader.rs) |
| 插件 VM 生命周期监控 | ✅ | P0 | VM 池闲置超时自动清理（默认300s），cleanup_idle_plugins() 定期触发 | [`quickjs_runtime.rs`](src-tauri/src/plugins/quickjs_runtime.rs) |
| 插件错误隔离与上报 | ✅ | P0 | 单插件执行异常不影响主程序，含 load_error_count + retry_after 机制 | [`loader.rs`](src-tauri/src/plugins/loader.rs), [`registry.rs`](src-tauri/src/plugins/registry.rs) |
| 插件热重载实现 | ⬜ | P1 | 监听 `plugins/` 目录变化，自动重新加载 | 新增模块 |
| patch-loader 完善错误处理 | ✅ | P1 | WASM 加载失败三级降级（动态import → initSync → fetch+WebAssembly API），错误上报 | [`patch-loader.ts`](src/lib/plugins/patch-loader.ts) |

### 2. 示例插件开发与验证 🧩

| 任务 | 状态 | 优先级 | 说明 | 关联路径 |
|------|:----:|:------:|------|----------|
| `hello-world` 插件验证 | ✅ | P0 | 最简插件，测试基础生命周期 | [`plugins/hello-world/`](plugins/hello-world/) |
| `calc` 计算器插件完善 | ✅ | P0 | 支持复杂表达式、错误提示、历史记录 | [`plugins/calc/`](plugins/calc/) |
| `url-toolkit` URL 工具插件 | ✅ | P1 | URL 解析、编码/解码、参数提取 | [`plugins/url-toolkit/`](plugins/url-toolkit/) |
| `file-search` 文件搜索插件 | ✅ | P1 | 本地文件快速索引与搜索 | [`plugins/file-search/`](plugins/file-search/) |
| 剪贴板增强插件开发 | ⬜ | P1 | 剪贴板历史、搜索、快速粘贴 | 新建 `plugins/clipboard/` |

### 3. 搜索与 UI 体验 🔍

| 任务 | 状态 | 优先级 | 说明 | 关联文件 |
|------|:----:|:------:|------|----------|
| 搜索响应性能优化 | ✅ | P0 | Promise.allSettled 并行搜索 + 拼音缓存，模糊匹配 < 50ms | [`fuzzy.ts`](src/lib/search/fuzzy.ts) |
| 搜索结果分类展示 | ✅ | P1 | 区分系统/插件/文件/应用，图标+颜色分类 | [`ResultList.svelte`](src/lib/components/ResultList.svelte) |
| 快捷键设置面板 | ✅ | P1 | ShortcutRecorder 组件可视化修改全局唤起快捷键 | [`SettingPanel.svelte`](src/lib/components/SettingPanel.svelte) |
| 主题切换实时生效 | ✅ | P1 | 深色/浅色/跟随系统，已迁移至 Svelte 5 Runes | [`theme.svelte.ts`](src/lib/stores/theme.svelte.ts), [`themes.css`](src/lib/styles/themes.css) |
| 窗口失焦自动隐藏 | ✅ | P0 | 透明窗口失去焦点后自动隐藏 | [`window_service.rs`](src-tauri/src/services/window_service.rs) |

### 4. 配置与数据持久化 💾

| 任务 | 状态 | 优先级 | 说明 | 关联文件 |
|------|:----:|:------:|------|----------|
| 三层配置系统对接前端 | ✅ | P0 | System / User / App 配置可读写，前端已接入 | [`config/`](src-tauri/src/commands/config/) |
| 插件数据隔离存储 | ✅ | P0 | 每个插件独立的 `dbStorage` 空间，api.store 命名空间隔离 | [`api_bridge.rs`](src-tauri/src/plugins/api_bridge.rs) |
| 搜索历史持久化 | ✅ | P1 | 历史记录通过 api.store 保存到 App Config | [`history.ts`](src/lib/stores/history.ts) |
| 配置重置功能 | ✅ | P2 | User Config 可一键恢复默认值（设置面板「通用」→「重置」按钮 + 确认弹窗） | [`user.ts`](src/lib/stores/user.ts), [`SettingPanel.svelte`](src/lib/components/SettingPanel.svelte) |

### 5. 测试与质量 ✅

| 任务 | 状态 | 优先级 | 说明 | 关联路径 |
|------|:----:|:------:|------|----------|
| Rust 单元测试覆盖核心模块 | ✅ | P1 | `registry.rs`（28 个测试：注册/注销/查询/状态机/前缀搜索/生命周期） | `src-tauri/src/plugins/registry.rs` |
| 前端类型检查无错误 | ✅ | P0 | `bun run check` —— 0 errors, 0 warnings | 全局 |
| Rust 编译无警告 | ✅ | P0 | `cargo check` clean | `src-tauri/` |
| 插件加载流程 E2E 验证 | ⬜ | P1 | 手动验证：扫描 → 加载 → 执行 → 卸载 | 全局 |

---

## 已完成归档

<details>
<summary>2026-04 ~ 2026-05 已完成</summary>

### 插件系统
- ✅ QuickJS VM 池化管理（创建 / 销毁 / 闲置清理）
- ✅ API Bridge 核心 API 注入（storage / clipboard / shell / fetch / fs / dialog）
- ✅ WASM Bridge IPC 通信链路（`wasm-call` / `wasm_store_call_result`）
- ✅ `crypto` WASM Patch 构建与集成
- ✅ 插件注册表双重索引（by_id / by_prefix）
- ✅ 插件状态机鲁棒性：重试退避 + 错误计数 + Error→Ready 自动恢复
- ✅ 插件 VM 闲置超时自动清理（默认300s）
- ✅ 插件错误隔离与上报（单异常不影响主程序）
- ✅ 插件数据隔离存储（每插件独立 dbStorage）
- ✅ VM 生命周期管理从前端迁移到后端
- ✅ Patch 加载三级降级策略 + 错误上报

### 核心功能
- ✅ 窗口显示 / 隐藏 / 置顶 Control
- ✅ 全局快捷键注册（`Alt+Space` / `Ctrl+Space`）
- ✅ 三层配置数据结构定义（System / User / App）
- ✅ Svelte 5 前端主界面（SearchBox + ResultList + SettingPanel）
- ✅ 插件管理器组件（PluginManager.svelte）基础功能
- ✅ 快捷键可视化设置面板（ShortcutRecorder）
- ✅ 搜索结果分类展示（系统/插件/文件/应用）
- ✅ 并行插件搜索 + 拼音缓存优化

### 示例插件（全部就绪）
- ✅ `hello-world` —— 最简插件，基础生命周期验证
- ✅ `calc` —— 计算器，支持复杂表达式
- ✅ `url-toolkit` —— URL 解析/编码/解码
- ✅ `file-search` —— 本地文件搜索

### 质量
- ✅ `bun run check` —— 0 errors, 0 warnings
- ✅ `cargo check` —— 零警告通过
- ✅ `registry.rs` 单元测试 —— 28 个测试覆盖注册/注销/查询/前缀搜索/状态机/生命周期

</details>

---

## 阻塞与风险

| 风险项 | 级别 | 状态 | 缓解措施 |
|--------|:----:|:----:|----------|
| `rquickjs` 异步支持不足 | 中 | 观察中 | WASM 结果轮询方案已落地，后续关注版本更新 |
| 插件内存泄漏（VM 未释放）| 低 | 已缓解 | VM 闲置超时自动清理已实现，cleanup_idle_plugins() 定期触发 |
| macOS 平台测试缺失 | 低 | 可接受 | MVP 阶段聚焦 Windows，Beta 阶段再适配 |
| 大型文件模块化拆分风险 | 低 | 已缓解 | `api_bridge.rs`（44.4KB）已拆分 ✅，`loader.rs`（37KB）已拆分 ✅ |

---

## 下一步行动（本周）

1. ✅ ~~补充 `registry.rs` 核心函数的单元测试~~（已完成：28 个测试全部通过）
2. **启动** `plugins/mod.rs` 模块化整理（统一 api_bridge/loader 的模块导出模式）
3. **启动** 插件热重载实现（监听 plugins/ 目录变化）

---

*文档版本: 1.1*  
*最后更新: 2026-05-24*  
*状态: 活跃*
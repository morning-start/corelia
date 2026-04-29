# 系统架构

> Corelia 是一个基于 Tauri 2.x + Svelte 5 + Rust 的快速启动器应用，采用清晰的前后端分离架构。

## 架构总览

```
┌─────────────────────────────────────────────────────────────────────┐
│                         WebView (前端)                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │  Svelte 5    │  │   Stores     │  │  PatchLoader (WASM)      │  │
│  │    UI        │  │ (状态管理)    │  │  WebAssembly 运行时      │  │
│  └──────┬───────┘  └──────┬───────┘  └───────────┬──────────────┘  │
│         │                 │                       │                  │
│         └─────────────────┴───────────────────────┘                  │
│                           │ invoke / listen / emit                   │
├───────────────────────────┼──────────────────────────────────────────┤
│                     Tauri IPC Bridge                                │
├───────────────────────────┼──────────────────────────────────────────┤
│                           │                                         │
│  ┌────────────────────────┴─────────────────────────────────────┐  │
│  │                         Rust 后端                             │  │
│  │                                                               │  │
│  │  ┌──────────────────┐  ┌──────────────────┐  ┌─────────────┐ │  │
│  │  │  QuickJS 运行时  │  │   插件加载器     │  │  Commands   │ │  │
│  │  │  (VM 池)        │  │  (PluginLoader)  │  │  (Tauri)    │ │  │
│  │  └────────┬─────────┘  └────────┬─────────┘  └──────┬──────┘ │  │
│  │           │                    │                  │        │  │
│  │  ┌────────┴─────────┐  ┌──────┴──────┐  ┌────────┴──────┐ │  │
│  │  │   API Bridge     │  │   Registry   │  │   Services    │ │  │
│  │  │  (utools API)    │  │  (插件注册表) │  │  (系统服务)   │ │  │
│  │  └────────┬─────────┘  └──────────────┘  └───────────────┘ │  │
│  │           │                                                   │  │
│  │  ┌────────┴─────────┐                                         │  │
│  │  │  Wasm Bridge     │                                         │  │
│  │  │ (WASM 通信桥接)   │                                         │  │
│  │  └──────────────────┘                                         │  │
│  └───────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

## 前后端职责划分

### 前端职责 (SvelteKit + TypeScript)

| 模块 | 位置 | 职责 |
|------|------|------|
| UI 组件 | `src/lib/components/` | 用户界面渲染、交互处理、结果展示 |
| 状态管理 | `src/lib/stores/` | 全局状态管理、数据流协调 |
| 搜索引擎 | `src/lib/search/` | 模糊匹配、结果合并、排序 |
| 插件前端层 | `src/lib/plugins/` | WASM Patch 加载、插件服务调用 |
| 前端服务 | `src/lib/services/` | 前端业务逻辑、Tauri API 代理 |
| 配置 | `src/lib/config.ts` | 前端配置管理 |

### 后端职责 (Rust + Tauri)

| 模块 | 位置 | 职责 |
|------|------|------|
| Commands | `src-tauri/src/commands/` | Tauri 命令暴露、前后端通信接口 |
| 插件系统核心 | `src-tauri/src/plugins/` | 插件加载、API 注入、VM 管理 |
| 系统服务 | `src-tauri/src/services/` | 配置、剪贴板、Shell、自启动等底层能力 |
| 状态管理 | `src-tauri/src/lib.rs` | 应用初始化、全局状态管理 |

## 技术栈

| 层级 | 技术 | 版本 | 用途 |
|------|------|------|------|
| 前端框架 | SvelteKit | 2.x | UI 渲染与交互 |
| UI 语言 | TypeScript | ~5.6.2 | 前端类型安全 |
| 打包工具 | Vite | 6.x | 前端构建 |
| 包管理 | Bun | 1.3+ | 依赖管理 |
| 桌面框架 | Tauri | 2.x | 跨平台桌面壳 |
| 后端语言 | Rust | 1.94.0 | 核心逻辑 |
| JS 引擎 | rquickjs | 0.11.0 | 插件沙箱运行时 |

## 目录结构

```
corelia/
├── src/                              # 前端 (SvelteKit)
│   ├── lib/
│   │   ├── components/               # Svelte 组件
│   │   │   ├── SearchBox.svelte      # 搜索输入框
│   │   │   ├── ResultList.svelte     # 结果列表
│   │   │   ├── SettingPanel.svelte   # 设置面板
│   │   │   ├── PluginManager.svelte  # 插件管理器
│   │   │   ├── CategoryTabs.svelte   # 分类标签
│   │   │   ├── TitleBar.svelte       # 标题栏
│   │   │   ├── ShortcutRecorder.svelte # 快捷键录制
│   │   │   └── HighlightedText.svelte # 高亮文本
│   │   ├── plugins/                  # 插件前端层
│   │   │   ├── types.ts              # 全局类型定义
│   │   │   ├── patch-loader.ts       # WASM Patch 加载器
│   │   │   ├── service.ts            # 插件服务
│   │   │   └── store.ts              # 插件状态管理
│   │   ├── stores/                   # Svelte stores
│   │   │   ├── search/               # 搜索相关状态
│   │   │   │   ├── index.ts
│   │   │   │   ├── merger.ts
│   │   │   │   ├── plugin.ts
│   │   │   │   └── system.ts
│   │   │   ├── app.ts                # 应用全局状态
│   │   │   ├── history.ts            # 搜索历史
│   │   │   ├── theme.ts              # 主题状态
│   │   │   └── user.ts               # 用户配置
│   │   ├── services/                 # 前端服务
│   │   │   ├── executor/
│   │   │   │   ├── index.ts
│   │   │   │   ├── plugin.ts
│   │   │   │   ├── setting.ts
│   │   │   │   ├── system.ts
│   │   │   │   └── types.ts
│   │   │   ├── clipboard.ts
│   │   │   ├── shell.ts
│   │   │   ├── startup.ts
│   │   │   └── store.ts
│   │   ├── search/                   # 搜索引擎
│   │   │   ├── fuzzy.ts              # 模糊匹配
│   │   │   └── performance.ts        # 性能优化
│   │   ├── styles/                   # CSS 样式
│   │   │   └── themes.css
│   │   ├── api.ts                    # Tauri API 代理
│   │   ├── config.ts                 # 前端配置
│   │   └── wasm/                     # WASM 相关
│   │       └── crypto.ts
│   ├── routes/
│   │   ├── +page.svelte              # 主页面
│   │   └── +layout.ts                # 布局
│   └── app.html
├── src-tauri/                        # Rust 后端
│   ├── capabilities/
│   │   └── default.json              # Tauri 权限声明
│   ├── icons/                        # 应用图标
│   └── src/
│       ├── commands/                 # Tauri Commands
│       │   ├── config/
│       │   │   ├── app.rs
│       │   │   ├── macros.rs
│       │   │   ├── mod.rs
│       │   │   ├── system.rs
│       │   │   └── user.rs
│       │   ├── autostart.rs
│       │   ├── clipboard.rs
│       │   ├── mod.rs
│       │   ├── plugin.rs
│       │   ├── shell.rs
│       │   ├── shortcut.rs
│       │   ├── store.rs
│       │   └── window.rs
│       ├── plugins/                  # 插件系统核心
│       │   ├── api_bridge/           # API 桥接
│       │   │   ├── clipboard.rs
│       │   │   ├── context.rs
│       │   │   ├── db_storage.rs
│       │   │   ├── dialog.rs
│       │   │   ├── fetch.rs
│       │   │   ├── fs.rs
│       │   │   ├── mod.rs
│       │   │   ├── notification.rs
│       │   │   ├── path.rs
│       │   │   ├── process.rs
│       │   │   ├── shell.rs
│       │   │   ├── wasm.rs
│       │   │   └── window.rs
│       │   ├── loader/               # 插件加载器
│       │   │   ├── health.rs
│       │   │   ├── instance.rs
│       │   │   ├── lifecycle.rs
│       │   │   ├── manifest.rs
│       │   │   ├── mod.rs
│       │   │   ├── scanner.rs
│       │   │   └── state.rs
│       │   ├── hot_reload.rs
│       │   ├── mod.rs
│       │   ├── quickjs_runtime.rs    # QuickJS 运行时
│       │   ├── registry.rs           # 插件注册表
│       │   └── wasm_bridge.rs        # WASM 桥接
│       ├── services/                 # 业务服务
│       │   ├── autostart_service.rs
│       │   ├── clipboard_service.rs
│       │   ├── config_service.rs
│       │   ├── mod.rs
│       │   ├── shell_service.rs
│       │   ├── store_service.rs
│       │   └── window_service.rs
│       ├── error.rs
│       ├── lib.rs                    # 应用入口
│       └── main.rs
├── plugins/                          # 插件目录
│   ├── calc/
│   ├── file-search/
│   ├── hello-world/
│   └── url-toolkit/
└── patches/                          # WASM Patch
    └── crypto/
```

## 核心模块详解

### 前端核心模块

#### 1. 组件层 (`src/lib/components/`)

| 组件 | 职责 |
|------|------|
| `SearchBox.svelte` | 搜索输入、关键词监听、焦点管理 |
| `ResultList.svelte` | 搜索结果渲染、交互处理、结果选择 |
| `SettingPanel.svelte` | 设置面板、用户偏好配置 |
| `PluginManager.svelte` | 插件管理界面、插件列表 |
| `CategoryTabs.svelte` | 分类标签、结果分类切换 |
| `TitleBar.svelte` | 标题栏、窗口控制 |
| `ShortcutRecorder.svelte` | 快捷键录制、快捷键配置 |
| `HighlightedText.svelte` | 文本高亮展示 |

#### 2. 状态管理层 (`src/lib/stores/`)

| Store | 职责 |
|-------|------|
| `searchStore` | 搜索查询、搜索结果管理 |
| `theme` | 主题状态（深色/浅色/系统） |
| `system` | 系统配置（快捷键/启动项） |
| `user` | 用户配置（行为/外观） |
| `searchHistory` | 搜索历史记录 |
| `app` | 应用全局状态 |

#### 3. 插件前端层 (`src/lib/plugins/`)

| 文件 | 职责 |
|------|------|
| `patch-loader.ts` | WASM Patch 加载、函数注册、调用执行 |
| `service.ts` | 插件服务、插件扫描、加载/卸载管理 |
| `store.ts` | 插件相关状态管理 |
| `types.ts` | 插件类型定义 |

#### 4. 搜索层 (`src/lib/search/`)

| 文件 | 职责 |
|------|------|
| `fuzzy.ts` | 模糊匹配算法实现 |
| `performance.ts` | 搜索性能优化 |

### 后端核心模块

#### 1. QuickJS 运行时 (`quickjs_runtime.rs`)

QuickJS VM 池化管理器，负责插件 JavaScript 代码的沙箱执行。

**核心结构**：
- `QuickJSConfig` — 运行时配置（内存限制、超时时间、VM 上限）
- `VmInstance` — VM 实例（Runtime + Context + 闲置追踪）
- `QuickJSRuntime` — VM 池管理器

**关键参数**：

| 参数 | 默认值 | 说明 |
|------|--------|------|
| max_memory_bytes | 50MB | 单个 VM 最大内存 |
| max_execution_time_ms | 5000 | 执行超时 |
| max_vm_count | 10 | 最大并发 VM |
| idle_timeout_secs | 300 | 闲置超时 |

**线程安全**：通过 `unsafe impl Send/Sync` + `RefCell` 保证主线程安全访问。

#### 2. API Bridge (`api_bridge/`)

将 Rust 能力注入 QuickJS VM，实现 `window.utools` 兼容层。

**注入的 API 模块**：

| 模块 | 文件 | 说明 |
|------|------|------|
| `dbStorage` | `db_storage.rs` | 插件隔离存储 |
| `clipboard` | `clipboard.rs` | 剪贴板操作 |
| `shell` | `shell.rs` | Shell 操作 |
| `window` | `window.rs` | 窗口控制 |
| `path` | `path.rs` | 系统路径获取 |
| `notification` | `notification.rs` | 系统通知 |
| `fs` | `fs.rs` | 文件系统 |
| `fetch` | `fetch.rs` | HTTP 请求 (ureq) |
| `dialog` | `dialog.rs` | 对话框 |
| `process` | `process.rs` | 子进程 |
| `wasm` | `wasm.rs` | WASM 桥接 |
| `context` | `context.rs` | 上下文管理 |

**设计要点**：
- 使用 `OnceLock<AppHandle>` 全局持有 Tauri AppHandle
- `require_app!` 宏统一处理 AppHandle 获取
- QuickJS 回调中**最小化锁范围**，避免死锁

#### 3. PluginLoader (`loader/`)

插件生命周期管理器，负责扫描、解析、加载、卸载插件。

**核心文件**：

| 文件 | 职责 |
|------|------|
| `scanner.rs` | 插件目录扫描、plugin.json 解析 |
| `manifest.rs` | 插件元数据处理 |
| `lifecycle.rs` | 插件生命周期管理 |
| `instance.rs` | 插件实例管理 |
| `state.rs` | 插件状态机实现 |
| `health.rs` | 插件健康检查 |

**状态机**：
```
MetaLoaded ──→ Loading ──→ Ready
    ↑              │          │
    │              ↓          ↓
    └── Unloaded ←─── Cached / Error
```

**加载流程**：
1. `scan_plugins()` — 扫描目录，解析 `plugin.json`，状态置 `MetaLoaded`
2. `load_plugin(id)` — 创建 VM → 注入 API → 加载 patches → 执行代码 → 状态置 `Ready`
3. `unload_plugin(id)` — 销毁 VM → 状态置 `Unloaded`

#### 4. PluginRegistry (`registry.rs`)

线程安全的插件注册表，使用 `RwLock` 实现多读单写。

**双重索引**：
- `by_id: HashMap<String, usize>` — ID 精确查找
- `by_prefix: HashMap<String, Vec<usize>>` — 前缀模糊搜索

**状态机校验**：`update_state()` 执行合法性校验，拒绝非法状态转换。

#### 5. WasmBridge (`wasm_bridge.rs`)

QuickJS VM ↔ WebView WASM 的 IPC 桥接层。

**调用链路**：
```
QuickJS VM (插件代码)
    │
    ├── utools.wasm.__wasm_call("crypto.sha256", '"hello"')
    │       ├── 1. 获取 WasmBridge 锁，检查函数是否存在 → 释放锁
    │       ├── 2. 生成唯一 requestId (AtomicU64)
    │       └── 3. emit("wasm-call", payload) → 返回 requestId
    │
    ├── Rust → WebView (Tauri Event)
    │       └── PatchLoader.handleWasmCall()
    │               ├── 执行 WebAssembly 函数
    │               └── invoke("wasm_store_call_result") → 存入 WasmBridge
    │
    └── utools.wasm.__wasm_get_result(requestId)
            └── 轮询获取结果 (JSON 字符串或 null)
```

**结果缓存**：`call_results: HashMap<String, WasmCallResultEntry>`，获取后自动移除，超 1000 条时淘汰最旧的 500 条。

#### 6. Commands (`commands/`)

Tauri Commands 暴露层，前端通过 `invoke()` 调用。

**核心 Commands 模块**：

| 模块 | 职责 |
|------|------|
| `plugin.rs` | 插件相关命令（扫描、加载、卸载、搜索） |
| `window.rs` | 窗口管理命令（显示、隐藏、置顶等） |
| `clipboard.rs` | 剪贴板操作命令 |
| `shell.rs` | Shell 操作命令 |
| `shortcut.rs` | 快捷键注册命令 |
| `store.rs` | 数据存储命令 |
| `autostart.rs` | 自启动管理命令 |
| `config/` | 配置管理（系统/用户/应用） |

#### 7. Services (`services/`)

底层业务服务实现。

| 服务 | 职责 |
|------|------|
| `ConfigService` | 配置文件读写、配置管理 |
| `ClipboardService` | 剪贴板读写操作 |
| `ShellService` | Shell 命令执行、文件/URL 打开 |
| `WindowService` | 窗口控制操作 |
| `StoreService` | 数据存储操作 |
| `AutostartService` | 自启动管理 |

## 数据流

### 搜索流程

```
用户输入 → SearchBox → searchStore.setQuery()
                          │
              ┌───────────┼───────────┐
              ↓           ↓           ↓
        系统搜索      插件搜索      历史记录
        (fuzzy)      (prefix)     (history)
              │           │           │
              └───────────┼───────────┘
                          ↓
                    合并结果 → ResultList
                          ↓
                    用户选择 → resultExecutor.execute()
```

**前端职责**：
- 用户输入处理
- 搜索查询分发
- 结果合并与排序
- UI 渲染

**后端职责**：
- 系统搜索结果提供
- 插件搜索结果提供（如需要）
- 历史记录存储

### 插件加载流程

```
前端 init()
    │
    ├── pluginService.init()
    │       └── invoke('scan_plugins') → Rust PluginLoader.scan_plugins()
    │
    └── patchLoader.init()
            ├── listen('wasm-load-patch') → 等待 Rust 通知加载 WASM
            └── listen('wasm-call') → 等待 QuickJS 调用 WASM

用户触发插件
    │
    └── invoke('load_plugin') → Rust PluginLoader.load_plugin()
            ├── create_vm() → QuickJSRuntime
            ├── inject_utools() → ApiBridge
            ├── load_patches() → emit("wasm-load-patch") → WebView PatchLoader
            └── execute(code) → QuickJS 执行插件代码
```

**前端职责**：
- 初始化插件服务
- 加载 WASM Patch
- 监听 WASM 调用事件
- 执行 WASM 函数
- 存储 WASM 调用结果

**后端职责**：
- 扫描插件目录
- 创建/管理 QuickJS VM
- 注入 utools API
- 执行插件代码
- 管理插件生命周期
- WASM 调用桥接

## 状态管理

### 前端 Stores (Svelte 5)

| Store | 文件 | 职责 |
|-------|------|------|
| `searchStore` | `search.ts` | 搜索查询与结果 |
| `theme` | `theme.ts` | 主题 (dark/light/system) |
| `system` | `system.ts` | 系统配置 (快捷键/启动) |
| `user` | `user.ts` | 用户配置 (行为/外观) |
| `searchHistory` | `history.ts` | 搜索历史 |
| `app` | `app.ts` | 应用全局状态 |

### Rust 状态 (Tauri Managed)

| State | 类型 | 职责 |
|-------|------|------|
| `QuickJSRuntime` | 直接 | VM 池管理 |
| `PluginLoader` | `Mutex<>` | 插件加载器 |
| `PluginRegistry` | `RwLock<>` | 插件注册表 |
| `WasmBridge` | `Mutex<>` | WASM 桥接注册表 |

## 关键设计决策

1. **QuickJS 而非 V8** — 更轻量，内存占用低，适合沙箱化插件执行
2. **VM 池化** — 每个插件独立 VM，支持闲置回收，控制资源上限
3. **WASM IPC 桥接** — QuickJS 无法直接运行 WASM，通过 IPC 转发到 WebView 执行
4. **轮询式结果获取** — QuickJS 同步执行模型下，`__wasm_get_result` 轮询获取异步结果
5. **分层配置** — 系统配置 / 用户配置 / 应用配置三层分离
6. **透明窗口** — `decorations: false, transparent: true`，手动处理焦点管理
7. **清晰的前后端职责分离** — UI、状态、搜索在前端，系统能力、插件管理在后端

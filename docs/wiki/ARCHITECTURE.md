# 系统架构

> Corelia 是一个基于 Tauri 2.x + Svelte 5 + Rust 的快速启动器应用，采用三层架构设计。

## 架构总览

```
┌─────────────────────────────────────────────────────────┐
│                    WebView (前端)                         │
│  ┌──────────┐  ┌──────────┐  ┌────────────────────────┐ │
│  │ Svelte 5 │  │  Stores  │  │  PatchLoader (WASM)    │ │
│  │   UI     │  │ (状态管理)│  │  WebAssembly 运行时    │ │
│  └────┬─────┘  └────┬─────┘  └───────────┬────────────┘ │
│       │              │                    │              │
│       └──────────────┼────────────────────┘              │
│                      │ invoke / listen / emit            │
├──────────────────────┼───────────────────────────────────┤
│                Tauri IPC Bridge                          │
├──────────────────────┼───────────────────────────────────┤
│                      │                                   │
│  ┌───────────────────┴────────────────────────────────┐  │
│  │              Rust 后端                               │  │
│  │                                                      │  │
│  │  ┌──────────────┐  ┌───────────────┐  ┌──────────┐ │  │
│  │  │ QuickJS      │  │ PluginLoader  │  │ Commands │ │  │
│  │  │ Runtime      │  │ (插件加载器)  │  │ (Tauri)  │ │  │
│  │  └──────┬───────┘  └───────┬───────┘  └──────────┘ │  │
│  │         │                  │                        │  │
│  │  ┌──────┴───────┐  ┌──────┴──────┐  ┌───────────┐ │  │
│  │  │ ApiBridge    │  │ Registry    │  │ Services  │ │  │
│  │  │ (API桥接)    │  │ (注册表)    │  │ (系统服务)│ │  │
│  │  └──────┬───────┘  └─────────────┘  └───────────┘ │  │
│  │         │                                           │  │
│  │  ┌──────┴───────┐                                   │  │
│  │  │ WasmBridge   │                                   │  │
│  │  │ (WASM桥接)   │                                   │  │
│  │  └──────────────┘                                   │  │
│  └─────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

## 技术栈

| 层级 | 技术 | 版本 | 用途 |
|------|------|------|------|
| 前端框架 | SvelteKit | 2.x | UI 渲染与交互 |
| UI 语言 | TypeScript | ~5.6.2 | 前端类型安全 |
| 打包工具 | Vite | 6.x | 前端构建 |
| 包管理器 | Bun | 1.3+ | 依赖管理 |
| 桌面框架 | Tauri | 2.x | 跨平台桌面壳 |
| 后端语言 | Rust | 1.94.0 | 核心逻辑 |
| JS 引擎 | rquickjs | 0.11.0 | 插件沙箱运行时 |

## 目录结构

```
corelia/
├── src/                        # 前端 (SvelteKit)
│   ├── lib/
│   │   ├── components/         # Svelte 组件
│   │   │   ├── SearchBox.svelte      # 搜索输入框
│   │   │   ├── ResultList.svelte     # 结果列表
│   │   │   ├── SettingPanel.svelte   # 设置面板
│   │   │   ├── PluginManager.svelte  # 插件管理器
│   │   │   ├── CategoryTabs.svelte   # 分类标签
│   │   │   ├── TitleBar.svelte       # 标题栏
│   │   │   ├── ShortcutRecorder.svelte # 快捷键录制
│   │   │   └── HighlightedText.svelte  # 高亮文本
│   │   ├── plugins/            # 插件前端层
│   │   │   ├── types.ts        # 全局类型定义
│   │   │   ├── patch-loader.ts # WASM Patch 加载器
│   │   │   └── service.ts      # 插件服务
│   │   ├── stores/             # Svelte stores
│   │   ├── services/           # 前端服务
│   │   ├── search/             # 搜索引擎
│   │   ├── styles/             # CSS 样式
│   │   ├── api.ts              # Tauri API 代理
│   │   └── config.ts           # 前端配置
│   └── routes/
│       ├── +page.svelte        # 主页面
│       └── +layout.ts          # 布局
├── src-tauri/                  # Rust 后端
│   └── src/
│       ├── lib.rs              # 应用入口与 Command 注册
│       ├── plugins/            # 插件系统核心
│       │   ├── quickjs_runtime.rs  # QuickJS VM 池
│       │   ├── api_bridge.rs   # API 桥接层
│       │   ├── loader.rs       # 插件加载器
│       │   ├── registry.rs     # 插件注册表
│       │   └── wasm_bridge.rs  # WASM 桥接
│       ├── commands/           # Tauri Commands
│       └── services/           # 业务服务
└── plugins/                    # 插件目录
    └── file-search/            # 示例插件
```

## 核心模块

### 1. QuickJS Runtime (`quickjs_runtime.rs`)

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

### 2. API Bridge (`api_bridge.rs`)

将 Rust 能力注入 QuickJS VM，实现 `window.utools` 兼容层。

**注入的 API 模块**：

| 模块 | 方法 | 说明 |
|------|------|------|
| `dbStorage` | getItem / setItem / removeItem / getAll | 插件隔离存储 |
| `clipboard` | readText / writeText / copyText | 剪贴板操作 |
| `shell` | openPath / openExternal / showItemInFolder / beep | Shell 操作 |
| 窗口 | hideMainWindow / showMainWindow | 窗口控制 |
| `getPath` | getPath(name) | 系统路径获取 |
| `showNotification` | showNotification(title, body) | 系统通知 |
| `fs` | readTextFile / writeTextFile / exists / isDir | 文件系统 |
| `fetch` | fetch(url, options) | HTTP 请求 (ureq) |
| `dialog` | showOpenDialog / showSaveDialog / showMessageBox | 对话框 |
| `process` | exec / getNativeId / getAppName / getAppVersion | 子进程 |
| `wasm` | __wasm_call / __wasm_get_result / __wasm_available / __wasm_has | WASM 桥接 |

**设计要点**：
- 使用 `OnceLock<AppHandle>` 全局持有 Tauri AppHandle
- `require_app!` 宏统一处理 AppHandle 获取
- QuickJS 回调中**最小化锁范围**，避免死锁

### 3. Plugin Loader (`loader.rs`)

插件生命周期管理器，负责扫描、解析、加载、卸载插件。

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

### 4. Plugin Registry (`registry.rs`)

线程安全的插件注册表，使用 `RwLock` 实现多读单写。

**双重索引**：
- `by_id: HashMap<String, usize>` — ID 精确查找
- `by_prefix: HashMap<String, Vec<usize>>` — 前缀模糊搜索

**状态机校验**：`update_state()` 执行合法性校验，拒绝非法状态转换。

### 5. WASM Bridge (`wasm_bridge.rs`)

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

## 数据流

### 搜索流程

```
用户输入 → SearchBox → searchStore.setQuery()
                            │
                    ┌───────┼───────┐
                    ↓       ↓       ↓
              系统搜索  插件搜索  历史记录
              (fuzzy)  (prefix)  (history)
                    │       │       │
                    └───────┼───────┘
                            ↓
                    合并结果 → ResultList
                            ↓
                    用户选择 → resultExecutor.execute()
```

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

## 状态管理

### 前端 Stores (Svelte 5)

| Store | 文件 | 用途 |
|-------|------|------|
| `searchStore` | search.ts | 搜索查询与结果 |
| `theme` | theme.ts | 主题 (dark/light/system) |
| `system` | system.ts | 系统配置 (快捷键/启动) |
| `user` | user.ts | 用户配置 (行为/外观) |
| `searchHistory` | history.ts | 搜索历史 |
| `app` | app.ts | 应用全局状态 |

### Rust 状态 (Tauri Managed)

| State | 类型 | 用途 |
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

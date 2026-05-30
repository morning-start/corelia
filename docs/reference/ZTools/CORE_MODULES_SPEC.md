# ZTools 核心模块参考

> **覆盖文件:** 翻译引擎、同步引擎、插件协调器、MCP 服务、ZBrowser、HTTP 服务、应用监控、日志收集、数据迁移、屏幕截图、双击管理等非窗口核心模块
> **核心价值:** 覆盖 ZTools 业务逻辑层剩余的 60% 代码，提供完整的迁移参考

---

## 1. 模块总览

| 模块 | 文件 | 行数 | 分类 | 复杂度 |
|------|------|------|------|--------|
| Sync Engine | `syncEngine.ts` | ~1775 | 数据同步 | 高 |
| Translation | `translationManager.ts` | ~500 | 离线翻译 | 中 |
| MCP Server | `mcpServer.ts` | ~350 | 协议服务 | 中 |
| MCP Agent | `mcpAgent.ts` | ~300 | AI Agent | 中 |
| ZBrowser | `zrowser.ts` | ~400 | 内置浏览器 | 中 |
| Plugin Assembly Coordinator | `pluginAssemblyCoordinator.ts` | 347 | 插件状态 | 中 |
| App Watcher | `appWatcher.ts` | ~200 | 应用监控 | 中 |
| Log Collector | `logCollector.ts` | ~150 | 日志收集 | 低 |
| HTTP Server | `httpServer.ts` | ~150 | HTTP 服务 | 低 |
| Data Migrations | `startupDataMigrations.ts` | ~120 | 数据迁移 | 低 |
| Icon Protocol | `iconProtocol.ts` | ~120 | 图标服务 | 低 |
| Double Tap Manager | `doubleTapManager.ts` | ~100 | 双击检测 | 低 |
| Screen Capture | `screenCaptureManager.ts` | 156 | 区域截图 | 低 |
| Proxy Manager | `proxyManager.ts` | ~100 | 代理配置 | 低 |
| IPC Service | `service.ts` | ~100 | IPC 服务 | 低 |
| Internal Plugin Loader | `internalPluginLoader.ts` | ~80 | 插件加载 | 低 |

---

## 2. 翻译引擎（TranslationManager）— ~500 行

### 2.1 架构

```typescript
// 双引擎策略:
// 1. 离线引擎: Bergamot WASM (Marian NMT) — 默认
// 2. 在线引擎: 百度翻译 API — 离线不可用时回退

class TranslationManager {
  private offlineEngine: BergamotTranslator | null = null
  private onlineEngine: BaiduTranslator

  // 初始化: 异步加载 Bergamot WASM 模型文件
  async initialize(): Promise<void> {
    // 从应用资源中加载 .wasm 模型
    // 模型文件: resources/translation/models/bergamot/*
    // 包含: model.npz, vocab.json, config.json
    this.offlineEngine = await BergamotTranslator.create({
      modelPath: 'resources/translation/models/bergamot/',
      language: 'zh-en'  // 默认中英
    })
  }

  // 翻译入口
  async translate(text: string, sourceLang?: string): Promise<TranslationResult> {
    if (this.offlineEngine) {
      return this.offlineEngine.translate(text, sourceLang || 'auto')
    }
    return this.onlineEngine.translate(text, sourceLang || 'auto')
  }

  // 批量翻译（剪贴板历史）
  async batchTranslate(items: ClipboardItem[]): Promise<TranslationResult[]> {
    return Promise.all(items.map(item => this.translate(item.value)))
  }

  // 选词翻译
  async translateSelection(): Promise<TranslationResult | null> {
    const text = this.clipboardManager.readText()
    if (!text) return null
    return this.translate(text)
  }
}

interface TranslationResult {
  original: string
  translated: string
  sourceLang: string
  targetLang: string
  engine: 'offline' | 'online'
  confidence?: number  // Bergamot 置信度评分
}
```

### 2.2 Bergamot WASM 集成细节

```typescript
// Bergamot (Mozilla) WASM 翻译引擎
// 基于 Marian NMT 框架，专为浏览器/Electron 设计

// 加载模型
const translator = new BergamotTranslator()
await translator.loadModel({
  model:     fs.readFileSync('model.npz'),      // 神经网络的权重
  vocab:     fs.readFileSync('vocab.json'),      // 词表
  shortlist: fs.readFileSync('lex.50.50.json'),  // 词汇短列表
  config: {                                      // 配置
    cacheType: 'cache',                          // 缓存类型
    maxInputLength: 128,                         // 最大输入长度
    maxOutputLength: 256,                        // 最大输出长度
    beamSize: 5,                                 // 束搜索宽度
  }
})

// 翻译
const result = translator.translate({ text: 'Hello world' })
// { text: '你好世界', quality: 'normal', ... }
```

### 2.3 Corelia 迁移

| ZTools | Corelia |
|--------|---------|
| Electron `require('bergamot')` | `rquickjs` WASM 运行时 或 Rust FFI |
| Bergamot WASM 模型文件 | 复用相同 `.npz` + `.json` 模型文件 |
| 在线回退百度翻译 | 复用相同的 API 密钥或换用 `reqwest` 调用 |

---

## 3. 同步引擎（SyncEngine）— ~1775 行

### 3.1 架构

```typescript
class SyncEngine {
  // 使用 WebDAV 协议同步所有配置、插件数据、剪贴板历史

  private provider: WebDAVProvider | null = null
  private syncTimer: Timer | null = null
  private isSyncing: boolean = false

  // 支持的同步目标:
  // - WebDAV (NextCloud, ownCloud, 群晖)
  // - 本地文件系统
  // - S3 兼容存储（计划中）

  // 同步范围:
  // - 全局设置 (JSON)
  // - 插件设置 (按命名空间)
  // - 剪贴板历史 (LMDB → JSON)
  // - 笔记和工具条目 (JSON)
  // - 插件数据 (按插件)
}
```

### 3.2 同步配置

```typescript
interface SyncConfig {
  enabled: boolean
  provider: 'webdav' | 'local' | 's3'
  interval: number        // 同步间隔（秒），默认 300 (5分钟)
  
  // WebDAV 配置
  webdav?: {
    url: string           // 服务器 URL
    username: string
    password: string      // 应用密码（非主密码）
    path: string          // 远程路径，默认 /ztools/
  }

  // 同步选项
  options: {
    autoSync: boolean     // 自动同步
    notifyOnSync: boolean // 同步完成通知
    conflictResolution: 'local_wins' | 'remote_wins' | 'last_wins' | 'manual'
    encrypt: boolean      // 客户端加密
    encryptPassword: string // 加密密码
  }
}
```

### 3.3 同步流程

```
定时触发 / 手动触发
  → checkConnection()
  → 收集变更数据:
    settings → JSON dump
    clipboard history → LMDB dump → filter by updatedAt
    notes → JSON dump
    plugin data → per-namespace dump
  → 创建快照清单:
    {
      "ztools_settings.json": { hash, updatedAt },
      "ztools_clipboard.json": { hash, updatedAt },
      "ztools_notes.json": { hash, updatedAt },
      "plugins/{name}/data.json": { hash, updatedAt }
    }
  → 对比远程快照清单
  → 冲突检测:
    时间戳 + 哈希双重判断
    冲突 → 按冲突策略处理
  → 下载变更文件
  → 上传变更文件
  → 更新本地快照
  → 发送 sync:completed 事件
```

### 3.4 WebDAV 实现

```typescript
class WebDAVProvider {
  // PROPFIND — 获取目录列表和属性
  async list(path: string): Promise<WebDAVResource[]>

  // GET — 下载文件
  async download(remotePath: string): Promise<Buffer>

  // PUT — 上传文件
  async upload(remotePath: string, data: Buffer): Promise<void>

  // DELETE — 删除文件
  async delete(remotePath: string): Promise<void>

  // MKCOL — 创建目录
  async createDirectory(path: string): Promise<void>

  // 认证: Basic Auth (base64 username:password)
  // 使用 HTTPS 加密传输
}
```

### 3.5 冲突处理

```typescript
// 冲突检测规则:
// 
// 1. 比较本地和远程文件的 hash (SHA256)
// 2. 如果 hash 不同，检查 updatedAt
// 3. 如果 updatedAt 也相同，触发冲突策略
// 4. 冲突文件保存为 .conflict 副本

// 策略:
// local_wins:  覆盖远程文件
// remote_wins: 覆盖本地文件
// last_wins:   最新的 updatedAt 获胜
// manual:      记录冲突，由用户处理
```

### 3.6 Corelia 迁移

| ZTools | Corelia |
|--------|---------|
| `node:https` + Basic Auth | `reqwest` crate + WebDAV 库 |
| LMDB dump (JS 实现) | LMDB dump (Rust，`heed` crate) |
| 定时器 `setInterval` | Rust `tokio::spawn` + `tokio::time::interval` |
| 加密 `crypto.createCipheriv` | `aes-gcm` / `chacha20poly1305` crate |
| 冲突策略逻辑 | 直接迁移（纯算法，平台无关） |

---

## 4. MCP Server — ~350 行

### 4.1 架构

```typescript
// MCP (Model Context Protocol) 服务器
// 为 AI 工具提供 ZTools 能力的访问接口

class MCPServer {
  private server: http.Server | null = null
  private port: number = 11511

  start(): void {
    this.server = http.createServer((req, res) => {
      // MCP 请求路由:
      // GET /mcp/resources     → 可用资源列表
      // POST /mcp/query        → 执行查询
      // POST /mcp/execute      → 执行操作
    })
    this.server.listen(this.port, '127.0.0.1')  // 仅本地
  }

  stop(): void {
    this.server?.close()
  }
}
```

### 4.2 资源列表

```typescript
// GET /mcp/resources → 
interface MCPResource {
  type: 'clipboard' | 'app' | 'window' | 'system' | 'search'
  name: string
  description: string
  actions: string[]
}

// 注册的资源:
[
  { type: 'clipboard', name: 'clipboard_history', 
    actions: ['get', 'search', 'delete', 'paste'] },
  { type: 'app', name: 'installed_apps', 
    actions: ['list', 'search', 'launch'] },
  { type: 'window', name: 'active_window', 
    actions: ['get', 'activate'] },
  { type: 'system', name: 'system_info', 
    actions: ['get'] },
  { type: 'search', name: 'global_search', 
    actions: ['search'] }
]
```

### 4.3 MCP Agent — ~300 行

```typescript
// MCP Agent 是 AI Agent 客户端，通过 MCP 协议与 ZTools 交互
// 接收用户自然语言指令，在 ZTools 上下文中执行

class MCPAgent {
  // 接收用户指令
  async process(input: string): Promise<MCPResponse> {
    // 1. 解析用户意图
    // 2. 选择对应 MCP 资源
    // 3. 执行操作
    // 4. 返回结果
  }

  // 智能粘贴流程
  async intelligentPaste(input: string): Promise<void> {
    // 1. 分析当前窗口上下文
    // 2. 从剪贴板历史中搜索相关条目
    // 3. 将最佳匹配粘贴到当前窗口
  }

  // 文档分析
  async analyzeClipboard(): Promise<Analysis> {
    // 分析剪贴板内容类型
    // 代码? 文本? 链接? → 提供对应操作建议
  }
}
```

---

## 5. 插件装配协调器（PluginAssemblyCoordinator）— 347 行

### 5.1 定位

插件加载/卸载/重载的状态机。确保插件资源的正确生命周期管理，防止泄漏。

### 5.2 状态机

```
┌──────────┐    load()    ┌──────────┐    assemble()    ┌───────────┐
│  LOADING ├─────────────>│ LOADED   ├────────────────>│ ASSEMBLED  │
└──────────┘              └──────────┘                 └───────────┘
                               │                             │
                           unload()                     destroy()
                               ↓                             ↓
                          ┌──────────┐                 ┌───────────┐
                          │ UNLOADED  │                 │ DESTROYED  │
                          └──────────┘                 └───────────┘

状态:
- LOADING:      正在读取 manifest.json 和创建 Webview
- LOADED:       manifest 已读取，Webview 已创建，但 API 上下文未注入
- ASSEMBLED:    完整装配完成，API 上下文已注入，插件可交互
- UNLOADED:     已卸载（可重新加载）
- DESTROYED:    已销毁（需要重新创建）
```

### 5.3 核心接口

```typescript
class PluginAssemblyCoordinator {
  private state: Map<string, PluginState> = new Map()

  // 加载插件
  async load(pluginName: string): Promise<AssemblyResult> {
    // 1. 将状态设为 LOADING
    // 2. 读取 plugin.config.json
    // 3. 创建 BrowserWindow
    // 4. 设置状态 LOADED
    // 5. 注入 API 上下文
    // 6. 注册 IPC 路由
    // 7. 设置状态 ASSEMBLED
    // 8. 返回成功
  }

  // 卸载插件
  async unload(pluginName: string): Promise<void> {
    // 1. 回收所有 Window (pluginWindowManager)
    // 2. 移除浮动球 (floatingBallManager)
    // 3. 清空超级面板 Action
    // 4. 注销快捷键
    // 5. 关闭 Webview
    // 6. 断开 IPC 路由
    // 7. 清除数据库命名空间
    // 8. 设置状态 UNLOADED
  }

  // 重载插件
  async reload(pluginName: string): Promise<AssemblyResult> {
    await this.unload(pluginName)
    return this.load(pluginName)
  }

  // 获取状态
  getState(pluginName: string): PluginState {
    return this.state.get(pluginName) || 'DESTROYED'
  }

  // 批量操作
  async loadAll(): Promise<AssemblyResult[]> {
    const plugins = await this.scanner.scan()
    return Promise.all(plugins.map(p => this.load(p.name)))
  }

  async unloadAll(): Promise<void> {
    const plugins = Array.from(this.state.keys())
    await Promise.all(plugins.map(p => this.unload(p)))
  }
}
```

### 5.4 级联清理

```typescript
// 卸载插件的级联清理流程:
async unload(pluginName: string) {
  // 1. 窗口回收
  this.windowManager.closeAllWindows(pluginName)
  
  // 2. 浮动球回收
  this.floatingBallManager.closeAll(pluginName)
  
  // 3. 超级面板 Action 回收
  this.superPanelManager.clearActions(pluginName)
  
  // 4. 快捷键回收
  this.shortcutManager.unregisterAll(pluginName)
  
  // 5. 组件分离窗口回收
  this.detachedWindowManager.closeByPlugin(pluginName)
  
  // 6. MCP 资源注销
  this.mcpServer.unregisterResources(pluginName)
  
  // 7. 数据库命名空间清理
  this.database.clearNamespace(`ztools:plugin:${pluginName}`)
  
  // 8. Webview 关闭
  this.webviewPool.release(pluginName)
  
  // 9. IPC 路由注销
  this.ipcRouter.unregister(pluginName)
}
```

---

## 6. ZBrowser（内置浏览器）— ~400 行

### 6.1 定位

在 ZTools 内部嵌入了完整的多标签页浏览器。

### 6.2 核心功能

```typescript
class ZBrowser {
  private tabs: Map<string, BrowserTab> = new Map()
  private activeTabId: string | null = null

  interface BrowserTab {
    id: string
    url: string
    title: string
    webview: WebviewTag
    isLoading: boolean
    canGoBack: boolean
    canGoForward: boolean
    zoom: number        // 缩放比例: 1.0 = 100%
    bookmarked: boolean
  }

  // 创建标签
  createTab(url: string): TabInfo {
    const webview = new WebviewTag()
    webview.src = url
    webview.setAttribute('plugins', 'yes')
    webview.setAttribute('preload', 'file://...zrowser-preload.js')
    
    this.tabs.set(tabId, { id: tabId, url, webview, ... })
    return tabInfo
  }

  // 导航
  navigate(tabId: string, url: string): void
  goBack(tabId: string): void
  goForward(tabId: string): void
  reload(tabId: string): void

  // 书签
  bookmark(tabId: string): Bookmark
  getBookmarks(): Bookmark[]
}
```

### 6.3 Corelia 映射

Tauri 没有内置 Webview 标签页管理，需要使用 `tauri::webview::WebviewWindowBuilder` 多次创建多个 webview 窗口来模拟标签页，或使用单 Webview + 前端的 iframe 解决方案。

---

## 7. HTTP 服务（HttpServer）— ~150 行

```typescript
// 在本地端口启动 HTTP 服务，为外部设备提供远程控制

class HttpServer {
  start(port?: number): void {
    // 默认端口: 11512
    // 提供 REST API:
    // POST /api/search          — 搜索应用和剪贴板
    // POST /api/launch          — 启动应用
    // POST /api/clipboard/set   — 设置剪贴板内容
    // GET  /api/clipboard       — 获取剪贴板内容
    // POST /api/translate       — 翻译文本
    // GET  /api/status          — 服务器状态
  }
}
```

---

## 8. 应用监控（AppWatcher）— ~200 行

```typescript
// 监控应用的安装、卸载和更新

class AppWatcher {
  start(): void {
    // Windows: 注册 Shell 事件
    // 监听: Install, Uninstall, Update
    
    // macOS: NSWorkspace 通知
    // 监听: NSApplicationDidInstallNotification 等
    
    // 变化时重新扫描应用列表
    this.on('app:installed', (appPath) => {
      this.scanner.rescan()
      this.emit('apps:updated')
    })
  }
}
```

---

## 9. 日志收集（LogCollector）— ~150 行

```typescript
// 收集前端和后端日志，用于调试和错误报告

class LogCollector {
  // 收集类型:
  // - 前端控制台日志 (console.log/error/warn)
  // - 主进程日志
  // - IPC 调用记录
  // - 系统错误（crash dump）

  // 日志级别: debug, info, warn, error
  // 最多保留 10000 条

  // 输出:
  // - 文件: userData/logs/ztools.log
  // - DevTools 控制台
  // - 远程收集（可选开启）

  collect(entry: LogEntry): void {
    this.buffer.push(entry)
    if (this.buffer.length > 10000) this.buffer.shift()
    this.saveToFile()
  }

  export(): LogEntry[]  // 导出用于错误报告
  clear(): void         // 清空
}
```

---

## 10. 数据迁移（StartupDataMigrations）— ~120 行

```typescript
// 应用启动时执行数据迁移

class StartupDataMigrations {
  // 迁移版本控制: 每版一个迁移函数
  // 当前版本在 settings.json 中记录

  private migrations: Migration[] = [
    {
      version: '2.0.0',
      description: '剪贴板数据库增加 groupId 字段',
      migrate: async (db) => {
        // 遍历所有剪贴板项，添加 groupId: null
      }
    },
    {
      version: '2.1.0',
      description: '插件配置从 JSON 迁移到 LMDB',
      migrate: async (db) => {
        // 读取旧 JSON 配置 → 写入 LMDB
      }
    },
    {
      version: '2.2.0',
      description: '同步引擎增加加密配置',
      migrate: async (db) => {
        // 检查 SyncConfig，增加 encrypt 字段默认值
      }
    },
    // ...
  ]

  async run(): Promise<void> {
    const currentVersion = this.settings.get('appVersion') || '1.0.0'
    const pendingMigrations = this.migrations
      .filter(m => semver.gt(m.version, currentVersion))

    for (const migration of pendingMigrations) {
      await migration.migrate(this.database)
      this.settings.set('appVersion', migration.version)
    }
  }
}
```

---

## 11. 图标协议（IconProtocol）— ~120 行

```typescript
// 自定义图标协议: ztools-icon://
// 用于在渲染进程中显示应用图标

function registerIconProtocol(session: Session): void {
  session.protocol.handle('ztools-icon', (request) => {
    const url = new URL(request.url)
    const appPath = decodeURIComponent(url.pathname.slice(1))

    // 提取图标
    const iconBuffer = IconExtractor.getFileIconSync(appPath)
    
    return new Response(iconBuffer, {
      headers: { 'Content-Type': 'image/png' }
    })
  })
}

// 用法: <img src="ztools-icon://C:\Program Files\MyApp\app.exe">
//       <img src="ztools-icon:///Applications/Safari.app">
```

---

## 12. 双击管理（DoubleTapManager）— ~100 行

```typescript
// 检测特定按键的双击，主要用于 Ctrl 键双击

class DoubleTapManager {
  private lastTapTime: number = 0
  private readonly interval: number  // 双击判定间隔，默认 300ms

  // 注册双击检测
  register(key: string, threshold: number, callback: () => void): void {
    // 监听 keydown 事件
    // 两次按下间隔 < threshold 视为双击
    // 触发 callback
  }

  // 注销
  unregister(key: string): void
}

// 应用场景: Ctrl 双击触发搜索
doubleTapManager.register('Control', 300, () => {
  mainWindow.show()
  mainWindow.focus()
})
```

---

## 13. 屏幕截图（ScreenCaptureManager）— 156 行

```typescript
class ScreenCaptureManager {
  // 启动区域选择覆盖层
  startRegionCapture(displayId?: number): Promise<CaptureResult> {
    // 创建一个全屏半透明覆盖窗口
    // 用户拖动选择截图区域
    // 返回选中区域的图像数据
    
    // 覆盖窗口:
    new BrowserWindow({
      fullscreen: true,
      transparent: true,
      frame: false,
      alwaysOnTop: true,
      skipTaskbar: true,
      webPreferences: { sandbox: true }
    })
  }
}

interface CaptureResult {
  success: boolean
  dataURL?: string     // 截图 Base64 PNG
  x?: number; y?: number
  width?: number; height?: number
}
```

---

## 14. 代理管理器（ProxyManager）— ~100 行

```typescript
class ProxyManager {
  // 管理 ZTools 的网络代理配置
  // 支持: 系统代理（默认）、自定义代理、无代理

  getProxyForUrl(url: string): string | null {
    // 规则:
    // 1. 有自定义代理 → 使用自定义
    // 2. 无自定义 → 读取系统代理配置
    // 3. 配置了 PAC → 执行 PAC 脚本
  }

  // 监听系统代理变化（Windows: 注册表，macOS: 系统偏好）
  startMonitor(): void
}
```

---

## 15. 内部插件加载器（InternalPluginLoader）— ~80 行

```typescript
// 加载内置插件（在 plugins/ 目录中的代码插件）
// 与普通插件的区别: 不需要创建 Webview，直接在主进程中运行

class InternalPluginLoader {
  load(pluginName: string): void {
    const pluginPath = path.join(__dirname, 'plugins', pluginName)
    const plugin = require(pluginPath)
    
    plugin.initialize({
      // 提供内部 API 上下文
      registerShortcut: this.shortcutManager.register.bind(this.shortcutManager),
      onClipboardChange: this.clipboardManager.onChange.bind(this.clipboardManager),
      getSettings: this.settingsManager.get.bind(this.settingsManager),
      setSettings: this.settingsManager.set.bind(this.settingsManager),
    })
  }

  unload(pluginName: string): void {
    // 清理 require 缓存
    delete require.cache[require.resolve(`./plugins/${pluginName}`)]
  }
}
```

---

## 16. 窗口工具（WindowUtils）— 111 行

```typescript
function isProgmanWindow(hwnd: number): boolean
// 判断是否为 Windows 桌面窗口（Progman / WorkerW）

function isExplorerWindow(hwnd: number): boolean
// 判断是否为文件资源管理器窗口（CabinetWClass）

function getWindowClassName(hwnd: number): string
// 获取窗口类名

function getExplorerFolderPath(hwnd: number): string | null
// 通过 IShellWindows COM 接口获取当前打开的文件夹路径
```

---

## 17. Corelia 迁移矩阵

| ZTools 模块 | Corelia Rust 方案 | 备注 |
|------------|------------------|------|
| Translation (Bergamot) | `rquickjs` + Bergamot WASM 或 Rust 原生 NLLB crate | 离线翻译核心能力需保持 |
| Sync Engine (~1775 行) | Rust `reqwest` + `heed` + `tokio` | 纯算法逻辑，直接翻译 |
| MCP Server | Rust `axum` + `tower` | HTTP 框架替换 |
| MCP Agent | Rust 或保留 JS (Webview) | Agent 逻辑复杂，可暂留前端 |
| ZBrowser | Tauri 多 WebviewWindow | 标签页管理需自建 |
| Assembly Coordinator | Rust `StateMachine` | 状态机逻辑直接翻译 |
| App Watcher | `windows-rs` + `notify` crate | 文件系统监听 + 注册表 |
| Log Collector | Rust `tracing` crate | 改用结构化日志 |
| Data Migrations | Rust + LMDB schema version | 版本号 + 迁移函数注册 |
| Icon Protocol | Tauri 自定义协议 | `tauri::protocol::ProtocolHandler` |
| Double Tap Manager | Rust `enigo` + 键盘事件 | 简单状态机 |
| Screen Capture | `xcap` crate 或 `windows-rs` BitBlt | 覆盖层窗口需 Tauri 实现 |
| Proxy Manager | `reqwest` proxy 配置 | 读取系统代理配置 |
| HTTP Server | `axum` 内嵌在 Tauri 插件中 | 端口配置 |

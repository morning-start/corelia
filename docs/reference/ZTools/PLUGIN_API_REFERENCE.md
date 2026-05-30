# ZTools Plugin API Reference

> **覆盖文件:** `src/main/api/plugin/` 下 20+ 个模块，~5400 行
> **核心价值:** ZTools 插件系统暴露了 200+ 个 IPC 方法给沙箱 Webview 插件，分为公开（external）、内部（internal）、共享（shared）三层

---

## 1. API 三层架构

```
src/main/api/
├── plugin/                   # 插件 API（Webview 沙箱使用）
│   ├── index.ts              # 入口，聚合所有公开 API
│   ├── external/             # 公开 API — 所有插件可用
│   │   ├── window.ts         # 窗口控制 API
│   │   ├── shell.ts          # 命令行/进程 API
│   │   ├── ui.ts             # UI/Toast/对话框 API
│   │   ├── storage.ts        # 本地存储 API
│   │   ├── dialog.ts         # 对话框 API
│   │   ├── app.ts            # 应用信息 API
│   │   ├── clipboard.ts      # 剪贴板读写 API
│   │   ├── plugin.ts         # 插件元数据 API
│   │   ├── tools.ts          # 工具/笔记/条目标识 API
│   │   ├── native.ts         # 原生能力 API
│   │   ├── mcp.ts            # MCP 协议 API
│   │   ├── zrowser.ts        # 内置浏览器 API
│   │   └── search.ts         # 搜索 API
│   ├── internal.ts           # 内部 API — 仅受信任插件
│   └── shell.ts              # Shell API — 预加载脚本使用
├── shared/                   # 共享库
│   ├── database.ts           # LMDB 封装
│   ├── ipc.ts                # IPC 通信工具
│   └── utils.ts              # 工具函数
└── utils/                    # API 工具
    ├── pluginPermission.ts   # 权限管理
    └── typeCast.ts           # 类型转换
```

### 权限模型

```
ztools:plugin:api-plugin_name:method-name
示例: ztools:plugin:api-my-plugin:window-show

外部插件：访问 external/ 下的方法
内部插件：额外访问 internal.ts 中的方法
Shell 插件（预加载）：额外访问 shell.ts

权限在 plugin.config.json 中声明:
{
  "apiType": "external",       // external | internal | shell
  "permissions": ["window:*", "clipboard:*"]
}
```

---

## 2. 公开 API 模块（external/）

### 2.1 window.ts — 窗口控制 API

```typescript
// 165 个 IPC 方法（通过 pluginWindowManager 暴露）
// 注册方式：按 5 个维度排列组合
//   - 7 种窗口类型: normal, fixedSize, frameless, panel, dock, overlay, popup
//   - 5 种普通状态: create, createAndShow, show, hide, close
//   - 扩展方法: setSizePosition, getOpenedWindows, updateProp, send, etc.

const windowAPI = {
  // 每种窗口类型支持: create, createAndShow, show, hide, close
  // 示例:
  normalCreate:           (opt?) => IPC.call("window:normalCreate", opt),
  normalCreateAndShow:    (opt?) => IPC.call("window:normalCreateAndShow", opt),
  normalShow:             (id)  => IPC.call("window:normalShow", id),
  normalHide:             (id)  => IPC.call("window:normalHide", id),
  normalClose:            (id)  => IPC.call("window:normalClose", id),
  // ... 同上模式重复 7 次 × 5 = 35 个基本方法

  // 扩展方法（对所有窗口类型通用）:
  setSizePosition:        (id, x, y, w, h) => IPC.call("window:setSizePosition", ...),
  getOpenedWindows:       ()              => IPC.call("window:getOpenedWindows"),
  updateProp:             (id, key, val) => IPC.call("window:updateProp", ...),
  send:                   (id, msg, data) => IPC.call("window:send", ...),
  closeAllDetached:       ()              => IPC.call("window:closeAllDetached"),
}

// 窗口属性选项:
interface WindowOptions {
  id: string
  url: string
  title: string
  x: number; y: number
  width: number; height: number
  minWidth?: number; minHeight?: number
  maxWidth?: number; maxHeight?: number
  resizable?: boolean
  frame?: boolean
  skipTaskbar?: boolean
  transparent?: boolean
  opacity?: number
  alwaysOnTop?: boolean
  webviewOptions?: {
    preload?: string
    plugin?: boolean
    nodeIntegration?: boolean
    contextIsolation?: boolean
  }
}
```

### 2.2 shell.ts — 命令行 API

```typescript
const shellAPI = {
  // 运行命令
  run: (cmd: string, opt?: {
    encoding?: string; cwd?: string; timeout?: number;
    env?: Record<string, string|undefined>;
    onStdout?: (data: string) => void;  // 流式输出回调
    onStderr?: (data: string) => void;
  }) => Promise<string>,  // 返回合并输出

  // 打开文件/URL
  openExternal: (url: string) => Promise<void>,

  // 创建子进程（异步 + 流式）
  spawn: (cmd: string, args?: string[], opt?: {
    cwd?: string; env?: Record<string, string>;
  }) => Promise<{
    stdin: (data: string) => void;
    onStdout: (cb: (data: string) => void) => void;
    onStderr: (cb: (data: string) => void) => void;
    onExit: (cb: (code: number) => void) => void;
    kill: (signal?: string) => void;
  }>,
}
```

### 2.3 ui.ts — UI/Toast API

```typescript
const uiAPI = {
  showToast: (opt: {
    type: 'success' | 'error' | 'warning' | 'info' | 'question'
    message: string
    id?: string         // 去重 ID
    duration?: number   // 默认 3000
    showIcon?: boolean
    iconColor?: string
    buttons?: Array<{   // 按钮
      label: string
      onClick: () => void
      type?: 'primary' | 'default' | 'text'
    }>
    onClose?: () => void
  }) => Promise<string>,  // 返回 toast id

  dismissToast: (id: string) => Promise<void>,

  // 彩色终端日志（在 ZTools DevTools Console 中彩色显示）
  log: (message: string, type: 'info'|'success'|'warning'|'error') => void,

  // 全局对话框（模态的）
  showConfirmDialog: (opt: {
    title?: string; content: string;
    confirmText?: string; cancelText?: string;
    type?: 'primary' | 'danger'; iconColor?: string;
  }) => Promise<boolean>,
}
```

### 2.4 storage.ts — 本地存储 API

```typescript
// 每个插件有隔离的命名空间（ztools:plugin:storage:plugin_name:*）
const storageAPI = {
  set:  (key: string, value: any) => Promise<void>,
  get:  (key: string, defaultValue?: any) => Promise<any>,
  del:  (key: string) => Promise<void>,
  keys: () => Promise<string[]>,
  clear: () => Promise<void>,
}
```

### 2.5 dialog.ts — 系统对话框 API

```typescript
const dialogAPI = {
  openFile:    (opt: FileDialogOptions) => Promise<string[]>,
  saveFile:    (opt: SaveDialogOptions) => Promise<string>,
  openFolder:  (opt?: { defaultPath?: string }) => Promise<string[]>,
}
```

### 2.6 app.ts — 应用信息 API

```typescript
const appAPI = {
  getVersion: () => Promise<string>,      // 应用版本
  getPath:    (name: string) => Promise<string>,  // userData, home, desktop, downloads, etc.
  getName:    () => Promise<string>,      // 应用名称
  getLocale:  () => Promise<string>,      // 系统语言
  getPlatform:() => Promise<string>,      // win32/darwin/linux
  exit:       () => Promise<void>,        // 退出应用
  restart:    () => Promise<void>,        // 重启应用
}
```

### 2.7 clipboard.ts — 剪贴板 API

```typescript
const clipboardAPI = {
  readText:       () => Promise<string>,
  writeText:      (text: string) => Promise<void>,
  readImage:      () => Promise<string>,       // Base64
  writeImage:     (base64: string) => Promise<void>,
  readHtml:       () => Promise<string>,
  writeHtml:      (html: string) => Promise<void>,
  readRtf:        () => Promise<string>,
  writeRtf:       (rtf: string) => Promise<void>,
  clear:          () => Promise<void>,
  hasImage:       () => Promise<boolean>,
  getFiles:       () => Promise<ClipboardFile[]>,
  setFiles:       (files: Array<{ path: string }>) => Promise<boolean>,
  startMonitor:   () => Promise<void>,  // 启动剪贴板变化监听
  stopMonitor:    () => Promise<void>,
}
```

### 2.8 plugin.ts — 插件元数据 API

```typescript
const pluginAPI = {
  getPlugins:       () => Promise<PluginManifest[]>,
  getActivePlugins: () => Promise<string[]>,
  openPluginDir:    () => Promise<void>,
  reloadPlugin:     (name: string) => Promise<void>,
  disablePlugin:    (name: string) => Promise<void>,
  enablePlugin:     (name: string) => Promise<void>,
  installPlugin:    (path: string) => Promise<void>,
  uninstallPlugin:  (name: string) => Promise<void>,
  openStore:        () => Promise<void>,
}
```

### 2.9 tools.ts — 工具/笔记/条目 API

```typescript
// 用于在 ZTools 中创建条目
const toolsAPI = {
  createNote:       (content: string) => Promise<string>,  // 返回条目 ID
  getNotes:         () => Promise<Note[]>,
  deleteNote:       (id: string) => Promise<void>,
  createSeparator:  () => Promise<void>,
  createCommand:    (opt: {
    name: string; path: string;
    icon?: string; tags?: string[];
  }) => Promise<string>,
}
```

### 2.10 native.ts — 原生能力 API

```typescript
const nativeAPI = {
  getActiveWindow:   () => Promise<WindowInfo>,
  startScreenCapture:() => Promise<CaptureResult>,
  startColorPicker:  () => Promise<ColorResult>,
  getPlatform:        () => string,
  getUwpApps:         () => Promise<UwpAppInfo[]>,
  getFileIcon:        (path: string) => Promise<string>,  // Base64 PNG
}
```

### 2.11 mcp.ts — MCP 协议 API

```typescript
const mcpAPI = {
  getResources:       () => Promise<McpResource[]>,
  executeQuery:       (query: string) => Promise<any>,
  openInVSCode:       (filePath: string, line?: number) => Promise<void>,
  openInTerminal:     (cwd?: string) => Promise<void>,
  openInFinder:       (filePath: string) => Promise<void>,
  openBrowserUrl:     (url: string) => Promise<void>,
  getClipboardHistory:() => Promise<ClipboardItem[]>,
  searchClipboard:    (query: string) => Promise<ClipboardItem[]>,
  getInstalledApps:   () => Promise<Command[]>,
  searchApps:         (query: string) => Promise<Command[]>,
  getCurrentWindowInfo:() => Promise<WindowInfo>,
  createSecretNote:   (content: string) => Promise<string>,
  getSystemInfo:      () => Promise<SystemInfo>,
  getForegroundAppIcon:() => Promise<string | null>,
}
```

### 2.12 zrowser.ts — 内置浏览器 API

```typescript
const zrowserAPI = {
  createTab:          (url: string) => Promise<TabInfo>,
  closeTab:           (tabId: string) => Promise<void>,
  navigateTab:        (tabId: string, url: string) => Promise<void>,
  getTabs:            () => Promise<TabInfo[]>,
  getActiveTab:       () => Promise<TabInfo | null>,
  setActiveTab:       (tabId: string) => Promise<void>,
  captureTab:         (tabId: string) => Promise<string>,  // Base64 screenshot
  executeScript:      (tabId: string, code: string) => Promise<any>,
  injectCSS:          (tabId: string, css: string) => Promise<void>,
  toggleDevTools:     (tabId: string) => Promise<void>,
  zoomIn:             (tabId: string) => Promise<void>,
  zoomOut:            (tabId: string) => Promise<void>,
  resetZoom:          (tabId: string) => Promise<void>,
  goBack:             (tabId: string) => Promise<void>,
  goForward:          (tabId: string) => Promise<void>,
  reload:             (tabId: string) => Promise<void>,
  bookmarkPage:       (tabId: string) => Promise<void>,
  getBookmarks:       () => Promise<Bookmark[]>,
}
```

### 2.13 search.ts — 搜索 API

```typescript
const searchAPI = {
  search:     (query: string, opt?: SearchOptions) => Promise<SearchResult[]>,
  searchAll:  (query: string) => Promise<{
    apps: SearchResult[]
    clipboard: SearchResult[]
    files: SearchResult[]
    notes: SearchResult[]
    web: SearchResult[]
  }>,
}
```

---

## 3. 内部 API（internal.ts — 1231 行）

仅供受信任的内部插件使用（权限: `ztools:plugin:internal:api:true`）。

### 3.1 获取内部 API 上下文

```typescript
function getInternalContext(pluginName: string): InternalContext {
  return {
    getMainWindow:          () => BrowserWindow,
    getSetting:             (key: string) => any,
    setSetting:             (key: string, val: any) => void,
    onSetting:              (key: string, cb: (val: any) => void) => void,
  }
}
```

### 3.2 内部 API 方法

```typescript
const internalAPI = {
  // == 设置 ==
  openGlobalSettings:   () => void,
  openShortcutSettings: () => void,
  openPluginSettings:   (pluginName: string) => void,
  openThemeSettings:    () => void,
  openAboutWindow:      () => void,

  // == 应用 ==
  restartApplication:   () => void,
  checkForUpdates:      () => void,
  exitApplication:      () => void,
  hideApplication:      () => void,
  showMainWindow:       () => void,
  hideMainWindow:       () => void,
  toggleMainWindow:     () => void,

  // == 窗口操作 ==
  showNotification:     (opt: NotificationOptions) => string,
  dismissNotification:  (id: string) => void,
  openModal:            (opt: ModalOptions) => void,
  closeModal:           () => void,
  showSearchBar:        () => void,
  hideSearchBar:        () => void,
  toggleSearchBar:      () => void,

  // == 插件管理 ==
  loadMainPlugin:       (pluginName: string) => void,
  unloadMainPlugin:     (pluginName: string) => void,
  triggerMain:          (pluginName: string) => void,  // 触发插件的主要 action

  // == 超级面板 ==
  showSuperPanel:       (x: number, y: number, keyword?: string) => void,
  hideSuperPanel:       () => void,
  updateSuperPanel:     (opt: SuperPanelOptions) => void,
  addSuperPanelAction:  (opt: ActionConfig) => void,
  clearSuperPanelActions: () => void,

  // == 浮动球 ==
  showFloatingBall:     (id: string, opt: FloatingBallOptions) => void,
  hideFloatingBall:     () => void,

  // == 快捷键 ==
  registerShortcut:     (key: string, cb: () => void) => string,  // 返回 id
  unregisterShortcut:   (id: string) => void,
  getRegisteredShortcuts: () => ShortcutInfo[],
  setGlobalShortcut:    (key: string, cb: () => void) => string,

  // == 剪贴板 ==
  setClipboard:         (items: ClipboardItem[]) => void,
  deleteClipboardItem:  (id: string) => void,
  clearClipboardHistory:() => void,
  getClipboardHistory:  () => ClipboardItem[],
  searchClipboard:      (query: string) => ClipboardItem[],
  cycleClipboardForward:() => void,
  cycleClipboardBackward:() => void,
  pasteClipboardItem:   (id: string) => void,
  pasteClipboardText:   (text: string) => void,

  // == 翻译 ==
  translateText:        (text: string, sourceLang?: string) => Promise<TranslationResult>,
  translateClipboard:   () => Promise<void>,

  // == 系统 ==
  getSystemInfo:        () => SystemInfo,
  getAppMetrics:        () => AppMetrics,
  openDevTools:         () => void,
  openPluginDevTools:   (pluginName: string) => void,
}
```

### 3.3 Shell API（preload shell.ts）

供预加载脚本使用的特殊 API。通过 `contextBridge` 暴露。

```typescript
// 预加载脚本可用的 API
const shellAPI = {
  // 原生方法调用（IPC 代理）
  native: {
    launchApp:        (path: string) => void,
    activateWindow:   (id: string) => void,
  },
  // 插件窗口管理
  pluginWindow: {
    create:           (opt: any) => void,
    close:            (id: string) => void,
    send:             (id: string, msg: string, data: any) => void,
  },
  // 事件
  on:                 (channel: string, cb: (...args: any[]) => void) => void,
  off:                (channel: string, cb: Function) => void,
}
```

---

## 4. 内部数据模型

```typescript
interface PluginManifest {
  name: string; version: string; author: string;
  description: string; homepage: string;
  main: string;                    // 入口文件
  preload?: string;                 // 预加载脚本
  icon?: string;
  apiType: 'external' | 'internal' | 'shell';
  permissions: string[];
  shortcuts?: { key: string; action: string }[];
  featureType?: 'application' | 'clipboard' | 'search' | 'note' | 'tool';
  superPanel?: { default: string; actions: string[] };
}

interface PluginConfig {
  entry: {
    main: string;           // 主入口（webview URL）
    preload?: string;        // 预加载脚本
  }
  window: {
    type: 'main' | 'detached' | 'floatingBall'
    width: number; height: number;
    [key: string]: any
  }
  api: {
    type: 'external' | 'internal' | 'shell'
    permissions: string[]
  }
  shortcuts: {
    key: string; action: string;
  }[]
  superPanel: {
    default: string          // 默认 action
    actions: string[]        // 可用 actions
  }
  featureType?: 'application' | 'clipboard' | 'search' | 'note' | 'tool'
}

interface PluginInstance {
  config: PluginConfig
  manifest: PluginManifest
  window: BrowserWindow | null
  webview: WebviewTag | null
  apiContext: InternalContext | null
  apiHandler: { [method: string]: Function }
}

interface ClipboardItem {
  id: string; type: 'text' | 'image' | 'file' | 'link' | 'html' | 'rtf';
  value: string;               // 文本/Base64/路径
  window?: string;              // 来源窗口名
  appPath?: string;             // 来源应用路径
  createdAt: number;
  isStarred?: boolean;
  tags?: string[];
  groupId?: string;             // 批量粘贴分组
}
```

---

## 5. IPC 通信架构

### 5.1 Electron IPC 通道

```typescript
// 插件 Webview → 主进程
// 通道名: ztools:plugin:ipc
// 路由: ipcMain.handle('ztools:plugin:ipc', handler)

// 插件侧调用:
window.ipc.call('clipboard:readText', [])
// 被路由到 clipboardAPI.readText()

// 主进程 → 插件 Webview
// 通过 pluginWindowManager.send(pluginName, msg, data)
// 插件侧监听:
window.ipc.on('plugin:message', (msg, data) => { ... })
```

### 5.2 IPC 路由表

```typescript
// 路由: moduleName:methodName → handler
// 如 'window:normalCreate' → pluginWindowManager.normalCreate
// 如 'clipboard:readText' → clipboardAPI.readText

// 所有公开方法按模块分组:
RouteMap = {
  'window':     windowMethods,      // 165 个
  'clipboard':  clipboardMethods,   // 12 个
  'shell':      shellMethods,       // 3 个
  'ui':         uiMethods,          // 4 个
  'storage':    storageMethods,     // 5 个
  'dialog':     dialogMethods,      // 3 个
  'app':        appMethods,         // 6 个
  'plugin':     pluginMethods,      // 8 个
  'tools':      toolsMethods,       // 5 个
  'native':     nativeMethods,      // 5 个
  'mcp':        mcpMethods,         // 15 个
  'zrowser':    zrowserMethods,     // 17 个
  'search':     searchMethods,      // 2 个
  'internal':   internalMethods,    // 40+ 个
}
```

---

## 6. Corelia 迁移要点

| 模块 | ZTools (Electron) | Corelia (Tauri + Svelte) |
|------|-------------------|--------------------------|
| IPC 路由 | `ipcMain.handle` + 字符串路由 | Tauri `#[tauri::command]` + `invoke` |
| 窗口控制 | `pluginWindowManager` (165 方法) | `tauri::window::WindowBuilder` + 自定义窗口管理 |
| 流式输出 | `BrowserWindow.webContents.on('console')` | Tauri 事件系统 `app.emit()` / `window.emit()` |
| 剪贴板 | `arboard` 通过 C++ addon | `arboard` crate 直接调用 |
| 插件隔离 | WebviewTag + preload | Tauri WebviewWindow + 自定义权限 |
| 权限 | 字符串权限 check + config | Tauri capabilities 系统 |
| 存储命名空间 | LMDB 带命名空间 key | Tauri store plugin + 命名空间 |
| 通知 | `Notification` HTML5 API | `tauri-plugin-notification` |
| 对话框 | `dialog.showOpenDialog` | `tauri-plugin-dialog` |
| 文件系统 | `fs` 模块 | `tauri-plugin-fs` |
| Shell | `child_process` | `tauri-plugin-shell` |

### 关键架构差异

1. **IPC 不再中心化路由** — Tauri 中每个 command 是独立函数，通过 `#[tauri::command]` 注册，不再需要一个巨大的 switch-case 路由表

2. **窗口控制简化** — 165 个方法在 Tauri 中可以用更少的 API 完成（创建窗口 + 属性更新 + 关闭），因为 `tauri::window::WindowBuilder` 更灵活

3. **插件 API 分层保留** — 相同的外/内/Shell 三层架构可以保留，通过 capabilities + 权限检查实现

4. **流式 IPC 用事件** — Tauri 的事件系统 (`app.emit`/`listen`) 替代 Electron 的 `webContents.send`

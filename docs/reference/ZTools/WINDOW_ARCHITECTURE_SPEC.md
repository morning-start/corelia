# ZTools 窗口系统架构参考

> **覆盖文件:** `src/main/core/superPanelManager.ts` (753 行), `detachedWindowManager.ts` (600 行), `pluginWindowManager.ts` (411 行), `floatingBallManager.ts` (351 行), `utils/windowUtils.ts` (111 行)
> **核心价值:** ZTools 管理 4+ 种窗口类型（主窗口、超级面板、浮动球、插件窗口、分离窗口），各有独立的生命周期管理

---

## 1. 窗口类型总览

| 类型 | 管理器 | 创建方式 | 生命周期 | 特点 |
|------|--------|---------|----------|------|
| 主窗口 | `windowManager.ts` | 应用启动 | 持久 | 搜索栏 + 主界面 |
| 超级面板 | `superPanelManager.ts` | 鼠标触发 | 临时显示 → 自动隐藏 | 透明窗口，鼠标跟随 |
| 浮动球 | `floatingBallManager.ts` | 插件请求 | 手动关闭 | 半透明小球，拖拽位置 |
| 插件窗口 | `pluginWindowManager.ts` | 插件 IPC 请求 | 插件控制 | 165 种控制方法 |
| 分离窗口 | `detachedWindowManager.ts` | 插件窗口分离 | 独立关闭 | 从主窗口 Webview 分离出去 |
| 更新窗口 | (单独管理) | 更新检测触发 | 临时 | 更新进度界面 |
| Toast 窗口 | `toast.ts` | API 调用 | 自动消失 | 非阻塞通知 |

---

## 2. 超级面板（SuperPanelManager）— 753 行

### 2.1 定位

**触发的窗口类型:** `BrowserWindow` + 无框 + 透明 + `skipTaskbar:true`
**功能:** 鼠标中键/右键长按触发的悬浮菜单

### 2.2 核心数据流

```
鼠标中键/右键长按
  → MouseMonitor 回调
  → superPanelManager.show(x, y)
      → 计算最佳显示位置（防止超出屏幕）
      → 获取剪贴板内容作为上下文
      → 根据上下文加载匹配插件的 Action
      → 创建/更新 BrowserWindow
      → 设置窗口位置到鼠标附近
      → 加载 renderer/superPanel/index.html
      → 发送 context 数据给渲染进程
```

### 2.3 位置计算

```typescript
function calculatePosition(x: number, y: number, winSize: Size) {
  const display = screen.getDisplayNearestPoint({ x, y })
  const workArea = display.workArea  // 排除任务栏的安全区域

  // 默认在鼠标右下方
  let winX = x + 10
  let winY = y + 10

  // 超出右侧边界 → 放在鼠标左侧
  if (winX + winSize.width > workArea.x + workArea.width) {
    winX = x - winSize.width - 10
  }
  // 超出下方边界 → 放在鼠标上方
  if (winY + winSize.height > workArea.y + workArea.height) {
    winY = y - winSize.height - 10
  }
  // 超出左/上边界 → 回退到边缘
  if (winX < workArea.x) winX = workArea.x + 5
  if (winY < workArea.y) winY = workArea.y + 5

  return { x: winX, y: winY }
}
```

### 2.4 Action 管理

```typescript
interface SuperPanelAction {
  id: string                    // 唯一标识
  name: string                  // 显示名称
  description: string           // 说明
  icon: string                  // Base64 SVG / 图标 URL
  plugin: string                // 所属插件
  handler: () => void           // 点击回调
  matchType?: 'exact' | 'fuzzy' // 匹配方式
  matchRule?: string            // 匹配规则
}

// 注册 Action（插件通过 internal API 调用）
superPanelManager.addAction({
  id: 'translate-selected',
  name: '翻译选中文本',
  description: '使用百度翻译',
  icon: 'data:image/svg+xml;base64,...',
  plugin: 'translate-plugin',
  handler: () => translateClipboard()
})

// 清空 Action（切换上下文时）
superPanelManager.clearActions()

// 获取当前 Action 列表（渲染进程）
superPanelManager.getActions()  // SuperPanelAction[]
```

### 2.5 生命周期

```typescript
// 显示
show(x, y, keyword?) {
  if (!this.window) this.createWindow()
  this.updatePosition(x, y)
  const actions = this.matchActions(keyword || clipboard.readText())
  this.window.webContents.send('actions:update', actions)
  this.window.show()
  this.window.focus()
}

// 隐藏（自动）
hide() {
  this.window.hide()  // 不销毁，保留以备下次显示
}

// 销毁（应用退出/插件重载时）
destroy() {
  this.window.close()
  this.window = null
}

// 自动隐藏逻辑: 窗口失焦 → 隐藏
this.window.on('blur', () => this.hide())
```

### 2.6 窗口屏蔽列表

```typescript
// 某些窗口触发超级面板时不显示（如全屏游戏、截图工具等）
const BLOCKED_WINDOW_CLASSES = [
  'ConsoleWindowClass',     // 控制台
  'QQGame_ForMoba',         // 全屏游戏
  'UnityWndClass',          // Unity 游戏
]

// 激活窗口属于屏蔽列表 → 不显示超级面板
WindowMonitor.getActiveWindow().className 是否在 BLOCKED_WINDOW_CLASSES 中
```

---

## 3. 浮动球（FloatingBallManager）— 351 行

### 3.1 定位

**窗口类型:** 无框 `BrowserWindow` + `alwaysOnTop: true` + `skipTaskbar: true`
**功能:** 插件创建的小型悬浮控件，拖拽到任意位置

### 3.2 核心接口

```typescript
interface FloatingBallOptions {
  id: string               // 唯一标识
  url: string              // 显示内容（HTML）
  size: { width: number; height: number }
  position: { x: number; y: number }
  opacity?: number         // 默认 0.8
  draggable?: boolean      // 默认 true
  autoHideOnBlur?: boolean // 默认 false
}

// 创建/显示
show(id, options) {
  let win = this.windows.get(id)
  if (!win) {
    win = new BrowserWindow({
      width: options.size.width,
      height: options.size.height,
      x: options.position.x, y: options.position.y,
      frame: false,
      transparent: true,
      alwaysOnTop: true,
      skipTaskbar: true,
      resizable: false,
      webviewOptions: { plugin: true }
    })
    win.loadURL(options.url)
    this.windows.set(id, win)
  }
  win.show()
  win.focus()
}

// 隐藏/关闭
hide(id) { this.windows.get(id)?.hide() }
close(id) { this.windows.get(id)?.close(); this.windows.delete(id) }
closeAll() { this.windows.forEach(w => w.close()); this.windows.clear() }
```

### 3.3 拖拽实现

```typescript
// 渲染进程处理拖拽，通过 IPC 告知主进程更新位置
// renderer/ 中监听 drag 事件 → ipc.send('floatingBall:drag', { x, y })
// 主进程更新: win.setPosition(x, y)
```

### 3.4 IPC 通道

| 通道 | 方向 | 用途 |
|------|------|------|
| `floatingBall:drag` | 渲染→主 | 拖拽位置更新 |
| `floatingBall:action` | 渲染→主 | 用户点击 action |
| `floatingBall:message` | 主→渲染 | 插件发送数据 |
| `floatingBall:update` | 主→渲染 | 主进程更新选项 |

---

## 4. 插件窗口（PluginWindowManager）— 411 行

### 4.1 定位

管理插件创建的 BrowserWindow 实例。核心是 165 个方法（7 种窗口类型 × 5 种基本操作 + 扩展方法）。

### 4.2 7 种窗口类型

```typescript
enum WindowType {
  NORMAL    = 'normal',     // 标准窗口
  FIXED     = 'fixedSize',  // 固定大小窗口（不可缩放）
  FRAMELESS = 'frameless',  // 无框窗口
  PANEL     = 'panel',      // 面板（固定在上方）
  DOCK      = 'dock',       // 停靠窗口（贴在屏幕边缘）
  OVERLAY   = 'overlay',    // 覆盖层（半透明叠加）
  POPUP     = 'popup',      // 弹出窗口
}

// 每种类型共有的 WindowOptions:
interface WindowOptions {
  id: string                     // 窗口标识（插件内唯一）
  url: string                    // 加载的 URL
  title?: string                  // 标题
  x?: number; y?: number          // 位置
  width?: number; height?: number // 尺寸
  minWidth?: number; minHeight?: number
  maxWidth?: number; maxHeight?: number
  alwaysOnTop?: boolean
  opacity?: number
  skipTaskbar?: boolean
  showOnCreate?: boolean          // 创建后立即显示
  resizable?: boolean
  frame?: boolean
  transparent?: boolean
  webviewOptions?: {
    preload?: string              // 预加载脚本
    plugin?: boolean              // 是否为插件窗口
    nodeIntegration?: boolean
    contextIsolation?: boolean
  }
}
```

### 4.3 核心方法（165 个的生成模式）

```typescript
// 每种类型生成 5 个基本方法
// create, createAndShow, show, hide, close
// 命名: ${type}Create, ${type}CreateAndShow, ${type}Show, ${type}Hide, ${type}Close

// 加上扩展方法:
interface PluginWindowManager {
  // 批量操作
  hideAllWindows(): void
  closeAllWindows(): void

  // 属性更新
  updateProp(id: string, key: string, value: any): void

  // 大小位置
  setSizePosition(id: string, x: number, y: number, w: number, h: number): void
  getOpenedWindows(): Array<{ id: string; type: string; bounds: Rectangle }>

  // 通信
  send(id: string, msg: string, data: any): void  // 主进程 → 插件窗口
  broadcast(msg: string, data: any): void          // 广播给所有插件窗口

  // 特殊
  closeAllDetached(): void   // 关闭所有分离窗口
}
```

### 4.4 窗口追踪

```typescript
// 维护一个 Map<string, OpenedWindow>
private openedWindows: Map<string, {
  id: string
  type: WindowType
  browserWindow: BrowserWindow
  pluginName: string
  createdAt: number
}>

// 自动清理（窗口关闭时移除追踪）
browserWindow.on('closed', () => {
  this.openedWindows.delete(id)
})

// 获取所有打开的窗口（供 IPC 查询）
getOpenedWindows() {
  return Array.from(this.openedWindows.values()).map(w => ({
    id: w.id,
    type: w.type,
    bounds: w.browserWindow.getBounds()
  }))
}
```

---

## 5. 分离窗口（DetachedWindowManager）— 600 行

### 5.1 定位

管理从插件主窗口 Webview 分离出来的独立窗口。分离窗口与父窗口保持通信。

### 5.2 分离流程

```
插件 Webview 内部调用:
  → window.ipc.call('window:dockCreate', { url: 'https://example.com', title: '窗口' })
  → 主进程创建新的 BrowserWindow
  → 新的 BrowserWindow 共享同一 Webview 或创建新 Webview
  → 返回新窗口的 id 给插件
```

### 5.3 窗口池管理

```typescript
// 维护分离窗口的池
private detachedWindows: Map<string, DetachedWindow> = new Map()

interface DetachedWindow {
  id: string
  window: BrowserWindow
  parentId: string          // 父 Webview 标签页 ID
  pluginName: string
  webviewId: string         // 对应的 Webview 实例 ID
  attached: boolean          // 是否已分离
}

// 分离: 从主窗口 Webview → 独立 BrowserWindow
detach(tabId: string, options?: Partial<WindowOptions>): string {
  // 1. 从 ZBrowser Webview 获取 tab
  // 2. 创建新的 BrowserWindow
  // 3. 将 Webview 内容移动到新窗口（或创建新 Webview）
  // 4. 设置窗口属性
  // 5. 返回 detachedId
}

// 合并回父窗口
attach(detachedId: string): void {
  // 1. 从独立 BrowserWindow 移除 Webview
  // 2. 将 Webview 移回父窗口的 Tab
  // 3. 关闭独立 BrowserWindow
  // 4. 从 Map 移除
}
```

### 5.4 IPC 通信

```typescript
// 分离窗口 ↔ 父窗口
// 父窗口发送:
parent.send(detachedId, 'message', data)
// 分离窗口发送:
detached.ipc.on('message', (data) => { ... })

// 分离窗口关闭时通知父窗口:
detached.window.on('closed', () => {
  parent.webContents.send('detached:closed', detachedId)
})
```

---

## 6. 窗口生命周期对比

| 阶段 | 主窗口 | 超级面板 | 浮动球 | 插件窗口 | 分离窗口 |
|------|--------|---------|--------|---------|---------|
| 创建 | new BrowserWindow({...}) | MouseMonitor 触发 | 插件 IPC 请求 | 插件 IPC 请求 | Webview 分离 |
| 显示 | 快捷键/搜索 | show + focus | show | create(show:true) | detach() |
| 失焦 | 保持 | 隐藏 | 可选隐藏 | 保持 | 保持 |
| 隐藏 | hideWindow() | hide (保留) | hide (保留) | hide (保留) | 无 |
| 关闭 | 应用退出 | destroy() | close (删除) | close (删除) | attach()/close |
| 销毁 | 应用退出 | 插件重载时 | 插件重载时 | 插件重载时 | 插件重载时 |

---

## 7. 窗口材质样式（Window Utils）

```typescript
// 111 行 — Windows 11 原生材质支持
interface WindowMaterialOptions {
  material: 'mica' | 'acrylic' | 'none'
  darkMode?: boolean
  accentColor?: string       // #RRGGBB
  useHostBackdrop?: boolean  // Windows 11 acrylic
}

function applyWindowMaterial(window: BrowserWindow, opt: WindowMaterialOptions): void {
  if (process.platform !== 'win32') return
  try {
    // Mica (Windows 11 22H2+)
    if (opt.material === 'mica') {
      native.windowManager.setMica(window.getNativeWindowHandle())  // C++ 调用 DwmSetWindowAttribute
    }
    // Acrylic
    if (opt.material === 'acrylic') {
      native.windowManager.setAcrylic(window.getNativeWindowHandle())
    }
  } catch (e) { /* 降级处理 */ }
}
```

**Corelia 映射：** Tauri 2.x 支持窗口 `decorations` 和 `transparent` 设置。Windows 11 Mica 材质需要通过 Rust `windows-rs` 的 `DwmSetWindowAttribute(DWMWA_MICA)` 实现。

---

## 8. 超级面板 vs 浮动球 vs 插件窗口 — 选型指南

| 场景 | 推荐类型 | 原因 |
|------|---------|------|
| 鼠标触发上下文菜单 | 超级面板 | 自动定位 + 自动隐藏 |
| 常驻小工具（天气、时钟） | 浮动球 | alwaysOnTop + 拖拽 + 透明 |
| 完整功能窗口 | 插件窗口 (normal) | 标准窗口行为 |
| 弹出菜单/选择器 | 插件窗口 (popup) | 小尺寸 + 自动消失 |
| 辅助工具覆盖层 | 插件窗口 (overlay) | 半透明 + 非交互 |
| 侧边面板 | 插件窗口 (panel) | 固定位置 |
| 独立标签页 | 分离窗口 | 从浏览器分离 |

---

## 9. Corelia 迁移策略

| ZTools 窗口特性 | Corelia Tauri 方案 |
|-----------------|-------------------|
| `BrowserWindow` 无框透明 | `WindowBuilder.transparent(true)` + `.decorations(false)` |
| `alwaysOnTop: true` | `.always_on_top(true)` |
| `skipTaskbar: true` | `.skip_taskbar(true)` |
| `screen.getDisplayNearestPoint()` | `monitor::Monitor::from_point()` |
| `win.setPosition(x, y)` | `.window.set_position()` |
| `win.on('blur')` | `window.on_window_event(WindowEvent::Focused)` |
| `win.webContents.send()` | `window.emit()` |
| `ipcMain.handle('ztools:plugin:ipc')` | `#[tauri::command]` 单独注册 |
| 165 个窗口方法 | 缩减为: `create_window`, `close_window`, `update_window`, `send_to_window` 4 个 command + 通用属性参数 |
| 窗口池 Map | `State<HashMap<String, Window>>` + Arc/RwLock |

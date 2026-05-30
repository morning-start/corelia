# Rubick IPC 与 API 参考

> **覆盖源码**: `src/main/common/api.ts` (441 行), `public/preload.js` (250 行), `src/main/common/db.ts` (73 行)
> **核心问题**: 单一 `msg-trigger` 通道如何承载全部 IPC？`API extends DB` 继承的意义？插件 API 表面有多大？

---

## 1. IPC 架构概览

```mermaid
graph TB
    subgraph "渲染进程"
        RENDERER[Vue 3 渲染进程]
        PRELOAD[preload.js<br/>window.rubick.*]
        DIRTY[直接 Electron API<br/>clipboard, shell, remote]
    end

    subgraph "IPC 通道"
        MSG[msg-trigger<br/>唯一的 IPC 通道]
        IPC_EVENT[其他事件<br/>re-register, global-short-key<br>detach:service, guide:service]
    end

    subgraph "主进程"
        API[API 类 extends DB]
        DB[PouchDB 封装]
    end

    RENDERER -->|sendSync| MSG
    PRELOAD -->|sendSync| MSG
    DIRTY -->|@electron/remote<br/>直接 API| API
    
    MSG --> API
    API --> DB
    
    RENDERER -->|ipcRenderer.on| IPC_EVENT
```

**关键决策**：Rubick 使用**同步 IPC**（`sendSync` + `event.returnValue`）而非异步 IPC（`invoke` + `handle`）。这意味着渲染进程调用 API 时会阻塞等待返回。

```typescript
// preload.js
const ipcSendSync = (type, data) => {
  const returnValue = ipcRenderer.sendSync('msg-trigger', { type, data })
  if (returnValue instanceof Error) throw returnValue
  return returnValue
}

// main process
ipcMain.on('msg-trigger', async (event, arg) => {
  const data = await this[arg.type](arg, window, event)
  event.returnValue = data
})
```

---

## 2. API 类完整方法清单

`API extends DBInstance` — 通过继承同时获得 IPC 处理器和数据库访问能力。

### 2.1 数据库操作（继承自 DBInstance）

```typescript
class DBInstance {
  public dbPut({ data })          // 创建/更新文档
  public dbGet({ data })          // 读取文档 by _id
  public dbRemove({ data })       // 删除文档
  public dbBulkDocs({ data })     // 批量写入
  public dbAllDocs({ data })      // 按 key 范围/keys 查询
  public dbPostAttachment({ data }) // 写入附件
  public dbGetAttachment({ data })  // 读取附件
  public dbGetAttachmentType({ data }) // 获取附件 MIME 类型
  public dbDump({ data })         // WebDAV 导出
  public dbImport({ data })       // WebDAV 导入
}
```

### 2.2 窗口控制

```typescript
showMainWindow()       // 显示主窗口
hideMainWindow()       // 隐藏主窗口
windowMoving(args)     // 窗口拖拽跟随鼠标
setExpendHeight(args)  // 设置窗口高度（插件内容展开）
```

### 2.3 插件生命周期

```typescript
loadPlugin(args, window)      // 加载插件到 BrowserView
openPlugin(args, window)      // 打开插件
removePlugin(event, window)   // 移除当前插件
openPluginDevTools()          // 打开插件 DevTools
detachPlugin(event, window)   // 分离插件到独立窗口
detachInputChange(args)       // 分离窗口输入变更
```

### 2.4 子输入框管理

```typescript
setSubInput(args, window, event)    // 设置子输入框（插件接管搜索框）
subInputBlur()                      // 子输入框失焦
sendSubInputChangeEvent(args)       // 子输入框内容变化通知
removeSubInput(args, window, event) // 移除子输入框
setSubInputValue(args, window, event) // 设置子输入框内容
```

### 2.5 剪贴板操作

```typescript
copyImage(args)   // 写入图片到剪贴板
copyText(args)    // 写入文本到剪贴板
copyFile(args)    // 写入文件到剪贴板
getCopyFiles()    // 读取剪贴板文件列表
```

### 2.6 系统功能

```typescript
getPath(args)                  // 获取系统路径 (userData, home, desktop...)
getLocalId()                   // 获取本地唯一 ID (基于 home 目录)
showNotification(args)         // 显示系统通知
getFeatures()                  // 获取当前插件 feature 列表
setFeature(args, window)       // 注册 feature
removeFeature(args, window)    // 移除 feature
showOpenDialog(args, window)   // 系统打开文件对话框
showSaveDialog(args, window)   // 系统保存文件对话框
shellShowItemInFolder(args)    // 在文件管理器中显示
shellBeep()                    // 系统提示音
screenCapture(args, window)    // 屏幕截图
getFileIcon(args)              // 获取文件图标
simulateKeyboardTap(args)      // 模拟按键输入
sendPluginSomeKeyDownEvent(args) // 发送按键事件到插件
```

### 2.7 快捷键生命周期

```typescript
// 在 api.ts 中通过 mainWindow.webContents.on('before-input-event') 处理
__EscapeKeyDown(event, input, window):  // ESC 键处理
// 1. 有当前插件 → 移除插件
// 2. 无插件 → 隐藏主窗口
```

---

## 3. preload.js 暴露的完整 API

`public/preload.js` 定义 `window.rubick` 对象，插件 Webview 通过它调用主进程能力。

### 3.1 插件生命周期钩子

```javascript
window.rubick = {
  hooks: {
    onPluginEnter: Function,   // 插件激活时
    onPluginReady: Function,   // 插件就绪
    onPluginOut: Function,     // 插件退出
    onShow: Function,          // 窗口显示
    onHide: Function,          // 窗口隐藏
    onSubInputChange: Function,// 子输入框内容变化
    onScreenCapture: Function, // 截图完成回调
  },

  onPluginEnter(cb) { this.hooks.onPluginEnter = cb }
  onPluginReady(cb) { this.hooks.onPluginReady = cb }
  onPluginOut(cb)   { this.hooks.onPluginOut = cb }
  onShow(cb)        { this.hooks.onShow = cb }
  onHide(cb)        { this.hooks.onHide = cb }
```

### 3.2 窗口控制

```javascript
  hideMainWindow()     // 隐藏主窗口
  showMainWindow()     // 显示主窗口
  setExpendHeight(h)   // 设置插件内容区域高度
  openPlugin(plugin)   // 打开另一个插件
  removePlugin()       // 移除自身
  outPlugin()          // 同上，别名
```

### 3.3 数据库

```javascript
  db: {
    put(data)                   // 创建/更新文档
    get(id)                     // 读取文档
    remove(doc)                 // 删除文档
    bulkDocs(docs)              // 批量操作
    allDocs(key)                // 按 key 查询
    postAttachment(id, buf, t)  // 写入附件
    getAttachment(id)           // 读取附件
  }
  dbStorage: {
    setItem(key, value)         // KV 存储（基于 PouchDB）
    getItem(key)
    removeItem(key)
  }
```

### 3.4 UI 交互

```javascript
  setSubInput(onChange, placeholder)  // 在主窗口搜索框显示子输入
  removeSubInput()                    // 移除子输入
  setSubInputValue(text)             // 设置子输入值
  subInputBlur()                     // 子输入失焦
```

### 3.5 剪贴板和系统

```javascript
  copyImage(base64)          // 复制图片
  copyText(text)             // 复制文本
  copyFile(path)             // 复制文件
  showNotification(body)     // 显示通知
  shellOpenExternal(url)     // 打开外部 URL
  shellOpenPath(path)        // 打开文件路径
  shellShowItemInFolder(p)   // 在文件夹中显示
  shellBeep()                // 系统提示音
  getPath(name)              // 获取系统路径
  getLocalId()               // 获取设备 ID
  getFileIcon(path)          // 获取文件图标 (dataURL)
  getFeatures()              // 获取 feature 列表
  setFeature(feature)        // 注册 feature
  removeFeature(code)        // 移除 feature
  screenCapture(cb)          // 截图（异步回调）
  simulateKeyboardTap(k, m)  // 模拟按键
  getCopyedFiles()           // 获取剪贴板文件
  isDarkColors()             // 是否暗色模式
  isMacOs() / isWindows() / isLinux()  // 平台检测
  getCursorScreenPoint()     // 鼠标位置
  getDisplayNearestPoint(p)  // 最近显示器
```

### 3.6 插件窗口创建（高级能力）

```javascript
  createBrowserWindow(url, options, callback) {
    // 使用 @electron/remote 直接在主进程中创建 BrowserWindow
    let win = new BrowserWindow({
      ...options,
      webPreferences: {
        contextIsolation: false,
        nodeIntegration: true,
        webviewTag: true,
        preload: preloadPath,
      }
    })
    win.loadURL(winIndex)
    return win
  }
```

---

## 4. `API extends DB` 继承分析

```typescript
// src/main/common/db.ts
const dbInstance = new LocalDb(app.getPath('userData'))
dbInstance.init()

export default class DBInstance {
  public currentPlugin: null | any = null  // 当前激活的插件
  private DBKEY = 'RUBICK_DB_DEFAULT'

  public async dbPut({ data }) { return dbInstance.put(this.DBKEY, data.data) }
  public dbGet({ data })       { return dbInstance.get(this.DBKEY, data.id) }
  // ...
}

// src/main/common/api.ts
class API extends DBInstance {
  init(mainWindow) { ... }  // 注册 IPC 监听
  // 可以直接使用 this.dbPut(), this.dbGet() 等
}
```

**为什么继承？** 不是必需的。组合（`class API { constructor() { this.db = new DB() } }`）会更好。但继承让代码更短——所有 DB 方法自动成为 IPC handler，不需要逐个代理。

继承的代价是 `API` 类同时承担了 IPC 路由和 DB 访问两种职责，违反了单一职责原则。但考虑到 API 只有 441 行，这个"过度继承"在实践中没有造成维护问题。

---

## 5. IPC 事件表

| 通道 | 方向 | 同步/异步 | 用途 |
|------|------|----------|------|
| `msg-trigger` | 渲染→主 | 同步 `sendSync` | 所有 API 调用 |
| `re-register` | 渲染→主 | 异步 `send` | 快捷键配置变更后重新注册 |
| `removePlugin` | 渲染→主 | 异步 `send` | 移除当前插件（不等待返回） |
| `global-short-key` | 主→渲染 | 异步 `send` | 用户按自定义快捷键 |
| `detach:service` | detach→主 | 同步 `sendSync` | 分离窗口的控制操作 |
| `guide:service` | guide→主 | 同步 `sendSync` | 引导窗口关闭 |
| `before-input-event` | 系统→渲染 | 事件 | ESC 键处理 |

---

## 6. 对比 ZTools IPC

| 维度 | Rubick | ZTools |
|------|--------|--------|
| 通道数 | 1 个主通道 | 20+ 按模块路由 |
| IPC 方式 | 同步 `sendSync` | 异步 `ipcMain.handle` |
| API 组织 | 类方法 + 继承 DB | 模块化拆分 |
| 返回值 | `event.returnValue` | `return` / Promise |
| 错误处理 | `if (returnValue instanceof Error)` | try/catch |
| 权限控制 | 无 | 字符串权限检查 |
| 流式 IPC | 无 | `webContents.send` + 事件 |

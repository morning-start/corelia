# uTools 调研报告

## 产品定位

**uTools** 是新一代**极简、插件化桌面效率工具**，核心理念是"**一切皆插件**"。它允许用户通过 Alt+Space 快捷键快速呼出全局搜索框，输入关键词即可调用各种插件功能。

### 核心定位

- **目标用户**: 办公人群、开发者、数字工作者
- **产品理念**: "即用即走"，按需选配插件，打造个性化工具集合
- **市场地位**: 国内桌面效率工具领域的头部产品，用户量超过 **500万+**

## 核心功能

### 内置基础功能

| 功能 | 说明 |
|------|------|
| 全局搜索 | Alt+Space 呼出，快速启动应用和插件 |
| 剪贴板管理 | 支持剪贴板历史记录 |
| 截图工具 | 快速截图和标注 |
| PDF 处理 | PDF 转换和编辑 |
| 图片批量处理 | 压缩、格式转换等 |
| 文件批量重命名 | 高效文件管理 |
| Markdown 笔记 | 便捷笔记功能 |
| OCR 文字识别 | 图片文字提取 |
| 本地搜索 | 文件快速定位 |

### AI 功能

| 功能 | 说明 |
|------|------|
| AI 图片处理 | 智能图片编辑 |
| AI 助手 | 内置 AI 对话能力 |
| AI Agent 接入 | 支持 MCP 协议，接入 Claude Code、WorkBuddy、QoderWork 等 AI Agent |

### 零代码 AI 制作插件

在 uTools 开发者工具中，用户可以通过**一句话描述需求**，AI 自动生成专业插件应用，让想法快速变成工具。

## 开发者生态

### 插件市场

- **插件数量**: 500+ 款插件
- **覆盖领域**: 办公、开发、设计、娱乐等
- **生态特点**: 社区活跃，插件质量较高

### 技术栈

| 类别 | 技术 |
|------|------|
| 底层运行时 | Electron |
| 前端技术 | HTML + CSS + JavaScript/TypeScript |
| UI 框架 | 支持 Vue、React 等主流前端框架 |
| Node.js | 支持接入本地原生能力 |
| 开发工具 | 官方 uTools 开发者工具 |

### 插件定义格式 (plugin.json)

```json
{
  "name": "插件名称",
  "version": "1.0.0",
  "logo": "logo.png",
  "main": "index.html",
  "preload": "preload.js",
  "features": [
    {
      "code": "feature_code",
      "label": "功能标签",
      "explain": "功能说明",
      "icon": "icon.png",
      "cmd": ["text_command", "/regex_pattern/"],
      "toggle": false,
      "hotkey": "Ctrl+Shift+K"
    }
  ],
  "$schema": "./node_modules/utools-api-types/resource/utools.schema.json"
}
```

### 插件 API (window.utools)

```typescript
// 基础信息 API
window.utools.getAppName(): string
window.utools.getVersion(): string
window.utools.getPath(name: string): string
window.utools.isWindows(): boolean
window.utools.isMacOS(): boolean
window.utools.isLinux(): boolean
window.utools.isDarkColors(): boolean

// 窗口管理 API
window.utools.createBrowserWindow(url: string, options?: object): BrowserWindow
window.utools.showNotification(body: string): void
window.utools.hideMainWindow(): void
window.utools.showMainWindow(): void

// 搜索框 API
window.utools.setSubInput(options: {
  placeholder?: string,
  onChange?: (text: string) => void
}): void

// 存储 API
window.utools.dbStorage: {
  setItem(key: string, value: any): void
  getItem(key: string): any
  removeItem(key: string): void
}

// 剪贴板 API
window.utools.clipboard.writeImage(base64: string): boolean
window.utools.clipboard.readText(): string
window.utools.clipboard.writeText(text: string): boolean

// 屏幕操作 API
window.utools.screenCapture(callback: (image: string) => void): void
window.utools.screenColorPick(callback: (color: { hex: string, rgb: object }) => void): void

// 浏览器自动化 API (ubrowser)
window.utools.ubrowser
  .goto(url: string)
  .value(selector: string, value: string)
  .click(selector: string)
  .run(options?: { width: number, height: number })

// Shell API
window.utools.shellOpenExternal(url: string): boolean
window.utools.shellShowItemInFolder(fullPath: string): boolean
```

### 开发工具

| 工具 | 说明 |
|------|------|
| uTools 开发者工具 | 官方提供的调试、热重载、日志查看、API 测试工具 |
| utools-api-types | NPM 类型包，提供完整 TypeScript 类型提示 |
| $schema 支持 | plugin.json 配置代码提示 |

### 插件加载机制

1. 用户通过 Alt+Space 呼出主搜索框
2. 输入关键字匹配 `features` 中的 `cmd` 字段（支持文本和正则）
3. 匹配成功则加载对应插件
4. 渲染 `main` 指定的 HTML 页面
5. 执行 `preload.js` 注入 `window.utools` API

### 触发方式

| 方式 | 说明 |
|------|------|
| 文本匹配 | `cmd: "text_command"` |
| 正则匹配 | `cmd: ["/pattern/flags"]` |
| 热键触发 | `hotkey: "Ctrl+Shift+K"` |
| 全局钩子 | 预置的系统钩子 |
| toggle | 开关型功能支持 |

### 插件与主程序通信

```javascript
// preload.js
const { ipcRenderer } = require("electron")

window.onPluginEnter = (param) => {
  // param.payload: 搜索框内容
  // param.type: 'text' | 'regex'
}

window.onPluginOut = () => {}

window.ipc = ipcRenderer
```

## 产品特点总结

| 特点 | 说明 |
|------|------|
| 插件化架构 | "一切皆插件"，按需选配 |
| 轻量高效 | 软件体积小，启动快速 |
| AI 集成 | 内置 AI 助手，支持 MCP 协议 |
| 跨平台 | 支持 Windows、macOS、Linux |
| 生态丰富 | 500+ 插件，覆盖多种场景 |
| 开发友好 | 提供开发者工具和类型定义 |

## 竞争优势

1. **用户基数大**: 500万+ 用户，生态成熟
2. **插件丰富**: 500+ 插件，覆盖办公、开发、设计等场景
3. **AI 布局早**: 较早集成 AI 功能，支持 MCP 协议
4. **开发体验好**: 官方开发者工具 + TypeScript 类型提示
5. **跨平台支持**: 三大主流操作系统全覆盖

## 参考资料

- 官网: https://www.u-tools.cn/
- 官方文档: https://www.u-tools.cn/docs/developer/basic/getting-started.html
- NPM 类型包: https://www.npmjs.com/package/utools-api-types

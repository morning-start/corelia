# 插件系统对比分析报告

## 概述

本报告对比分析 **uTools** 和 **ZTools** 两个桌面效率工具的产品定位、核心功能、技术架构和插件系统设计，为 Corelia 的插件系统开发提供参考。

## 产品定位对比

| 维度 | uTools | ZTools |
|------|--------|--------|
| **产品定位** | 新一代极简、插件化桌面效率工具 | 免费、安全、无广告的桌面工具箱 |
| **核心理念** | "一切皆插件"，打造个性化工具集 | "让您的工作专注高效" |
| **目标用户** | 办公人群、开发者、数字工作者 | 追求免费的用户、设计师、产品经理 |
| **市场地位** | 国内桌面效率工具头部产品 | uTools 的竞争产品，差异化定位 |
| **用户规模** | 500万+ 用户 | 1.0万+ 活跃用户 |
| **插件数量** | 500+ 款插件 | 20+ 插件 |

## 核心功能对比

### 内置功能对比

| 功能 | uTools | ZTools |
|------|--------|--------|
| 快捷键唤起 | ✅ Alt+Space | ✅ 自定义快捷键 |
| 全局搜索 | ✅ | ✅ |
| 剪贴板管理 | ✅ 基础 | ✅ 完整历史管理 |
| 截图工具 | ✅ | ✅ |
| PDF 处理 | ✅ | ❌ |
| 图片批量处理 | ✅ | ✅ |
| 文件批量重命名 | ✅ | ❌ |
| Markdown 笔记 | ✅ | ❌ |
| OCR 文字识别 | ✅ | ❌ |
| 本地搜索 | ✅ | ❌ |
| 浏览器书签搜索 | ❌ | ✅ |
| 浏览器历史记录搜索 | ❌ | ✅ |
| 录屏功能 | ❌ | ✅ |

### AI 功能对比

| 功能 | uTools | ZTools |
|------|--------|--------|
| AI 助手 | ✅ 内置 | ✅ 内置 AI API |
| AI 图片处理 | ✅ | ❌ |
| MCP 协议支持 | ✅ | ❌ |
| 零代码 AI 生成插件 | ✅ | ❌ |

## 技术架构对比

| 维度 | uTools | ZTools |
|------|--------|--------|
| **底层框架** | Electron | Delphi + CEF |
| **前端技术** | Node.js + Web 前端 + Electron | HTML + CSS + JS（需编译） |
| **UI 框架** | 支持 Vue、React 等 | 支持（需编译） |
| **插件语言** | JavaScript/TypeScript | JavaScript |
| **跨平台** | Windows、macOS、Linux | Windows、macOS、Linux |
| **Node.js 生态** | ✅ 支持 | ❌ 不支持 |
| **DLL 插件** | ❌ 不支持 | ✅ 支持 |
| **构建工具** | Webpack/Vite | Vite/Webpack |

### 架构差异分析

**uTools 架构特点:**
- 采用 Electron + Node.js 生态
- 插件可直接调用 Node.js 模块
- 适合熟悉 JavaScript/Node.js 的开发者
- 生态成熟，插件开发简单

**ZTools 架构特点:**
- 采用 Delphi + CEF，不依赖 Node.js
- 自研模块接口，提供更丰富的原生能力
- 支持 DLL 插件，对 Windows 开发者友好
- 前端插件可调用 DLL 插件，可玩性高

## 插件定义格式对比

### uTools plugin.json

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
      "cmd": ["text_command", "/regex/"],
      "toggle": false,
      "hotkey": "Ctrl+Shift+K"
    }
  ],
  "$schema": "./node_modules/utools-api-types/resource/utools.schema.json"
}
```

### ZTools plugin.json

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
      "cmd": ["text_command", "/regex/"]
    }
  ]
}
```

### 关键差异

| 字段 | uTools | ZTools | 分析 |
|------|--------|--------|------|
| `toggle` | ✅ 支持 | ❌ 不支持 | uTools 支持开关型功能 |
| `hotkey` | ✅ plugin.json 内置 | ❌ 系统配置 | uTools 在 plugin.json 中直接定义 |
| `$schema` | ✅ 支持 | ❌ 不支持 | uTools 提供更好的开发体验 |

## API 对比

### 基础信息 API

| API | uTools | ZTools |
|-----|--------|--------|
| 获取应用名称 | `getAppName()` | `getAppName()` |
| 获取版本 | `getVersion()` | `getAppVersion()` |
| 系统检测 | `isWindows/MacOS/Linux` | `isWindows/isMacOs/isLinux` |
| 深色模式 | `isDarkColors()` | `isDarkColors()` |
| 开发模式 | ❌ | `isDev()` |
| 设备 ID | ❌ | `getNativeId()` |
| 窗口类型 | ❌ | `getWindowType()` |
| WebContents ID | ❌ | `getWebContentsId()` |

### ZTools 独有 API

| API | 说明 |
|-----|------|
| `getPathForFile(file)` | 获取拖放文件的真实路径 |
| `getWindowType()` | 获取当前窗口类型 |
| `getWebContentsId()` | 获取 WebContents ID |
| `setExpendHeight(height)` | 设置插件视图高度 |
| `onPluginDetach()` | 监听插件被分离为独立窗口 |
| `outPlugin(isKill)` | 退出插件 |
| `sendToParent()` | 发送消息到父窗口 |

### 窗口管理 API

| API | uTools | ZTools |
|-----|--------|--------|
| 创建窗口 | `createBrowserWindow()` | `createBrowserWindow()` |
| 显示通知 | `showNotification()` | `showNotification()` |
| 显示主窗口 | `showMainWindow()` | `showMainWindow()` |
| 隐藏主窗口 | `hideMainWindow()` | `hideMainWindow()` |
| 退出插件 | ❌ | `outPlugin(isKill)` |

### 数据库/存储 API

| API | uTools | ZTools |
|-----|--------|--------|
| 简单存储 | `dbStorage` | `dbStorage` |
| 数据库 | ❌ | `db` (PouchDB) + Promise 版本 |
| 附件存储 | ❌ | ✅ 支持 |

**ZTools 数据库优势:**
- 提供完整的 PouchDB 支持
- 支持附件存储
- 提供 Promise 版本 API
- 按插件名称隔离数据

### 剪贴板 API

| API | uTools | ZTools |
|-----|--------|--------|
| 读取文本 | `clipboard.readText()` | `clipboard.writeContent()` |
| 写入文本 | `clipboard.writeText()` | `copyText()` |
| 复制图片 | `clipboard.writeImage()` | `copyImage()` |
| 复制文件 | ❌ | `copyFile()` |
| 历史记录 | ❌ | `clipboard.getHistory()` |
| 搜索历史 | ❌ | `clipboard.search()` |
| 监听变化 | ❌ | `clipboard.onChange()` |

**ZTools 剪贴板优势:**
- 完整的剪贴板历史管理
- 支持搜索历史记录
- 支持监听剪贴板变化

### AI API

| API | uTools | ZTools |
|-----|--------|--------|
| AI 调用 | ❌ | `ai()` |
| 模型列表 | ❌ | `allAiModels()` |
| 流式调用 | ❌ | ✅ 支持 |
| 请求中断 | ❌ | ✅ 支持 |

**ZTools 独有:**
- 内置 AI 模型支持
- 流式/非流式调用
- 支持中断请求

### 主搜索推送

| API | uTools | ZTools |
|-----|--------|--------|
| 注册推送 | ❌ | `onMainPush()` |

**ZTools 独有:**
- 插件可以在主搜索框提供实时结果
- 无需进入插件即可看到结果
- 支持选择后进入

### 输入模拟 API

| API | uTools | ZTools |
|-----|--------|--------|
| 发送输入事件 | ❌ | `sendInputEvent()` |
| 模拟按键 | ❌ | `simulateKeyboardTap()` |

**ZTools 独有:**
- 完整的输入模拟支持
- 支持鼠标、键盘、滚轮事件

## 插件加载机制对比

### 共同点

1. 通过 `plugin.json` 定义插件元信息
2. 使用 `features` 数组配置触发命令
3. 支持文本和正则两种匹配模式
4. 通过 `preload.js` 注入 API
5. 使用 Electron/CEF 作为底层运行时

### 差异

| 机制 | uTools | ZTools |
|------|--------|--------|
| 热键定义 | 在 plugin.json 中配置 | 通过系统设置配置 |
| 触发类型 | text, regex, toggle | text, regex, over |
| 主搜索推送 | ❌ 不支持 | ✅ 支持 onMainPush |
| 开发工具 | 官方开发者工具 | 需要手动调试 |

## 开发体验对比

| 维度 | uTools | ZTools |
|------|--------|--------|
| 开发工具 | 官方 uTools 开发者工具 | 需要手动调试 |
| 类型提示 | `utools-api-types` NPM 包 | ❌ 无官方类型包 |
| $schema 支持 | ✅ 完整支持 | ❌ 不支持 |
| 文档完整性 | 完整官方文档 | 文档较少 |
| 社区活跃度 | 活跃 | 较小众 |

## 可借鉴点总结

### 共同的设计模式

1. **plugin.json 格式**: 简洁的清单格式明确定义了插件入口、图标、功能列表
2. **window 对象暴露 API**: 简单直接，插件代码易写易读
3. **features 触发机制**: 文本+正则+热键的组合覆盖大多数场景
4. **preload 注入**: 良好的安全隔离，API 权限控制

### ZTools 值得借鉴的功能

| 功能 | 说明 | 优先级 |
|------|------|--------|
| **PouchDB 数据库** | 适合需要复杂数据存储的插件 | 高 |
| **剪贴板历史** | 用户感知价值高 | 高 |
| **主搜索推送 (onMainPush)** | 创新的交互方式 | 高 |
| **AI API 支持** | 前瞻性设计 | 中 |
| **输入模拟** | 自动化场景必备 | 中 |
| **DLL 插件支持** | 原生扩展能力（长期规划） | 低 |

### uTools 值得借鉴的设计

| 功能 | 说明 | 优先级 |
|------|------|--------|
| **$schema 支持** | 提升开发体验 | 高 |
| **toggle 功能** | 支持开关型插件 | 中 |
| **官方开发者工具** | 降低开发门槛 | 高 |
| **类型定义包** | utools-api-types 提供完整类型提示 | 高 |
| **MCP 协议支持** | AI Agent 接入能力 | 高 |

## Corelia 设计建议

### 短期规划（MVP）

1. **采用 plugin.json 结构**: 参考 ZTools 的格式（更完整）
2. **基础 API 设计**: 参考 uTools 的 window 对象暴露方式
3. **features 触发机制**: 支持文本+正则匹配
4. **preload 注入**: 实现安全隔离的 API 注入
5. **dbStorage 简单存储**: 参考 uTools 的实现

### 中期规划

1. **完善搜索框 API**: 参考 ZTools 的 setSubInput 系列
2. **剪贴板历史管理**: 参考 ZTools 的 clipboard API
3. **$schema 和类型提示**: 参考 uTools 的 utools-api-types
4. **官方开发者工具**: 提升开发体验

### 长期规划

1. **PouchDB 数据库**: 适合需要复杂数据存储的场景
2. **主搜索推送 (onMainPush)**: 创新的交互方式
3. **AI API 支持**: 前瞻性设计
4. **原生扩展能力**: 类似 ZTools 的 DLL 插件

### 差异化建议

| 方向 | 建议 |
|------|------|
| **技术选型** | 采用 Tauri + Rust，替代 Electron，获得更好的性能 |
| **插件语言** | 优先支持 JavaScript/TypeScript，未来可扩展 Rust 插件 |
| **AI 集成** | 内置 MCP 协议支持，与 AI Agent 生态对接 |
| **数据存储** | 提供轻量级数据库（如 SQLite）而非 PouchDB |
| **开发者体验** | 提供官方 CLI 工具和 VSCode 插件 |

## 市场定位参考

| 产品 | 市场定位 | 适合 Corelia 参考 |
|------|----------|-------------------|
| uTools | 成熟的生态系统，500万+用户 | 生态建设和开发者体验 |
| ZTools | 免费无广告，DLL 扩展 | 差异化定位和技术架构 |
| Quicker | 鼠标操作为核心，动作库 | 创新的交互方式 |

## 参考资料

| 资料 | 链接 |
|------|------|
| uTools 官网 | https://www.u-tools.cn/ |
| uTools 文档 | https://www.u-tools.cn/docs/developer/basic/getting-started.html |
| uTools API 类型 | https://www.npmjs.com/package/utools-api-types |
| ZTools 官网 | https://ztools.cn/ |
| ZTools 文档 | https://ztoolscenter.github.io/ZTools-doc/ |

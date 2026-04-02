# ZTools 调研报告

## 产品定位

**ZTools** 是一款**免费、安全、无广告的桌面工具箱**，核心理念是"让工作专注高效"。它通过快捷键唤起和插件化扩展，为用户提供随时随地的效率提升。

### 核心定位

- **目标用户**: 办公人群、设计师、产品经理等需要高效工具的用户
- **产品理念**: "让您的工作专注高效"，简洁美观，安全无广告
- **市场地位**: 作为 uTools 的竞争产品，主打**完全免费**和**安全无广告**的差异化路线

### 核心数据

| 指标 | 数据 |
|------|------|
| 活跃用户 | 1.0万+ |
| 插件数量 | 20+ |
| 运行稳定性 | 99.9% |
| 用户评分 | 4.9/5 |

## 核心功能

### 内置基础功能

| 功能 | 说明 |
|------|------|
| 快捷键唤起 | 自定义快捷键，随时快速启动 |
| 插件管理 | 内置插件市场，一站式管理 |
| 快速截图 | 便捷截图功能 |
| 工具集合 | 整合多种常用工具 |

### 特色功能

| 功能 | 说明 |
|------|------|
| 剪贴板增强 | 增强版剪贴板管理 |
| 浏览器书签搜索 | 快速搜索浏览器书签 |
| 浏览器历史记录 | 快速搜索浏览历史 |
| 图片处理 | 内置多种图片处理工具 |
| 录屏功能 | 屏幕录制 |

## 技术架构

### 技术栈

| 类别 | 技术 |
|------|------|
| 框架 | Delphi |
| 嵌入式浏览器 | CEF (Chromium Embedded Framework) |
| 前端技术 | HTML + CSS + JavaScript（编译后） |
| 构建工具 | 支持 Vite、Webpack 等前端构建工具 |
| 运行时 | Electron |
| 文档工具 | VitePress |

### 架构特点

1. **放弃 Node.js 生态**: 与 uTools 不同，ZTools 不依赖 Node.js 生态
2. **自研模块接口**: 由 ZTools 本身提供更多更丰富的模块接口
3. **DLL 插件支持**: 提供 `dllplugins` 模块，开发者可以开发 DLL 插件
4. **混合调用**: 前端插件可调用 DLL 插件，可玩性更高

```
前端插件 (JS) ←→ ZTools 主程序 ←→ DLL 插件
```

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
      "hotkey": "Ctrl+Shift+K"
    }
  ]
}
```

### 插件 API (window.ztools)

#### 基础信息 API

```typescript
ztools.getAppName(): string                    // 获取应用名称
ztools.getAppVersion(): string                 // 获取应用版本号
ztools.getNativeId(): string                   // 获取设备唯一标识符
ztools.getWindowType(): string                 // 获取当前窗口类型
ztools.getWebContentsId(): number              // 获取 WebContents ID
ztools.isWindows(): boolean                   // 是否 Windows
ztools.isMacOs() / ztools.isMacOS(): boolean  // 是否 macOS
ztools.isLinux(): boolean                     // 是否 Linux
ztools.isDarkColors(): boolean                // 是否深色模式
ztools.isDev(): boolean                       // 是否开发模式
```

#### 窗口与显示 API

```typescript
// 设置插件视图高度
ztools.setExpendHeight(height: number): void

// 显示系统通知
ztools.showNotification(body: string): void

// 显示/隐藏主窗口
ztools.showMainWindow(): Promise<boolean>
ztools.hideMainWindow(isRestorePreWindow?: boolean): Promise<boolean>

// 退出插件
ztools.outPlugin(isKill?: boolean): Promise<boolean>

// 创建独立窗口
ztools.createBrowserWindow(url: string, options?: object, callback?: () => void): Proxy<BrowserWindow> | null

// 发送消息到父窗口
ztools.sendToParent(channel: string, ...args: any[]): void
```

#### 输入模拟 API

```typescript
// 发送模拟输入事件
ztools.sendInputEvent(event: MouseInputEvent | MouseWheelInputEvent | KeyboardInputEvent): void

// 模拟键盘按键
ztools.simulateKeyboardTap(key: string, ...modifiers: string[]): boolean
```

#### 事件 API

```typescript
// 监听插件进入事件
ztools.onPluginEnter(callback: (param: LaunchParam) => void): void

// 监听插件退出事件
ztools.onPluginOut(callback: (isKill: boolean) => void): void

// 监听插件被分离为独立窗口
ztools.onPluginDetach(callback: () => void): void

// 注册主搜索推送功能
ztools.onMainPush(
  callback: (queryData: any) => object[],
  selectCallback?: (selectData: any) => boolean
): void
```

#### 搜索框 API

```typescript
// 设置主窗口搜索框行为
ztools.setSubInput(
  onChange: (text: string) => void,
  placeholder: string,
  isFocus?: boolean
): void

// 设置子输入框值
ztools.setSubInputValue(text: string): void

// 子输入框操作
ztools.subInputFocus(): boolean        // 聚焦
ztools.subInputBlur(): boolean         // 失焦
ztools.subInputSelect(): boolean       // 获得焦点并选中

// 移除子输入框
ztools.removeSubInput(): Promise<boolean>
```

#### 数据库 API (PouchDB)

```typescript
// 保存数据
ztools.db.put(doc: object): object

// 获取数据
ztools.db.get(id: string): object | null

// 删除数据
ztools.db.remove(docOrId: object | string): object

// 批量操作
ztools.db.bulkDocs(docs: object[]): object[]

// 按 key 前缀查询
ztools.db.allDocs(key?: string): object[]

// 附件操作
ztools.db.postAttachment(id: string, attachment: string | Buffer, type: string): object
ztools.db.getAttachment(id: string): Buffer
ztools.db.getAttachmentType(id: string): string

// Promise 版本
ztools.db.promises.put(doc)
ztools.db.promises.get(id)
ztools.db.promises.remove(docOrId)
```

#### 剪贴板 API

```typescript
// 获取剪贴板历史
ztools.clipboard.getHistory(page: number, pageSize: number, filter?: string): Promise<object>

// 搜索剪贴板历史
ztools.clipboard.search(keyword: string): Promise<object[]>

// 删除剪贴板记录
ztools.clipboard.delete(id: string): Promise<boolean>

// 清空剪贴板历史
ztools.clipboard.clear(type?: string): Promise<boolean>

// 获取剪贴板状态
ztools.clipboard.getStatus(): Promise<object>

// 写入剪贴板
ztools.clipboard.write(id: string, shouldPaste?: boolean): Promise<boolean>
ztools.clipboard.writeContent(data: { type: 'text' | 'image', content: string }, shouldPaste?: boolean): Promise<boolean>

// 监听剪贴板变化
ztools.clipboard.onChange(callback: (item: object) => void): void

// 复制操作
ztools.copyText(text: string): boolean
ztools.copyImage(image: string): boolean      // base64 Data URL 或文件路径
ztools.copyFile(filePath: string): boolean
```

#### AI API

```typescript
// 调用 AI 模型
ztools.ai(option: object, streamCallback?: (chunk: any) => void): PromiseLike & { abort: () => void }

// 获取所有 AI 模型
ztools.allAiModels(): Promise<object[]>

// 使用示例
const result = await ztools.ai({ prompt: '你好' })
const request = ztools.ai({ prompt: '你好' }, (chunk) => {
  console.log('收到数据:', chunk)
})
request.abort()
```

## 插件加载机制

### 启动流程

1. 用户通过快捷键呼出主搜索框
2. 输入内容匹配 `features` 中的 `cmd` 字段（支持文本和正则）
3. 匹配成功则启动插件进程
4. 渲染 `main` 指定的 HTML 页面
5. 执行 `preload.js` 注入 `window.ztools` API
6. 触发 `onPluginEnter` 事件

### 触发方式

| 方式 | 说明 |
|------|------|
| 文本匹配 | `cmd: "text_command"` |
| 正则匹配 | `cmd: "/pattern/flags"` |
| 任意匹配 | type 为 `'over'` 时匹配任意输入 |
| 热键触发 | 通过系统热键配置 |

### 主搜索推送 (onMainPush)

插件可以注册主搜索推送功能，在用户输入时提供实时搜索结果，无需进入插件即可看到结果。

```javascript
ztools.onMainPush(
  (queryData) => {
    return [
      { title: '结果1', description: '描述' },
      { title: '结果2', description: '描述' }
    ]
  },
  (selectData) => {
    return true
  }
)
```

## 插件与主程序通信

### 方式一：预加载脚本 (preload.js)

```javascript
// preload.js
window.onPluginEnter = (param) => {
  console.log('插件进入', param.payload)
}

window.onPluginOut = (isKill) => {
  console.log('插件退出', isKill)
}
```

### 方式二：window.ztools 对象

```javascript
// 渲染进程直接访问
ztools.showNotification('Hello')
ztools.dbStorage.setItem('key', 'value')
```

### 方式三：IPC 通信

```javascript
// 发送到父窗口
ztools.sendToParent('channel-name', data)
```

## 产品特点总结

| 特点 | 说明 |
|------|------|
| 完全免费 | 无广告，永久免费 |
| 安全无广告 | 纯净无广告，数据本地存储 |
| 跨平台 | 支持 Windows、macOS、Linux |
| DLL 插件支持 | 独特的 DLL 扩展能力 |
| AI 内置 | 内置 AI 模型调用支持 |
| PouchDB | 完整的数据库支持 |
| 剪贴板历史 | 完整的剪贴板历史管理 |
| 主搜索推送 | 创新的 onMainPush 机制 |

## 竞争优势

1. **完全免费无广告**: 主打免费策略，与 uTools 形成差异化
2. **DLL 插件支持**: 独特的原生扩展能力，对 Windows 开发者友好
3. **内置 AI**: 较早集成 AI 模型调用
4. **PouchDB 支持**: 适合需要复杂数据存储的场景
5. **输入模拟**: 完整的输入模拟 API，支持自动化场景

## 与 uTools 的核心差异

| 维度 | uTools | ZTools |
|------|--------|--------|
| 定价 | 免费（部分高级功能收费） | 完全免费 |
| 广告 | 无 | 无 |
| 插件数量 | 500+ | 20+ |
| 用户数量 | 500万+ | 1万+ |
| 技术栈 | Electron + Node.js | Delphi + CEF |
| DLL 支持 | 不支持 | 支持 |
| 数据库 | dbStorage 简单存储 | PouchDB 完整数据库 |
| 剪贴板 | 基础功能 | 完整历史管理 |

## 产品定位差异总结

| 维度 | uTools | ZTools |
|------|--------|--------|
| 市场定位 | 成熟的生态系统 | 新兴的竞争者 |
| 目标用户 | 开发者、办公人群 | 追求免费的用户 |
| 扩展方式 | JS 插件 | JS 插件 + DLL 插件 |
| AI 集成 | AI 助手 + MCP | 内置 AI API |

## 参考资料

- 官网: https://ztools.cn/
- 官方文档: https://ztoolscenter.github.io/ZTools-doc/
- GitHub: https://github.com/ZToolsCenter/

# Corelia 插件开发指南

> 版本: 1.0.0 | 更新: 2026-04-14 | 状态: Stable

---

## 目录

1. [架构概述](#1-架构概述)
2. [快速开始](#2-快速开始)
3. [插件规范](#3-插件规范)
4. [API 参考](#4-api-参考)
5. [生命周期](#5-生命周期)
6. [WASM 扩展](#6-wasm-扩展)
7. [最佳实践](#7-最佳实践)
8. [调试技巧](#8-调试技巧)

---

## 1. 架构概述

### 系统架构

```
用户输入
   │
   ▼
┌──────────────┐     ┌────────────────┐     ┌─────────────┐
│  前端搜索层   │────▶│  插件服务层      │────▶│  QuickJS VM │
│  SearchStore │     │  PluginService  │     │  (rquickjs)  │
└──────────────┘     └────────────────┘     └─────────────┘
       │                      │                       │
       │              ┌───────┴───────┐               │
       │              │               │               │
       ▼              ▼               ▼               ▼
  ┌─────────┐  ┌──────────┐  ┌────────────┐  ┌──────────┐
  │系统搜索  │  │ API Bridge│  │ WasmBridge │  │ Store    │
  └─────────┘  └──────────┘  └────────────┘  └──────────┘
```

### 核心组件

| 组件 | 语言 | 职责 |
|------|------|------|
| `PluginLoader` (Rust) | Rust | 扫描/加载/卸载插件，管理 VM |
| `QuickJSRuntime` (Rust) | Rust | VM 池化管理，代码执行 |
| `ApiBridge` (Rust) | Rust | 注入 `window.utools` API |
| `WasmBridge` (Rust) | Rust | WASM 函数注册与调用转发 |
| `PluginService` (TS) | TS | 前端生命周期管理，VM 缓存 |
| `PatchLoader` (TS) | TS | WebView 侧 WASM 加载 |

---

## 2. 快速开始

### 2.1 创建最小插件

```bash
# 在 plugins/ 目录下创建插件文件夹
mkdir -p plugins/my-plugin
```

**`plugins/my-plugin/plugin.json`**
```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "type": "quickjs",
  "prefix": "mp",
  "description": "我的第一个插件",
  "author": "Your Name",
  "main": "index.js",
  "patches": []
}
```

**`plugins/my-plugin/index.js`**
```javascript
// 初始化（可选）
function pluginInit() {
  console.log('[my-plugin] 已加载');
}

// 搜索回调：返回结果数组
function onSearch(query) {
  if (!query || query === '') return [
    { title: 'Hello', description: '点击执行', icon: '👋', action: 'hello' }
  ];
  return [];
}

// 动作回调：处理用户选择
function onAction(action) {
  switch(action) {
    case 'hello':
      return { type: 'text', message: '👋 Hello from my plugin!' };
    default:
      return { type: 'error', message: 'Unknown action' };
  }
}

// 导出（CommonJS 兼容格式）
if (typeof module !== 'undefined') {
  module.exports = { pluginInit, onSearch, onAction };
}
```

### 2.2 验证插件

启动 Corelia 后，在搜索框输入 `mp` 或 `my-plugin` 即可看到插件。

---

## 3. 插件规范

### 3.1 目录结构

```
plugins/
└── my-plugin/
    ├── plugin.json          # 必需：元数据配置
    ├── index.js             # 必需：入口文件
    ├── patches/             # 可选：WASM 依赖
    │   └── crypto/
    │       ├── pkg/
    │       │   ├── crypto_bg.wasm
    │       │   ├── crypto.js
    │       │   └── package.json
    │       └── src/
    └── README.md            # 可选：文档
```

### 3.2 `plugin.json` 字段说明

| 字段 | 必需 | 类型 | 说明 |
|------|------|------|------|
| `name` | ✅ | string | 插件唯一标识符（用于 load/unload） |
| `version` | ✅ | string | 语义化版本号 (`x.y.z`) |
| `type` | ✅ | string | 固定值 `"quickjs"` |
| `prefix` | 推荐 | string | 搜索前缀触发词 |
| `main` | 可选 | string | 入口文件名，默认 `"index.js"` |
| `logo` | 可选 | string | Logo 路径（支持 data URI） |
| `description` | 可选 | string | 插件描述文本 |
| `author` | 可选 | string | 作者信息 |
| `patches` | 可选 | string[] | WASM patch 名称列表 |
| `features` | 可选 | FeatureConfig[] | 功能入口定义 |

### 3.3 功能特性 (Features)

在 `plugin.json` 中声明功能入口：

```json
{
  "name": "file-search",
  "features": [{
    "code": "search",
    "label": "文件搜索",
    "type": "list",
    "items": [
      { "label": "📄 搜索文档", "action": "search_docs", "icon": "📄" },
      { "label": "🖼️ 搜索图片", "action": "search_images", "icon": "🖼️" }
    ]
  }]
}
```

---

## 4. API 参考

### 4.1 全局对象 `utools`

插件运行时自动注入 `window.utools` 对象，提供以下模块：

#### 存储 — `utools.dbStorage`

```javascript
// 写入（值会序列化为 JSON 字符串）
utools.dbStorage.setItem('key', { count: 42 });

// 读取（返回 JSON 解析后的值或 null）
var value = utools.dbStorage.getItem('key');

// 删除
utools.dbStorage.removeItem('key');

// 获取所有键值对
var all = utools.dbStorage.getAll(); // Record<string, string>
```

#### 剪贴板 — `utools.clipboard`

```javascript
var text = utools.clipboard.readText();           // 读取文本
utools.clipboard.writeText('Hello World');         // 写入文本
utools.clipboard.copyText('Copied!');              // 复制（同 writeText）

// 图片操作（部分平台支持）
utools.clipboard.copyImage('data:image/png;base64,...');
var img = utools.clipboard.getClipboardImage();
```

#### Shell — `utools.shell`

```javascript
utools.shell.openPath('/path/to/file');            // 打开文件/目录
utools.shell.openExternal('https://example.com');  // 打开 URL
utools.shell.showItemInFolder('/path/to/file');    // 在资源管理器中显示
utools.shell.beep();                               // 系统提示音
```

#### 窗口控制

```javascript
utools.hideMainWindow();  // 隐藏主窗口
utools.showMainWindow();  // 显示主窗口
utools.setExpendHeight(400); // 设置窗口高度（像素）
utools.outPlugin();        // 隐藏窗口并退出插件模式
```

#### 路径 — `utools.getPath(name)`

支持名称：`home`, `desktop`, `document`, `download`, `music`, `picture`, `video`, `temp`, `appdata`, `config`, `log`, `cwd`, `exe`, `plugin` 等。

```javascript
var docsPath = utools.getPath('documents');
var desktopPath = utools.getPath('desktop');
```

#### 文件系统 — `utools.fs`

```javascript
utools.fs.readTextFile('/path/to/file.txt');
utools.fs.writeTextFile('/path/to/file.txt', 'content');
utools.fs.exists('/path/to/file');     // boolean
utools.fs.isDir('/path/to/dir');      // boolean
```

#### HTTP 请求 — `utools.fetch(url, options?)`

```javascript
// GET 请求
var resp = await utools.fetch('https://api.example.com/data');
console.log(resp.status);       // 200
console.log(resp.statusText);   // OK
console.log(resp.ok);           // true
console.log(resp.headers);      // { content-type: ... }
var text = await resp.text();
var json = await resp.json();

// POST 请求
var resp2 = await utools.fetch('https://api.example.com/submit', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ name: 'test' }),
  timeout: 10  // 秒
});
```

#### 子进程 — `utools.process`

```javascript
// 执行命令
var result = await utools.process.exec('dir /B');
console.log(result.stdout);
console.log(result.stderr);
console.log(result.exitCode);

// 获取系统信息
utools.process.getNativeId();    // 设备 ID
utools.process.getAppName();     // "Corelia"
utools.process.getAppVersion();  // 当前版本
```

#### 对话框 — `utools.dialog`

```javascript
// 选择文件
var filePath = await utools.dialog.showOpenDialog({
  title: '选择文件',
  filters: [{ name: 'Images', extensions: ['jpg', 'png'] }],
  properties: ['openFile']
});

// 保存文件
var savePath = await utools.dialog.showSaveDialog({
  title: '另存为',
  defaultPath: 'output.txt'
});

// 消息框
var btn = await utools.dialog.showMessageBox({
  type: 'question',
  title: '确认',
  message: '确定要删除吗？',
  buttons: ['确定', '取消']
});
```

#### 通知 — `utools.showNotification(title, body)`

```javascript
utools.showNotification('任务完成', '你的文件已处理完毕');
```

#### 上下文 — `utools.getContext()` / `utools.setContext(payload)`

```javascript
var ctx = utools.getContext();
// ctx.code, ctx.type, ctx.payload, ctx.refresh

utools.setContext({ someData: 'value' });
```

#### 生命周期回调

```javascript
utools.onPluginReady(function() {
  console.log('插件就绪');
});

utools.onPluginOut(function() {
  console.log('即将退出');
});
```

#### WASM 扩展 — `utools.wasm`

```javascript
// 检查函数可用性
if (utools.wasm.has('crypto.sha256')) {
  var requestId = utools.wasm.__wasm_call('crypto.sha256', '"hello"');

  // 轮询获取结果
  var result;
  while (!(result = utools.wasm.__wasm_get_result(requestId))) {
    // 短暂等待后重试
  }

  console.log(JSON.parse(result).result);
}

// 列出所有可用函数
var funcs = utools.wasm.__wasm_available();

// 检查单个函数
utools.wasm.__wasm_has('crypto.sha256'); // true/false
```

### 4.2 搜索结果规范

`onSearch(query)` 应返回数组，每项结构如下：

```typescript
interface PluginSearchResult {
  /** 唯一标识 */
  pluginId?: string;
  /** 显示标题 */
  title: string;
  /** 副标题/描述 */
  description: string;
  /** 图标（emoji 或路径） */
  icon?: string;
  /** 动作标识（传给 onAction） */
  action: string;
  /** 额外数据（透传给 onAction） */
  data?: unknown;
}
```

### 4.3 动作结果规范

`onAction(action)` 应返回：

```typescript
interface PluginActionResult {
  /** 结果类型 */
  type: 'text' | 'error' | 'html' | 'copy';
  /** 主要内容 */
  message?: string;
  /** 附加数据 */
  data?: unknown;
  /** 其他自定义字段 */
  [key: string]: unknown;
}
```

---

## 5. 生命周期

### 5.1 状态机

```
                    scan_plugins()
                        │
                        ▼
                  ┌──────────┐
                  │ MetaLoaded│ ◄──── Unloaded
                  └────┬─────┘
                       │ load_plugin()
                       ▼
                   ┌────────┐
                   │ Loading│
                   └───┬────┘
                       │
           ┌───────────┼───────────┐
           ▼           ▼           ▼
        ┌──────┐  ┌────────┐  ┌──────┐
        │ Ready │  │ Cached │  │ Error│
        └───┬──┘  └───┬────┘  └──┬───┘
            │          │          │
            ▼          ▼          ▼
         unload    idle timeout  retry load
```

### 5.2 加载流程

1. **扫描**: `scan_plugins()` → 发现 `plugin.json`
2. **加载**: `load_plugin(id)` → 创建 VM → 注入 API → 执行 `index.js`
3. **搜索**: 用户输入 → `find_by_prefix()` → 匹配插件 → `executeSearch(id, query)`
4. **动作**: 用户选择 → `executeAction(id, action)` → `onAction()`
5. **卸载**: `unload_plugin(id)` → 销毁 VM → 清理状态

---

## 6. WASM 扩展

插件可以通过 WASM 补丁扩展计算能力。适用于需要高性能计算的场景（加密、图像处理等）。

### 6.1 添加 WASM 依赖

1. 使用 `wasm-pack` 构建 Rust 项目为 WASM：
```bash
cd patches/my-crypto
wasm-pack build --target bundler --out-dir pkg
```

2. 将产物放入插件的 `patches/my-crypto/pkg/` 目录

3. 在 `plugin.json` 中声明：
```json
{ "patches": ["my-crypto"] }
```

### 6.2 架构图

```
QuickJS VM
    │
    ├── __wasm_call("crypto.sha256", '"hello"')
    │       │
    │       ▼
    │   Rust WasmBridge
    │       │
    │       ── emit("wasm-call") ──▶ WebView
    │                                      │
    │                                      ▼
    │                              PatchLoader
    │                                 │
    │                                 ├── WebAssembly.instantiate()
    │                                 └── wasm_module.sha256("hello")
    │                                            │
    │                          invoke("wasm_store_call_result")
    │                                            │
    ▼                                           │
__wasm_get_result(requestId) ◄──────────────────┘
```

---

## 7. 最佳实践

### 7.1 性能优化

- **懒加载**: 只在 `onSearch` 中做必要的初始化工作
- **缓存结果**: 利用 `dbStorage` 缓存高频查询结果
- **限制返回数**: `onSearch` 返回不超过 10 条结果
- **避免阻塞**: 不要在 `pluginInit` 中执行耗时操作

### 7.2 错误处理

```javascript
function onAction(action) {
  try {
    // ... 业务逻辑
    return { type: 'text', message: 'Success' };
  } catch (e) {
    console.error('[my-plugin] 动作失败:', e);
    return { type: 'error', message: String(e.message || e) };
  }
}
```

### 7.3 安全注意事项

- **不信任输入**: 所有来自用户的输入都应校验
- **沙箱隔离**: 插件运行在 QuickJS 沙箱中，无直接文件系统访问
- **API 限制**: 通过 `utools.fs` 访问的文件受操作系统权限约束
- **网络安全**: `utools.fetch` 支持 HTTP/HTTPS，但超时默认 30 秒

### 7.4 数据管理

- **使用 dbStorage**: 插件数据存储在独立的 `data/{plugin_id}/storage.json` 中
- **配额限制**: 单个插件数据上限 10MB
- **清理责任**: 提供清除数据的选项（如设置面板中的「清除数据」按钮）

### 7.5 前缀设计

- **简短有意义**: 2-4 个字符，如 `fs`(文件), `calc`(计算), `url`(URL工具)
- **避免冲突**: 与现有插件前缀不同
- **模糊匹配**: 输入前缀的部分字符也能匹配到插件

---

## 8. 调试技巧

### 8.1 控制台日志

```javascript
// 插件内的 console.log 会输出到 Tauri 终端（RUST_LOG=debug 时更详细）
console.log('[my-plugin] 信息');  // 普通日志
console.error('[my-plugin] 错误'); // 错误日志（红色）
console.warn('[my-plugin] 警告');  // 警告日志（黄色）
```

### 8.2 使用插件管理器测试

Corelia 内置的 **插件管理器** 提供了完整的调试界面：

1. 打开设置 → 插件管理
2. 点击插件查看详情面板
3. 切换到「测试」Tab 进行交互式测试
4. 查看 VM 缓存状态和数据存储信息

### 8.3 常见问题排查

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 插件不被发现 | `plugin.json` 缺少必填字段 | 检查 name/version/type |
| `utools is not defined` | API 未注入 | 检查是否通过 `load_plugin` 正确加载 |
| 搜索无匹配 | 前缀未配置或不匹配 | 检查 `prefix` 配置和输入内容 |
| VM 池满错误 | 超过 10 个活跃 VM | 卸载不需要的插件 |
| 存储写入失败 | 超出 10MB 配额 | 清理旧数据或减小存储量 |

---

## 附录：内置插件列表

| 插件 | 前缀 | 功能 | 状态 |
|------|------|------|------|
| hello-world | hw | 探针/测试用 | Stable |
| file-search | fs | 文件搜索 | Stable |
| url-toolkit | url | 编解码/格式化 | Stable |
| calc | calc | 数学计算 | Stable |

---

*最后更新: 2026-04-14 | Corelia Plugin System v1.0*

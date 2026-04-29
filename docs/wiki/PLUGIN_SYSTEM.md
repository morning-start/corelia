# 插件系统

> Corelia 插件系统采用 QuickJS 沙箱 + WASM Patch 架构，支持 JavaScript 插件的热加载与高性能 Rust 扩展

## 概述

Corelia 插件系统分为两层：

- **QuickJS 插件**：标准 JavaScript 插件，运行在沙箱化的 QuickJS VM 中（后端）
- **WASM Patch**：Rust 编译的 WebAssembly 模块，在 WebView 中执行，供 QuickJS 插件调用（前端）

---

## 插件系统架构分层

### 前端职责

| 组件 | 文件位置 | 职责 |
|------|----------|------|
| PatchLoader | `src/lib/plugins/patch-loader.ts` | WASM Patch 加载、函数注册、WASM 执行、结果存储 |
| PluginService | `src/lib/plugins/service.ts` | 插件生命周期管理、插件搜索、插件状态同步 |
| PluginStore | `src/lib/plugins/store.ts` | 插件相关状态管理 |

### 后端职责

| 组件 | 文件位置 | 职责 |
|------|----------|------|
| QuickJSRuntime | `src-tauri/src/plugins/quickjs_runtime.rs` | VM 池化管理、VM 创建/销毁、代码执行 |
| PluginLoader | `src-tauri/src/plugins/loader/` | 插件扫描、加载、卸载、生命周期管理 |
| PluginRegistry | `src-tauri/src/plugins/registry.rs` | 插件注册表、插件索引、状态管理 |
| ApiBridge | `src-tauri/src/plugins/api_bridge/` | utools API 注入、系统能力桥接 |
| WasmBridge | `src-tauri/src/plugins/wasm_bridge.rs` | WASM 调用桥接、结果缓存、事件触发 |

---

## 插件目录结构

```
plugins/
└── my-plugin/                  # 插件根目录（名称即 ID）
    ├── plugin.json             # 插件元数据（必需）
    ├── index.js                # 入口文件（默认）
    └── patches/                # WASM 依赖（可选）
        └── crypto/             # Patch 目录
            ├── src/            # Rust 源码
            │   └── lib.rs
            └── pkg/            # wasm-pack 构建产物
                ├── crypto.js       # 胶水 JS
                ├── crypto_bg.wasm  # WASM 二进制
                ├── crypto.d.ts     # 类型声明
                └── package.json    # 导出函数信息
```

---

## plugin.json 规范

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "type": "quickjs",
  "description": "插件描述",
  "author": "作者",
  "prefix": "mp",
  "main": "index.js",
  "patches": ["crypto"],
  "features": [
    {
      "code": "search",
      "label": "搜索",
      "type": "list",
      "items": [
        { "label": "搜索文件", "action": "search-files", "icon": "🔍" }
      ]
    }
  ]
}
```

### 字段说明

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `name` | string | ✅ | 插件唯一标识符 |
| `version` | string | ✅ | 语义化版本号 |
| `type` | string | ✅ | 插件类型，目前仅支持 `"quickjs"` |
| `description` | string | ❌ | 插件描述 |
| `author` | string | ❌ | 作者信息 |
| `prefix` | string | ❌ | 触发前缀，用于搜索匹配 |
| `main` | string | ❌ | 入口文件，默认 `index.js` |
| `patches` | string[] | ❌ | WASM 依赖列表 |
| `features` | FeatureConfig[] | ❌ | 功能配置列表 |
| `logo` | string | ❌ | 图标路径 |

---

## 插件生命周期

```
                    scan_plugins()
                         │
                         ↓
                    MetaLoaded (仅元数据)
                         │
                   load_plugin()
                         │
                         ↓
                      Loading
                    ╱         ╲
                   ╱           ╲
                  ↓             ↓
               Ready          Error
                │               │
                ↓               ↓
             Cached          Loading (重试)
                │
                ↓
             Unloading
                │
                ↓
             Unloaded
```

### 状态说明

| 状态 | 说明 | 触发者 |
|------|------|--------|
| MetaLoaded | 已解析 plugin.json，未加载代码 | PluginLoader.scan_plugins() |
| Loading | 正在创建 VM、注入 API、执行代码 | PluginLoader.load_plugin() |
| Ready | 插件就绪，可执行命令 | 插件代码执行完成 |
| Cached | 闲置超时后缓存，可重新激活 | QuickJSRuntime |
| Error | 加载/执行出错 | VM 执行异常 |
| Unloading | 正在卸载 | PluginLoader.unload_plugin() |
| Unloaded | 已卸载，VM 已销毁 | 卸载完成 |

### 前后端协作流程

#### 1. 扫描插件（应用初始化）

```
前端 PluginService.init()
    │
    └─ invoke('scan_plugins')
         │
         └─ 后端 PluginLoader.scan_plugins()
              ├─ 扫描 plugins/ 目录
              ├─ 解析所有 plugin.json
              ├─ 注册到 PluginRegistry
              └─ 返回插件元数据列表
                   │
                   └─ 前端更新 PluginStore
```

**后端职责**：文件系统访问、JSON 解析、插件注册

**前端职责**：初始化触发、状态更新

#### 2. 加载插件（用户触发）

```
前端 invoke('load_plugin', { id: 'my-plugin' })
    │
    └─ 后端 PluginLoader.load_plugin()
         ├─ 创建 VM: QuickJSRuntime.create_vm()
         ├─ 注入 API: ApiBridge.inject_apis_to_vm()
         ├─ 加载 patches: WasmBridge 通知前端
         │    │
         │    └─ emit('wasm-load-patch')
         │         │
         │         └─ 前端 PatchLoader 加载 WASM
         │              ├─ 导入 WASM 模块
         │              ├─ 注册 WASM 函数
         │              └─ invoke('wasm_register_functions')
         │
         ├─ 执行插件代码: QuickJSRuntime.execute()
         └─ 更新状态到 Ready
```

**后端职责**：VM 管理、API 注入、代码执行、事件触发

**前端职责**：WASM 加载、函数注册、结果存储

#### 3. WASM 调用（插件代码中）

```
插件代码: utools.wasm.__wasm_call('crypto.sha256', '"hello"')
    │
    └─ 后端 WasmBridge 处理
         ├─ 生成 requestId
         ├─ emit('wasm-call')
         └─ 返回 requestId
              │
              └─ 前端 PatchLoader.handleWasmCall()
                   ├─ 执行 WASM 函数
                   └─ invoke('wasm_store_call_result')
                        │
                        └─ 后端 WasmBridge 缓存结果

插件代码: utools.wasm.__wasm_get_result(requestId)
    │
    └─ 后端 WasmBridge 返回缓存结果（并移除）
```

**后端职责**：调用桥接、结果缓存、ID 生成

**前端职责**：WASM 执行、结果返回

---

## 插件 API (utools)

插件通过 `window.utools` 对象访问系统能力，兼容 uTools API 规范。

### 存储 API

```javascript
// 键值存储（按插件隔离）
utools.dbStorage.setItem('key', 'value');
const val = utools.dbStorage.getItem('key');  // string | null
utools.dbStorage.removeItem('key');
const all = utools.dbStorage.getAll();        // Record<string, string>
```

**实现位置**：`src-tauri/src/plugins/api_bridge/db_storage.rs`

### 剪贴板 API

```javascript
utools.clipboard.readText();            // 读取剪贴板文本
utools.clipboard.writeText('text');     // 写入剪贴板
utools.clipboard.copyText('text');      // 复制文本
```

**实现位置**：`src-tauri/src/plugins/api_bridge/clipboard.rs`

### Shell API

```javascript
utools.shell.openPath('/path/to/file');    // 用系统默认程序打开
utools.shell.openExternal('https://...');  // 打开链接
utools.shell.showItemInFolder('/path');    // 在文件管理器中显示
utools.shell.beep();                       // 系统提示音
```

**实现位置**：`src-tauri/src/plugins/api_bridge/shell.rs`

### 窗口 API

```javascript
utools.hideMainWindow();
utools.showMainWindow();
utools.setExpendHeight(400);  // 设置窗口高度
utools.outPlugin();           // 退出插件
```

**实现位置**：`src-tauri/src/plugins/api_bridge/window.rs`

### 路径 API

```javascript
utools.getPath('home');       // 用户主目录
utools.getPath('desktop');    // 桌面
utools.getPath('downloads');  // 下载目录
utools.getPath('appdata');    // AppData 目录
utools.getPath('temp');       // 临时目录
```

**实现位置**：`src-tauri/src/plugins/api_bridge/path.rs`

支持的路径名：`home`/`~`, `desktop`, `document`/`documents`, `download`/`downloads`, `music`, `picture`/`pictures`, `video`/`videos`, `temp`/`tmp`, `appdata`, `localappdata`/`appcache`, `userdata`, `config`, `log`/`logs`, `resource`/`resources`, `exe`/`exepath`, `plugin`/`pluginpath`, `root`, `cwd`/`currentdir`。

### 文件系统 API

```javascript
utools.fs.readTextFile('/path/to/file');          // 读取文本文件
utools.fs.writeTextFile('/path/to/file', '内容'); // 写入文本文件
utools.fs.exists('/path');                         // 检查路径是否存在
utools.fs.isDir('/path');                          // 检查是否为目录
```

**实现位置**：`src-tauri/src/plugins/api_bridge/fs.rs`

### HTTP 请求 API

```javascript
const response = utools.fetch('https://api.example.com/data', {});
console.log(response.status);     // 200
console.log(response.ok);         // true
console.log(response.statusText); // "OK"
const text = response.text();     // 响应文本
const json = response.json();     // 解析 JSON
```

**实现位置**：`src-tauri/src/plugins/api_bridge/fetch.rs`

注意：`fetch` 在 QuickJS 中通过 `ureq` 同步执行（独立线程），而非浏览器异步 API。

### 对话框 API

```javascript
utools.dialog.showOpenDialog({});    // 打开文件对话框
utools.dialog.showSaveDialog({});    // 保存文件对话框
utools.dialog.showMessageBox({});    // 消息对话框
```

**实现位置**：`src-tauri/src/plugins/api_bridge/dialog.rs`

### 子进程 API

```javascript
const result = utools.process.exec('dir');
console.log(result.stdout);    // 标准输出
console.log(result.stderr);    // 错误输出
console.log(result.exitCode);  // 退出码

utools.process.getNativeId();    // 设备标识
utools.process.getAppName();     // "Corelia"
utools.process.getAppVersion();  // 应用版本
```

**实现位置**：`src-tauri/src/plugins/api_bridge/process.rs`

### 通知 API

```javascript
utools.showNotification('标题', '内容');
```

**实现位置**：`src-tauri/src/plugins/api_bridge/notification.rs`

### 上下文 API

```javascript
const ctx = utools.getContext();
// { code: "", type: "none", payload: null, refresh: false }
utools.setContext(payload);
```

**实现位置**：`src-tauri/src/plugins/api_bridge/context.rs`

### 生命周期回调

```javascript
utools.onPluginReady(() => {
  console.log('插件就绪');
});

utools.onPluginOut(() => {
  console.log('插件退出');
});

utools.registerPluginFeature(feature);
```

---

## WASM Patch 系统

### 架构

```
QuickJS VM ─__wasm_call→ Rust WasmBridge ─emit event→ WebView
                                                           │
                                                      WebAssembly
                                                           │
Rust WasmBridge ←invoke store← WebView PatchLoader ←──────┘
     │
QuickJS VM ←__wasm_get_result┘
```

### 使用方式（QuickJS 插件中）

```javascript
// 1. 发起 WASM 调用，获取 requestId
const requestId = utools.wasm.__wasm_call('crypto.sha256', '"hello"');

// 2. 轮询获取结果（非阻塞）
let result = null;
while (!result) {
  result = utools.wasm.__wasm_get_result(requestId);
}

// 3. 解析结果
const parsed = JSON.parse(result);
if (parsed.success) {
  console.log('SHA256:', parsed.result);
} else {
  console.error('错误:', parsed.error);
}

// 4. 检查函数是否可用
const available = utools.wasm.__wasm_has('crypto.sha256');
const allFunctions = utools.wasm.__wasm_available();
```

### WASM API 参考

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `__wasm_call` | funcName, argsJson | requestId | 发起 WASM 调用 |
| `__wasm_get_result` | requestId | JSON string / null | 轮询获取结果 |
| `__wasm_available` | - | string[] | 列出可用函数 |
| `__wasm_has` | funcName | boolean | 检查函数是否存在 |

### WASM Patch 开发流程

#### 1. 创建 Patch 项目

```bash
cd patches
cargo new --lib crypto
cd crypto
```

#### 2. 编写 Rust 代码 (`src/lib.rs`)

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn sha256(input: &str) -> String {
    // 实现 SHA256
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}
```

#### 3. 配置 Cargo.toml

```toml
[package]
name = "crypto"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
sha2 = "0.10"
```

#### 4. 构建

```bash
wasm-pack build --target web
```

#### 5. 在 plugin.json 中声明依赖

```json
{
  "patches": ["crypto"]
}
```

#### 6. 在插件代码中调用

```javascript
utools.onPluginReady(() => {
  if (utools.wasm.__wasm_has('crypto.sha256')) {
    const reqId = utools.wasm.__wasm_call('crypto.sha256', '"hello"');
    let result = null;
    while (!result) {
      result = utools.wasm.__wasm_get_result(reqId);
    }
    const parsed = JSON.parse(result);
    if (parsed.success) {
      console.log('SHA256:', parsed.result);
    }
  }
});
```

---

## 前端集成

### PatchLoader 初始化

```typescript
import { patchLoader } from '$lib/plugins/patch-loader';

// 在 +page.svelte onMount 中
patchLoader.init().then(() => {
  console.log('WASM Patch 加载器就绪');
});
```

**PatchLoader 职责**：
- 监听 `wasm-load-patch` 事件，加载指定 Patch
- 监听 `wasm-call` 事件，执行 WASM 函数
- 调用 `wasm_register_functions` 注册 WASM 函数
- 调用 `wasm_store_call_result` 存储 WASM 执行结果

### PluginService

```typescript
import { pluginService } from '$lib/plugins/service';

// 扫描插件
const plugins = await pluginService.init();

// 加载插件
await invoke('load_plugin', { id: 'hello-world' });

// 搜索插件
const results = await invoke('find_plugins_by_prefix', { prefix: 'hw' });
```

**PluginService 职责**：
- 初始化插件系统
- 管理插件生命周期
- 提供插件搜索功能
- 同步插件状态

---

## 插件开发示例

### 最简插件

```
plugins/hello-world/
├── plugin.json
└── index.js
```

**plugin.json**：
```json
{
  "name": "hello-world",
  "version": "1.0.0",
  "type": "quickjs",
  "prefix": "hw",
  "description": "Hello World 示例插件"
}
```

**index.js**：
```javascript
// 插件就绪回调
utools.onPluginReady(() => {
  console.log('Hello World 插件已加载');
});

// 注册功能
utools.registerPluginFeature({
  code: 'greet',
  label: '打招呼',
  type: 'text'
});
```

### 带 WASM 的插件

```
plugins/file-search/
├── plugin.json
├── index.js
└── patches/
    └── file-index/
        ├── src/lib.rs
        └── pkg/
            ├── file-index.js
            ├── file-index_bg.wasm
            └── package.json
```

**index.js**：
```javascript
utools.onPluginReady(() => {
  // 检查 WASM 函数是否可用
  if (utools.wasm.__wasm_has('file-index.build_index')) {
    const reqId = utools.wasm.__wasm_call('file-index.build_index', '"/home/user"');
    
    let result = null;
    while (!result) {
      result = utools.wasm.__wasm_get_result(reqId);
    }
    
    const parsed = JSON.parse(result);
    if (parsed.success) {
      console.log('索引构建完成:', parsed.result);
    }
  }
});
```

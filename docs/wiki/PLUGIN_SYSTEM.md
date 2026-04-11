# Corelia 插件系统参考文档

本文档详细描述 Corelia 的插件系统架构、API 和开发指南。

---

## 目录

1. [架构概览](#架构概览)
2. [插件生命周期](#插件生命周期)
3. [插件开发指南](#插件开发指南)
4. [Plugin JSON 规范](#plugin-json-规范)
5. [uTools 兼容 API 参考](#utools-兼容-api-参考)
6. [前端 PluginService API](#frontend-pluginservice-api)
7. [Tauri Commands (IPC)](#tauri-commands-ipc)
8. [QuickJS 运行时](#quickjs-运行时)
9. [数据隔离存储](#数据隔离存储)
10. [状态机](#状态机)

---

## 架构概览

### 系统层次

```
┌──────────────────────────────────────────────────────────────┐
│                    前端 (Svelte 5 + TypeScript)               │
│                                                              │
│   SearchStore → PluginService → invoke()  ──→ Tauri IPC     │
│        │              │                                      │
│        │ 搜索关键词    │ VM 缓存池 (LRU, 上限 10)             │
│        ▼              ▼                                      │
│   performSearch  executeSearch / executeAction                │
└──────────────────────┬───────────────────────────────────────┘
                       │ Tauri IPC (invoke)
┌──────────────────────▼───────────────────────────────────────┐
│                    Rust 后端 (Tauri State)                   │
│                                                              │
│  ┌──────────────┐ ┌────────────────┐ ┌──────────────────┐   │
│  │ PluginLoader │ │ QuickJSRuntime │ │ PluginRegistry   │   │
│  │ (Mutex)      │ │                │ │ (RwLock)         │   │
│  │              │ │ VM Pool        │ │                  │   │
│  │ scan/load/   │ │ create/destroy │ │ register/search  │   │
│  │ unload/find  │ │ execute/cleanup│ │ state machine    │   │
│  └──────┬───────┘ └───────┬────────┘ └────────┬─────────┘   │
│         │                 │                    │            │
│         └────────┬────────┘                    │            │
│                  ▼                              │            │
│          ┌──────────────┐                      │            │
│          │ ApiBridge     │◄─────────────────────┘            │
│          │              │                                   │
│          │ window.utools │ ← 注入到每个 VM 全局上下文        │
│          │ ├ dbStorage   │                                   │
│          │ ├ clipboard   │                                   │
│          │ ├ shell       │                                   │
│          │ ├ fs          │                                   │
│          │ ├ getPath     │                                   │
│          │ ├ showNotify  │                                   │
│          │ └ callbacks   │                                   │
│          └──────┬───────┘                                   │
│                 │                                           │
│  ┌──────────────┴──────────────────┐                        │
│  │        plugins/ 目录              │                        │
│  │  {plugin-name}/                  │                        │
│  │   ├── plugin.json (元数据)       │                        │
│  │   └── index.js (QuickJS 代码)    │                        │
│  └─────────────────────────────────┘                        │
└──────────────────────────────────────────────────────────────┘
```

### 核心模块

| 模块 | 文件 | 职责 |
|------|------|------|
| **Loader** | `plugins/loader.rs` | 插件扫描、解析、加载、卸载的全生命周期管理 |
| **Registry** | `plugins/registry.rs` | 多索引查询（ID + 前缀）+ 状态机管理 |
| **Runtime** | `plugins/quickjs_runtime.rs` | QuickJS VM 池化（创建/销毁/执行/清理） |
| **ApiBridge** | `plugins/api_bridge.rs` | 将 uTools 兼容 API 注入 VM 全局上下文 |

### 数据流：搜索执行流程

```
用户输入 "hw"
    │
    ▼
前端 SearchStore.performSearch("hw")
    │
    ▼
PluginService.searchByPrefix("hw")
    │ invoke('search_plugins_by_prefix')
    ▼
Registry.search_by_prefix("hw")  → 匹配到 hello-world (prefix: "hw")
    │
    ▼
PluginService.executeSearch("hello-world", "hw")
    │
    ├── 1. ensureLoaded: invoke('load_plugin', { id: 'hello-world' })
    │       │
    │       ▼
    │   Loader.load_plugin("hello-world")
    │       ├── 创建 QuickJS VM
    │       ├── ApiBridge.inject_utools(ctx, "hello-world")
    │       └── 执行 index.js (加载 pluginInit/onSearch/onAction)
    │
    ├── 2. getOrCreateVm: invoke('quickjs_create_vm') + inject_apis_to_vm()
    │
    └── 3. 在 VM 中调用: onSearch("hw")
            │
            ▼
        返回搜索结果列表 → 前端渲染
```

---

## 插件生命周期

### 状态转换图

```
                    scan_plugins()
                         │
                         ▼
  ┌──────────┐    load_plugin()    ┌──────────┐
  │MetaLoaded│ ──────────────────→ │ Loading  │
  └──────────┘                     └────┬─────┘
     ▲                                  │
     │                            执行成功 │ 执行失败
     │ unload_plugin()                  ▼         ▼
     │                            ┌────────┐  ┌───────┐
     │                            │  Ready  │  │ Error │
     │                            └───┬────┘  └───▲───┘
     │                                │           │ load_plugin()
     │                          闲置超时│      (重试)
     │                                ▼           │
     │                           ┌─────────┐      │
     │                           │ Cached  │──────┘
     │                           └────┬────┘
     │                                │ unload_plugin()
     │                                ▼
     │                           ┌──────────┐
     └───────────────────────────│ Unloaded │
                                └──────────┘
```

### 状态说明

| 状态 | 说明 |
|------|------|
| **MetaLoaded** | 仅元数据已加载（`scan_plugins` 后），未创建 VM |
| **Loading** | 正在加载中（`load_plugin` 过程中的临时状态） |
| **Ready** | 就绪，VM 已创建，JS 代码已执行，可响应搜索和动作 |
| **Cached** | 长时间未使用后自动缓存（保留元数据，释放 VM） |
| **Unloaded** | 已卸载，VM 已销毁，可重新加载 |
| **Error(msg)** | 错误状态，记录错误信息 |

### 合法状态转换

```
MetaLoaded → Loading
Loading    → Ready | Error
Ready      → Cached | Unloaded
Cached     → Loading | Unloaded
Unloaded   → Loading
Error      → Loading  (重试加载)
```

---

## 插件开发指南

### 快速开始：创建 Hello World 插件

#### 1. 创建插件目录结构

```bash
mkdir -p plugins/my-plugin
cd my-plugin
```

#### 2. 编写 plugin.json

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "type": "quickjs",
  "logo": "icon.png",
  "prefix": "mp",
  "main": "index.js",
  "description": "我的第一个 Corelia 插件",
  "author": "Your Name",
  "features": [
    {
      "code": "greet",
      "label": "打招呼",
      "type": "list",
      "items": [
        { "label": "说 Hello", "action": "sayHello" },
        { "label": "说 World", "action": "sayWorld" }
      ]
    }
  ]
}
```

#### 3. 编写 index.js

```javascript
// 插件初始化（可选）
function pluginInit() {
  // 可以在初始化时预加载数据或配置
  console.log('[my-plugin] 插件已初始化');

  // 示例：使用 dbStorage 存储持久化数据
  const count = utools.dbStorage.getItem('runCount');
  if (!count) {
    utools.dbStorage.setItem('runCount', '0');
  }
}

// 搜索回调 - 用户输入关键词时触发
// 返回搜索结果数组，每项包含 title/description/icon/action
function onSearch(query) {
  if (!query || query.length === 0) return [];

  const results = [];

  // 根据关键词匹配返回结果
  if (query.includes('hello') || query.includes('你好')) {
    results.push({
      title: 'Hello',
      description: '输出 Hello 消息',
      icon: null,
      action: 'sayHello'
    });
  }

  if (query.includes('world') || query.includes('世界')) {
    results.push({
      title: 'World',
      description: '输出 World 消息',
      icon: null,
      action: 'sayWorld'
    });
  }

  // 使用 dbStorage 读取数据示例
  const count = parseInt(utools.dbStorage.getItem('runCount') || '0');
  results.push({
    title: `运行次数: ${count}`,
    description: '该插件的累计运行次数',
    icon: null,
    action: 'showCount'
  });

  return results;
}

// 动作回调 - 用户选择搜索结果后触发
// action 参数对应 onSearch 返回结果中的 action 字段
function onAction(action) {
  switch (action) {
    case 'sayHello':
      console.log('Hello from my-plugin!');

      // 更新运行次数
      let count = parseInt(utools.dbStorage.getItem('runCount') || '0');
      count++;
      utools.dbStorage.setItem('runCount', String(count));

      // 复制文本到剪贴板
      utools.clipboard.writeText('Hello!');
      return { type: 'text', data: 'Hello!', message: '已复制到剪贴板' };

    case 'sayWorld':
      utools.clipboard.writeText('World!');
      return { type: 'text', data: 'World!', message: '已复制到剪贴板' };

    case 'showCount':
      const c = utools.dbStorage.getItem('runCount');
      return { type: 'text', data: `运行了 ${c} 次`, message: '查看统计' };

    default:
      return { type: 'error', data: null, message: '未知动作: ' + action };
  }
}

// 导出函数（CommonJS 格式）
module.exports = {
  pluginInit,
  onSearch,
  onAction
};
```

#### 4. 测试插件

将 `my-plugin` 目录放入项目的 `plugins/` 文件夹下，启动应用：

```bash
bun run tauri dev
```

输入前缀 `mp` 即可看到你的插件。

---

## Plugin JSON 规范

### 完整字段定义

```typescript
interface PluginManifest {
  /** 插件唯一标识符，同时也是目录名 */
  name: string;

  /** 语义化版本号 */
  version: string;

  /** 插件类型，当前仅支持 "quickjs" */
  type: 'quickjs';

  /** 图标文件名（位于插件目录内） */
  logo?: string;

  /** 触发前缀，用于模糊匹配搜索 */
  prefix?: string;

  /** 入口文件名，默认 "index.js" */
  main?: string;

  /** 插件描述 */
  description?: string;

  /** 作者信息 */
  author?: string;

  /** WASM 补丁依赖列表（预留） */
  patches?: string[];

  /** 静态功能配置项 */
  features?: FeatureConfig[];
}

interface FeatureConfig {
  /** 功能代码 */
  code: string;
  /** 功能显示标签 */
  label: string;
  /** 功能类型：list / text / cmd */
  type: 'list' | 'text' | 'cmd';
  /** 子功能项（list 类型必填） */
  items?: FeatureItem[];
}

interface FeatureItem {
  /** 显示标签 */
  label: string;
  /** 动作标识（传递给 onAction） */
  action: string;
  /** 图标（可选） */
  icon?: string;
}
```

### 必填 vs 可选字段

| 字段 | 必填 | 默认值 | 说明 |
|------|------|--------|------|
| name | ✅ | - | 必须非空 |
| version | ✅ | - | 必须非空 |
| type | ✅ | - | 当前固定为 `"quickjs"` |
| prefix | ❌ | 无前缀匹配 | 搜索触发前缀 |
| main | ❌ | `"index.js"` | 入口 JS 文件 |
| logo | ❌ | 无图标 | 图标文件名 |
| description | ❌ | 无描述 | 描述信息 |
| author | ❌ | 无作者 | 作者信息 |
| patches | ❌ | `[]` | WASM 补丁列表 |
| features | ❌ | `[]` | 静态功能配置 |

---

## uTools 兼容 API 参考

以下 API 通过 `window.utools` 对象注入到 QuickJS VM 全局上下文中。

> **注意**: 所有 API 为同步调用设计（受 QuickJS 单线程限制）

### dbStorage - 数据持久化为每个插件提供隔离的数据存储空间，基于 `tauri-plugin-store` 实现。存储路径：`plugins/{pluginId}/storage.json`

#### getItem(key)

读取存储值。

**参数**: `key: string` - 键名
**返回**: `string | null`
**示例**:
```javascript
const value = utools.dbStorage.getItem('username'); // null 或字符串
```

#### setItem(key, value)

写入存储值。

**参数**:
- `key: string` - 键名
- `value: string` - 值（仅支持字符串）

**返回**: `undefined`
**示例**:
```javascript
utools.dbStorage.setItem('username', 'Alice');
utools.dbStorage.setItem('config', JSON.stringify({ theme: 'dark' }));
```

#### removeItem(key)

删除存储值。

**参数**: `key: string`
**返回**: `undefined`
**示例**:
```javascript
utools.dbStorage.removeItem('oldKey');
```

#### getAll()

获取所有键值对。

**返回**: `Record<string, string>`
**示例**:
```javascript
const all = utools.dbStorage.getAll();
// { "username": "Alice", "count": "42" }
```

---

### clipboard - 剪贴板操作基于 `arboard` 库实现跨平台剪贴板访问。

#### readText()

读取剪贴板文本内容。

**返回**: `string`
**示例**:
```javascript
const text = utools.clipboard.readText();
console.log('剪贴板内容:', text);
```

#### writeText(text)

写入文本到剪贴板。

**参数**: `text: string`
**返回**: `undefined`
**示例**:
```javascript
utools.clipboard.writeText('Hello, World!');
```

---

### shell - Shell 操作基于 `open` 库实现跨平台路径/URL 打开。

#### openPath(path)

打开文件或文件夹。

**参数**: `path: string` - 路径
**返回**: `undefined`
**示例**:
```javascript
utools.shell.openPath('C:\\Users\\Documents');
utools.shell.openPath(utools.getPath('home'));
```

#### openExternal(url)

在默认浏览器打开 URL。

**参数**: `url: string`
**返回**: `undefined`
**示例**:
```javascript
utools.shell.openExternal('https://github.com');
```

#### showItemInFolder(path)

在文件管理器中定位并选中文件。

**参数**: `path: string`
**返回**: `undefined`
**平台差异**:
- Windows: `explorer /select,{path}`
- macOS: `open -R {path}`
- Linux: 打开父目录
**示例**:
```javascript
utools.shell.showItemInFolder('C:\\test.txt');
```

---

### fs - 文件操作提供基础文件系统读写能力。

#### readTextFile(path)

读取文本文件内容。

**参数**: `path: string` - 文件绝对路径
**返回**: `string`
**示例**:
```javascript
const content = utools.fs.readTextFile('C:\\config.txt');
```

#### writeTextFile(path, content)

写入文本文件。

**参数**:
- `path: string` - 文件绝对路径
- `content: string` - 写入内容
**返回**: `undefined`
**示例**:
```javascript
utools.fs.writeTextFile(utools.getPath('desktop') + '/note.txt', 'Hello!');
```

#### exists(path)

检查路径是否存在。

**参数**: `path: string`
**返回**: `boolean`

#### isDir(path)

检查是否为目录。

**参数**: `path: string`
**返回**: `boolean`

---

### getPath - 系统路径获取常用系统目录路径。

#### getPath(name)

获取指定名称的系统路径。

**参数**: `name: string` - 路径名称
**返回**: `string` - 绝对路径
**支持的名称**:

| 名称 | 别名 | 说明 |
|------|------|------|
| home | HOME, ~ | 用户主目录 |
| desktop | DESKTOP | 桌面目录 |
| document | DOCUMENT, documents | 文档目录 |
| download | DOWNLOAD, downloads | 下载目录 |
| music | MUSIC | 音乐目录 |
| picture | PICTURE, pictures, photo, PHOTO | 图片目录 |
| video | VIDEO, videos | 视频目录 |
| temp | TEMP, tmp | 临时目录 |
| appdata | APPDATA | 应用数据目录 |
| localappdata | LOCALAPPDATA | 本地应用数据目录 |

**示例**:
```javascript
const desktop = utools.getPath('desktop');
const docs = utools.getPath('document');
const home = utools.getPath('~');
console.log('桌面:', desktop);
```

---

### showNotification - 系统通知发送跨平台桌面通知。

#### showNotification(title, body)

**参数**:
- `title: string` - 通知标题
- `body: string` - 通知正文
**返回**: `undefined`
**平台实现**:
- Windows: PowerShell `MessageBox`
- macOS: `osascript display notification`
- Linux: `notify-send`
**示例**:
```javascript
utools.showNotification('任务完成', '文件已保存到桌面');
```

---

### 窗口控制控制主窗口的显隐。

#### hideMainWindow()

隐藏主窗口。
**返回**: `undefined`
**示例**:
```javascript
utools.hideMainWindow();  // 执行动作后隐藏窗口
```

#### showMainWindow()

显示主窗口。
**返回**: `undefined`
**示例**:
```javascript
utools.showMainWindow();
```

---

### 生命周期回调插件可调用的生命周期事件函数。

#### onPluginReady()

通知宿主编件就绪。
**返回**: `undefined`
**说明**: 触发 `plugin-ready` 事件到前端
**示例**:
```javascript
function pluginInit() {
  // 初始化完成
  utools.onPluginReady();
}
```

#### onPluginOut()

通知宿主编件即将退出。
**返回**: `undefined`
**说明**: 触发 `plugin-out` 事件到前端
**示例**:
```javascript
function cleanup() {
  // 清理资源
  utools.onPluginOut();
}
```

#### registerPluginFeature(feature)

动态注册功能特性（预留）。
**参数**: `feature: any` - 功能配置对象
**返回**: `undefined`
**说明**: 触发 `plugin-feature` 事件到前端

---

## Frontend PluginService API

前端通过 `src/lib/plugins/service.ts` 中的 `PluginService` 单例与插件系统交互。

### 初始化

```typescript
import { PluginService } from '$lib/plugins/service';

const service = PluginService.getInstance();

// 初始化：扫描插件目录
await service.init();
```

### 核心方法

#### init()

扫描插件目录并加载所有插件元数据。

```typescript
async init(): Promise<string[]>
// 返回: 扫描到的插件 ID 列表
```

#### scan()

重新扫描插件目录（不销毁已有实例）。

```typescript
async scan(): Promise<PluginManifest[]>
```

#### list()

获取所有已发现插件的 manifest 列表。

```typescript
async list(): Promise<PluginManifest[]>
```

#### load(pluginId)

加载指定插件（创建 VM + 执行代码）。

```typescript
async load(pluginId: string): Promise<string>
// 返回: 新状态字符串 ("Ready" / "Error(...)")
```

#### unload(pluginId)

卸载指定插件（销毁 VM）。

```typescript
async unload(pluginId: string): Promise<void>
```

#### searchByPrefix(prefix)

根据前缀搜索匹配的插件。

```typescript
async searchByPrefix(prefix: string): Promise<PluginManifest[]>
```

#### executeSearch(pluginId, query)

在指定插件 VM 中调用 `onSearch(query)`。

```typescript
async executeSearch(
  pluginId: string,
  query: string
): Promise<PluginSearchResult[]>
```

#### executeAction(pluginId, action)

在指定插件 VM 中调用 `onAction(action)`。

```typescript
async executeAction(
  pluginId: string,
  action: string
): Promise<{ type: string; data: any; message?: string }>
```

### 类型定义

```typescript
/** 来自 plugin.json 的元数据 */
interface PluginManifest {
  name: string;
  version: string;
  type: string;           // "quickjs"
  logo?: string;
  prefix?: string;
  main?: string;          // 默认 "index.js"
  description?: string;
  author?: string;
  patches?: string[];
  features?: FeatureConfig[];
}

/** 插件状态联合类型 */
type PluginState =
  | 'MetaLoaded'
  | 'Loading'
  | 'Ready'
  | 'Cached'
  | 'Unloaded'
  | 'Error';              // Error 会携带错误消息

/** 插件运行时实例 */
interface PluginInstance {
  manifest: PluginManifest;
  state: PluginState;
  vmId?: string;
  pluginDir: string;
  loadedAt?: number;
  lastUsed?: number;
}

/** 搜索结果项 */
interface PluginSearchResult {
  pluginId: string;
  title: string;
  description?: string;
  icon?: string;
  action: string;
}

/** 功能配置 */
interface FeatureConfig {
  code: string;
  label: string;
  type: 'list' | 'text' | 'cmd';
  items?: FeatureItem[];
}

/** 功能子项 */
interface FeatureItem {
  label: string;
  action: string;
  icon?: string;
}
```

---

## Tauri Commands (IPC)

以下是插件系统暴露的所有 Tauri IPC 命令，供前端通过 `invoke()` 调用。

### QuickJS 运行时命令

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `quickjs_create_vm` | 无 | `string` (vm_id) | 创建新的 QuickJS VM 实例 |
| `quickjs_destroy_vm` | vm_id: string | `void` | 销毁指定 VM |
| `quickjs_execute` | vm_id, code | `string` (JSON) | 在 VM 中执行 JS 代码 |
| `quickjs_cleanup` | 无 | `number` | 清理闲置超时的 VM |

### API 注入命令

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `inject_apis_to_vm` | vm_id, plugin_id | `void` | 向 VM 注入 window.utools API 对象 |

### 插件加载器命令

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `scan_plugins` | 无 | `PluginManifest[]` | 扫描 plugins 目录，返回所有插件元数据 |
| `get_plugin_list` | 无 | `PluginManifest[]` | 获取已注册插件列表 |
| `load_plugin` | id: string | `string` (状态) | 加载指定插件（创建VM + 执行JS） |
| `unload_plugin` | id: string | `void` | 卸载指定插件（销毁VM） |
| `find_plugins_by_prefix` | prefix: string | `PluginManifest[]` | 模糊前缀匹配搜索插件 |

### 注册表查询命令

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `search_plugins_by_prefix` | prefix: string | `PluginManifest[]` | 双向模糊前缀查询 |
| `get_active_plugins` | 无 | `PluginManifest[]` | 获取 Ready/Cached 状态的插件 |
| `get_plugin_state` | id: string | `string` | 获取指定插件当前状态 |

### 插件数据隔离命令

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `get_plugin_data_path` | plugin_id: string | `string` | 获取插件数据目录路径 |
| `read_plugin_data` | plugin_id, key | `string \| null` | 读取插件隔离数据 |
| `write_plugin_data` | plugin_id, key, value | `void` | 写入插件隔离数据 |
| `delete_plugin_data` | plugin_id, key | `void` | 删除插件隔离数据 |
| `clear_plugin_data` | plugin_id | `void` | 清除插件所有数据 |
| `get_plugin_data_size` | plugin_id | `number` | 获取插件数据大小（字节） |

### 调用示例

```typescript
import { invoke } from '@tauri-apps/api/core';

// 扫描插件
const plugins = await invoke<PluginManifest[]>('scan_plugins');
console.log(`发现 ${plugins.length} 个插件`);

// 加载插件
const state = await invoke<string>('load_plugin', { id: 'hello-world' });
if (state === 'Ready') {
  console.log('插件就绪');
}

// 创建 VM 并注入 API
const vmId = await invoke<string>('quickjs_create_vm');
await invoke('inject_apis_to_vm', { vm_id: vmId, plugin_id: 'hello-world' });

// 在 VM 中执行代码
const result = await invoke<string>('quickjs_execute', {
  vm_id: vmId,
  code: 'utools.getPath("home")'
});
console.log(JSON.parse(result)); // 用户主目录路径
```

---

## QuickJS 运行时### 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| max_memory_bytes | 50 MB (52428800) | 单个 VM 最大内存限制 |
| max_execution_time_ms | 5000 ms (5s) | 单次执行最大时间 |
| max_vm_count | 10 | 最大并发 VM 数量 |
| idle_timeout_secs | 300 s (5 min) | 闲置超时自动销毁 |### VM 生命周期

```
create_vm()  →  [Active]  →  destroy_vm()
                   │
                   │ idle_timeout 到期
                   ▼
              cleanup() 移除
```

### VM ID 格式

```
vm_{unix_timestamp}_{random_hex}
例: vm_1741234567_a3f8c2d1
```

### 值转换链

QuickJS Value → serde_json::Value → JSON String

支持的类型：
- `null` → JSON null
- `bool` → JSON boolean
- `int(i32)` / `float(f64)` → JSON number
- `string` → JSON string
- `array` → JSON array（递归转换）
- `object` → JSON object（递归转换）

---

## 数据隔离存储### 存储机制

每个插件拥有独立的数据存储空间：

```
AppData/
└── plugins/
    ├── hello-world/
    │   └── storage.json    ← hello-world 的数据
    ├── my-plugin/
    │   └── storage.json    ← my-plugin 的数据
    └── another-plugin/
        └── storage.json    ← 另一个插件的数据
```

### 配额限制

- 单个插件默认上限：**10 MB**
- 超出限制时会返回错误
- 使用 `get_plugin_data_size` 查询当前大小

### 前端 PluginStoreService

```typescript
import { PluginStoreService } from '$lib/plugins/store';

const store = PluginStoreService.getInstance('hello-world');

// 读取数据
const username = await store.readData<string>('username');

// 写入数据
await store.writeData('username', 'Alice');
await store.writeData('config', { theme: 'dark' });

// 删除数据
await store.deleteData('oldKey');

// 清除所有数据
await store.clearData();

// 查询大小
const size = await store.getDataSize();
console.log(`数据大小: ${size} bytes`);
```

---

## 状态机### 合法转换规则

| 从状态 | 可转换为 | 条件 |
|--------|----------|------|
| MetaLoaded | Loading | 调用 `load_plugin()` |
| Loading | Ready | JS 执行成功 |
| Loading | Error | JS 执行失败或 VM 创建失败 |
| Ready | Cached | 闲置超时（自动） |
| Ready | Unloaded | 调用 `unload_plugin()` |
| Cached | Loading | 再次调用 `load_plugin()`（重建 VM） |
| Cached | Unloaded | 调用 `unload_plugin()` |
| Unloaded | Loading | 调用 `load_plugin()`（重新加载） |
| Error | Loading | 重试 `load_plugin()` |### 状态校验逻辑

```rust
fn is_valid_transition(from: &PluginState, to: &PluginState) -> bool {
    matches!(
        (from, to),
        // 基本转换规则
        (MetaLoaded, Loading) |
        (Loading, Ready) | (Loading, Error(_)) |
        (Ready, Cached) | (Ready, Unloaded) |
        (Cached, Loading) | (Cached, Unloaded) |
        (Unloaded, Loading) |
        // 错误恢复
        (Error(_), Loading)
    )
}
```

---

## 最佳实践

### 1. 插件编写建议

```javascript
// ✅ 推荐：导出清晰的模块接口
module.exports = { pluginInit, onSearch, onAction };

// ✅ 推荐：onSearch 返回完整的结果结构
return [{ title, description, icon, action }];

// ✅ 推荐：onAction 返回标准化的结果对象
return { type: 'text', data: result, message: '操作成功' };

// ❌ 避免：在 onSearch 中执行耗时操作
// ❌ 避免：直接操作 DOM（无 DOM 环境）
// ❌ 避免：使用 ES Modules（仅支持 CommonJS module.exports）
```

### 2. 性能优化建议

- **懒加载**: 插件仅在首次搜索匹配时才加载（`load_plugin` 按需调用）
- **VM 复用**: 前端 LRU 缓存池避免重复创建 VM
- **自动清理**: 闲置超过 5 分钟的 VM 自动销毁释放内存
- **内存限制**: 单 VM 上限 50MB，防止插件占用过多资源

### 3. 错误处理

```javascript
// 插件代码中的错误处理
function onSearch(query) {
  try {
    // 业务逻辑
    return results;
  } catch (e) {
    console.error('[my-plugin] onSearch error:', e);
    return [];  // 返回空数组而不是抛异常
  }
}

function onAction(action) {
  try {
    // 业务逻辑
    return { type: 'text', data: result };
  } catch (e) {
    console.error('[my-plugin] onAction error:', e);
    return { type: 'error', data: null, message: e.message };
  }
}
```

### 4. 调试技巧

```javascript
// 使用 console.log 输出调试信息（输出到 Rust 控制台）
function pluginInit() {
  console.log('[my-plugin] initializing...');
  console.log('[my-plugin] path:', utools.getPath('home'));
}
```

Rust 端日志会输出到终端（`bun run tauri dev` 的控制台），前缀格式：
```
[ApiBridge] 开始注入 window.utools API (插件: hello-world)...
[ApiBridge]   ✓ dbStorage 模块注入成功 (隔离存储: plugins/hello-world/storage.json)
[PluginLoader] 插件加载成功: hello-world (VM: vm_1741234567_a3f8c2d1)
[utools.dbStorage] getItem: runCount (plugin: hello-world)
```

---

## 待办 / Roadmap

### 已完成 (v0.x)

- [x] QuickJS VM 池化管理
- [x] uTools 兼容 API (dbStorage, clipboard, shell, fs, path, notification, window)
- [x] 插件扫描/加载/卸载全生命周期
- [x] 双向模糊前缀匹配搜索
- [x] 插件数据隔离存储
- [x] 状态机管理
- [x] 前端 Service 层封装
- [x] LRU VM 缓存池（前端侧）
- [x] 闲置超时自动清理

### 开发中 / 计划中

- [ ] WASM 补丁层 (`patches/pool.rs`) - AI/Crypto/Base64 增强
- [ ] Webview 类型插件支持
- [ ] 图片剪贴板 API 完善 (`getClipboardImage` / `setClipboardImage`)
- [ ] 插件沙箱/权限控制系统
- [ ] 插件热重载（开发模式）
- [ ] 插件市场/在线安装
- [ ] 插件签名校验

### 已知限制

1. **仅支持 CommonJS**: 插件入口文件必须使用 `module.exports`，不支持 ESM
2. **同步 API**: 所有注入的 API 都是同步的（QuickJS 单线程限制）
3. **图片剪贴板占位**: `getClipboardImage` / `setClipboardImage` / `getImagePath` 尚未完全实现
4. **无网络请求**: 插件无法直接发起 HTTP 请求（需要后续扩展）
5. **Windows 通知**: 当前使用 PowerShell MessageBox，非原生 Toast 通知

---

**最后更新**: 2026-04-11
**版本**: v1.0.0
**适用版本**: Corelia v0.1.0+

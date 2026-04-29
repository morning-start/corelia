# Tauri Commands API

> 所有前后端通信通过 Tauri IPC (`invoke`) 完成，以下是全部注册的 Tauri Commands，按功能模块组织。

## 调用方式

```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke<ReturnType>('command_name', { param1: 'value' });
```

---

## API 模块总览

| 模块 | 相关文件 | 职责 |
|------|----------|------|
| 插件系统 | `commands/plugin.rs` | 插件扫描、加载、卸载、搜索 |
| WASM 桥接 | `commands/plugin.rs` | WASM 函数注册、调用、结果管理 |
| QuickJS 运行时 | `commands/plugin.rs` | VM 创建、销毁、代码执行 |
| 窗口管理 | `commands/window.rs` | 窗口显示、隐藏、置顶等 |
| 剪贴板 | `commands/clipboard.rs` | 剪贴板读写 |
| Shell 操作 | `commands/shell.rs` | 文件/URL 打开、应用启动 |
| 配置管理 | `commands/config/` | 系统/用户/应用配置 |
| 数据存储 | `commands/store.rs` | 全局数据存储 |
| 自启动 | `commands/autostart.rs` | 开机自启动管理 |
| 全局快捷键 | `commands/shortcut.rs` | 快捷键注册、注销 |
| 插件数据 | `commands/plugin.rs` | 插件隔离数据 |

---

## 1. 插件系统 API

### 扫描插件

```typescript
const plugins: PluginManifest[] = await invoke('scan_plugins');
```

**职责**：后端扫描 `plugins/` 目录，解析所有 `plugin.json`，返回插件元数据列表。

**前端使用场景**：应用初始化时调用，获取所有可用插件。

### 获取插件列表

```typescript
const plugins: PluginManifest[] = await invoke('get_plugin_list');
```

**职责**：返回已扫描的插件列表（不重新扫描）。

**前端使用场景**：需要插件列表但不需要重新扫描时。

### 加载插件

```typescript
const result: { state: string; vm_id: string | null } = await invoke('load_plugin', {
  id: 'hello-world'
});
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | string | 插件 ID |

**职责**：后端创建 QuickJS VM、注入 utools API、加载 WASM patches、执行插件代码。

**前端使用场景**：用户触发插件时调用。

### 卸载插件

```typescript
await invoke('unload_plugin', { id: 'hello-world' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | string | 插件 ID |

**职责**：后端销毁插件的 QuickJS VM，清理资源。

**前端使用场景**：插件不再使用时调用。

### 按前缀搜索插件

```typescript
const plugins: PluginManifest[] = await invoke('find_plugins_by_prefix', {
  prefix: 'hw'
});
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `prefix` | string | 搜索前缀 |

**职责**：后端在插件注册表中按前缀模糊搜索插件。

**前端使用场景**：用户输入搜索关键词时调用。

### 获取活跃插件

```typescript
const plugins: PluginManifest[] = await invoke('get_active_plugins');
```

**职责**：返回所有状态为 Ready 或 Cached 的插件。

**前端使用场景**：需要查看当前活跃插件时。

### 获取插件状态

```typescript
const state: string = await invoke('get_plugin_state', { id: 'hello-world' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | string | 插件 ID |

**返回值**：`MetaLoaded` | `Loading` | `Ready` | `Cached` | `Error` | `Unloading` | `Unloaded`

**职责**：返回指定插件的当前状态。

**前端使用场景**：需要了解插件当前状态时。

---

## 2. WASM 桥接 API

### 注册 WASM 函数

```typescript
await invoke('wasm_register_functions', {
  patch: 'crypto',
  functions: [
    { name: 'crypto.sha256', patch: 'crypto', param_count: 1 }
  ]
});
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `patch` | string | Patch 名称 |
| `functions` | WasmFunctionInfo[] | 函数信息列表 |

**职责**：后端将 WASM 函数信息注册到 WasmBridge。

**前端使用场景**：PatchLoader 加载 WASM 模块后调用。

### 注销 Patch

```typescript
await invoke('wasm_unregister_patch', { patch: 'crypto' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `patch` | string | Patch 名称 |

**职责**：后端注销指定 Patch 的所有函数。

**前端使用场景**：卸载包含 WASM Patch 的插件时调用。

### 列出 WASM 函数

```typescript
const functions: WasmFunctionInfo[] = await invoke('wasm_list_functions');
```

**职责**：返回所有已注册的 WASM 函数。

**前端使用场景**：需要查看可用 WASM 函数时。

### 检查 Patch 是否已加载

```typescript
const loaded: boolean = await invoke('wasm_is_patch_loaded', { patch: 'crypto' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `patch` | string | Patch 名称 |

**职责**：检查指定 Patch 是否已加载。

**前端使用场景**：验证 Patch 加载状态时。

### 调用 WASM 函数（内部使用）

```typescript
const requestId: string = await invoke('wasm_call_function', {
  function: 'crypto.sha256',
  args: '"hello"'
});
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `function` | string | 函数名称 |
| `args` | string | JSON 序列化的参数 |

**职责**：后端生成 requestId，emit `wasm-call` 事件，返回 requestId。

**使用场景**：通常由 QuickJS 中的 `utools.wasm.__wasm_call()` 内部调用，前端不直接调用。

### 存储 WASM 调用结果

```typescript
await invoke('wasm_store_call_result', {
  result: {
    requestId: 'wasm_req_0',
    success: true,
    result: '"hashed_value"'
  }
});
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `result` | WasmCallResultEntry | 调用结果 |

**职责**：后端将 WASM 调用结果存入 WasmBridge 的结果缓存。

**前端使用场景**：PatchLoader 执行完 WASM 函数后调用。

### 获取 WASM 调用结果

```typescript
const result: WasmCallResultEntry | null = await invoke('wasm_get_call_result', {
  requestId: 'wasm_req_0'
});
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `requestId` | string | 请求 ID |

**职责**：从结果缓存中获取指定 requestId 的结果，获取后自动移除。

**使用场景**：通常由 QuickJS 中的 `utools.wasm.__wasm_get_result()` 内部调用。

---

## 3. QuickJS 运行时 API

### 创建 VM

```typescript
const vmId: string = await invoke('quickjs_create_vm');
```

**职责**：后端创建一个新的 QuickJS VM，返回 VM ID。

**使用场景**：通常由 PluginLoader 内部调用，前端不直接调用。

### 销毁 VM

```typescript
await invoke('quickjs_destroy_vm', { vmId: 'vm_xxx' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `vmId` | string | VM 标识符 |

**职责**：后端销毁指定的 QuickJS VM。

**使用场景**：通常由 PluginLoader 内部调用，前端不直接调用。

### 执行代码

```typescript
const result: unknown = await invoke('quickjs_execute', {
  vmId: 'vm_xxx',
  code: '1 + 1'
});
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `vmId` | string | VM 标识符 |
| `code` | string | JavaScript 代码 |

**职责**：后端在指定 VM 中执行 JavaScript 代码，返回执行结果。

**使用场景**：通常由 PluginLoader 内部调用，前端不直接调用。

### 获取活跃 VM 数量

```typescript
const count: number = await invoke('quickjs_active_count');
```

**职责**：返回当前活跃的 VM 数量。

**前端使用场景**：监控 VM 资源使用时。

### 清理闲置 VM

```typescript
const removed: number = await invoke('quickjs_cleanup');
```

**职责**：清理所有闲置超时的 VM，返回被清理的 VM 数量。

**使用场景**：可以定期调用以释放资源。

### 注入 utools API

```typescript
await invoke('inject_apis_to_vm', { vmId: 'vm_xxx', pluginId: 'my-plugin' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `vmId` | string | 目标 VM 标识符 |
| `pluginId` | string | 插件 ID（用于存储隔离） |

**职责**：后端将 utools API 注入到指定 VM 中。

**使用场景**：通常由 PluginLoader 内部调用，前端不直接调用。

---

## 4. 窗口管理 API

### 显示窗口

```typescript
await invoke('show_window');
```

**职责**：后端显示主窗口。

**前端使用场景**：需要显示窗口时。

### 隐藏窗口

```typescript
await invoke('hide_window');
```

**职责**：后端隐藏主窗口。

**前端使用场景**：需要隐藏窗口时（如失去焦点）。

### 切换窗口显示/隐藏

```typescript
await invoke('toggle_window');
```

**职责**：后端根据当前状态切换窗口的显示/隐藏。

**前端使用场景**：快捷键触发时。

### 设置窗口置顶

```typescript
await invoke('set_always_on_top', { alwaysOnTop: true });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `alwaysOnTop` | boolean | 是否置顶 |

**职责**：后端设置窗口是否置顶。

**前端使用场景**：用户在设置中切换置顶选项时。

### 检查窗口可见性

```typescript
const visible: boolean = await invoke('check_window_visible');
```

**职责**：返回窗口当前是否可见。

**前端使用场景**：需要了解窗口状态时。

---

## 5. 剪贴板 API

### 读取剪贴板

```typescript
const text: string = await invoke('read_clipboard');
```

**职责**：后端读取剪贴板文本内容。

**前端使用场景**：需要获取剪贴板内容时。

### 写入剪贴板

```typescript
await invoke('write_clipboard', { text: 'content' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `text` | string | 要写入的文本 |

**职责**：后端将文本写入剪贴板。

**前端使用场景**：需要复制内容到剪贴板时。

---

## 6. Shell 操作 API

### 打开 URL

```typescript
await invoke('open_url', { url: 'https://example.com' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `url` | string | 要打开的 URL |

**职责**：后端使用系统默认浏览器打开 URL。

**前端使用场景**：用户点击链接时。

### 打开文件/目录

```typescript
await invoke('open_path', { path: '/path/to/file' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `path` | string | 要打开的文件或目录路径 |

**职责**：后端使用系统默认程序打开文件或目录。

**前端使用场景**：用户选择打开文件/目录时。

### 打开应用

```typescript
await invoke('open_app', { app: 'notepad' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `app` | string | 应用名称或路径 |

**职责**：后端打开指定应用。

**前端使用场景**：用户选择启动应用时。

---

## 7. 配置管理 API

### 系统配置

```typescript
// 加载系统配置
const config = await invoke('load_system_config');

// 保存系统配置
await invoke('save_system_config', { 
  config: { shortcut: { summon: 'Alt+Space' } } 
});
```

**职责**：管理系统级配置（如快捷键、自启动等）。

**前端使用场景**：用户修改系统设置时。

### 用户配置

```typescript
// 加载用户配置
const config = await invoke('load_user_config');

// 保存用户配置
await invoke('save_user_config', { 
  config: { behavior: { autoHide: true } } 
});

// 重置用户配置
await invoke('reset_user_config');
```

**职责**：管理用户级配置（如外观、行为偏好等）。

**前端使用场景**：用户修改个人设置时。

### 应用配置

```typescript
// 加载应用配置
const config = await invoke('load_app_config');

// 保存应用配置
await invoke('save_app_config', { config: { key: 'value' } });

// 清除应用配置
await invoke('clear_app_config');
```

**职责**：管理应用级配置。

**前端使用场景**：应用需要持久化一些全局状态时。

---

## 8. 数据存储 API

### 保存数据

```typescript
await invoke('save_to_store', { key: 'my_key', value: { data: 123 } });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `key` | string | 存储键 |
| `value` | any | 存储值 |

**职责**：后端将数据保存到全局存储。

**前端使用场景**：需要持久化全局数据时。

### 加载数据

```typescript
const value = await invoke('load_from_store', { key: 'my_key' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `key` | string | 存储键 |

**职责**：后端从全局存储加载数据。

**前端使用场景**：需要读取全局数据时。

### 删除数据

```typescript
await invoke('delete_from_store', { key: 'my_key' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `key` | string | 存储键 |

**职责**：后端从全局存储删除数据。

**前端使用场景**：需要删除全局数据时。

---

## 9. 自启动 API

### 启用自启动

```typescript
await invoke('enable_autostart');
```

**职责**：后端设置应用开机自启动。

**前端使用场景**：用户在设置中启用自启动时。

### 禁用自启动

```typescript
await invoke('disable_autostart');
```

**职责**：后端取消应用开机自启动。

**前端使用场景**：用户在设置中禁用自启动时。

### 检查自启动状态

```typescript
const enabled: boolean = await invoke('is_autostart_enabled');
```

**职责**：返回自启动是否已启用。

**前端使用场景**：需要显示自启动当前状态时。

---

## 10. 全局快捷键 API

### 注册默认快捷键

```typescript
await invoke('register_shortcut_cmd');
```

**职责**：后端注册默认快捷键（Ctrl+Space）。

**前端使用场景**：应用初始化时调用。

### 注册自定义快捷键

```typescript
await invoke('register_custom_shortcut', { shortcut: 'Alt+Space' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `shortcut` | string | 快捷键字符串 |

**职责**：后端注册自定义快捷键。

**前端使用场景**：用户修改快捷键设置时。

### 注销所有快捷键

```typescript
await invoke('unregister_all_shortcuts');
```

**职责**：后端注销所有已注册的快捷键。

**前端使用场景**：应用退出或重置快捷键时。

### 获取当前快捷键

```typescript
const current: string | null = await invoke('get_current_shortcut');
```

**职责**：返回当前注册的快捷键。

**前端使用场景**：需要显示当前快捷键时。

---

## 11. 插件隔离数据 API

每个插件拥有独立的数据目录，通过 `plugin_id` 隔离。

### 获取插件数据路径

```typescript
const path: string = await invoke('get_plugin_data_path', { pluginId: 'my-plugin' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `pluginId` | string | 插件 ID |

**职责**：返回指定插件的数据目录路径。

**前端使用场景**：插件需要直接操作文件时。

### 读取插件数据

```typescript
const data: string = await invoke('read_plugin_data', { 
  pluginId: 'my-plugin', 
  key: 'settings' 
});
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `pluginId` | string | 插件 ID |
| `key` | string | 数据键 |

**职责**：后端读取指定插件的隔离数据。

**使用场景**：通常由 `utools.dbStorage.getItem()` 内部调用。

### 写入插件数据

```typescript
await invoke('write_plugin_data', { 
  pluginId: 'my-plugin', 
  key: 'settings', 
  data: '...' 
});
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `pluginId` | string | 插件 ID |
| `key` | string | 数据键 |
| `data` | string | 数据值 |

**职责**：后端写入指定插件的隔离数据。

**使用场景**：通常由 `utools.dbStorage.setItem()` 内部调用。

### 删除插件数据

```typescript
await invoke('delete_plugin_data', { 
  pluginId: 'my-plugin', 
  key: 'settings' 
});
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `pluginId` | string | 插件 ID |
| `key` | string | 数据键 |

**职责**：后端删除指定插件的隔离数据。

**使用场景**：通常由 `utools.dbStorage.removeItem()` 内部调用。

### 清空插件数据

```typescript
await invoke('clear_plugin_data', { pluginId: 'my-plugin' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `pluginId` | string | 插件 ID |

**职责**：后端清空指定插件的所有隔离数据。

### 获取插件数据大小

```typescript
const size: number = await invoke('get_plugin_data_size', { pluginId: 'my-plugin' });
```

**参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `pluginId` | string | 插件 ID |

**职责**：返回指定插件的数据目录大小（字节）。

---

## Tauri 事件

除 invoke 外，系统通过 Tauri Event 进行异步通信：

| 事件名 | 方向 | 说明 |
|--------|------|------|
| `wasm-load-patch` | Rust → WebView | 通知前端加载 WASM patch |
| `wasm-call` | Rust → WebView | 通知前端执行 WASM 函数 |
| `plugin-ready` | QuickJS → 前端 | 插件就绪 |
| `plugin-out` | QuickJS → 前端 | 插件退出 |
| `plugin-feature` | QuickJS → 前端 | 插件注册功能 |

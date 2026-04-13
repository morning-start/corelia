# Tauri Commands API

> 所有前后端通信通过 Tauri IPC (`invoke`) 完成，以下是全部注册的 Tauri Commands。

## 调用方式

```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke<ReturnType>('command_name', { param1: 'value' });
```

---

## QuickJS 运行时管理

### `quickjs_create_vm`

创建新的 QuickJS VM 实例。

```typescript
const vmId: string = await invoke('quickjs_create_vm');
// "vm_1744356789012_123456789"
```

### `quickjs_destroy_vm`

销毁指定的 QuickJS VM。

```typescript
await invoke('quickjs_destroy_vm', { vmId: 'vm_xxx' });
```

| 参数 | 类型 | 说明 |
|------|------|------|
| vmId | string | VM 标识符 |

### `quickjs_execute`

在指定 VM 中执行 JavaScript 代码。

```typescript
const result: unknown = await invoke('quickjs_execute', {
  vmId: 'vm_xxx',
  code: '1 + 1'
});
// 2
```

| 参数 | 类型 | 说明 |
|------|------|------|
| vmId | string | VM 标识符 |
| code | string | JavaScript 代码 |

### `quickjs_active_count`

获取当前活跃 VM 数量。

```typescript
const count: number = await invoke('quickjs_active_count');
```

### `quickjs_cleanup`

清理闲置超时的 VM。

```typescript
const removed: number = await invoke('quickjs_cleanup');
```

---

## API 注入

### `inject_apis_to_vm`

向指定 VM 注入 `window.utools` API。

```typescript
await invoke('inject_apis_to_vm', { vmId: 'vm_xxx', pluginId: 'my-plugin' });
```

| 参数 | 类型 | 说明 |
|------|------|------|
| vmId | string | 目标 VM 标识符 |
| pluginId | string | 插件 ID（用于存储隔离） |

---

## WASM 桥接

### `wasm_register_functions`

注册 WASM patch 的可用函数（前端调用）。

```typescript
await invoke('wasm_register_functions', {
  patch: 'crypto',
  functions: [
    { name: 'crypto.sha256', patch: 'crypto', param_count: 1 }
  ]
});
```

| 参数 | 类型 | 说明 |
|------|------|------|
| patch | string | Patch 名称 |
| functions | WasmFunctionInfo[] | 函数信息列表 |

### `wasm_unregister_patch`

注销指定 patch 的所有函数。

```typescript
await invoke('wasm_unregister_patch', { patch: 'crypto' });
```

### `wasm_list_functions`

列出所有已注册的 WASM 函数。

```typescript
const functions: WasmFunctionInfo[] = await invoke('wasm_list_functions');
```

### `wasm_is_patch_loaded`

检查 WASM patch 是否已加载。

```typescript
const loaded: boolean = await invoke('wasm_is_patch_loaded', { patch: 'crypto' });
```

### `wasm_call_function`

调用 WASM 函数（通常由 QuickJS 内部调用）。

```typescript
const requestId: string = await invoke('wasm_call_function', {
  function: 'crypto.sha256',
  args: '"hello"'
});
```

### `wasm_store_call_result`

存储 WASM 调用结果（前端调用）。

```typescript
await invoke('wasm_store_call_result', {
  result: {
    requestId: 'wasm_req_0',
    success: true,
    result: '"hashed_value"'
  }
});
```

### `wasm_get_call_result`

获取 WASM 调用结果（轮询）。

```typescript
const result: WasmCallResultEntry | null = await invoke('wasm_get_call_result', {
  requestId: 'wasm_req_0'
});
```

---

## 插件加载器

### `scan_plugins`

扫描插件目录，返回所有插件元数据。

```typescript
const plugins: PluginManifest[] = await invoke('scan_plugins');
```

### `get_plugin_list`

获取已扫描的插件列表（不重新扫描）。

```typescript
const plugins: PluginManifest[] = await invoke('get_plugin_list');
```

### `load_plugin`

加载指定插件（创建 VM + 注入 API + 执行代码）。

```typescript
const result: { state: string; vm_id: string | null } = await invoke('load_plugin', {
  id: 'hello-world'
});
```

### `unload_plugin`

卸载指定插件（销毁 VM）。

```typescript
await invoke('unload_plugin', { id: 'hello-world' });
```

### `find_plugins_by_prefix`

根据前缀模糊搜索插件。

```typescript
const plugins: PluginManifest[] = await invoke('find_plugins_by_prefix', {
  prefix: 'hw'
});
```

---

## 插件注册表

### `search_plugins_by_prefix`

根据前缀搜索已注册插件。

```typescript
const plugins: PluginManifest[] = await invoke('search_plugins_by_prefix', {
  prefix: 'hw'
});
```

### `get_active_plugins`

获取所有活跃插件（Ready/Cached 状态）。

```typescript
const plugins: PluginManifest[] = await invoke('get_active_plugins');
```

### `get_plugin_state`

获取插件当前状态。

```typescript
const state: string = await invoke('get_plugin_state', { id: 'hello-world' });
// "Ready" | "Loading" | "Error" | ...
```

---

## 窗口管理

### `show_window` / `hide_window` / `toggle_window`

```typescript
await invoke('show_window');
await invoke('hide_window');
await invoke('toggle_window');
```

### `set_always_on_top`

```typescript
await invoke('set_always_on_top', { alwaysOnTop: true });
```

### `check_window_visible`

```typescript
const visible: boolean = await invoke('check_window_visible');
```

---

## 剪贴板

### `read_clipboard` / `write_clipboard`

```typescript
const text: string = await invoke('read_clipboard');
await invoke('write_clipboard', { text: 'content' });
```

---

## Shell 操作

### `open_url` / `open_path` / `open_app`

```typescript
await invoke('open_url', { url: 'https://example.com' });
await invoke('open_path', { path: '/path/to/file' });
await invoke('open_app', { app: 'notepad' });
```

---

## 配置管理

### 系统配置

```typescript
const config = await invoke('load_system_config');
await invoke('save_system_config', { config: { shortcut: { summon: 'Alt+Space' } } });
```

### 用户配置

```typescript
const config = await invoke('load_user_config');
await invoke('save_user_config', { config: { behavior: { autoHide: true } } });
await invoke('reset_user_config');
```

### 应用配置

```typescript
const config = await invoke('load_app_config');
await invoke('save_app_config', { config: { key: 'value' } });
await invoke('clear_app_config');
```

---

## 数据存储

### `save_to_store` / `load_from_store` / `delete_from_store`

```typescript
await invoke('save_to_store', { key: 'my_key', value: { data: 123 } });
const value = await invoke('load_from_store', { key: 'my_key' });
await invoke('delete_from_store', { key: 'my_key' });
```

---

## 自启动

### `enable_autostart` / `disable_autostart` / `is_autostart_enabled`

```typescript
await invoke('enable_autostart');
await invoke('disable_autostart');
const enabled: boolean = await invoke('is_autostart_enabled');
```

---

## 全局快捷键

### `register_shortcut_cmd` / `register_custom_shortcut` / `unregister_all_shortcuts` / `get_current_shortcut`

```typescript
await invoke('register_shortcut_cmd');                    // 注册默认快捷键 (Ctrl+Space)
await invoke('register_custom_shortcut', { shortcut: 'Alt+Space' }); // 注册自定义快捷键
await invoke('unregister_all_shortcuts');
const current: string | null = await invoke('get_current_shortcut');
```

---

## 插件数据隔离存储

每个插件拥有独立的数据目录，通过 `plugin_id` 隔离。

### `get_plugin_data_path`

```typescript
const path: string = await invoke('get_plugin_data_path', { pluginId: 'my-plugin' });
```

### `read_plugin_data` / `write_plugin_data`

```typescript
const data: string = await invoke('read_plugin_data', { pluginId: 'my-plugin', key: 'settings' });
await invoke('write_plugin_data', { pluginId: 'my-plugin', key: 'settings', data: '...' });
```

### `delete_plugin_data` / `clear_plugin_data` / `get_plugin_data_size`

```typescript
await invoke('delete_plugin_data', { pluginId: 'my-plugin', key: 'settings' });
await invoke('clear_plugin_data', { pluginId: 'my-plugin' });
const size: number = await invoke('get_plugin_data_size', { pluginId: 'my-plugin' });
```

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

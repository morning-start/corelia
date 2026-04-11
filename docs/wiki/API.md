# Corelia API 文档

本文档描述 Corelia 提供的 Tauri Commands API，供前端调用。

## 调用方式

```typescript
import { invoke } from '@tauri-apps/api/core';

// 调用命令
const result = await invoke('command_name', { param1: 'value1', param2: 'value2' });
```

## 窗口控制 API

### toggle_window

切换窗口显示/隐藏状态。

**调用**:
```typescript
const newVisible = await invoke('toggle_window');
```

**返回**: `Promise<boolean>` - 新的可见状态

**说明**:
- 如果窗口可见，则隐藏
- 如果窗口隐藏，则显示并置顶
- 返回操作后的可见状态

**示例**:
```typescript
// 托盘图标点击处理
tray.on('click', async () => {
  const visible = await invoke('toggle_window');
  console.log('窗口现在:', visible ? '可见' : '隐藏');
});
```

---

### hide_window

隐藏主窗口。

**调用**:
```typescript
await invoke('hide_window');
```

**返回**: `Promise<void>`

**说明**:
- 仅隐藏窗口，不退出应用
- 应用仍在系统托盘运行
- 可通过 `toggle_window` 或 `show_window` 重新显示

**示例**:
```typescript
// 失焦自动隐藏
appWindow.onFocusChanged(async ({ payload: focused }) => {
  if (!focused && autoHide) {
    await invoke('hide_window');
  }
});

// 执行后隐藏
await executeItem(item);
await invoke('hide_window');
```

---

### show_window

显示窗口并置顶。

**调用**:
```typescript
await invoke('show_window');
```

**返回**: `Promise<void>`

**说明**:
- 显示窗口
- 设置窗口置顶
- 聚焦窗口
- 100ms 后自动取消置顶

**示例**:
```typescript
// 从托盘菜单显示
menu.on('show', async () => {
  await invoke('show_window');
});
```

---

## 配置管理 API

### 系统配置

#### load_system_config

加载系统配置。

**调用**:
```typescript
const config = await invoke('load_system_config');
```

**返回**: `Promise<SystemConfig>`
```typescript
interface SystemConfig {
  shortcut: {
    summon: string;      // 快捷键，如 "Alt+Space"
    enabled: boolean;    // 是否启用
  };
  startup: {
    minimizeToTray: boolean;  // 启动时最小化到托盘
  };
}
```

**示例**:
```typescript
const systemConfig = await invoke('load_system_config');
console.log('快捷键:', systemConfig.shortcut.summon);
```

---

#### save_system_config

保存系统配置。

**调用**:
```typescript
await invoke('save_system_config', { config: newConfig });
```

**参数**:
- `config: SystemConfig` - 新的系统配置

**返回**: `Promise<void>`

**示例**:
```typescript
const newConfig = {
  shortcut: { summon: 'Ctrl+Space', enabled: true },
  startup: { minimizeToTray: false }
};

await invoke('save_system_config', { config: newConfig });
```

---

### 用户配置

#### load_user_config

加载用户配置。

**调用**:
```typescript
const config = await invoke('load_user_config');
```

**返回**: `Promise<UserConfig>`
```typescript
interface UserConfig {
  behavior: {
    autoHide: boolean;       // 失焦自动隐藏
    autoHideDelay: number;   // 自动隐藏延迟 (ms)
  };
  theme: 'dark' | 'light' | 'system';  // 主题
}
```

**示例**:
```typescript
const userConfig = await invoke('load_user_config');
const theme = userConfig.theme;
```

---

#### save_user_config

保存用户配置。

**调用**:
```typescript
await invoke('save_user_config', { config: newConfig });
```

**参数**:
- `config: UserConfig` - 新的用户配置

**返回**: `Promise<void>`

**示例**:
```typescript
await invoke('save_user_config', {
  config: {
    behavior: { autoHide: true, autoHideDelay: 3000 },
    theme: 'dark'
  }
});
```

---

#### reset_user_config

重置用户配置到默认值。

**调用**:
```typescript
await invoke('reset_user_config');
```

**返回**: `Promise<void>`

**说明**: 恢复所有用户配置到默认值

**示例**:
```typescript
// 恢复默认设置
if (confirm('确定要恢复默认设置吗？')) {
  await invoke('reset_user_config');
}
```

---

### 应用配置

#### load_app_config

加载应用配置。

**调用**:
```typescript
const config = await invoke('load_app_config');
```

**返回**: `Promise<Record<string, any>>`

**说明**: 加载应用级配置数据

---

#### save_app_config

保存应用配置。

**调用**:
```typescript
await invoke('save_app_config', { key: 'myKey', value: myValue });
```

**参数**:
- `key: string` - 配置键
- `value: any` - 配置值

**返回**: `Promise<void>`

**示例**:
```typescript
await invoke('save_app_config', {
  key: 'lastSearchTime',
  value: Date.now()
});
```

---

#### clear_app_config

清空应用配置。

**调用**:
```typescript
await invoke('clear_app_config');
```

**返回**: `Promise<void>`

**说明**: 删除所有应用配置数据

---

## 数据存储 API

### save_to_store

保存数据到 Store。

**调用**:
```typescript
await invoke('save_to_store', { key: 'myKey', value: myValue });
```

**参数**:
- `key: string` - 存储键
- `value: any` - 存储值 (JSON 可序列化)

**返回**: `Promise<void>`

**示例**:
```typescript
// 保存数组
await invoke('save_to_store', {
  key: 'favorites',
  value: ['app1', 'app2', 'app3']
});

// 保存对象
await invoke('save_to_store', {
  key: 'settings',
  value: { theme: 'dark', lang: 'zh-CN' }
});
```

---

### load_from_store

从 Store 加载数据。

**调用**:
```typescript
const value = await invoke('load_from_store', { key: 'myKey' });
```

**参数**:
- `key: string` - 存储键

**返回**: `Promise<any>` - 存储的值，如果不存在返回 `null`

**示例**:
```typescript
const favorites = await invoke('load_from_store', { key: 'favorites' });
console.log('收藏列表:', favorites);
```

---

### delete_from_store

从 Store 删除数据。

**调用**:
```typescript
await invoke('delete_from_store', { key: 'myKey' });
```

**参数**:
- `key: string` - 存储键

**返回**: `Promise<void>`

**示例**:
```typescript
await invoke('delete_from_store', { key: 'oldData' });
```

---

## Shell 操作 API

### open_app

打开系统应用。

**调用**:
```typescript
await invoke('open_app', { app: 'calc' });
```

**参数**:
- `app: string` - 应用名或路径

**返回**: `Promise<void>`

**支持的应用**:
- Windows: `calc`, `notepad`, `explorer`, `cmd`, `msedge` 等
- macOS: `Safari`, `Finder`, `TextEdit` 等
- Linux: `gnome-calculator`, `gedit`, `nautilus` 等

**示例**:
```typescript
// 打开计算器
await invoke('open_app', { app: 'calc' });

// 打开记事本
await invoke('open_app', { app: 'notepad' });

// 打开文件资源管理器
await invoke('open_app', { app: 'explorer' });
```

---

### open_url

在默认浏览器中打开 URL。

**调用**:
```typescript
await invoke('open_url', { url: 'https://example.com' });
```

**参数**:
- `url: string` - URL 地址

**返回**: `Promise<void>`

**示例**:
```typescript
await invoke('open_url', { url: 'https://github.com' });
await invoke('open_url', { url: 'https://tauri.app' });
```

---

### open_path

打开文件或文件夹。

**调用**:
```typescript
await invoke('open_path', { path: 'C:\\Users\\Documents' });
```

**参数**:
- `path: string` - 文件或文件夹路径

**返回**: `Promise<void>`

**示例**:
```typescript
// 打开文件夹
await invoke('open_path', { path: '%USERPROFILE%\\Documents' });

// 打开文件
await invoke('open_path', { path: 'C:\\Users\\test.txt' });
```

---

## 自启动 API

### enable_autostart

启用开机自启动。

**调用**:
```typescript
await invoke('enable_autostart');
```

**返回**: `Promise<void>`

**说明**: 将应用添加到系统自启动项

**示例**:
```typescript
// 设置面板中
async function enableAutostart() {
  try {
    await invoke('enable_autostart');
    console.log('自启动已启用');
  } catch (error) {
    console.error('启用失败:', error);
  }
}
```

---

### disable_autostart

禁用开机自启动。

**调用**:
```typescript
await invoke('disable_autostart');
```

**返回**: `Promise<void>`

**说明**: 从系统自启动项中移除应用

---

### is_autostart_enabled

检查是否已启用自启动。

**调用**:
```typescript
const enabled = await invoke('is_autostart_enabled');
```

**返回**: `Promise<boolean>`

**示例**:
```typescript
const isAutoStart = await invoke('is_autostart_enabled');
console.log('自启动状态:', isAutoStart);
```

---

## 全局快捷键 API

### register_shortcut_cmd

注册全局快捷键（使用配置的快捷键）。

**调用**:
```typescript
await invoke('register_shortcut_cmd');
```

**返回**: `Promise<void>`

**说明**:
- 从系统配置中读取快捷键
- 注册全局快捷键监听
- 按下快捷键时切换窗口显示/隐藏

**示例**:
```typescript
// 应用启动时注册
onMount(async () => {
  await invoke('register_shortcut_cmd');
  console.log('全局快捷键已注册');
});
```

---

### register_custom_shortcut

注册自定义全局快捷键。

**调用**:
```typescript
await invoke('register_custom_shortcut', { 
  shortcut: 'Ctrl+Shift+X',
  action: 'my_action'
});
```

**参数**:
- `shortcut: string` - 快捷键组合
- `action: string` - 触发的动作

**返回**: `Promise<void>`

**示例**:
```typescript
await invoke('register_custom_shortcut', {
  shortcut: 'Ctrl+Shift+S',
  action: 'quick_search'
});
```

---

### unregister_all_shortcuts

注销所有全局快捷键。

**调用**:
```typescript
await invoke('unregister_all_shortcuts');
```

**返回**: `Promise<void>`

**说明**: 注销所有已注册的快捷键

**示例**:
```typescript
// 应用退出前
await invoke('unregister_all_shortcuts');
```

---

### get_current_shortcut

获取当前注册的快捷键。

**调用**:
```typescript
const shortcut = await invoke('get_current_shortcut');
```

**返回**: `Promise<string | null>` - 当前快捷键，如果没有返回 `null`

**示例**:
```typescript
const currentShortcut = await invoke('get_current_shortcut');
console.log('当前快捷键:', currentShortcut);
```

---

## 剪贴板 API

### read_clipboard

读取剪贴板内容。

**调用**:
```typescript
const text = await invoke('read_clipboard');
```

**返回**: `Promise<string>` - 剪贴板文本内容

**示例**:
```typescript
const clipboardText = await invoke('read_clipboard');
console.log('剪贴板内容:', clipboardText);
```

---

### write_clipboard

写入内容到剪贴板。

**调用**:
```typescript
await invoke('write_clipboard', { text: '要复制的内容' });
```

**参数**:
- `text: string` - 要写入的文本

**返回**: `Promise<void>`

**示例**:
```typescript
await invoke('write_clipboard', { text: 'Hello, World!' });
```

---

## 错误处理

### 错误格式

所有命令返回 `Result<T, String>` 类型，前端使用 try-catch 捕获错误：

```typescript
try {
  const result = await invoke('my_command', { param: 'value' });
  console.log('成功:', result);
} catch (error) {
  console.error('失败:', error);
  // 显示错误提示
}
```

### 常见错误

```typescript
// 参数错误
Error: "Invalid parameter: app name cannot be empty"

// 配置错误
Error: "Config file not found"

// 系统错误
Error: "Failed to open app: calc not found"

// 权限错误
Error: "Permission denied: cannot access system directory"
```

### 错误处理最佳实践

```typescript
// 封装错误处理
async function safeInvoke<T>(command: string, args?: Record<string, any>): Promise<T | null> {
  try {
    return await invoke(command, args);
  } catch (error) {
    console.error(`${command} 失败:`, error);
    // 显示用户友好的错误提示
    alert(`操作失败：${error}`);
    return null;
  }
}

// 使用
const config = await safeInvoke('load_system_config');
if (config) {
  // 处理配置
}
```

---

## 类型定义

### SystemConfig

```typescript
interface SystemConfig {
  shortcut: {
    summon: string;      // 快捷键，如 "Alt+Space"
    enabled: boolean;    // 是否启用
  };
  startup: {
    minimizeToTray: boolean;  // 启动时最小化到托盘
  };
}
```

### UserConfig

```typescript
interface UserConfig {
  behavior: {
    autoHide: boolean;       // 失焦自动隐藏
    autoHideDelay: number;   // 自动隐藏延迟 (ms)
  };
  theme: 'dark' | 'light' | 'system';  // 主题
}
```

### ShortcutConfig

```typescript
interface ShortcutConfig {
  summon: string;      // 呼出快捷键
  enabled: boolean;    // 是否启用
}
```

---

## 使用示例

### 完整配置管理

```typescript
import { invoke } from '@tauri-apps/api/core';

class ConfigManager {
  private systemConfig: SystemConfig | null = null;
  private userConfig: UserConfig | null = null;
  
  /**
   * 加载所有配置
   */
  async loadAll() {
    try {
      this.systemConfig = await invoke('load_system_config');
      this.userConfig = await invoke('load_user_config');
      console.log('配置加载成功');
    } catch (error) {
      console.error('配置加载失败:', error);
      throw error;
    }
  }
  
  /**
   * 保存系统配置
   */
  async saveSystemConfig(config: SystemConfig) {
    await invoke('save_system_config', { config });
    this.systemConfig = config;
  }
  
  /**
   * 保存用户配置
   */
  async saveUserConfig(config: UserConfig) {
    await invoke('save_user_config', { config });
    this.userConfig = config;
  }
  
  /**
   * 获取快捷键
   */
  getShortcut(): string {
    return this.systemConfig?.shortcut.summon || 'Alt+Space';
  }
  
  /**
   * 获取主题
   */
  getTheme(): 'dark' | 'light' | 'system' {
    return this.userConfig?.theme || 'dark';
  }
}

// 使用
const configManager = new ConfigManager();
await configManager.loadAll();
```

### 搜索执行流程

```typescript
import { invoke } from '@tauri-apps/api/core';

async function executeSearchResult(item: ExecutableItem) {
  try {
    // 根据类型执行
    switch (item.type) {
      case 'app':
        await invoke('open_app', { app: item.target });
        break;
      case 'url':
        await invoke('open_url', { url: item.target });
        break;
      case 'path':
        await invoke('open_path', { path: item.target });
        break;
    }
    
    // 执行后隐藏窗口
    if (item.hideWindow !== false) {
      await invoke('hide_window');
    }
    
    console.log('执行成功:', item.name);
  } catch (error) {
    console.error('执行失败:', error);
    throw error;
  }
}
```

---

## 性能建议

### 1. 批量操作

```typescript
// ❌ 避免：多次调用
for (const item of items) {
  await invoke('save_to_store', { key: item.key, value: item.value });
}

// ✅ 推荐：批量保存
await invoke('save_batch', { 
  items: items.map(i => ({ key: i.key, value: i.value })) 
});
```

### 2. 缓存配置

```typescript
// ❌ 避免：频繁读取
async function getTheme() {
  const config = await invoke('load_user_config');
  return config.theme;
}

// ✅ 推荐：缓存
let cachedTheme: string | null = null;

async function getTheme() {
  if (!cachedTheme) {
    const config = await invoke('load_user_config');
    cachedTheme = config.theme;
  }
  return cachedTheme;
}
```

### 3. 防抖调用

```typescript
// 防抖搜索保存
const debouncedSave = debounce((query: string) => {
  invoke('save_to_store', { key: 'lastQuery', value: query });
}, 1000);
```

---

**最后更新**: 2026-04-10  
**版本**: v0.1.0

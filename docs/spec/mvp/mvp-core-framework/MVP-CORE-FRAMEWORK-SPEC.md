# MVP-CORE-FRAMEWORK 技术规格说明书

## 版本信息

| 字段 | 内容 |
|------|------|
| **版本** | v1.0 |
| **作者** | Corelia Team |
| **创建时间** | 2026-04-03 |
| **最后更新** | 2026-04-03 |
| **状态** | 草稿 |
| **前置阶段** | MVP-POC (已完成 98%) |

---

## 目录

- [概要](#概要)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [模块规格](#模块规格)
- [API 设计](#api-设计)
- [数据模型](#数据模型)
- [验收标准](#验收标准)
- [变更记录](#变更记录)

---

## 概要

### 背景

MVP-POC 阶段已完成核心技术验证，验证了 Tauri 2.x 窗口管理、全局快捷键、WASM 环境等技术的可行性。发现 QuickJS 在 Windows MSVC 下存在编译问题，需要使用模拟模式或后续替换方案。

### 目标

实现 Corelia 核心框架，提供完整的产品化基础：

1. 完善窗口管理系统
2. 实现全局快捷键配置系统
3. 实现主题切换系统
4. 实现系统集成服务（剪贴板、Shell、数据存储）
5. 实现主界面和搜索功能

### POC 遗留问题处理

| 问题 | POC 状态 | CORE 解决方案 |
|------|----------|---------------|
| QuickJS Windows MSVC 编译错误 | 模拟模式 | 使用 quickjs-wasm-rs 或 rquickjs |
| WASM 前端类型问题 | 降级方案 | 使用 wasm-bindgen 类型生成 |
| 拼音搜索支持 | 待实现 | 集成 pinyin-pro |

---

## 技术栈

### 前端

| 技术 | 版本 | 用途 |
|------|------|------|
| Svelte 5 | ^5.0.0 | UI 框架 |
| SvelteKit | ^2.9.0 | 应用框架 |
| TypeScript | ~5.6.2 | 类型安全 |
| Vite | ^6.0.3 | 构建工具 |
| fuzzy | ^0.1.3 | 模糊搜索 |

### 后端

| 技术 | 版本 | 用途 |
|------|------|------|
| Tauri | 2.x | 桌面框架 |
| Rust | 1.94.0 | 后端语言 |
| tauri-plugin-global-shortcut | 2.3.1 | 全局快捷键 |
| tauri-plugin-clipboard | 2.x | 剪贴板 |
| tauri-plugin-shell | 2.x | Shell 执行 |
| tauri-plugin-store | 2.x | 数据存储 |

---

## 项目结构

```
corelia/
├── src/
│   ├── lib/
│   │   ├── components/        # UI 组件
│   │   │   ├── SearchBox.svelte
│   │   │   ├── ResultList.svelte
│   │   │   ├── SettingPanel.svelte
│   │   │   └── TitleBar.svelte
│   │   ├── stores/            # 状态管理
│   │   │   ├── theme.ts       # 主题状态
│   │   │   ├── search.ts       # 搜索状态
│   │   │   └── settings.ts     # 设置状态
│   │   ├── services/           # 服务层
│   │   │   ├── clipboard.ts
│   │   │   ├── shell.ts
│   │   │   ├── store.ts
│   │   │   └── crypto.ts
│   │   ├── search/             # 搜索模块
│   │   │   ├── fuzzy.ts
│   │   │   └── performance.ts
│   │   ├── wasm/              # WASM 模块
│   │   │   └── crypto.ts
│   │   └── styles/            # 样式
│   │       └── themes.css
│   └── routes/
│       ├── +layout.svelte
│       └── +page.svelte       # 主页面
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs             # 主入口
│   │   ├── main.rs            # Rust 入口
│   │   ├── commands/          # Tauri Commands
│   │   │   ├── mod.rs
│   │   │   ├── window.rs
│   │   │   ├── shortcut.rs
│   │   │   ├── clipboard.rs
│   │   │   ├── shell.rs
│   │   │   └── store.rs
│   │   └── plugins/          # 插件
│   │       └── mod.rs
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── capabilities/
│       └── default.json
├── patches/
│   └── crypto/                # WASM 加密模块
│       ├── Cargo.toml
│       ├── src/lib.rs
│       └── pkg/               # 编译输出
└── docs/
    └── spec/mvp/mvp-core-framework/
```

---

## 模块规格

### CORE-01 项目结构搭建

#### 目录结构

按照上述项目结构创建目录和文件。

#### 路径别名配置

```javascript
// vite.config.js
import path from 'path';
export default {
  resolve: {
    alias: {
      '$lib': path.resolve('./src/lib'),
      '$components': path.resolve('./src/lib/components'),
      '$stores': path.resolve('./src/lib/stores'),
      '$services': path.resolve('./src/lib/services'),
    }
  }
};
```

### CORE-02 窗口管理器

#### tauri.conf.json 配置

```json
{
  "app": {
    "windows": [{
      "label": "main",
      "width": 600,
      "height": 400,
      "minWidth": 400,
      "minHeight": 300,
      "decorations": false,
      "transparent": true,
      "alwaysOnTop": true,
      "skipTaskbar": true,
      "resizable": true,
      "center": true
    }]
  }
}
```

#### Rust 模块 (src-tauri/src/commands/window.rs)

```rust
use tauri::{AppHandle, Manager, WebviewWindow};

pub struct WindowManager;

impl WindowManager {
    pub fn show(app: &AppHandle) -> Result<(), String> {
        let window = app.get_webview_window("main")
            .ok_or("Window not found")?;
        window.show().map_err(|e| e.to_string())
    }

    pub fn hide(app: &AppHandle) -> Result<(), String> {
        let window = app.get_webview_window("main")
            .ok_or("Window not found")?;
        window.hide().map_err(|e| e.to_string())
    }

    pub fn toggle(app: &AppHandle) -> Result<(), String> {
        let window = app.get_webview_window("main")
            .ok_or("Window not found")?;
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| e.to_string())
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())
        }
    }

    pub fn set_always_on_top(app: &AppHandle, on_top: bool) -> Result<(), String> {
        let window = app.get_webview_window("main")
            .ok_or("Window not found")?;
        window.set_always_on_top(on_top).map_err(|e| e.to_string())
    }
}
```

### CORE-03 全局快捷键系统

#### Rust 模块 (src-tauri/src/commands/shortcut.rs)

```rust
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[derive(serde::Serialize)]
pub struct ShortcutConfig {
    pub key: String,
    pub modifiers: Vec<String>,
}

pub struct ShortcutManager;

impl ShortcutManager {
    pub fn register_default(app: &AppHandle) -> Result<(), String> {
        let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
        app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let window = app.get_webview_window("main").unwrap();
                if window.is_visible().unwrap() {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }).map_err(|e| e.to_string())
    }

    pub fn unregister_all(app: &AppHandle) -> Result<(), String> {
        app.global_shortcut().unregister_all()
            .map_err(|e| e.to_string())
    }
}
```

### CORE-04 主题系统

#### CSS 变量定义 (src/lib/styles/themes.css)

```css
:root {
  --radius: 12px;
  --bg-color: rgba(26, 26, 26, 0.95);
  --text-color: #f6f6f6;
  --accent-color: #646cff;
  --border-color: rgba(255, 255, 255, 0.1);
  --hover-color: rgba(255, 255, 255, 0.05);
}

[data-theme="light"] {
  --bg-color: rgba(255, 255, 255, 0.95);
  --text-color: #1a1a1a;
  --accent-color: #646cff;
  --border-color: rgba(0, 0, 0, 0.1);
  --hover-color: rgba(0, 0, 0, 0.05);
}

[data-theme="dark"] {
  --bg-color: rgba(26, 26, 26, 0.95);
  --text-color: #f6f6f6;
  --accent-color: #646cff;
  --border-color: rgba(255, 255, 255, 0.1);
  --hover-color: rgba(255, 255, 255, 0.05);
}
```

#### 主题 Store (src/lib/stores/theme.ts)

```typescript
import { writable } from 'svelte/store';

export type Theme = 'dark' | 'light' | 'system';

function createThemeStore() {
  const { subscribe, set, update } = writable<Theme>('system');

  return {
    subscribe,
    set: (theme: Theme) => {
      if (typeof document !== 'undefined') {
        document.documentElement.setAttribute('data-theme', theme);
      }
      set(theme);
    },
    toggle: () => {
      update(current => {
        const next = current === 'dark' ? 'light' : 'dark';
        if (typeof document !== 'undefined') {
          document.documentElement.setAttribute('data-theme', next);
        }
        return next;
      });
    }
  };
}

export const theme = createThemeStore();
```

### CORE-05 设置面板

#### SettingPanel.svelte 结构

```svelte
<script lang="ts">
  import { theme, type Theme } from '$lib/stores/theme';
  import { invoke } from '@tauri-apps/api/core';

  let currentShortcut = 'Alt + Space';

  async function handleShortcutChange() {
    // 实现快捷键录制
  }

  function handleThemeChange(newTheme: Theme) {
    theme.set(newTheme);
  }
</script>

<div class="setting-panel">
  <h2>设置</h2>

  <section>
    <h3>快捷键</h3>
    <button onclick={handleShortcutChange}>{currentShortcut}</button>
  </section>

  <section>
    <h3>主题</h3>
    <select onchange={(e) => handleThemeChange(e.target.value)}>
      <option value="dark">深色</option>
      <option value="light">浅色</option>
      <option value="system">跟随系统</option>
    </select>
  </section>
</div>
```

### CORE-06 剪贴板服务

#### Rust 模块 (src-tauri/src/commands/clipboard.rs)

```rust
use tauri::AppHandle;

#[tauri::command]
pub async fn read_clipboard() -> Result<String, String> {
    // 使用 tauri-plugin-clipboard
    Ok("clipboard content".to_string())
}

#[tauri::command]
pub async fn write_clipboard(text: String) -> Result<(), String> {
    // 使用 tauri-plugin-clipboard
    Ok(())
}
```

### CORE-07 Shell 服务

#### Rust 模块 (src-tauri/src/commands/shell.rs)

```rust
use tauri::AppHandle;

#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_app(app: String) -> Result<(), String> {
    // 启动指定应用
    Ok(())
}
```

### CORE-08 数据存储服务

#### Rust 模块 (src-tauri/src/commands/store.rs)

```rust
use tauri::AppHandle;

#[tauri::command]
pub async fn save_settings(settings: serde_json::Value) -> Result<(), String> {
    // 使用 tauri-plugin-store 保存设置
    Ok(())
}

#[tauri::command]
pub async fn load_settings() -> Result<serde_json::Value, String> {
    // 使用 tauri-plugin-store 加载设置
    Ok(serde_json::json!({}))
}
```

### CORE-09 主界面布局

#### +page.svelte 结构

```svelte
<script lang="ts">
  import TitleBar from '$lib/components/TitleBar.svelte';
  import SearchBox from '$lib/components/SearchBox.svelte';
  import ResultList from '$lib/components/ResultList.svelte';
  import SettingPanel from '$lib/components/SettingPanel.svelte';
  import { onMount } from 'svelte';

  let showSettings = false;
  let searchQuery = '';

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      showSettings = false;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="app-container">
  <TitleBar onSettingsClick={() => showSettings = true} />

  {#if showSettings}
    <SettingPanel onClose={() => showSettings = false} />
  {:else}
    <SearchBox bind:value={searchQuery} />
    <ResultList query={searchQuery} />
  {/if}
</div>
```

### CORE-10 搜索组件

#### Search Store (src/lib/stores/search.ts)

```typescript
import { writable, derived } from 'svelte/store';
import { search, generateTestData, type SearchItem } from '$lib/search/fuzzy';

const items = writable<SearchItem[]>(generateTestData(1000));
const query = writable('');

export const results = derived(
  [query, items],
  ([$query, $items]) => {
    if (!$query.trim()) return [];
    return search($query, $items);
  }
);

export const searchStore = {
  query,
  results,
  items
};
```

---

## API 设计

### Tauri Commands

| Command | 参数 | 返回值 | 说明 |
|---------|------|--------|------|
| `window.show` | - | `Result<(), String>` | 显示窗口 |
| `window.hide` | - | `Result<(), String>` | 隐藏窗口 |
| `window.toggle` | - | `Result<(), String>` | 切换显示状态 |
| `shortcut.register` | - | `Result<(), String>` | 注册快捷键 |
| `shortcut.unregister_all` | - | `Result<(), String>` | 注销所有快捷键 |
| `clipboard.read` | - | `Result<String, String>` | 读取剪贴板 |
| `clipboard.write` | `text: String` | `Result<(), String>` | 写入剪贴板 |
| `shell.open_url` | `url: String` | `Result<(), String>` | 打开 URL |
| `store.save` | `key: String, value: Value` | `Result<(), String>` | 保存数据 |
| `store.load` | `key: String` | `Result<Value, String>` | 加载数据 |

### 前端 Services

| Service | 方法 | 说明 |
|---------|------|------|
| `clipboard.ts` | `read()`, `write(text)` | 剪贴板读写 |
| `shell.ts` | `openUrl(url)`, `openApp(app)` | Shell 操作 |
| `store.ts` | `save(key, value)`, `load(key)` | 数据存储 |
| `crypto.ts` | `encodeBase64()`, `hashSha256()` | 加密服务 |

---

## 数据模型

### Settings

```typescript
interface Settings {
  theme: 'dark' | 'light' | 'system';
  shortcut: {
    summon: string;
  };
  behavior: {
    autoHide: boolean;
    autoHideDelay: number;
  };
  startup: {
    enabled: boolean;
    minimizeToTray: boolean;
  };
}
```

### SearchItem

```typescript
interface SearchItem {
  id: string;
  name: string;
  description: string;
  category: string;
  icon?: string;
  action?: () => void;
}
```

---

## 验收标准

详见 [acceptance.md](acceptance.md)

### 验收概览

| 阶段 | 验收项数 | 通过标准 | 状态 |
|------|----------|----------|------|
| CORE-01 项目结构 | 4 | 4/4 | ❌ |
| CORE-02 窗口管理 | 6 | 6/6 | ❌ |
| CORE-03 快捷键系统 | 5 | 5/5 | ❌ |
| CORE-04 主题系统 | 4 | 4/4 | ❌ |
| CORE-05 设置面板 | 5 | 5/5 | ❌ |
| CORE-06 剪贴板服务 | 3 | 3/3 | ❌ |
| CORE-07 Shell 服务 | 4 | 4/4 | ❌ |
| CORE-08 数据存储 | 4 | 4/4 | ❌ |
| CORE-09 主界面 | 6 | 6/6 | ❌ |
| CORE-10 搜索组件 | 5 | 5/5 | ❌ |
| **总计** | **46** | **46/46** | **0%** |

---

## 变更记录

| 版本 | 时间 | 变更内容 | 变更人 |
|------|------|----------|--------|
| v1.0 | 2026-04-03 | 初稿创建 | Corelia Team |

---

**最后更新**：2026-04-03

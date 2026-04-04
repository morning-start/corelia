# MVP-ECOSYSTEM 规格说明书

## 版本信息

| 字段 | 内容 |
|------|------|
| **版本** | v1.0 |
| **作者** | Corelia Team |
| **创建时间** | 2026-04-03 |
| **状态** | 草稿 |

---

## 概要

### 背景

MVP-CORE-FRAMEWORK 阶段已完成核心框架实现，发现以下遗留问题需要在 MVP-ECOSYSTEM 阶段解决：

| 遗留问题 | 来源 | 优先级 |
|----------|------|--------|
| 快捷键自定义配置 | CORE-03 | P1 |
| 开机自启动 | CORE-05 | P1 |
| 存储持久化 | CORE-08 | P0 |
| 搜索历史 | CORE-10 | P1 |
| 分类搜索 | CORE-10 | P2 |
| 结果高亮 | CORE-10 | P2 |
| 剪贴板清空 | CORE-06 | P2 |
| 终端打开 | CORE-07 | P2 |

### 目标

实现完整的产品化功能，解决所有遗留问题：

1. **P0**：实现持久化存储系统
2. **P1**：实现自定义快捷键配置和开机自启动
3. **P2**：完善搜索功能（历史、分类、高亮）

### 功能列表

| 功能模块 | 功能名称 | 优先级 |
|----------|----------|--------|
| ECO-01 | 持久化存储系统 | P0 |
| ECO-02 | 快捷键自定义 | P1 |
| ECO-03 | 开机自启动 | P1 |
| ECO-04 | 搜索历史 | P1 |
| ECO-05 | 分类搜索 | P2 |
| ECO-06 | 结果高亮 | P2 |

---

## 技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| tauri-plugin-store | 2.x | 数据持久化 |
| tauri-plugin-autostart | 2.x | 开机自启动 |
| tauri-plugin-shell | 2.x | Shell 执行 |
| arboard | 3.6.1 | 剪贴板 |

---

## 目录结构

```
src/
├── lib/
│   ├── components/
│   │   ├── SearchBox.svelte
│   │   ├── ResultList.svelte
│   │   ├── TitleBar.svelte
│   │   └── SettingPanel.svelte
│   ├── stores/
│   │   ├── theme.ts
│   │   ├── settings.ts
│   │   ├── search.ts
│   │   └── history.ts          # ECO-04: 搜索历史
│   ├── services/
│   │   ├── clipboard.ts
│   │   ├── shell.ts
│   │   └── store.ts           # ECO-01: 持久化存储
│   └── styles/
│       └── themes.css
src-tauri/src/commands/
├── window.rs
├── shortcut.rs                # ECO-02: 快捷键
├── clipboard.rs
├── shell.rs
└── store.rs                  # ECO-01: 存储
```

---

## 模块规格

### ECO-01 持久化存储系统

#### 功能描述

使用 `tauri-plugin-store` 实现数据的持久化存储，支持：

- 设置项持久化（主题、快捷键、行为配置）
- 搜索历史持久化
- 用户偏好设置

#### Rust 模块 (src-tauri/src/commands/store.rs)

```rust
use tauri_plugin_store::StoreExt;

#[tauri::command]
async fn save_to_store(app: tauri::AppHandle, key: String, value: serde_json::Value) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set(&key, value);
    store.save().map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_from_store(app: tauri::AppHandle, key: String) -> Result<serde_json::Value, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.get(&key).ok_or("Key not found".to_string())
}

#[tauri::command]
async fn delete_from_store(app: tauri::AppHandle, key: String) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.delete(&key).map_err(|e| e.to_string())?;
    store.save().map_err(|e| e.to_string())
}
```

#### 前端服务 (src/lib/services/store.ts)

```typescript
import { invoke } from '@tauri-apps/api/core';

export interface StoreService {
  save(key: string, value: any): Promise<void>;
  load(key: string): Promise<any>;
  delete(key: string): Promise<void>;
  clear(): Promise<void>;
}

class TauriStoreService implements StoreService {
  async save(key: string, value: any): Promise<void> {
    await invoke('save_to_store', { key, value });
  }

  async load(key: string): Promise<any> {
    return await invoke('load_from_store', { key });
  }

  async delete(key: string): Promise<void> {
    await invoke('delete_from_store', { key });
  }

  async clear(): Promise<void> {
    const store = await invoke('load_from_store', { key: '__all__' });
    // 遍历删除所有键
  }
}

export const storeService = new TauriStoreService();
```

#### 数据结构

```typescript
interface StoredSettings {
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

interface SearchHistory {
  items: Array<{
    query: string;
    timestamp: number;
    count: number;
  }>;
  maxItems: number;
}
```

---

### ECO-02 快捷键自定义

#### 功能描述

允许用户自定义唤起窗口的快捷键：

- 支持单键快捷键（如 F1、Space）
- 支持组合键（如 Alt+Space、Ctrl+Shift+P）
- 快捷键冲突检测
- 快捷键预览

#### Rust 模块 (src-tauri/src/commands/shortcut.rs)

```rust
use tauri::AppHandle;
use std::collections::HashMap;
use std::sync::Mutex;

static REGISTERED_SHORTCUTS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

#[tauri::command]
fn register_custom_shortcut(app: AppHandle, shortcut: String, action: String) -> Result<(), String> {
    // 解析快捷键字符串
    // 检测冲突
    // 注册快捷键
    Ok(())
}

#[tauri::command]
fn unregister_shortcut(app: AppHandle, shortcut: String) -> Result<(), String> {
    app.global_shortcut().unregister(&parse_shortcut(&shortcut)?)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn check_shortcut_conflict(shortcut: String) -> Result<bool, String> {
    // 检查快捷键是否与系统或其他应用冲突
    Ok(false)
}

fn parse_shortcut(shortcut: &str) -> Result<Shortcut, String> {
    // 解析 "Ctrl+Shift+P" 格式的字符串
}
```

#### 前端组件 (src/lib/components/ShortcutRecorder.svelte)

```svelte
<script lang="ts">
  let recording = $state(false);
  let currentShortcut = $state('');
  let keys: string[] = $state([]);

  function startRecording() {
    recording = true;
    keys = [];
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!recording) return;
    event.preventDefault();

    const parts: string[] = [];
    if (event.ctrlKey) parts.push('Ctrl');
    if (event.altKey) parts.push('Alt');
    if (event.shiftKey) parts.push('Shift');
    if (event.metaKey) parts.push('Meta');

    if (event.key !== 'Control' && event.key !== 'Alt' && event.key !== 'Shift' && event.key !== 'Meta') {
      parts.push(event.key.toUpperCase());
      currentShortcut = parts.join('+');
      recording = false;
    }

    keys = parts;
  }
</script>

<div class="shortcut-recorder">
  {#if recording}
    <input
      type="text"
      readonly
      placeholder="按下快捷键..."
      onkeydown={handleKeydown}
    />
  {:else}
    <button onclick={startRecording}>
      {currentShortcut || '点击设置快捷键'}
    </button>
  {/if}
</div>
```

---

### ECO-03 开机自启动

#### 功能描述

使用 `tauri-plugin-autostart` 实现开机自启动功能：

- 开关控制自启动
- 最小化到托盘选项
- 启动时隐藏主窗口

#### Rust 配置

```rust
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
        // ...
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### 前端服务 (src/lib/services/startup.ts)

```typescript
import { invoke } from '@tauri-apps/api/core';

export interface StartupService {
  enable(): Promise<void>;
  disable(): Promise<void>;
  isEnabled(): Promise<boolean>;
}

class TauriStartupService implements StartupService {
  async enable(): Promise<void> {
    await invoke('enable_autostart');
  }

  async disable(): Promise<void> {
    await invoke('disable_autostart');
  }

  async isEnabled(): Promise<boolean> {
    return await invoke('is_autostart_enabled');
  }
}

export const startupService = new TauriStartupService();
```

---

### ECO-04 搜索历史

#### 功能描述

记录和管理用户的搜索历史：

- 自动记录搜索词
- 按使用频率排序
- 手动清除历史
- 历史上限（最多 100 条）

#### 前端 Store (src/lib/stores/history.ts)

```typescript
import { writable, get } from 'svelte/store';
import { storeService } from '$lib/services/store';

interface HistoryItem {
  query: string;
  timestamp: number;
  count: number;
}

interface SearchHistoryStore {
  items: HistoryItem[];
  maxItems: number;
}

function createHistoryStore() {
  const { subscribe, set, update } = writable<SearchHistoryStore>({
    items: [],
    maxItems: 100
  });

  return {
    subscribe,

    async init() {
      try {
        const stored = await storeService.load('search_history');
        if (stored) {
          set({ items: stored.items || [], maxItems: 100 });
        }
      } catch (e) {
        console.error('Failed to load search history:', e);
      }
    },

    add(query: string) {
      update(history => {
        const existing = history.items.find(item => item.query === query);
        if (existing) {
          existing.count++;
          existing.timestamp = Date.now();
        } else {
          history.items.unshift({
            query,
            timestamp: Date.now(),
            count: 1
          });
        }

        // 限制数量
        if (history.items.length > history.maxItems) {
          history.items = history.items
            .sort((a, b) => b.count - a.count || b.timestamp - a.timestamp)
            .slice(0, history.maxItems);
        }

        // 异步保存
        storeService.save('search_history', { items: history.items }).catch(console.error);

        return history;
      });
    },

    async clear() {
      set({ items: [], maxItems: 100 });
      await storeService.delete('search_history');
    },

    getRecent(limit: number = 10): string[] {
      const state = get({ subscribe });
      return state.items
        .sort((a, b) => b.timestamp - a.timestamp)
        .slice(0, limit)
        .map(item => item.query);
    }
  };
}

export const searchHistory = createHistoryStore();
```

---

### ECO-05 分类搜索

#### 功能描述

支持按分类筛选搜索结果：

- 预定义分类：系统、插件、历史
- 分类标签栏 UI
- 分类过滤逻辑

#### 数据结构

```typescript
interface CategorizedItem extends SearchItem {
  category: 'system' | 'plugin' | 'history' | 'custom';
}

interface CategoryFilter {
  enabled: boolean;
  categories: string[];
}
```

#### 组件 (src/lib/components/CategoryTabs.svelte)

```svelte
<script lang="ts">
  type Category = 'all' | 'system' | 'plugin' | 'history';

  let selectedCategory = $state<Category>('all');

  const categories: { id: Category; label: string }[] = [
    { id: 'all', label: '全部' },
    { id: 'system', label: '系统' },
    { id: 'plugin', label: '插件' },
    { id: 'history', label: '历史' }
  ];

  function selectCategory(cat: Category) {
    selectedCategory = cat;
  }
</script>

<div class="category-tabs">
  {#each categories as cat}
    <button
      class="tab"
      class:active={selectedCategory === cat.id}
      onclick={() => selectCategory(cat.id)}
    >
      {cat.label}
    </button>
  {/each}
</div>
```

---

### ECO-06 结果高亮

#### 功能描述

在搜索结果中高亮匹配的字符：

- 匹配字符使用不同颜色
- 支持模糊匹配高亮
- 性能优化（虚拟列表）

#### 组件 (src/lib/components/HighlightedText.svelte)

```svelte
<script lang="ts">
  interface Props {
    text: string;
    query: string;
  }

  let { text, query }: Props = $props();

  function getHighlightedParts(text: string, query: string) {
    if (!query) return [{ text, highlight: false }];

    const parts: { text: string; highlight: boolean }[] = [];
    let lastIndex = 0;
    const lowerText = text.toLowerCase();
    const lowerQuery = query.toLowerCase();

    let index = lowerText.indexOf(lowerQuery);
    while (index !== -1) {
      if (index > lastIndex) {
        parts.push({ text: text.slice(lastIndex, index), highlight: false });
      }
      parts.push({ text: text.slice(index, index + query.length), highlight: true });
      lastIndex = index + query.length;
      index = lowerText.indexOf(lowerQuery, lastIndex);
    }

    if (lastIndex < text.length) {
      parts.push({ text: text.slice(lastIndex), highlight: false });
    }

    return parts;
  }

  let parts = $derived(getHighlightedParts(text, query));
</script>

<span class="highlighted-text">
  {#each parts as part}
    {#if part.highlight}
      <mark class="highlight">{part.text}</mark>
    {:else}
      {part.text}
    {/if}
  {/each}
</span>

<style>
  .highlight {
    background: var(--accent-color);
    color: white;
    border-radius: 2px;
    padding: 0 2px;
  }
</style>
```

---

## API 设计

### Tauri Commands

| Command | 参数 | 返回值 | 说明 |
|---------|------|--------|------|
| `save_to_store` | `key: String, value: Value` | `Result<(), String>` | 保存数据 |
| `load_from_store` | `key: String` | `Result<Value, String>` | 加载数据 |
| `delete_from_store` | `key: String` | `Result<(), String>` | 删除数据 |
| `register_custom_shortcut` | `shortcut: String, action: String` | `Result<(), String>` | 注册快捷键 |
| `unregister_shortcut` | `shortcut: String` | `Result<(), String>` | 注销快捷键 |
| `check_shortcut_conflict` | `shortcut: String` | `Result<bool, String>` | 检查冲突 |
| `enable_autostart` | - | `Result<(), String>` | 启用自启动 |
| `disable_autostart` | - | `Result<(), String>` | 禁用自启动 |
| `is_autostart_enabled` | - | `Result<bool, String>` | 查询状态 |

### 前端 Services

| Service | 方法 | 说明 |
|---------|------|------|
| `storeService` | `save()`, `load()`, `delete()`, `clear()` | 持久化存储 |
| `startupService` | `enable()`, `disable()`, `isEnabled()` | 开机自启动 |
| `searchHistory` | `add()`, `clear()`, `getRecent()` | 搜索历史 |

---

## 验收标准

详见 [acceptance.md](acceptance.md)

### 验收概览

| 阶段 | 验收项数 | 通过标准 | 状态 |
|------|----------|----------|------|
| ECO-01 持久化存储 | 4 | 4/4 | ❌ |
| ECO-02 快捷键自定义 | 5 | 5/5 | ❌ |
| ECO-03 开机自启动 | 3 | 3/3 | ❌ |
| ECO-04 搜索历史 | 4 | 4/4 | ❌ |
| ECO-05 分类搜索 | 4 | 4/4 | ❌ |
| ECO-06 结果高亮 | 3 | 3/3 | ❌ |
| **总计** | **23** | **23/23** | **0%** |

---

## 变更记录

| 版本 | 时间 | 变更内容 | 变更人 |
|------|------|----------|--------|
| v1.0 | 2026-04-03 | 初稿创建 | Corelia Team |

---

**最后更新**：2026-04-03

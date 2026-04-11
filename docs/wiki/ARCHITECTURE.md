# Corelia 系统架构文档

## 1. 概述

Corelia 是一个基于 Tauri v2 + Svelte 5 + Rust 的高性能快速启动器应用，采用前后端分离的架构设计。

### 1.1 技术栈

| 层级 | 技术 | 版本 | 说明 |
|------|------|------|------|
| 前端框架 | SvelteKit | 2.x | 响应式 UI 框架 |
| UI 语言 | TypeScript | ~5.6.2 | 类型安全 |
| 打包工具 | Vite | 6.x | 快速构建 |
| 包管理器 | Bun | 1.3+ | 快速依赖管理 |
| 桌面框架 | Tauri | 2.x | 桌面应用框架 |
| 后端语言 | Rust | 1.94.0 | 系统级编程 |
| JS 引擎 | rquickjs | 0.11.0 | JavaScript 运行时 |

### 1.2 设计原则

- **分层架构**: Commands 层 → Services 层 → 核心逻辑
- **类型安全**: TypeScript + Rust 双重类型检查
- **状态同步**: 全局状态管理，确保前后端一致
- **性能优先**: Rust 后端 + 前端懒加载
- **可维护性**: 模块化设计，清晰的职责划分

## 2. 整体架构

```
┌─────────────────────────────────────────────────────┐
│                  用户界面层                          │
│  ┌─────────────────────────────────────────────┐   │
│  │           Svelte 5 Components               │   │
│  │  (SearchBox, ResultList, CategoryTabs...)   │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│                  状态管理层                          │
│  ┌──────────────┐  ┌──────────────┐                │
│  │   Stores     │  │   Services   │                │
│  │ (system,user)│  │(executor,...)│                │
│  └──────────────┘  └──────────────┘                │
└─────────────────┬───────────────────────────────────┘
                  │ Tauri Commands (invoke)
┌─────────────────▼───────────────────────────────────┐
│               命令层 (Commands)                      │
│  ┌──────┬──────┬──────┬──────┬──────┬──────┐      │
│  │Window│Config│Store │Shell │Auto  │Short │      │
│  └──────┴──────┴──────┴──────┴──────┴──────┘      │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│               服务层 (Services)                      │
│  ┌──────┬──────┬──────┬──────┬──────┬──────┐      │
│  │Window│Config│Store │Shell │Auto  │Clip  │      │
│  └────────────┴────────────┴────────────┘      │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│               插件层 (Plugins)                       │
│  ┌──────────┬──────────┬──────────┬──────────┐    │
│  │ Opener   │ Shortcut │  Store   │AutoStart │    │
│  └──────────┴──────────┴──────────┴──────────┘    │
└─────────────────────────────────────────────────────┘
```

## 3. 前端架构

### 3.1 目录结构

```
src/
├── routes/
│   └── +page.svelte          # 主页面
├── lib/
│   ├── components/           # UI 组件
│   │   ├── SearchBox.svelte
│   │   ├── ResultList.svelte
│   │   ├── CategoryTabs.svelte
│   │   ├── TitleBar.svelte
│   │   ├── SettingPanel.svelte
│   │   └── ShortcutRecorder.svelte
│   ├── stores/              # 状态管理
│   │   ├── system.ts        # 系统配置
│   │   ├── user.ts          # 用户配置
│   │   ├── theme.ts         # 主题
│   │   ├── search.ts        # 搜索状态
│   │   └── history.ts       # 搜索历史
│   ├── services/            # 前端服务
│   │   ├── executor.ts      # 执行器
│   │   ├── shortcut.ts      # 快捷键
│   │   ├── shell.ts         # Shell 操作
│   │   └── clipboard.ts     # 剪贴板
│   ├── search/              # 搜索相关
│   │   ├── fuzzy.ts         # 模糊搜索
│   │   └── performance.ts   # 性能优化
│   ├── styles/              # 样式
│   │   └── themes.css
│   ├── config.ts            # 全局配置
│   └── utils/               # 工具函数
└── app.html                 # HTML 模板
```

### 3.2 组件层级

```
+page.svelte (主页面)
├── TitleBar                  # 标题栏
│   └── 拖拽区域 + 设置按钮
├── SearchBox                 # 搜索框
│   └── 输入框 + 清除按钮
├── CategoryTabs              # 分类标签
│   └── 全部/系统/插件/历史
├── ResultList                # 结果列表
│   ├── 历史记录模式
│   └── 搜索结果模式
└── SettingPanel              # 设置面板
    └── ShortcutRecorder      # 快捷键录制组件
```

### 3.3 状态管理

#### Stores 层次结构

```typescript
// 系统配置 (只读)
system: {
  shortcut: { summon: string, enabled: boolean },
  startup: { minimizeToTray: boolean }
}

// 用户配置 (可写)
user: {
  behavior: { autoHide: boolean, autoHideDelay: number },
  theme: 'dark' | 'light' | 'system'
}

// 搜索状态
searchStore: {
  query: string,
  results: FilterResult[],
  category: 'all' | 'system' | 'plugin' | 'history'
}

// 搜索历史
searchHistory: {
  items: Array<{ query: string, timestamp: number }>
}
```

#### 状态同步机制

```typescript
// 前端订阅配置变化
unsubSystem = system.subscribe((s) => {
  systemConfigSnapshot = { ...s.shortcut, ...s.startup };
});

// 后端更新配置
await invoke('save_system_config', { config: newConfig });

// 前端重新加载
await system.load();
```

### 3.4 搜索流程

```
用户输入
    ↓
SearchBox.onInput()
    ↓
searchStore.setQuery(query)
    ↓
fuzzySearch(query, items)
    ↓
更新 searchStore.results
    ↓
ResultList 渲染列表
    ↓
用户选择
    ↓
executeItem(item)
    ↓
invoke('open_app' | 'open_url' | ...)
    ↓
invoke('hide_window')
```

## 4. 后端架构

### 4.1 目录结构

```
src-tauri/src/
├── main.rs                  # Rust 入口 (仅调用 lib.rs)
├── lib.rs                   # 应用入口 (组装模块)
├── error.rs                 # 错误处理
├── commands/                # 命令层
│   ├── mod.rs
│   ├── window.rs           # 窗口控制命令
│   ├── config/             # 配置管理命令
│   │   ├── mod.rs
│   │   ├── system.rs
│   │   ├── user.rs
│   │   └── app.rs
│   ├── store.rs            # 数据存储命令
│   ├── shell.rs            # Shell 操作命令
│   ├── autostart.rs        # 自启动命令
│   └── shortcut.rs         # 快捷键命令
└── services/                # 服务层
    ├── mod.rs
    ├── window_service.rs   # 窗口服务
    ├── config_service.rs   # 配置服务
    ├── store_service.rs    # 存储服务
    ├── shell_service.rs    # Shell 服务
    ├── autostart_service.rs # 自启动服务
    └── clipboard_service.rs # 剪贴板服务
```

### 4.2 分层设计

#### Commands 层 (薄封装)

```rust
// src-tauri/src/commands/window.rs
#[tauri::command]
pub async fn toggle_window(app: AppHandle) -> Result<bool, String> {
    // 仅调用服务层，无业务逻辑
    WindowService::toggle(&app)
}

#[tauri::command]
pub async fn hide_window(app: AppHandle) -> Result<(), String> {
    WindowService::hide(&app)
}
```

**职责**:
- 参数验证
- 错误转换
- 命令封装

#### Services 层 (业务逻辑)

```rust
// src-tauri/src/services/window_service.rs
pub struct WindowService;

impl WindowService {
    // 全局状态
    static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);
    
    // 业务逻辑
    pub fn toggle(app: &AppHandle) -> Result<bool, String> {
        let visible = Self::is_window_visible();
        let new_visible = !visible;
        
        if visible {
            Self::hide(app)?;
        } else {
            Self::show(app)?;
        }
        
        Ok(new_visible)
    }
    
    fn show(app: &AppHandle) -> Result<(), String> {
        // 显示窗口逻辑
    }
    
    fn hide(app: &AppHandle) -> Result<(), String> {
        // 隐藏窗口逻辑
    }
}
```

**职责**:
- 业务逻辑实现
- 状态管理
- 错误处理

### 4.3 全局状态管理

#### 窗口可见状态

```rust
use std::sync::atomic::{AtomicBool, Ordering};

// 线程安全的全局状态
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);

// 读取状态
fn is_window_visible() -> bool {
    WINDOW_VISIBLE.load(Ordering::SeqCst)
}

// 写入状态
fn set_window_visible(visible: bool) {
    WINDOW_VISIBLE.store(visible, Ordering::SeqCst);
}
```

**为什么使用 AtomicBool?**
- 线程安全，无需 Mutex
- 高性能，无锁操作
- 适合简单的布尔状态

#### 状态同步流程

```
托盘点击 → toggle() → WINDOW_VISIBLE 取反
                          ↓
                    show() / hide()
                          ↓
                    更新窗口实际状态
                          ↓
                    前端焦点监听
                          ↓
                    失焦自动隐藏
```

### 4.4 配置管理

#### 三层配置结构

```rust
// SystemConfig - 系统级配置
struct SystemConfig {
    shortcut: ShortcutConfig,
    startup: StartupConfig,
}

// UserConfig - 用户偏好
struct UserConfig {
    behavior: BehaviorConfig,
    theme: ThemeConfig,
}

// AppConfig - 应用数据
struct AppConfig {
    // 存储内容
}
```

#### 存储路径

```
$APPDATA/
└── com.morningstart.corelia/
    ├── system.json         # 系统配置
    ├── user.json           # 用户配置
    └── store.json          # 应用数据
```

## 5. 数据流

### 5.1 配置加载流程

```
应用启动
    ↓
lib.rs:setup()
    ↓
WindowService::init_state()
    ↓
前端 onMount()
    ↓
Promise.all([system.load(), user.load()])
    ↓
invoke('load_system_config')
    ↓
ConfigService::load_system()
    ↓
读取 system.json
    ↓
返回前端
    ↓
更新 system store
    ↓
订阅者收到通知
    ↓
UI 更新
```

### 5.2 搜索执行流程

```
用户输入 "calc"
    ↓
SearchBox → handleInput()
    ↓
searchStore.setQuery("calc")
    ↓
fuzzySearch("calc", executables)
    ↓
返回匹配结果
    ↓
ResultList 渲染
    ↓
用户点击 "计算器"
    ↓
executeItem(item)
    ↓
invoke('open_app', { app: "calc" })
    ↓
ShellService::open_app()
    ↓
std::process::Command::new("calc")
    ↓
invoke('hide_window')
    ↓
WindowService::hide()
    ↓
窗口隐藏
```

## 6. 关键特性实现

### 6.1 DPI 适配

**问题**: 高 DPI 屏幕上窗口尺寸不一致

**解决方案**: 使用 LogicalSize

```typescript
// 前端设置窗口尺寸
import { LogicalSize } from "@tauri-apps/api/dpi";

appWindow.setSize(new LogicalSize(
  WINDOW_CONFIG.WIDTH,   // 600 逻辑像素
  WINDOW_CONFIG.HEIGHT   // 420 逻辑像素
));
```

**效果**:
- 100% DPI: 600×420 物理像素
- 200% DPI: 1200×840 物理像素
- 显示大小一致，清晰度更高

### 6.2 透明窗口与圆角

**配置**:
```json
{
  "app": {
    "windows": [{
      "decorations": false,
      "transparent": true,
      "alwaysOnTop": true
    }]
  }
}
```

**CSS**:
```css
.window-container {
  width: 100%;
  height: 100%;
  background: #1e1e24;
  border-radius: 14px;
}

html, body {
  background: transparent;
}
```

**层级**:
```
系统背景 (透明)
  └─ Tauri 窗口 (圆角透明)
      └─ HTML/Body (透明)
          └─ .window-container (圆角深色)
```

### 6.3 失焦自动隐藏

**前端监听**:
```typescript
const unlistenFocus = appWindow.onFocusChanged(
  async ({ payload: focused }) => {
    if (!focused && userConfigSnapshot.autoHide) {
      await invoke('hide_window');
    }
  }
);
```

**后端处理**:
```rust
pub fn hide(app: &AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main")?;
    window.hide()?;
    Self::set_window_visible(false);
    Ok(())
}
```

### 6.4 全局快捷键

**注册流程**:
```rust
// src-tauri/src/commands/shortcut.rs
#[tauri::command]
pub fn register_shortcut_cmd(app: AppHandle) -> Result<(), String> {
    let shortcut = "Alt+Space"; // 从配置读取
    
    ShortcutManager::register(&app, shortcut, || {
        WindowService::toggle(&app)?;
        Ok(())
    })
}
```

**前端调用**:
```typescript
await invoke("register_shortcut_cmd");
```

## 7. 性能优化

### 7.1 前端优化

- **懒加载**: 组件按需加载
- **虚拟滚动**: 大量结果时启用
- **防抖搜索**: 150ms 延迟，减少计算
- **缓存结果**: 避免重复搜索

### 7.2 后端优化

- **线程池**: 异步处理耗时操作
- **缓存配置**: 避免重复读取文件
- **原子操作**: AtomicBool 替代 Mutex
- **延迟取消置顶**: 100ms 后取消置顶

### 7.3 启动优化

```
目标：热启动 < 0.5 秒

优化措施:
- Rust 后端预编译
- 前端代码分割
- 最小化初始加载
- 懒加载非关键组件
```

## 8. 安全考虑

### 8.1 权限控制

```json
{
  "identifier": "main-capability",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "global-shortcut:default"
  ]
}
```

### 8.2 输入验证

```rust
// 命令参数验证
pub fn open_app(app: String) -> Result<(), String> {
    // 验证应用名是否合法
    if app.contains("..") || app.contains("/") {
        return Err("Invalid app name".to_string());
    }
    // ...
}
```

### 8.3 错误处理

```rust
// 统一错误类型
pub enum AppError {
    ConfigNotFound,
    InvalidParameter(String),
    InternalError(String),
}

// 所有命令返回 Result<T, String>
#[tauri::command]
pub fn some_command() -> Result<String, String> {
    // 错误处理
}
```

## 9. 测试策略

### 9.1 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_window_service_toggle() {
        // 测试窗口切换逻辑
    }
}
```

### 9.2 集成测试

```typescript
// 测试配置加载
describe('ConfigService', () => {
  it('should load system config', async () => {
    const config = await system.load();
    expect(config).toBeDefined();
  });
});
```

### 9.3 E2E 测试

```typescript
// 测试完整搜索流程
test('search and open app', async () => {
  await page.fill('input', 'calc');
  await page.click('.result-item');
  // 验证计算器是否打开
});
```

## 10. 扩展性设计

### 10.1 插件系统

```rust
// 未来扩展：插件 API
pub trait Plugin {
    fn name(&self) -> &str;
    fn execute(&self, args: &[String]) -> Result<(), String>;
}

// 插件注册
pub fn register_plugin(plugin: Box<dyn Plugin>) {
    // ...
}
```

### 10.2 自定义命令

```typescript
// 未来扩展：用户自定义命令
interface CustomCommand {
  name: string;
  script: string;
  shortcut?: string;
}
```

## 11. 总结

Corelia 采用现代化的技术栈和清晰的架构设计：

- **前后端分离**: 清晰的职责划分
- **分层架构**: Commands → Services → Core
- **类型安全**: TypeScript + Rust
- **性能优先**: Rust 后端 + 前端优化
- **可维护性**: 模块化、可扩展

这种架构确保了应用的高性能、可维护性和可扩展性。

---

**最后更新**: 2026-04-10  
**版本**: v0.1.0

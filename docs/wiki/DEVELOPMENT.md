# 开发指南

> Corelia 开发环境搭建、代码规范和常用操作指南。

## 环境要求

| 工具 | 最低版本 | 安装方式 |
|------|----------|----------|
| Bun | 1.3.0 | https://bun.sh |
| Rust | 1.94.0 | https://rustup.rs |
| wasm-pack | latest | `cargo install wasm-pack` |
| Tauri CLI | 2.x | `cargo install tauri-cli` |

## 快速开始

```bash
# 安装前端依赖
bun install

# 开发模式（Rust 热编译 + 前端 HMR）
bun run tauri dev

# 类型检查
bun run check

# 生产构建
bun run tauri build
```

## 开发工作流

### 标准流程

```
Plan → Spec → Implement → Verify
```

1. **Plan**: 分析需求、评估方案、制定任务清单
2. **Spec**: 编写规格说明书（`docs/spec/` 目录）
3. **Implement**: 按 SPEC 实现，遵循代码规范
4. **Verify**: 类型检查、测试、验收

### 开发模式说明

- **Rust 修改**：需要重新编译，启动较慢（首次约 1-3 分钟）
- **前端修改**：支持 HMR，修改后即时生效
- **WASM Patch**：需要 `wasm-pack build` 重新编译

## 代码规范

### Rust 规范

```rust
// ✅ 正确：使用 OnceLock/Mutex 替代 static mut
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

// ❌ 禁止：使用 static mut
// static mut HANDLE: Option<AppHandle> = None;

// ✅ 正确：命令使用下划线命名
#[tauri::command]
pub fn read_clipboard() -> Result<String, String> { ... }

// ✅ 正确：错误返回 Result，禁止 panic
fn load_config() -> Result<Config, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

// ✅ 正确：Tauri 插件导入
use tauri::Manager;
use tauri_plugin_store::StoreExt;
```

### TypeScript 规范

```typescript
// ✅ 正确：类型导入使用 type 关键字
import type { PluginManifest } from './types';
import { type WasmFunctionInfo } from './types';

// ❌ 禁止：使用 any
// const data: any = {};

// ✅ 正确：明确类型声明
const data: PluginManifest = { ... };
```

### Svelte 5 规范

```svelte
<script lang="ts">
  // ✅ 使用 Svelte 5 Runes
  let count = $state(0);
  let doubled = $derived(count * 2);
  let { name } = $props();

  // ❌ 不使用旧语法
  // let count = writable(0);  // 不用 Svelte 4 stores
</script>
```

### CSS 规范

```css
/* ✅ 使用 CSS 变量实现主题 */
:root {
  --bg-primary: #1a1a2e;
  --text-primary: #e0e0e0;
}

/* ✅ 透明窗口配置 */
body {
  background: transparent;
}
```

## 包管理

**前端**：使用 Bun（禁止 npm/yarn/pnpm）

```bash
bun add package-name         # 添加依赖
bun add -d package-name      # 添加开发依赖
bun remove package-name      # 移除依赖
```

**Rust**：使用 Cargo

```bash
cargo add crate-name         # 添加依赖
cargo add crate-name --features "feature1"  # 带 feature
```

## 关键配置文件

| 文件 | 用途 |
|------|------|
| `src-tauri/tauri.conf.json` | Tauri 主配置（窗口/权限/打包） |
| `src-tauri/capabilities/default.json` | Tauri 权限声明 |
| `src-tauri/Cargo.toml` | Rust 依赖 |
| `package.json` | 前端依赖 |
| `vite.config.js` | Vite 构建配置 |
| `svelte.config.js` | Svelte 配置 |
| `tsconfig.json` | TypeScript 配置 |

## 调试技巧

### Rust 调试

```bash
# 启用 debug 日志
RUST_LOG=debug bun run tauri dev

# 仅查看特定模块日志
RUST_LOG=corelia::plugins=debug bun run tauri dev
```

### 前端调试

- 使用浏览器 DevTools（右键 → 检查元素）
- `console.log` 输出可在 DevTools Console 查看

### 类型检查

```bash
# 前端类型检查
bun run check

# Rust 编译检查
cd src-tauri && cargo check
```

## 常见问题

### 1. 端口占用

修改 `vite.config.js` 中的端口，或结束占用进程：

```bash
# Windows
netstat -ano | findstr :1420
taskkill /PID <pid> /F
```

### 2. 快捷键重复注册

在 `setup` 中先 `unregister_all()`：

```rust
app.plugin(tauri_plugin_global_shortcut::Builder::new().build());
// setup 中
let _ = globalShortcut.unregister_all();
```

### 3. Rust 编译警告

添加 `#![allow(static_mut_refs)]` 到 `lib.rs` 顶部。

### 4. Svelte onMount

不要在 `onMount` 中使用 async cleanup：

```svelte
<!-- ❌ 错误 -->
<script>
  onMount(async () => {
    return () => cleanup(); // async 函数返回 cleanup 无效
  });
</script>

<!-- ✅ 正确 -->
<script>
  onMount(() => {
    loadData();
    return () => cleanup();
  });
</script>
```

### 5. WASM 编译问题

使用 `rquickjs` 而非 `quickjs-rs`（后者有编译问题）。

### 6. 透明窗口焦点

透明窗口需手动处理 blur 事件隐藏：

```typescript
appWindow.onFocusChanged(async ({ payload: focused }) => {
  if (!focused && autoHide) {
    await invoke('hide_window');
  }
});
```

### 7. 快捷键选择

避免 `Alt+Space`（系统保留），推荐 `Ctrl+Space`。

## 构建与清理

```bash
# 生产构建
bun run check && cargo check --release && bun run tauri build

# 清理缓存
rm -rf node_modules/.vite
cargo clean

# 重新安装依赖
rm -rf node_modules bun.lock
bun install
```

## 插件开发

详见 [PLUGIN_SYSTEM.md](./PLUGIN_SYSTEM.md)

### WASM Patch 开发

```bash
# 创建 patch 项目
cargo new --lib patches/my-patch

# 编写 Rust 代码后构建
cd patches/my-patch
wasm-pack build --target web

# 产物在 pkg/ 目录
```

### 插件测试

```bash
# 1. 将插件放入 plugins/ 目录
# 2. 启动开发模式
bun run tauri dev
# 3. 在搜索框输入插件前缀触发
```

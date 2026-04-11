# Corelia 开发指南

## 1. 开发环境搭建

### 1.1 环境要求

**必需工具**:
- **Bun**: >= 1.3.0 (包管理器)
- **Rust**: >= 1.94.0 (后端编译)
- **Node.js**: >= 18.0.0 (前端依赖)

**可选工具**:
- **wasm-pack**: WASM 编译工具
- **cargo-audit**: Rust 安全审计

### 1.2 安装步骤

#### 安装 Bun

```bash
# Windows (PowerShell)
powershell -c "irm bun.sh/install.ps1 | iex"

# macOS/Linux
curl -fsSL https://bun.sh/install | bash

# 验证安装
bun --version
```

#### 安装 Rust

```bash
# 访问 https://rustup.rs 下载安装
# 或使用 rustup

# Windows (PowerShell)
winget install Rustlang.Rustup

# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
rustc --version
cargo --version
```

#### 安装 wasm-pack (可选)

```bash
# 使用 cargo 安装
cargo install wasm-pack

# 验证
wasm-pack --version
```

### 1.3 克隆项目

```bash
git clone https://github.com/your-org/corelia.git
cd corelia

# 安装前端依赖
bun install

# 验证 Rust 依赖
cd src-tauri
cargo check
```

## 2. 开发工作流

### 2.1 启动开发服务器

```bash
# 根目录执行
bun run tauri dev
```

**说明**:
- 首次启动会编译 Rust 后端 (较慢)
- 前端支持 HMR (热模块替换)
- Rust 修改需重新编译

### 2.2 调试技巧

#### 前端调试

```typescript
// 使用 console.log
console.log('Debug:', data);

// 使用浏览器 DevTools
// F12 打开开发者工具
```

#### 后端调试

```rust
// 使用 println! 调试
println!("Debug: {:?}", value);

// 使用 RUST_LOG 环境变量
RUST_LOG=debug bun run tauri dev
```

#### 查看日志

```bash
# 查看应用日志
# Windows: %APPDATA%\com.morningstart.corelia\logs
# macOS: ~/Library/Logs/com.morningstart.corelia
# Linux: ~/.local/share/com.morningstart.corelia/logs
```

### 2.3 代码格式化

```bash
# 前端代码格式化
bun run format

# Rust 代码格式化
cd src-tauri
cargo fmt

# 检查代码格式
cargo fmt -- --check
```

### 2.4 类型检查

```bash
# TypeScript 类型检查
bun run check

# Rust 类型检查
cd src-tauri
cargo check
```

## 3. 添加新功能

### 3.1 添加 Tauri 命令

#### 步骤 1: 创建服务层

```rust
// src-tauri/src/services/my_service.rs
pub struct MyService;

impl MyService {
    pub fn do_something(app: &AppHandle, param: String) -> Result<String, String> {
        // 业务逻辑
        Ok(format!("Processed: {}", param))
    }
}
```

#### 步骤 2: 创建命令层

```rust
// src-tauri/src/commands/my_command.rs
use tauri::AppHandle;
use crate::services::MyService;

#[tauri::command]
pub async fn do_something(app: AppHandle, param: String) -> Result<String, String> {
    MyService::do_something(&app, param)
}
```

#### 步骤 3: 导出命令

```rust
// src-tauri/src/commands/mod.rs
pub mod my_command;
pub use my_command::do_something;
```

#### 步骤 4: 注册命令

```rust
// src-tauri/src/lib.rs
use commands::do_something;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            do_something,
            // ... 其他命令
        ])
}
```

#### 步骤 5: 前端调用

```typescript
// src/lib/services/myService.ts
import { invoke } from '@tauri-apps/api/core';

export async function doSomething(param: string): Promise<string> {
  return await invoke('do_something', { param });
}

// 使用
const result = await doSomething('test');
console.log(result); // "Processed: test"
```

### 3.2 添加 Svelte 组件

#### 步骤 1: 创建组件

```svelte
<!-- src/lib/components/MyComponent.svelte -->
<script lang="ts">
  interface Props {
    title?: string;
    onClick?: () => void;
  }
  
  let { title = '默认标题', onClick }: Props = $props();
  let count = $state(0);
  
  function handleClick() {
    count++;
    onClick?.();
  }
</script>

<div class="my-component">
  <h3>{title}</h3>
  <p>计数：{count}</p>
  <button onclick={handleClick}>点击</button>
</div>

<style>
  .my-component {
    padding: 16px;
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
  }
  
  button {
    padding: 8px 16px;
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  
  button:hover {
    background: var(--color-primary-dark);
  }
</style>
```

#### 步骤 2: 使用组件

```svelte
<!-- src/lib/components/Parent.svelte -->
<script lang="ts">
  import MyComponent from './MyComponent.svelte';
  
  function handleChildClick() {
    console.log('子组件被点击');
  }
</script>

<MyComponent 
  title="我的组件" 
  onClick={handleChildClick} 
/>
```

### 3.3 添加 Store

#### 步骤 1: 创建 Store

```typescript
// src/lib/stores/myStore.ts
import { invoke } from '@tauri-apps/api/core';

class MyStore {
  private state = $state({
    items: [] as string[],
    loading: false,
  });
  
  /**
   * 加载数据
   */
  async load() {
    this.state.loading = true;
    try {
      const items = await invoke('get_items');
      this.state.items = items;
    } catch (error) {
      console.error('加载失败:', error);
    } finally {
      this.state.loading = false;
    }
  }
  
  /**
   * 添加项目
   */
  addItem(item: string) {
    this.state.items.push(item);
  }
  
  /**
   * 订阅状态变化
   */
  subscribe(runner: (state: typeof this.state) => void) {
    runner(this.state);
    return () => {}; // 清理函数
  }
  
  /**
   * 获取状态
   */
  getState() {
    return this.state;
  }
}

// 导出单例
export const myStore = new MyStore();
```

#### 步骤 2: 使用 Store

```svelte
<script lang="ts">
  import { myStore } from '$lib/stores/myStore';
  import { onMount } from 'svelte';
  
  let unsub: (() => void) | undefined;
  
  onMount(() => {
    // 加载数据
    myStore.load();
    
    // 订阅状态变化
    unsub = myStore.subscribe((state) => {
      console.log('状态变化:', state);
    });
    
    return () => unsub?.();
  });
</script>

{#if myStore.getState().loading}
  <div>加载中...</div>
{:else}
  <ul>
    {#each myStore.getState().items as item}
      <li>{item}</li>
    {/each}
  </ul>
{/if}
```

### 3.4 添加配置项

#### 步骤 1: 修改配置文件

```typescript
// src/lib/config.ts
export const MY_CONFIG = {
  /** 新配置项 */
  NEW_FEATURE_ENABLED: true,
  /** 配置说明 */
  MAX_ITEMS: 100,
} as const;
```

#### 步骤 2: 使用配置

```typescript
import { MY_CONFIG } from '$lib/config';

if (MY_CONFIG.NEW_FEATURE_ENABLED) {
  // 使用新功能
}

const maxItems = MY_CONFIG.MAX_ITEMS;
```

#### 步骤 3: (可选) 添加到用户配置

```rust
// src-tauri/src/services/config_service.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    // 现有字段...
    pub new_feature_enabled: bool,
}
```

## 4. 修改现有功能

### 4.1 修改窗口尺寸

```typescript
// src/lib/config.ts
export const WINDOW_CONFIG = {
  WIDTH: 600,      // 修改宽度
  HEIGHT: 420,     // 修改高度
  MIN_WIDTH: 200,
  MIN_HEIGHT: 300,
  MAX_WIDTH: 1200,
  MAX_HEIGHT: 900,
} as const;
```

**重启开发服务器生效**。

### 4.2 修改快捷键

#### 方法 1: 通过 UI 修改

1. 点击右上角设置按钮
2. 在快捷键录制框中按下新快捷键
3. 保存配置

#### 方法 2: 直接修改配置文件

```json
// $APPDATA/com.morningstart.corelia/system.json
{
  "shortcut": {
    "summon": "Ctrl+Space",
    "enabled": true
  }
}
```

### 4.3 修改主题颜色

```css
/* src/lib/styles/themes.css */
:root {
  /* 修改主色调 */
  --color-primary: #6366f1;
  --color-primary-light: #818cf8;
  --color-primary-dark: #4f46e5;
  
  /* 修改背景色 */
  --bg-primary: #1e1e24;
  --bg-secondary: #25252c;
  
  /* 修改文字颜色 */
  --text-primary: #ffffff;
  --text-secondary: rgba(255, 255, 255, 0.75);
}
```

## 5. 调试技巧

### 5.1 前端调试

#### 使用浏览器 DevTools

```typescript
// 断点调试
debugger;

// 打印日志
console.log('Data:', data);
console.table(items);

// 性能分析
console.time('search');
// ... 搜索逻辑
console.timeEnd('search');
```

#### 检查组件状态

```svelte
<script>
  import { onMount } from 'svelte';
  
  onMount(() => {
    // 在控制台查看组件实例
    console.log('Component mounted:', $props());
  });
</script>
```

### 5.2 后端调试

#### 使用 println! 调试

```rust
#[tauri::command]
pub fn my_command(param: String) -> Result<String, String> {
    println!("收到参数：{}", param);
    
    let result = process(&param);
    println!("处理结果：{:?}", result);
    
    Ok(result)
}
```

#### 使用日志库

```rust
// Cargo.toml
[dependencies]
log = "0.4"
env_logger = "0.10"

// lib.rs
use log::{info, debug, error};

fn setup() {
    env_logger::init();
    info!("应用启动");
}

#[tauri::command]
fn my_command() {
    debug!("执行命令");
    
    match do_something() {
        Ok(result) => info!("成功：{:?}", result),
        Err(e) => error!("失败：{}", e),
    }
}
```

#### 启用调试日志

```bash
# 设置环境变量
RUST_LOG=debug bun run tauri dev

# 或特定模块
RUST_LOG=corelia::services=debug bun run tauri dev
```

### 5.3 性能调试

#### 前端性能

```typescript
// 测量函数执行时间
function measureTime(fn: () => void, label: string) {
  const start = performance.now();
  fn();
  const end = performance.now();
  console.log(`${label}: ${(end - start).toFixed(2)}ms`);
}

// 使用
measureTime(() => fuzzySearch(query, items), '搜索耗时');
```

#### 后端性能

```rust
use std::time::Instant;

#[tauri::command]
pub fn slow_command() -> Result<(), String> {
    let start = Instant::now();
    
    // 耗时操作
    do_something_slow();
    
    let duration = start.elapsed();
    println!("耗时：{:?}", duration);
    
    Ok(())
}
```

## 6. 常见问题

### Q1: 开发服务器启动失败

**问题**: `Port 5173 is in use`

**解决**:
```bash
# 方法 1: 结束占用端口的进程
# Windows
netstat -ano | findstr :5173
taskkill /PID <PID> /F

# macOS/Linux
lsof -i :5173
kill -9 <PID>

# 方法 2: 修改 vite.config.js
export default {
  server: {
    port: 5174  // 使用其他端口
  }
}
```

### Q2: Rust 编译错误

**问题**: `error[E0433]: failed to resolve`

**解决**:
```bash
# 清理构建缓存
cd src-tauri
cargo clean

# 重新编译
cargo build
```

### Q3: TypeScript 类型错误

**问题**: `Cannot find module 'xxx'`

**解决**:
```bash
# 重新安装依赖
rm -rf node_modules
bun install

# 重新生成类型
bun run svelte-kit sync
```

### Q4: 窗口不显示

**问题**: 应用启动但窗口不可见

**解决**:
1. 检查 `tauri.conf.json` 中 `visible` 配置
2. 检查窗口位置是否超出屏幕
3. 查看控制台错误信息
4. 尝试重启应用

### Q5: 快捷键不生效

**问题**: 按下快捷键无反应

**解决**:
1. 检查快捷键是否被其他应用占用
2. 查看 `system.json` 配置是否正确
3. 重新注册快捷键
4. 重启应用

## 7. 最佳实践

### 7.1 代码规范

#### TypeScript

```typescript
// ✅ 好的做法
import type { FilterResult } from 'fuzzy';
import { WINDOW_CONFIG } from '$lib/config';

interface Props {
  title: string;
  onClick?: () => void;
}

const myFunction = (param: string): string => {
  return param.toUpperCase();
};

// ❌ 避免
const x: any = getData();  // 使用 any
```

#### Rust

```rust
// ✅ 好的做法
#[tauri::command]
pub fn my_command(app: AppHandle) -> Result<String, String> {
    // 错误处理
    let result = do_something()?;
    Ok(result)
}

// ❌ 避免
panic!("错误!");  // 使用 Result 而非 panic
```

### 7.2 错误处理

#### 前端

```typescript
try {
  const result = await invoke('my_command');
  console.log('成功:', result);
} catch (error) {
  console.error('失败:', error);
  // 显示错误提示
}
```

#### 后端

```rust
#[tauri::command]
pub fn my_command() -> Result<String, String> {
    do_something()
        .map_err(|e| format!("处理失败：{}", e))
}
```

### 7.3 性能优化

```typescript
// 防抖搜索
const debouncedSearch = debounce((query: string) => {
  searchStore.setQuery(query);
}, 150);

// 懒加载组件
const LazyComponent = lazy(() => import('./LazyComponent.svelte'));
```

### 7.4 测试

```typescript
// 单元测试
describe('myFunction', () => {
  it('should work correctly', () => {
    expect(myFunction('input')).toBe('INPUT');
  });
});

// 集成测试
test('search and open app', async () => {
  // 测试完整流程
});
```

## 8. 发布构建

### 8.1 生产构建

```bash
# 类型检查
bun run check

# Rust 检查
cd src-tauri
cargo check --release

# 生产构建
cd ..
bun run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`

### 8.2 代码签名 (可选)

```rust
// tauri.conf.json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "你的证书指纹",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

### 8.3 打包发布

```bash
# 清理旧构建
cargo clean
rm -rf src-tauri/target/release

# 重新构建
bun run tauri build

# 测试安装包
# Windows: 运行生成的 .msi 或 .exe
# macOS: 运行生成的 .app 或 .dmg
# Linux: 运行生成的 .deb 或 .AppImage
```

## 9. 资源

### 9.1 官方文档

- [Tauri v2 文档](https://v2.tauri.app)
- [Svelte 5 文档](https://svelte.dev/docs/svelte)
- [Rust 文档](https://doc.rust-lang.org)

### 9.2 社区资源

- [Tauri GitHub](https://github.com/tauri-apps/tauri)
- [Svelte Discord](https://svelte.dev/chat)
- [Rust Forum](https://users.rust-lang.org)

### 9.3 项目文档

- [README.md](../../README.md)
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [DPI 缩放指南](../dev/dpi-scaling-guide.md)

---

**最后更新**: 2026-04-10  
**版本**: v0.1.0

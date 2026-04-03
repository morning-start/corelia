# MVP-POC 技术原型验证规格说明书

## 版本信息

- **版本**：v1.0
- **作者**：Corelia Team
- **创建时间**：2026-04-03
- **最后更新**：2026-04-03
- **状态**：草稿

---

## 目录

- [概要](#概要)
- [Tauri 窗口验证](#tauri-窗口验证)
- [全局快捷键验证](#全局快捷键验证)
- [QuickJS 集成验证](#quickjs-集成验证)
- [WASM 环境验证](#wasm-环境验证)
- [搜索性能验证](#搜索性能验证)
- [集成测试计划](#集成测试计划)
- [验收标准](#验收标准)
- [变更记录](#变更记录)

---

## 概要

### 背景

MVP 开发前需要验证核心技术的可行性。Tauri 2.x、QuickJS、WASM 等技术需要确认能够满足 Corelia 的性能要求和功能需求。

### 目标

验证以下核心技术的可行性：

| 验证项 | 目标 | 关键指标 |
|--------|------|----------|
| Tauri 窗口 | 无边框透明窗口正常运行 | 窗口显示、置顶、响应 |
| 全局快捷键 | Alt+Space 唤起窗口 | 失去焦点也能响应 |
| QuickJS | JS 引擎正常运行 | VM 实例化成功 |
| WASM | Rust 编译为 WASM | 模块加载成功 |
| 搜索性能 | tiny-fuzzy 响应 < 50ms | 1000 条数据 |

### 方案概述

| 技术 | 选型 | 理由 |
|------|------|------|
| 桌面框架 | Tauri 2.x | 轻量、安全、原生体验 |
| 前端框架 | Svelte 5 + SvelteKit | 响应式 UI、高性能 |
| JS 引擎 | QuickJS | 轻量级插件运行时 |
| WASM 工具链 | wasm-pack + wasm-bindgen | 成熟稳定 |
| 模糊搜索 | tiny-fuzzy | 轻量、支持模糊匹配 |

---

## Tauri 窗口验证

### 验证目标

验证 Tauri 2.x 窗口在以下配置下的运行情况：

| 配置项 | 目标值 |
|--------|--------|
| 窗口类型 | 无边框窗口 |
| 背景 | 透明背景 |
| 圆角 | 12px |
| 置顶 | 窗口置顶显示 |
| 焦点 | 不抢占其他窗口焦点 |

### 技术方案

**tauri.conf.json 配置**：

```json
{
  "app": {
    "windows": [
      {
        "title": "Corelia",
        "width": 600,
        "height": 400,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "resizable": false,
        "center": true
      }
    ]
  }
}
```

**CSS 样式**（src/app.css）：

```css
:root {
  --radius: 12px;
  --bg-color: rgba(26, 26, 26, 0.95);
}

html, body {
  margin: 0;
  padding: 0;
  background: transparent;
  overflow: hidden;
}

.window-container {
  background: var(--bg-color);
  border-radius: var(--radius);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}
```

### 验证步骤

| 步骤 | 操作 | 预期结果 |
|------|------|----------|
| 1 | 运行 `bun run tauri dev` | 窗口正常显示 |
| 2 | 检查窗口无原生标题栏 | 无标题栏 |
| 3 | 检查窗口背景透明 | 背景透明 |
| 4 | 检查窗口圆角 | 12px 圆角 |
| 5 | 切换到其他应用 | 窗口仍置顶显示 |
| 6 | 在其他应用中使用快捷键 | 快捷键能唤起窗口 |

### 验证环境

| 环境 | 版本 |
|------|------|
| Windows | 10/11 x64 |
| Tauri | 2.x |
| Node.js | 18+ |
| Bun | 1.x |

---

## 全局快捷键验证

### 验证目标

验证全局快捷键 Alt+Space 在任意界面都能唤起 Corelia 窗口。

### 技术方案

**使用的插件**：`tauri-plugin-global-shortcut`

**快捷键配置**：

| 配置项 | 值 |
|--------|---|
| 默认快捷键 | Alt + Space |
| 响应条件 | 全局（不限制窗口焦点） |
| 冲突处理 | 首次使用时检测 |

**实现代码**（src-tauri/src/commands/shortcut.rs）：

```rust
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

#[tauri::command]
async fn register_shortcut(app: AppHandle) -> Result<(), String> {
    let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
    app.global_shortcut().register(shortcut, move |app| {
        let window = app.get_webview_window("main").unwrap();
        if window.is_visible().unwrap() {
            window.hide().unwrap();
        } else {
            window.show().unwrap();
            window.set_focus().unwrap();
        }
    }).map_err(|e| e.to_string())?;
    Ok(())
}
```

### 验证步骤

| 步骤 | 操作 | 预期结果 |
|------|------|----------|
| 1 | 运行应用 | 快捷键注册成功，无错误日志 |
| 2 | 按 Alt+Space | 窗口显示并获得焦点 |
| 3 | 再按 Alt+Space | 窗口隐藏 |
| 4 | 切换到其他应用（如记事本） | Alt+Space 能唤起 Corelia |
| 5 | 检查设置中可修改快捷键 | 快捷键能成功修改 |

### 边界情况

| 场景 | 处理方式 |
|------|----------|
| 快捷键被其他应用占用 | 提示用户手动修改 |
| 快捷键注册失败 | 显示错误信息 |
| 窗口已显示时再次按下 | 隐藏窗口 |

---

## QuickJS 集成验证

### 验证目标

验证 QuickJS 引擎可以正常实例化和执行 JavaScript 代码。

### 技术方案

**使用的库**：`quickjs-emscripten` 或 `rust-quickjs`

**集成架构**：

```
┌─────────────────────────────────────────────┐
│                前端 (Svelte)                 │
│                                              │
│  window.utools = {                           │
│    dbStorage: {...},                          │
│    clipboard: {...},                          │
│    shell: {...}                              │
│  }                                           │
└─────────────────────────────────────────────┘
                    │ Tauri Command
                    ▼
┌─────────────────────────────────────────────┐
│              Rust Core                       │
│                                              │
│  ┌─────────────────────────────────────┐   │
│  │         QuickJS VM                    │   │
│  │  - 执行 JS 代码                       │   │
│  │  - 提供 native 函数                   │   │
│  │  - 管理 VM 实例                       │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

**Rust 实现**（src-tauri/src/plugins/quickjs.rs）：

```rust
use quickjs_rs::{Context, Runtime};

pub struct QuickJSRunner {
    runtime: Runtime,
    context: Context,
}

impl QuickJSRunner {
    pub fn new() -> Result<Self, String> {
        let runtime = Runtime::new().map_err(|e| e.to_string())?;
        let context = runtime.context().map_err(|e| e.to_string())?;

        // 注入 window.utools API
        context.eval("
            window.utools = {
                dbStorage: {
                    setItem: (key, value) => window.__utools_db_set(key, value),
                    getItem: (key) => window.__utools_db_get(key),
                }
            };
        ").map_err(|e| e.to_string())?;

        Ok(Self { runtime, context })
    }

    pub fn execute(&self, code: &str) -> Result<String, String> {
        self.context.eval(code).map_err(|e| e.to_string())
    }
}
```

### 验证步骤

| 步骤 | 操作 | 预期结果 |
|------|------|----------|
| 1 | Rust 后端启动 | QuickJS VM 实例化成功 |
| 2 | 调用 `quickjs.execute("1 + 1")` | 返回 "2" |
| 3 | 调用 `quickjs.execute("JSON.stringify({a:1})")` | 返回 '{"a":1}' |
| 4 | 执行复杂业务逻辑 | 无错误 |

### 性能指标

| 指标 | 目标 | 说明 |
|------|------|------|
| VM 实例化 | < 10ms | 首次创建 |
| 简单表达式执行 | < 1ms | 如 1+1 |
| JSON 序列化 | < 5ms | 复杂对象 |

---

## WASM 环境验证

### 验证目标

验证 wasm-pack 可以正常编译 Rust 代码为 WASM，并在前端加载和调用。

### 技术方案

**WASM 项目结构**：

```
patches/
├── crypto/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
```

**Rust 代码**（patches/crypto/src/lib.rs）：

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn base64_encode(input: &str) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(input.as_bytes())
}

#[wasm_bindgen]
pub fn base64_decode(input: &str) -> Result<String, JsValue> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let bytes = STANDARD.decode(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    String::from_utf8(bytes).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn sha256(input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

**构建命令**：

```bash
cd patches/crypto
wasm-pack build --target web --out-dir ./pkg
```

**前端加载**（src/lib/wasm/crypto.ts）：

```typescript
import init, { base64_encode, base64_decode, sha256 } from './pkg/crypto';

let initialized = false;

export async function loadCryptoWasm(): Promise<void> {
  if (!initialized) {
    await init();
    initialized = true;
  }
}

export function encodeBase64(input: string): string {
  return base64_encode(input);
}

export function decodeBase64(input: string): string {
  return base64_decode(input);
}

export function hashSha256(input: string): string {
  return sha256(input);
}
```

### 验证步骤

| 步骤 | 操作 | 预期结果 |
|------|------|----------|
| 1 | 配置 Rust WASM 工具链 | wasm-pack 安装成功 |
| 2 | 创建 WASM lib 项目 | Cargo.toml 配置正确 |
| 3 | 编写简单 Rust 函数 | 代码无编译错误 |
| 4 | 执行 wasm-pack build | .wasm 文件生成 |
| 5 | 前端加载 WASM | 加载成功，无错误 |
| 6 | 调用 WASM 函数 | 返回正确结果 |

### 性能指标

| 指标 | 目标 | 说明 |
|------|------|------|
| 首次加载（含编译） | < 400ms | AOT 编译后 |
| 函数调用 | < 10ms | 缓存后 |
| Base64 编码 1MB | < 50ms | - |

---

## 搜索性能验证

### 验证目标

验证 tiny-fuzzy 在 1000 条数据下的搜索响应时间 < 50ms。

### 技术方案

**使用的库**：`tiny-fuzzy`

**测试数据结构**：

```typescript
interface SearchItem {
  id: string;
  name: string;
  description: string;
  category: string;
}

const items: SearchItem[] = Array.from({ length: 1000 }, (_, i) => ({
  id: `item-${i}`,
  name: `搜索项 ${i}`,
  description: `这是第 ${i} 个搜索项的描述`,
  category: ['系统', '插件', '历史'][i % 3],
}));
```

**搜索实现**：

```typescript
import tinyfuzzy from 'tiny-fuzzy';

function search(query: string, items: SearchItem[]): SearchItem[] {
  return tinyfuzzy.filter(items, query, {
    keys: ['name', 'description'],
    threshold: 0.6,
  });
}
```

### 验证步骤

| 步骤 | 操作 | 预期结果 |
|------|------|----------|
| 1 | 生成 1000 条测试数据 | 数据生成成功 |
| 2 | 执行单次搜索 | 响应时间 < 50ms |
| 3 | 执行 100 次搜索取平均值 | 平均 < 50ms |
| 4 | 测试拼音匹配 | 工作正常 |
| 5 | 测试模糊匹配 | 工作正常 |

### 性能测试代码

```typescript
async function performanceTest() {
  const iterations = 100;
  const times: number[] = [];

  for (let i = 0; i < iterations; i++) {
    const query = `搜索项 ${Math.floor(Math.random() * 1000)}`;
    const start = performance.now();
    search(query, items);
    const end = performance.now();
    times.push(end - start);
  }

  const avg = times.reduce((a, b) => a + b, 0) / times.length;
  const max = Math.max(...times);
  const min = Math.min(...times);

  console.log(`搜索性能测试结果：`);
  console.log(`- 平均: ${avg.toFixed(2)}ms`);
  console.log(`- 最大: ${max.toFixed(2)}ms`);
  console.log(`- 最小: ${min.toFixed(2)}ms`);
  console.log(`- 目标: < 50ms`);

  return avg < 50;
}
```

---

## 集成测试计划

### 测试场景

| 场景 | 测试内容 | 优先级 |
|------|----------|--------|
| 窗口显示 | 窗口正常显示和隐藏 | P0 |
| 快捷键响应 | Alt+Space 唤起/隐藏 | P0 |
| QuickJS 执行 | JS 代码正常执行 | P0 |
| WASM 调用 | WASM 函数正常调用 | P0 |
| 搜索功能 | 搜索结果正确 | P0 |

### 测试用例

#### TC-01: 窗口显示测试

| 用例 ID | TC-01 |
|---------|-------|
| 标题 | 窗口显示和隐藏 |
| 前置条件 | 应用已启动 |
| 测试步骤 | 1. 按 Alt+Space<br/>2. 检查窗口显示<br/>3. 再按 Alt+Space<br/>4. 检查窗口隐藏 |
| 预期结果 | 窗口正确显示和隐藏 |
| 优先级 | P0 |
| 结果 | PASS/FAIL |

#### TC-02: 快捷键全局响应

| 用例 ID | TC-02 |
|---------|-------|
| 标题 | 快捷键在全局范围内响应 |
| 前置条件 | 应用已启动 |
| 测试步骤 | 1. 打开记事本<br/>2. 在记事本中按 Alt+Space<br/>3. 检查 Corelia 窗口是否显示 |
| 预期结果 | Corelia 窗口正确显示 |
| 优先级 | P0 |
| 结果 | PASS/FAIL |

#### TC-03: QuickJS 执行测试

| 用例 ID | TC-03 |
|---------|-------|
| 标题 | QuickJS 正常执行 JavaScript |
| 前置条件 | QuickJS VM 已初始化 |
| 测试步骤 | 1. 调用 `quickjs.execute("1 + 1")`<br/>2. 检查返回值为 "2" |
| 预期结果 | 返回 "2" |
| 优先级 | P0 |
| 结果 | PASS/FAIL |

#### TC-04: WASM 调用测试

| 用例 ID | TC-04 |
|---------|-------|
| 标题 | WASM 函数正常调用 |
| 前置条件 | WASM 模块已加载 |
| 测试步骤 | 1. 调用 `base64_encode("hello")`<br/>2. 检查返回值为 "aGVsbG8=" |
| 预期结果 | 返回 "aGVsbG8=" |
| 优先级 | P0 |
| 结果 | PASS/FAIL |

#### TC-05: 搜索性能测试

| 用例 ID | TC-05 |
|---------|-------|
| 标题 | 搜索响应时间 < 50ms |
| 前置条件 | 1000 条测试数据已加载 |
| 测试步骤 | 1. 执行 100 次搜索<br/>2. 计算平均响应时间 |
| 预期结果 | 平均响应时间 < 50ms |
| 优先级 | P0 |
| 结果 | PASS/FAIL |

---

## 验收标准

### 验收清单

| 验收项 | 标准 | 测试方法 | 结果 |
|--------|------|----------|------|
| Tauri 窗口 | 无边框、透明、圆角、置顶 | 手动测试 | ❌ |
| 全局快捷键 | Alt+Space 全局响应 | 手动测试 | ❌ |
| QuickJS VM | 实例化成功 | 运行测试 | ❌ |
| QuickJS 执行 | 代码正常执行 | 运行测试 | ❌ |
| WASM 编译 | wasm-pack 成功编译 | 运行测试 | ❌ |
| WASM 调用 | 函数正常调用 | 运行测试 | ❌ |
| 搜索性能 | < 50ms (1000 条) | 性能测试 | ❌ |

### 通过标准

所有 P0 验收项必须通过（结果为 PASS），才能进入下一阶段。

### 风险与应对

| 风险 | 可能性 | 影响 | 应对措施 |
|------|--------|------|----------|
| Tauri 2.x API 不稳定 | 中 | 高 | 封装抽象层，锁定 minor 版本 |
| QuickJS 集成复杂度 | 中 | 中 | 寻找官方示例 |
| WASM 编译问题 | 中 | 中 | 预留 2 天 buffer |

---

## 变更记录

| 版本 | 时间 | 变更内容 | 变更人 |
|------|------|----------|--------|
| v1.0 | 2026-04-03 | 初稿创建 | Corelia Team |

---

**最后更新**：2026-04-03

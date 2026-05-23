---
name: corelia-agents
version: v2.0.0
author: corelia-team
description: Corelia 快速启动器项目 Agent 配置 — Tauri 2.x + Svelte 5 + Rust 桌面应用开发规范
tags: [tauri, svelte, rust, desktop-app, launcher, wasm]
---

# Corelia Agent 配置

## 身份与角色

你是 **Corelia 桌面应用开发专家**，专精于 Tauri 2.x + Svelte 5 + Rust 技术栈的快速启动器应用开发。

### ✅ 你擅长
- Tauri 2.x 窗口管理、全局快捷键、插件系统开发
- Svelte 5 Runes ($state/$derived/$props) 组件开发
- Rust 安全编码（禁止 unsafe/panic）、WASM 集成（rquickjs）
- 桌面应用性能优化与调试

### ❌ 你不负责
- 后端 API 服务设计 → 这是纯桌面应用
- 数据库设计与 ORM 配置 → 项目不涉及
- CI/CD 流水线配置 → 使用现有 Bun + Tauri CLI
- npm/yarn/pnpm 操作 → **只用 Bun**

## 触发条件

当用户提及以下任一内容时激活本配置：
- Corelia 项目文件（`src/`、`src-tauri/`、`plugins/`）
- Tauri / Svelte 5 / Rust 相关开发任务
- 桌面应用功能开发（窗口、快捷键、插件、主题）
- 关键词：`corelia`、`启动器`、`launcher`、`tauri`、`svelte5`

## 意图路由表

> 这是本文件最重要的章节，覆盖 90% 的日常开发场景。

| 用户意图 | 触发词示例 | 执行动作 | 优先级 |
|---------|-----------|---------|--------|
| 🆕 创建新功能 | "新增"/"实现"/"添加功能" | Plan → Spec → Implement → Verify 流程 | P0 |
| 🐛 修复 Bug | "报错"/"bug"/"不工作"/"修复" | → 先定位错误源（Rust/TS），再修复 | P0 |
| 🔧 修改现有功能 | "修改"/"重构"/"优化" | → 先读相关代码，再改，最后 `bun run check` | P1 |
| 📦 添加依赖 | "安装"/"add"/"引入" | → 前端 `bun add`，Rust `cargo add` | P1 |
| 🧪 运行测试/检查 | "检查"/"test"/"类型检查" | → `bun run check` + `cargo check` | P1 |
| 🚀 构建/部署 | "构建"/"build"/"打包" | → `bun run check && cargo check --release && bun run tauri build` | P2 |
| 📖 查阅文档 | "文档"/"怎么用"/"API" | → 先搜 `docs/spec/` 和 `.trae/skills/` | P2 |
| 🎨 UI/UX 开发 | "样式"/"主题"/"组件"/"界面" | → 调用 ui-ux-pro-max / svelte-skills | P1 |
| 🔌 插件开发 | "插件"/"plugin"/"扩展" | → 参考三层插件架构（QuickJS/Webview/WASM） | P2 |

## 通用工作流

```
理解需求 → 制定 SPEC → 实现代码 → 验证通过
```

1. **理解需求**: 分析问题域，明确改动范围
2. **制定 SPEC**: 按模板写规格说明书（存 `docs/spec/`），含 API 和验收标准
3. **实现代码**: 按 SPEC 编码，Rust 改后需重新编译，前端支持 HMR
4. **验证通过**: `bun run check` + `cargo check --release`，零错误才能交付

## 核心规则速查

### Rust 规范（强制）

| # | 规则 | ✅ 正确 | ❌ 错误 |
|---|------|---------|--------|
| R1 | 禁止 unsafe | `OnceLock`/`Mutex` | `static mut` |
| R2 | 禁止 panic | `Result<T, String>` | `unwrap()`/`expect()` |
| R3 | 命名下划线 | `read_clipboard` | `readClipboard` |
| R4 | 插件导入格式 | `use tauri::Manager;` | 直接调用无导入 |

### TypeScript 规范（强制）

| # | 规则 | ✅ 正确 | ❌ 错误 |
|---|------|---------|--------|
| T1 | 类型导入分离 | `import type { X } from 'y'` | `import { X } from 'y'`（仅类型时）|
| T2 | Svelte 5 Runes | `$state()`, `$derived()`, `$props()` | Svelte 4 旧语法 |
| T3 | 禁止 any | 明确类型声明 | `any` / 不声明类型 |

### CSS 规范

- 主题通过 `--var` CSS 变量实现
- 透明窗口全局 `background: transparent`

## 常用命令

### 开发
```bash
bun install              # 安装依赖（禁止 npm/yarn/pnpm）
bun run tauri dev        # 启动开发模式（前端 HMR）
bun run check            # TypeScript 类型检查
```

### 调试
```bash
RUST_LOG=debug bun run tauri dev   # Rust 调试日志
# 前端：浏览器 DevTools (F12)
```

### 构建与部署
```bash
bun run check && cargo check --release && bun run tauri build   # 完整构建
rm -rf node_modules/.vite && cargo clean                          # 清理缓存
```

## 项目结构速览

```
src/                    # 前端 (SvelteKit)
  lib/components/       # UI 组件
  lib/stores/           # 状态管理
  lib/services/         # 服务层
  routes/               # 页面路由
src-tauri/              # Rust 后端
  src/commands/         # Tauri Commands
  src/plugins/          # 插件系统
  src/patches/          # WASM 补丁
  capabilities/         # 权限配置 (default.json)
plugins/                # 用户插件目录
docs/spec/              # 规格说明书
```

## 技能索引

匹配到相关任务时，**优先调用对应技能**而非直接回答：

| 场景 | 技能 | 触发条件 |
|------|------|---------|
| Tauri 窗口/事件/插件 | tauri-skills | Tauri v2 API 开发 |
| Svelte 5 组件/状态 | svelte-skills | Runes/组件开发 |
| Rust 编码/并发 | rust-skills | Rust 代码编写 |
| UI/UX 设计 | ui-ux-pro-max | 界面设计/主题/配色 |
| 文档生成 | project-wiki | README/架构文档 |
| 代码审查 | TRAE-code-review | 审查 PR/代码质量 |

## 技术约束

| 约束 | 说明 |
|------|------|
| **包管理器** | 前端 `bun add`，Rust `cargo add`（禁止 npm/yarn/pnpm） |
| **先 Spec 再实现** | 任何新功能必须先写规格说明书 |
| **透明窗口** | `decorations: false, transparent: true` |
| **快捷键** | 避免 Alt+Space（系统冲突），用 Ctrl+Space |

⚠️ **环境要求**: Bun ≥ 1.3.0 / Rust ≥ 1.94.0 / wasm-pack 已安装 / Windows 10/11 x64

## 常见问题速查

| 问题 | 解决方案 |
|------|---------|
| 端口占用 | 修改 `vite.config.js` 或结束占用进程 |
| 快捷键重复注册 | `setup` 中先 `unregister_all()` |
| Rust `static_mut_refs` 警告 | `#![allow(static_mut_refs)]` |
| Svelte onMount async cleanup | 不用 async cleanup 函数 |
| TS 导入报错 | 确保 `moduleResolution: "bundler"` |
| WASM 编译失败 | 使用 `rquickjs` 替代 `quickjs-rs` |

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2026-04-03 | 初稿创建 |
| v1.1 | 2026-04-03 | 增加 Spec 流程规范 |
| v1.2 | 2026-04-04 | 添加技能使用规范章节 |
| v1.3 | 2026-04-09 | 补充 ui-ux-pro-max、web-design-guidelines、software-design 技能 |
| **v2.0** | **2026-05-24** | **重构：新增 YAML 前言、意图路由表、角色边界；压缩技能章节；按频率重排** |

---

**最后更新**: 2026-05-24
# Corelia 开发规范与操作指南

## 项目概述

Corelia 是一个基于 Tauri 2.x + Svelte 5 + Rust 的快速启动器应用。

### 技术栈

| 层级 | 技术 | 版本要求 |
|------|------|----------|
| 前端框架 | SvelteKit | 2.x |
| UI 语言 | TypeScript | ~5.6.2 |
| 打包工具 | Vite | 6.x |
| 包管理器 | Bun | 1.3+ |
| 桌面框架 | Tauri | 2.x |
| 后端语言 | Rust | 1.94.0 |
| JS 引擎 | rquickjs | 0.11.0 |

---

## 开发流程

**标准流程**: Plan → Spec → Implement → Verify

1. **计划**: 分析需求、评估方案、制定任务清单
2. **Spec**: 编写规格说明书（模板见下）
3. **实现**: 按 SPEC 实现，遵循代码规范
4. **验证**: 类型检查、测试、验收

**Spec 文档核心结构**:
- 版本信息（版本/作者/时间/状态）
- 概要（背景/目标/功能列表）
- 模块规格（功能描述/API/数据结构）
- 验收标准（验收项/验证方法/通过标准）

---

## 代码规范

### Rust 规范
- **禁止 unsafe**: 使用 `OnceLock`/`Mutex` 替代 `static mut`
- **命令命名**: 下划线命名（`read_clipboard`）
- **错误处理**: 返回 `Result<T, String>`, 禁止 panic
- **插件导入**: `use tauri::Manager;` `use tauri_plugin_xxx::XxxExt;`

### TypeScript 规范
- **类型导入**: `import type { X } from 'y'` 或 `import { type X } from 'y'`
- **Svelte 5**: 使用 `$state()`, `$derived()`, `$props()`
- **禁止 any**: 使用明确类型声明

### CSS 规范
- **CSS 变量**: 使用 `--var` 实现主题
- **透明窗口**: `background: transparent` 全局重置

---

## 开发环境

**环境要求**:
- Bun >= 1.3.0
- Rust >= 1.94.0
- wasm-pack 已安装

**快速开始**:
```bash
bun install          # 安装依赖
bun run tauri dev    # 开发模式
bun run check        # 类型检查
bun run tauri build  # 生产构建
```

**常见问题**:
1. **端口占用**: 修改 `vite.config.js` 或结束占用进程
2. **快捷键重复注册**: `setup` 中先 `unregister_all()`
3. **Rust 警告**: 添加 `#![allow(static_mut_refs)]`
4. **Svelte onMount**: 不使用 async cleanup
5. **TypeScript 导入**: 确保 `moduleResolution: "bundler"`
6. **WASM 编译**: 使用 `rquickjs` 替代 `quickjs-rs`

---

## 技术约束

**强制约束**:
- ✅ 先 Spec 再实现
- ✅ 不使用 `static mut`
- ✅ 错误返回 `Result`, 禁止 panic
- ✅ 透明窗口配置：`decorations: false, transparent: true`

**包管理器**: 前端用 `bun add`, Rust 用 `cargo add` (禁止 npm/yarn/pnpm)

**目录结构**:
```
src/           # 前端 (SvelteKit)
src-tauri/     # Rust 后端
docs/spec/     # 规格说明书
.trae/skills/  # 技能库
```

## 技能使用规范

**技能库** (`.trae/skills/`):
- **tauri-skills**: Tauri v2 开发（窗口/事件/权限/插件）
- **svelte-skills**: Svelte 5 开发（Runes/组件/状态管理）
- **rust-skills**: Rust 编程（所有权/并发/异步）
- **project-wiki**: 文档生成（README/ARCHITECTURE 等）
- **gstack**: QA 测试/部署/代码审查
- **ui-ux-pro-max**: UI/UX 设计（50+ 样式/161 配色/57 字体组合）
- **web-design-guidelines**: Web 界面规范审查（可访问性/UX 最佳实践）
- **software-design**: 软件设计与编码规范（程序设计/状态管理/模块化/错误处理）

**调用规则**:
1. **优先调用技能**: 匹配技能时不直接回答，调用技能工具
2. **自动触发**: 明确请求时（"创建技能"→skill-manager）
3. **询问触发**: 不确定时（"优化组件"→询问是否用 svelte-skills）

**技能路由** (CLAUDE.md):
```markdown
## Skill routing
- 产品创意/头脑风暴 → office-hours
- Bug/错误 → investigate
- 部署/上线 → ship
- QA/测试 → qa
- 代码审查 → review
- 文档更新 → document-release
```

**Gstack Browse 速查**:
```bash
$B goto <url>        # 导航
$B snapshot -i       # 查看交互元素
$B click @e3         # 点击
$B fill @e4 "val"    # 填充
$B screenshot        # 截图
$B is visible ".x"   # 断言
```

---

## 注意事项

**开发注意**:
- ✅ 先 Spec 再实现
- ✅ Rust 修改需重新编译，前端支持 HMR
- ✅ 权限检查：`src-tauri/capabilities/default.json`
- ✅ 透明窗口需手动处理焦点（blur 事件隐藏）
- ✅ 快捷键避免 Alt+Space（用 Ctrl+Space）

**调试**:
- Rust: `RUST_LOG=debug bun run tauri dev`
- 前端：浏览器 DevTools
- 类型检查：`bun run check`

**部署**:
- 构建前：`bun run check && cargo check --release && bun run tauri build`
- 清理：`rm -rf node_modules/.vite && cargo clean`

---

## 变更记录

| 版本 | 时间 | 变更内容 |
|------|------|----------|
| v1.0 | 2026-04-03 | 初稿创建 |
| v1.1 | 2026-04-03 | 增加 Spec 流程规范 |
| v1.2 | 2026-04-04 | 添加技能使用规范章节 |
| v1.3 | 2026-04-09 | 补充 ui-ux-pro-max、web-design-guidelines、software-design 技能 |

---

**最后更新**: 2026-04-09

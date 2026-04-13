# Corelia

> 基于 Tauri 2.x + Svelte 5 + Rust 的快速启动器应用

## 简介

Corelia 是一款桌面快速启动器，通过关键字前缀快速触发插件功能。采用三层架构：Rust 后端提供系统能力，QuickJS 沙箱运行插件代码，Svelte 5 前端呈现交互界面。

## 核心特性

- 🔍 **快速搜索** — 模糊匹配 + 前缀触发
- 🧩 **插件系统** — QuickJS 沙箱 + WASM Patch 高性能扩展
- 🎨 **主题切换** — 深色/浅色/跟随系统
- ⌨️ **全局快捷键** — 可自定义呼出快捷键
- 📋 **剪贴板集成** — 读写剪贴板内容
- 🪟 **透明窗口** — 无边框设计，失焦自动隐藏
- 🚀 **自启动** — 开机自动启动，最小化到托盘

## 快速开始

```bash
# 安装依赖
bun install

# 开发模式
bun run tauri dev

# 生产构建
bun run tauri build
```

## 文档

完整文档请查看 [Wiki 索引](./wiki/INDEX.md)

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | SvelteKit 2.x + TypeScript 5.6 + Vite 6 |
| 桌面 | Tauri 2.x |
| 后端 | Rust 1.94 + rquickjs 0.11 |
| 包管理 | Bun 1.3+ |

## 许可证

MIT

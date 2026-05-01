# Corelia

> 轻量、安全、可扩展的桌面效率工具平台

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows%2010%2F11-blue.svg)](https://www.microsoft.com/windows)
[![Tech: Tauri 2.x](https://img.shields.io/badge/Tech-Tauri%202.x-钟山.svg)](https://tauri.app/)

## 简介

Corelia 是一款面向桌面端的效率工具，定位为类似 uTools、Listary、Alfred 的**快速启动器 + 插件平台**。用户通过全局快捷键唤起搜索框，快速搜索并执行系统功能、插件功能或第三方工具。

### 核心特性

- ⚡ **快速唤起**：全局快捷键毫秒级响应
- 🔍 **模糊搜索**：输入即匹配，支持插件和系统功能
- 🔌 **插件生态**：支持用户自行开发、加载第三方插件
- 🛡️ **安全沙箱**：插件运行在隔离环境中
- 🎨 **主题定制**：支持深色/浅色/跟随系统
- ⚙️ **WASM 扩展**：高性能原生能力集成

## 技术栈

| 层级 | 技术选型 |
|------|----------|
| 框架 | Tauri 2.x |
| 前端 | Svelte 5 + SvelteKit |
| 后端 | Rust |
| JS 引擎 | QuickJS |
| WASM | wasm-pack + wasm-bindgen |
| 构建 | Vite + Bun |

## 快速开始

### 环境要求

- Windows 10/11 x64
- Node.js 18+
- Rust 1.75+
- Bun 1.x

### 安装依赖

```bash
# 安装 Bun（如果尚未安装）
curl -fsSL https://bun.sh/install | bash

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装项目依赖
bun install
```

### 开发

```bash
# 启动开发服务器
bun run dev

# 或启动 Tauri 开发模式
bun run tauri dev
```

### 构建

```bash
# 构建生产版本
bun run tauri build
```

## 项目结构

```
corelia/
├── src/                      # 前端源码
│   ├── lib/
│   │   ├── components/       # UI 组件
│   │   ├── stores/           # 状态管理
│   │   ├── services/         # 服务层
│   │   └── quickjs/         # QuickJS 运行时
│   └── routes/               # 页面路由
├── src-tauri/                # Rust 后端
│   └── src/
│       ├── commands/         # Tauri Commands
│       ├── plugins/          # 插件系统
│       └── patches/          # WASM 补丁
├── plugins/                  # 插件目录
├── patches/                  # WASM 补丁目录
└── wiki/                     # 项目文档
```

## 插件系统

### 三层架构

Corelia 采用三层插件架构：

1. **QuickJS 插件**：轻量快速，适合简单功能
2. **Webview 插件**：支持复杂 UI，适合需要丰富界面的功能
3. **WASM 补丁**：高性能原生能力，供插件调用

### MVP 插件清单

| 插件 | 类型 | 前缀 | 说明 |
|------|------|------|------|
| 快捷命令 | QuickJS | `sc` | 用户自定义命令脚本 |
| 剪贴板增强 | QuickJS | `cb` | 剪贴板历史、格式转换 |
| 计算器 | QuickJS | `calc` | 数学运算、单位换算 |
| 文件搜索 | QuickJS | - | 本地文件快速定位 |
| 应用启动器 | QuickJS | `app` | 快速打开常用应用 |
| 二维码工具 | QuickJS | `qr` | 生成/解析二维码 |
| 截图增强 | QuickJS | `ss` | 截图 + 历史管理 |
| URL 工具 | QuickJS | - | URL 编解码工具 |

### 插件开发

详见 [插件开发文档](wiki/SRS.md#22-插件系统)

## 文档

- [SRS 功能需求规格说明书](wiki/SRS.md)
- [系统架构设计](wiki/ARCHITECTURE.md)
- [项目路线图](wiki/ROADMAP.md)
- [问题讨论记录](wiki/problem/)

## 路线图

- [x] MVP 架构设计
- [x] MVP 实现（Phase 1-3 完成）
- [x] 插件系统开发（10 个插件）
- [ ] Beta 发布
- [ ] v1.0 正式发布

## MVP 发布检查清单

- [x] 所有 P0 插件完成（5 个）
- [x] 所有 P0 优化完成
- [x] 无编译错误
- [x] 无 Svelte 警告
- [ ] 性能测试达标
- [ ] 核心功能测试通过
- [ ] 文档完整

## License

MIT License - 详见 [LICENSE](LICENSE) 文件

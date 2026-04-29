# Corelia Wiki

> Corelia — 基于 Tauri 2.x + Svelte 5 + Rust 的快速启动器应用

## 文档总览

### 核心文档

| 文档 | 说明 | 重点内容 |
|------|------|----------|
| [README](./README.md) | 项目简介 | 项目介绍、快速开始、技术栈 |
| [系统架构](./ARCHITECTURE.md) | 完整架构说明 | 前后端职责划分、模块详解、数据流 |
| [插件系统](./PLUGIN_SYSTEM.md) | 插件开发指南 | 插件规范、API 参考、WASM Patch 开发 |
| [API 参考](./API.md) | 完整 API 文档 | 11 个功能模块、前后端调用场景 |
| [开发指南](./DEVELOPMENT.md) | 开发环境与规范 | 环境搭建、代码规范、调试技巧 |
| [构建部署](./DEPLOYMENT.md) | 构建与发布流程 | 生产构建、签名、发布流程 |

---

## 快速导航

### 我是前端开发者

- 了解前端架构：[系统架构 - 前端核心模块](./ARCHITECTURE.md#前端核心模块)
- 查看前端 API 调用：[API 参考 - API 模块总览](./API.md#api-模块总览)
- 组件开发参考：[系统架构 - 组件层](./ARCHITECTURE.md#1-组件层-src-lib-components)
- 状态管理参考：[系统架构 - 状态管理层](./ARCHITECTURE.md#2-状态管理层-src-lib-stores)
- 搜索功能开发：[系统架构 - 搜索层](./ARCHITECTURE.md#4-搜索层-src-lib-search)
- 插件前端集成：[系统架构 - 插件前端层](./ARCHITECTURE.md#3-插件前端层-src-lib-plugins)

### 我是后端开发者

- 了解后端架构：[系统架构 - 后端核心模块](./ARCHITECTURE.md#后端核心模块)
- 插件系统核心：[系统架构 - PluginLoader](./ARCHITECTURE.md#3-pluginloader-loader)
- QuickJS 运行时：[系统架构 - QuickJS 运行时](./ARCHITECTURE.md#1-quickjs-运行时-quickjs_runtime.rs)
- API 桥接开发：[系统架构 - API Bridge](./ARCHITECTURE.md#2-api-bridge-api_bridge)
- Tauri Commands 开发：[系统架构 - Commands](./ARCHITECTURE.md#6-commands-commands)
- Rust 服务开发：[系统架构 - Services](./ARCHITECTURE.md#7-services-services)

### 我是插件开发者

- 插件开发入门：[插件系统 - 插件开发示例](./PLUGIN_SYSTEM.md#插件开发示例)
- 插件规范：[插件系统 - plugin.json 规范](./PLUGIN_SYSTEM.md#pluginjson-规范)
- utools API 参考：[插件系统 - 插件 API (utools)](./PLUGIN_SYSTEM.md#插件-api-utools)
- WASM Patch 开发：[插件系统 - WASM Patch 系统](./PLUGIN_SYSTEM.md#wasm-patch-系统)
- 插件生命周期：[插件系统 - 插件生命周期](./PLUGIN_SYSTEM.md#插件生命周期)

### 我想搭建开发环境

- 环境搭建：[开发指南 - 环境要求](./DEVELOPMENT.md#环境要求)
- 快速开始：[开发指南 - 快速开始](./DEVELOPMENT.md#快速开始)
- 代码规范：[开发指南 - 代码规范](./DEVELOPMENT.md#代码规范)
- 调试技巧：[开发指南 - 调试技巧](./DEVELOPMENT.md#调试技巧)

### 我想构建发布

- 构建流程：[构建部署 - 生产构建](./DEPLOYMENT.md#生产构建)
- 代码签名：[构建部署 - 代码签名](./DEPLOYMENT.md#代码签名)
- 发布流程：[构建部署 - 版本发布流程](./DEPLOYMENT.md#版本发布流程)

---

## 技术栈一览

### 前端技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| SvelteKit | 2.x | 前端框架 |
| TypeScript | ~5.6.2 | 类型安全 |
| Vite | 6.x | 构建工具 |
| Bun | 1.3+ | 包管理、运行时 |

### 后端技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| Tauri | 2.x | 桌面框架 |
| Rust | 1.94.0 | 后端语言 |
| rquickjs | 0.11.0 | JavaScript 引擎 |

---

## 目录结构概览

```
corelia/
├── src/                      # 前端 (SvelteKit)
│   ├── lib/
│   │   ├── components/       # Svelte 组件
│   │   ├── plugins/          # 插件前端层
│   │   ├── stores/           # 状态管理
│   │   ├── services/         # 前端服务
│   │   ├── search/           # 搜索引擎
│   │   └── ...
│   └── ...
├── src-tauri/                # Rust 后端
│   └── src/
│       ├── commands/         # Tauri Commands
│       ├── plugins/          # 插件系统核心
│       └── services/         # 业务服务
├── plugins/                  # 用户插件目录
├── patches/                  # WASM Patch 目录
└── docs/wiki/                # 文档目录
```

详细结构说明：[系统架构 - 目录结构](./ARCHITECTURE.md#目录结构)

---

## 关键设计原则

1. **清晰的前后端分离**：前端负责 UI、状态、搜索，后端负责系统能力、插件管理
2. **模块化设计**：各模块职责明确，接口清晰
3. **插件优先**：核心功能通过插件实现，保持核心精简
4. **性能优化**：VM 池化、闲置回收、WASM 加速
5. **安全隔离**：插件沙箱运行，数据隔离存储

详细设计决策：[系统架构 - 关键设计决策](./ARCHITECTURE.md#关键设计决策)

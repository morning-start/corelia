# Corelia 文档中心

> **最后更新**: 2026-05-01  
> **项目状态**: MVP 开发阶段

---

## 快速导航

根据你的身份选择合适的文档：

### 👤 普通用户

- [ ] 快速开始指南
- [ ] 用户使用手册

### 🔌 插件开发者

- [ ] 插件开发入门
- [ ] QuickJS 插件开发
- [ ] Webview 插件开发
- [ ] API 参考文档

### 💻 核心开发者

- [**项目架构分析报告**](./PROJECT_ANALYSIS_REPORT.md) - ⭐ 必读
- [**MVP 开发指南**](./MVP_DEVELOPMENT_GUIDE.md) - ⭐ 必读
- [架构优化分析](./analysis/ARCHITECTURE_OPTIMIZATION_REPORT.md)
- [组件详细文档](./components/)

---

## 文档结构

```
docs/
├── README.md                    # 本文档（总入口）
├── PROJECT_ANALYSIS_REPORT.md   # 项目架构分析报告
├── MVP_DEVELOPMENT_GUIDE.md     # MVP 开发指南
├── analysis/                    # 分析文档
│   └── ARCHITECTURE_OPTIMIZATION_REPORT.md
├── components/                  # 模块详细文档
│   ├── README.md
│   ├── frontend.md
│   ├── plugin-system.md
│   └── quickjs-runtime.md
└── wiki/                        # 技术文档（旧文档参考）
```

---

## 项目简介

**Corelia** 是一款轻量、安全、可扩展的桌面效率工具平台。

### 核心特性

- ⚡ 全局快捷键唤起（Alt+Space）
- 🔍 智能模糊搜索
- 🔌 可扩展插件系统
- 🛡️ 安全沙箱隔离
- 🎨 主题定制
- ⚙️ WASM 原生扩展

### 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.x |
| 前端框架 | Svelte 5 + SvelteKit |
| 后端语言 | Rust |
| JS 引擎 | QuickJS |
| 包管理 | Bun |

---

## 开发资源

### 快速开始

```bash
# 安装依赖
bun install

# 开发模式
bun run tauri dev

# 类型检查
bun run check
```

### 核心模块索引

| 模块 | 文件位置 |
|------|----------|
| QuickJS 运行时 | `src-tauri/src/plugins/quickjs_runtime.rs` |
| 插件加载器 | `src-tauri/src/plugins/loader.rs` |
| API 桥接 | `src-tauri/src/plugins/api_bridge.rs` |
| 搜索系统 | `src/lib/stores/search/` |
| 执行器 | `src/lib/services/executor/` |

---

## 其他参考

### 旧文档位置

以下是旧文档位置，仅供参考：

- `wiki/` - 原始需求规格与架构设计
- `docs/wiki/` - 部分重复的技术文档

### 竞品分析

- [uTools 插件系统](../wiki/reference/uTools-plugin-system.md)
- [ZTools 插件系统](../wiki/reference/ZTools-plugin-system.md)

---

## 贡献指南

### 文档贡献

如果需要补充或修改文档：

1. 对于核心文档更新，请联系项目维护者
2. 文档使用 Markdown 格式
3. 保持与现有文档风格一致

---

## 联系方式

- 项目问题: GitHub Issues
- 技术讨论: GitHub Discussions

---

*Happy coding! 🚀*

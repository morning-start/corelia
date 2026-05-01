# Corelia 文档中心

> **最后更新**: 2026-05-01  
> **项目状态**: MVP 开发阶段

---

## 快速导航

### 💻 核心开发者

| 文档 | 说明 |
|------|------|
| [架构概览](./architecture/) | 架构设计总览 |
| [开发指南](./development/) | MVP 开发指南 |
| [性能分析](./architecture/performance.md) | 性能瓶颈与优化 |

### 🔌 插件开发者

| 文档 | 说明 |
|------|------|
| [插件开发指南](./development/plugin-development.md) | QuickJS/Webview 插件开发 |
| [插件系统架构](./architecture/plugin-system.md) | 三层插件架构设计 |

### 📚 参考资料

| 文档 | 说明 |
|------|------|
| [竞品分析](./reference/competitors.md) | uTools/ZTools 对比 |
| [API 参考](./reference/api.md) | API 文档 |

---

## 文档结构

```
docs/
├── README.md                    # 本文档
├── architecture/                # 架构文档
│   ├── README.md               # 架构概览
│   ├── tech-stack.md           # 技术栈分析
│   ├── plugin-system.md        # 插件系统架构
│   ├── performance.md          # 性能分析
│   └── security.md             # 安全性分析
├── development/                 # 开发文档
│   ├── README.md               # 开发概览
│   ├── environment.md          # 环境配置
│   ├── mvp-guide.md            # MVP 指南
│   ├── plugin-development.md   # 插件开发
│   └── conventions.md          # 代码规范
├── reference/                   # 参考资料
│   └── competitors.md          # 竞品分析
├── components/                  # 模块详细文档
│   ├── README.md
│   ├── frontend.md
│   ├── plugin-system.md
│   └── quickjs-runtime.md
└── analysis/                    # 分析报告
    └── ARCHITECTURE_OPTIMIZATION_REPORT.md
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

### 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.x |
| 前端框架 | Svelte 5 + SvelteKit |
| 后端语言 | Rust |
| JS 引擎 | QuickJS |

---

## 快速开始

```bash
# 安装依赖
bun install

# 开发模式
bun run tauri dev

# 类型检查
bun run check
```

---

## 旧文档位置

以下旧文档仅供参考，内容可能已过时：

- `wiki/` - 原始需求规格与架构设计
- `docs/wiki/` - 部分重复的技术文档
- `docs/PROJECT_ANALYSIS_REPORT.md` - 已拆分到 architecture/
- `docs/MVP_DEVELOPMENT_GUIDE.md` - 已拆分到 development/

---

## 联系方式

- 项目问题: GitHub Issues
- 技术讨论: GitHub Discussions

---

*Happy coding! 🚀*

# Corelia Wiki 索引

欢迎使用 Corelia Wiki，这里包含了 Corelia 项目的完整文档。

---

## 📚 文档导航

### 核心文档

| 文档 | 说明 | 状态 |
|------|------|------|
| [SRS 功能需求规格说明书](SRS.md) | 完整的功能需求规格说明 | ✅ 已完成 |
| [ARCHITECTURE 系统架构设计](ARCHITECTURE.md) | 系统架构、插件系统、数据流设计 | ✅ 已完成 |
| [ROADMAP 项目路线图](ROADMAP.md) | 项目发展阶段和里程碑 | ✅ 已完成 |
| [SYSTEM-CONFIG 系统配置管理](SYSTEM-CONFIG.md) | 系统配置管理与设置功能设计 | ✅ 已完成 |
| [插件系统参考文档](../docs/wiki/PLUGIN_SYSTEM.md) | 插件系统架构、API 和开发指南 | ✅ 已完成 |
| [项目配置](config.yaml) | 项目基础配置信息 | ✅ 已维护 |

### 问题讨论记录

| 文档 | 说明 | 状态 |
|------|------|------|
| [问题总结](problem/00-summary.md) | 所有问题的优先级总结 | ✅ 已更新 |
| [项目管理问题](problem/01-project-management.md) | MVP 边界、插件数量等决策 | ✅ 已完成 |
| [技术架构问题](problem/02-technical-architecture.md) | 插件 API、QuickJS 集成等决策 | ✅ 已完成 |
| [用户体验问题](problem/03-user-experience.md) | 快捷键、Onboarding、主题等决策 | ✅ 已完成 |
| [生态系统问题](problem/04-ecosystem-community.md) | 插件开发、分发、社区运营决策 | ✅ 已完成 |
| [资源与风险问题](problem/05-resources-risk.md) | 开发时间、技术风险评估 | ⏳ 进行中 |

### 参考资料

| 文档 | 说明 | 状态 |
|------|------|------|
| [插件系统对比](reference/plugin-system-comparison.md) | uTools、ZTools 插件系统对比 | ✅ 已完成 |
| [uTools 插件系统分析](reference/uTools-plugin-system.md) | uTools 插件系统详细分析 | ✅ 已完成 |
| [ZTools 插件系统](reference/ZTools-plugin-system.md) | ZTools 插件系统分析 | ✅ 已完成 |

---

## 🗂️ 文档分类

### 需求与设计

- **[SRS.md](SRS.md)** - 功能需求规格说明书
  - 项目概述
  - 功能需求（全局唤起、插件系统、系统交互、数据管理）
  - 生态与社区
  - 非功能需求（性能、安全性、兼容性）
  - 数据流设计
  - 验收标准

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - 系统架构设计
  - 整体架构分层
  - 三层插件架构（QuickJS + Webview + WASM）
  - 插件运行时设计
  - 模块设计
  - 目录结构
  - 性能模型

- **[SYSTEM-CONFIG.md](SYSTEM-CONFIG.md)** - 系统配置管理设计
  - 配置系统架构
  - 配置数据结构
  - 核心模块设计（前端 Store、后端 Service、UI 组件）
  - 数据流设计（加载、保存、快捷键注册）
  - 配置项详解（主题、快捷键、行为、启动）
  - 扩展性设计（导入导出、云端同步）
  - 安全性设计
  - 测试策略

- **[插件系统参考文档](../docs/wiki/PLUGIN_SYSTEM.md)** - 插件系统完整文档
  - 架构概览（系统层次、核心模块、数据流）
  - 插件生命周期（状态转换图、状态说明）
  - 插件开发指南（Hello World 示例）
  - Plugin JSON 规范（完整字段定义）
  - uTools 兼容 API 参考（dbStorage、clipboard、shell、fs、getPath、通知、窗口控制）
  - 前端 PluginService API（初始化、核心方法、类型定义）
  - Tauri Commands (IPC)（QuickJS 运行时、API 注入、加载器、注册表、数据隔离）
  - QuickJS 运行时（配置参数、VM 生命周期、值转换链）
  - 数据隔离存储（存储机制、配额限制、前端 PluginStoreService）
  - 状态机（合法转换规则、状态校验逻辑）
  - 最佳实践与调试技巧

### 项目规划

- **[ROADMAP.md](ROADMAP.md)** - 项目路线图
  - 发展阶段规划
  - 各阶段目标
  - 发布时间表

- **[config.yaml](config.yaml)** - 项目配置
  - 项目基本信息
  - 技术栈
  - 功能列表
  - 里程碑

### 问题决策记录

- **[problem/00-summary.md](problem/00-summary.md)** - 问题优先级总结
  - 已完成决策列表
  - 待讨论问题清单

- **[problem/01-project-management.md](problem/01-project-management.md)** - 项目管理问题
  - MVP 边界定义
  - 插件数量目标
  - 需求变更流程

- **[problem/02-technical-architecture.md](problem/02-technical-architecture.md)** - 技术架构问题
  - 插件 API 兼容性
  - QuickJS 集成方案
  - Webview 隔离策略
  - WASM 技术选型

- **[problem/03-user-experience.md](problem/03-user-experience.md)** - 用户体验问题
  - 快捷键方案
  - Onboarding 流程
  - 中文分词方案
  - 主题系统设计

- **[problem/04-ecosystem-community.md](problem/04-ecosystem-community.md)** - 生态系统问题
  - 插件开发文档
  - 插件分发机制
  - 跨平台适配
  - AI 集成方案
  - 社区运营计划

### 参考资料

- **[reference/plugin-system-comparison.md](reference/plugin-system-comparison.md)** - 插件系统对比
  - uTools vs ZTools vs Corelia
  - 架构对比
  - 性能对比

- **[reference/uTools-plugin-system.md](reference/uTools-plugin-system.md)** - uTools 插件系统分析
  - 插件结构
  - API 设计
  - 通信机制

- **[reference/ZTools-plugin-system.md](reference/ZTools-plugin-system.md)** - ZTools 插件系统分析
  - 插件结构
  - 特色功能

---

## 📊 文档状态

| 文档类型 | 已完成 | 进行中 | 待开始 |
|----------|--------|--------|--------|
| 需求与设计 | 5 | 0 | 0 |
| 项目规划 | 2 | 0 | 0 |
| 问题决策 | 5 | 1 | 0 |
| 参考资料 | 3 | 0 | 0 |
| **总计** | **15** | **1** | **0** |

---

## 🔗 快速链接

### 开发相关

- [AGENTS.md](../AGENTS.md) - 开发规范与操作指南
- [MVP-ECOSYSTEM 规格](../docs/spec/mvp/mvp-ecosystem/MVP-ECOSYSTEM-SPEC.md) - MVP 阶段生态功能规格

### 外部资源

- [Tauri 2.x 文档](https://v2.tauri.app/)
- [Svelte 5 文档](https://svelte.dev/docs)
- [Bun 文档](https://bun.sh/docs)

---

## 📝 文档维护

### 更新记录

| 时间 | 更新内容 | 更新人 |
|------|----------|--------|
| 2026-04-11 | 新增插件系统参考文档索引 | Corelia Team |
| 2026-04-05 | 新增 SYSTEM-CONFIG 系统配置管理文档 | Corelia Team |
| 2026-04-02 | 更新 SRS.md 决策记录 | Corelia Team |
| 2026-04-01 | 完成问题讨论系列文档 | Corelia Team |
| 2026-03-05 | 创建 SRS.md 初稿 | Corelia Team |

### 文档规范

- 使用中文撰写
- 遵循 Markdown 语法
- 重要决策添加决策记录表格
- 架构图使用 Mermaid 或 ASCII Art
- 代码示例标注语言类型

---

**最后更新**: 2026-04-11

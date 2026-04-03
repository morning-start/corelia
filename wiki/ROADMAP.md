# Corelia 项目路线图

## 概述

本文档描述 Corelia 项目的发展规划、里程碑和技术路线。

---

## 当前阶段：MVP（进行中 🔄）

**目标**：完成 Corelia 桌面效率工具的核心功能

### MVP 交付物

#### 核心功能

- [x] 全局快捷键唤起（Alt+Space，可自定义）
- [x] 模糊搜索（tiny-fuzzy）
- [x] 插件系统架构设计
  - [x] QuickJS 插件运行时
  - [x] Webview 插件支持（iframe 隔离）
  - [x] WASM 补丁层（crypto + AI）
- [x] 主题系统（CSS Custom Properties，3 个内置主题）
- [x] 首次使用引导（Onboarding，4 步）

#### MVP 插件清单（8 个）

| 序号 | 插件名称 | 类型 | 优先级 |
|------|----------|------|--------|
| 1 | 快捷命令 | QuickJS | 🔴 必须 |
| 2 | 剪贴板增强 | QuickJS | 🔴 必须 |
| 3 | 计算器 | QuickJS | 🔴 必须 |
| 4 | 二维码 | QuickJS + WASM crypto | 🔴 必须 |
| 5 | 文件搜索 + 应用搜索 | QuickJS | 🔴 必须 |
| 6 | 截图增强 | QuickJS + WASM AI | 🔴 必须 |
| 7 | 云端同步配置 | Webview | 🔴 必须 |

#### 技术架构

- [x] Tauri 2.x 窗口管理
- [x] Svelte 5 前端框架
- [x] 统一插件 API（window.utools 兼容 + window.corelia 扩展）
- [x] Tauri Event 通信机制
- [x] 插件懒加载机制
- [x] 插件热重载

#### 生态建设

- [x] 插件分发机制（GitHub Release + .zip + 拖拽安装）
- [x] 插件开发文档规划
- [x] TypeScript SDK 规划

### MVP 里程碑

- [ ] 技术原型验证（Proof of Concept）
- [ ] 核心框架实现
- [ ] MVP 插件开发
- [ ] 内测
- [ ] Beta 发布

---

## 计划阶段：v1.0（规划中 📋）

**目标**：正式发布 Corelia 1.0

### 新功能

- [ ] 插件市场/应用商店界面
- [ ] 用户词典（自定义分词）
- [ ] 自定义主题支持
- [ ] 插件预加载策略（智能预加载常用插件）
- [ ] 插件签名与安全机制

### 技术升级

- [ ] macOS 11+ 适配（Intel + Apple Silicon）
- [ ] Linux 适配（Ubuntu 20.04+, Fedora 35+）
- [ ] 多窗口支持（替代 iframe）
- [ ] VSCode 插件开发支持

### 性能优化

- [ ] 启动时间优化（目标 < 2 秒）
- [ ] 搜索响应优化（目标 < 50ms）
- [ ] 内存占用控制（空闲 < 100MB）

### 生态建设

- [ ] 完整插件开发文档
- [ ] 插件模板仓库
- [ ] 插件 Showcase 展示
- [ ] GitHub Discussions 社区运营

### v1.0 里程碑

- [ ] 技术选型完成
- [ ] 架构设计完成
- [ ] MVP 发布
- [ ] Beta 测试
- [ ] 正式发布（预计 2026-Q4）

---

## 未来规划（长期 🎯）

### v2.0（预计 2027）

#### 生态系统

- [ ] 插件市场正式上线
- [ ] 第三方集成 API
- [ ] 开发者平台
- [ ] 主题市场

#### 新功能

- [ ] AI 助手插件系统
- [ ] 智能推荐（基于使用习惯）
- [ ] 高级工作流自动化
- [ ] 团队协作功能

#### 企业级功能

- [ ] 企业级 SSO
- [ ] 高级权限管理
- [ ] 配置集中管理
- [ ] 合规性报告

### v3.0（预计 2028）

#### 技术创新

- [ ] 引入机器学习模型
- [ ] 实现边缘计算
- [ ] 云原生架构
- [ ] 无服务器支持

#### 生态扩展

- [ ] 移动端应用
- [ ] Web 版本
- [ ] 浏览器扩展
- [ ] 更多平台支持

---

## 技术路线

### 核心技术决策

| 技术 | 决策 | 说明 |
|------|------|------|
| 框架 | Tauri 2.x | 轻量、安全、原生体验 |
| 前端 | Svelte 5 + SvelteKit | 响应式 UI、高性能 |
| 后端 | Rust | 系统级能力、高性能 |
| JS 引擎 | QuickJS | 轻量级插件运行时 |
| WASM | wasm-pack + wasm-bindgen | 原生能力扩展 |
| 构建 | Vite + Bun | 快速开发、热更新 |

### 跨平台路线图

```
Phase 1（MVP）: Windows 10/11 x64
    │
    ▼
Phase 2 (v1.0): macOS 11+ (Intel + Apple Silicon)
    │
    ▼
Phase 3 (v1.x): Linux (Ubuntu 20.04+, Fedora 35+)
```

---

## 风险和挑战

### 技术风险

- **Tauri 2.x API 不稳定**：封装抽象层应对 API 变化
- **QuickJS 集成难度**：寻找成熟方案，降低风险
- **WASM 编译复杂**：MVP 优先使用社区成熟库

### 时间风险

- **个人开发时间不足**：MVP 优先，砍功能
- **外部依赖延迟**：预留 buffer 时间

### 社区风险

- **插件开发者不足**：先做官方插件，建立示范
- **开源社区反馈冷淡**：主动推广，积极运营

### 缓解措施

- 定期进行技术评审
- 建立自动化测试和部署
- 收集用户反馈并快速响应
- 建立应急响应机制

---

## 待确定问题

### P1 重要但未详细讨论

| 序号 | 问题 | 所属分类 | 说明 |
|------|------|----------|------|
| 1.2 | 版本发布节奏 | 项目管理 | 发布周期未确定 |
| 1.4 | 质量门禁标准 | 项目管理 | CI/CD 测试流程未确定 |
| 5.1 | 开发时间估算 | 资源风险 | 各阶段耗时未估算 |
| 5.3 | Tauri 2.x 稳定性评估 | 资源风险 | 版本锁定策略未确定 |
| 2.5 | 插件签名与安全 | 技术架构 | P2 可延后 |
| 2.6 | 数据存储方案 | 技术架构 | SQLite 等未讨论 |
| 2.7 | 前端状态管理方案 | 技术架构 | Svelte 5 Runes 未讨论 |
| 2.8 | 热更新策略 | 技术架构 | 细节未讨论 |
| 3.3 | 插件市场形态 | 用户体验 | P2 可延后 |
| 3.6 | 搜索交互细节 | 用户体验 | 结果数量等未讨论 |
| 3.7 | 窗口行为细节 | 用户体验 | 动画等未讨论 |

---

## 参考资料

- [SRS 功能需求规格说明书](SRS.md)
- [项目问题讨论记录](problem/)
  - [00-summary.md](problem/00-summary.md) - 问题优先级总结
  - [01-project-management.md](problem/01-project-management.md) - 项目管理层面问题
  - [02-technical-architecture.md](problem/02-technical-architecture.md) - 技术架构层面问题
  - [03-user-experience.md](problem/03-user-experience.md) - 用户体验层面问题
  - [04-ecosystem-community.md](problem/04-ecosystem-community.md) - 生态与社区层面问题
  - [05-resources-risk.md](problem/05-resources-risk.md) - 资源与风险层面问题
- [参考文档](reference/)

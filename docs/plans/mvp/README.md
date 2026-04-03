# MVP 执行计划总览

本文档汇总 MVP 所有阶段的执行计划，提供统一的索引和依赖关系管理。

---

## 阶段总览

| 阶段 | 计划 ID | 计划名称 | 预计工期 | 前置阶段 | 状态 |
|------|----------|----------|----------|----------|------|
| Phase 0 | MVP-POC | 技术原型验证 | 2-3 周 | - | ❌ |
| Phase 1 | MVP-CORE-FRAMEWORK | 核心框架实现 | 3-4 周 | MVP-POC | ❌ |
| Phase 2 | MVP-PLUGIN-SYSTEM | 插件系统实现 | 4-5 周 | MVP-CORE-FRAMEWORK | ❌ |
| Phase 3 | MVP-PLUGINS | MVP 插件开发 | 3-4 周 | MVP-PLUGIN-SYSTEM | ❌ |
| Phase 4 | MVP-UX | 用户体验优化 | 1-2 周 | MVP-PLUGINS | ❌ |
| Phase 5 | MVP-ECOSYSTEM | 生态建设 | 2-3 周 | MVP-UX | ❌ |

**总工期**：约 15-21 周

---

## 阶段依赖关系

```
MVP-POC (Phase 0)
    │
    ▼
MVP-CORE-FRAMEWORK (Phase 1)
    │
    ▼
MVP-PLUGIN-SYSTEM (Phase 2)
    │
    ▼
MVP-PLUGINS (Phase 3)
    │
    ▼
MVP-UX (Phase 4)
    │
    ▼
MVP-ECOSYSTEM (Phase 5)
```

---

## 计划索引

### [MVP-POC 技术原型验证计划](MVP-POC.md)

**目标**：验证核心技术的可行性

**核心任务**：
- Tauri 窗口配置验证
- 全局快捷键验证
- QuickJS 集成验证
- WASM 基础环境验证
- tiny-fuzzy 搜索测试

**关键验收标准**：
- Tauri 窗口可在无边框模式下运行
- 全局快捷键 Alt+Space 可唤起窗口
- QuickJS VM 可正常实例化
- WASM 模块可正常加载
- tiny-fuzzy 搜索响应 < 50ms

---

### [MVP-CORE-FRAMEWORK 核心框架实现计划](MVP-CORE-FRAMEWORK.md)

**目标**：实现 Corelia 的核心框架

**核心任务**：
- 项目结构搭建
- 窗口管理器实现
- 全局快捷键系统
- 主题系统实现
- 搜索组件实现

**关键验收标准**：
- Alt+Space 可唤起/隐藏窗口
- 窗口置顶且不抢占焦点
- 主题切换（深/浅/跟随）正常
- 快捷键可自定义且生效
- 剪贴板读写功能正常
- Shell 命令执行正常
- 用户配置持久化

---

### [MVP-PLUGIN-SYSTEM 插件系统实现计划](MVP-PLUGIN-SYSTEM.md)

**目标**：实现 Corelia 的插件系统

**核心任务**：
- 插件基础设施
- QuickJS 运行时
- Webview 插件支持
- 插件懒加载机制
- 插件 API 实现
- WASM 补丁层

**关键验收标准**：
- 插件目录扫描正常
- QuickJS 插件加载 < 10ms
- Webview 插件 iframe 正常
- Tauri Event 通信正常
- 插件懒加载机制正常
- window.utools API 可用
- WASM crypto 补丁正常
- WASM AI 补丁正常

---

### [MVP-PLUGINS MVP 插件开发计划](MVP-PLUGINS.md)

**目标**：实现 7 个 QuickJS 插件 + 1 个 Webview 插件

**插件清单**：

| 序号 | 插件名称 | 类型 | 优先级 |
|------|----------|------|--------|
| 1 | 快捷命令 | QuickJS | P0 |
| 2 | 剪贴板增强 | QuickJS | P0 |
| 3 | 计算器 | QuickJS | P0 |
| 4 | 二维码 | QuickJS + crypto | P0 |
| 5 | 文件搜索 + 应用搜索 | QuickJS | P0 |
| 6 | 截图增强 | QuickJS + AI | P0 |
| 7 | 云端同步配置 | Webview | P0 |

---

### [MVP-UX 用户体验优化计划](MVP-UX.md)

**目标**：优化 Corelia 的用户体验

**核心任务**：
- Onboarding 引导（4 步）
- 快捷键冲突检测
- 搜索结果高亮
- 搜索历史
- 搜索结果分组
- 窗口动画效果
- 多显示器支持

**关键验收标准**：
- Onboarding 4 步引导正常
- 快捷键自定义生效
- 快捷键冲突提示正常
- 搜索响应 < 50ms
- 主题切换正常
- 结果分组显示正常

---

### [MVP-ECOSYSTEM 生态建设计划](MVP-ECOSYSTEM.md)

**目标**：建设 Corelia 生态系统

**核心任务**：
- GitHub 仓库初始化
- 插件分发实现
- 插件模板仓库
- TypeScript SDK
- 插件开发文档
- GitHub Actions CI/CD
- 版本发布准备

**关键验收标准**：
- 插件可通过拖拽安装
- 插件可正确解压到目录
- 开发文档完整可用
- TypeScript SDK 可用
- GitHub 仓库结构完整
- Discussions 可用

---

## 工时汇总

| 阶段 | 计划名称 | 预计工时 |
|------|----------|----------|
| Phase 0 | MVP-POC | 34h |
| Phase 1 | MVP-CORE-FRAMEWORK | 124h |
| Phase 2 | MVP-PLUGIN-SYSTEM | 156h |
| Phase 3 | MVP-PLUGINS | 136h |
| Phase 4 | MVP-UX | 84h |
| Phase 5 | MVP-ECOSYSTEM | 120h |
| **总计** | | **654h** |

---

## 资源需求汇总

### 开发环境

| 工具 | 版本 | 用途 |
|------|------|------|
| Windows | 10/11 x64 | 开发平台 |
| Rust | 1.75+ | 后端开发 |
| Node.js | 18+ | 前端开发 |
| Bun | 1.x | 包管理、运行 |
| wasm-pack | latest | WASM 编译 |

### 前端库

| 库 | 版本 | 用途 |
|------|------|------|
| Svelte | 5.x | UI 框架 |
| SvelteKit | 2.x | 应用框架 |
| TypeScript | 5.x | 类型安全 |
| tiny-fuzzy | latest | 模糊搜索 |

### Tauri 插件

| 插件 | 用途 |
|------|------|
| tauri-plugin-global-shortcut | 全局快捷键 |
| tauri-plugin-clipboard | 剪贴板 |
| tauri-plugin-shell | Shell 执行 |
| tauri-plugin-store | 数据存储 |

---

## 风险汇总

| 风险 | 可能性 | 影响 | 应对措施 |
|------|--------|------|----------|
| Tauri 2.x API 不稳定 | 中 | 高 | 封装抽象层，锁定版本 |
| QuickJS 集成复杂度高 | 中 | 中 | 寻找成熟方案 |
| WASM 编译环境问题 | 中 | 中 | 预留 buffer |
| 个人开发时间不足 | 高 | 高 | MVP 优先，砍功能 |
| 插件开发者不足 | 中 | 中 | 先做官方插件 |

---

## 下一步行动

1. **立即开始**：执行 MVP-POC 技术原型验证
2. **并行工作**：Phase 1-5 可开始预研和准备工作
3. **定期评审**：每阶段结束后进行评审
4. **持续优化**：根据执行情况调整计划

---

## 参考资料

- [SRS 功能需求规格说明书](../../wiki/SRS.md)
- [系统架构设计](../../wiki/ARCHITECTURE.md)
- [项目路线图](../../wiki/ROADMAP.md)
- [问题讨论记录](../../wiki/problem/)

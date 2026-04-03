# MVP-CORE-FRAMEWORK 核心框架实现规格文档

本文档包含 MVP-CORE-FRAMEWORK 阶段的所有规格说明和执行清单。

---

## 文档列表

| 文档 | 说明 |
|------|------|
| [MVP-CORE-FRAMEWORK-SPEC.md](MVP-CORE-FRAMEWORK-SPEC.md) | 技术规格说明书 |
| [checklist.md](checklist.md) | 任务执行清单 |
| [acceptance.md](acceptance.md) | 验收标准 |

---

## 规格说明书内容

### 1. 概要

基于 MVP-POC 验证结果，实现 Corelia 核心框架：

- 窗口管理系统
- 全局快捷键系统
- 主题切换系统
- 系统集成服务
- 主界面和搜索功能

### 2. POC 遗留问题处理

| 问题 | POC 状态 | CORE 解决方案 |
|------|----------|---------------|
| QuickJS Windows MSVC 编译错误 | 模拟模式 | 使用 quickjs-wasm-rs 或 rquickjs |
| WASM 前端类型问题 | 降级方案 | 使用 wasm-bindgen 类型生成 |
| 拼音搜索支持 | 待实现 | 集成 pinyin-pro |

### 3. 技术栈

- **前端**：Svelte 5 + SvelteKit + TypeScript
- **后端**：Tauri 2.x + Rust
- **搜索**：fuzzy
- **存储**：tauri-plugin-store

---

## 执行流程

1. **阅读规格说明书** → [MVP-CORE-FRAMEWORK-SPEC.md](MVP-CORE-FRAMEWORK-SPEC.md)
2. **执行任务** → [checklist.md](checklist.md)
3. **记录问题和解决方案**
4. **验收** → [acceptance.md](acceptance.md)
5. **编写阶段报告**

---

## 下一阶段

CORE-FRAMEWORK 验证通过后，进入下一阶段：MVP-ECOSYSTEM

---

## 变更记录

| 版本 | 时间 | 变更内容 | 变更人 |
|------|------|----------|--------|
| v1.0 | 2026-04-03 | 初稿创建 | Corelia Team |

---

**最后更新**：2026-04-03

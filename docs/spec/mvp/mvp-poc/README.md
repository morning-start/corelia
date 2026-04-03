# MVP-POC 技术原型验证文档

本文档包含 MVP-POC 阶段的所有规格说明和执行清单。

---

## 文档列表

| 文档 | 说明 |
|------|------|
| [MVP-POC-SPEC.md](MVP-POC-SPEC.md) | 技术原型验证规格说明书 |
| [checklist.md](checklist.md) | 任务执行清单 |
| [acceptance.md](acceptance.md) | 验收标准 |

---

## 规格说明书内容

### 1. 概要

验证以下核心技术的可行性：
- Tauri 窗口配置
- 全局快捷键
- QuickJS 集成
- WASM 环境
- 搜索性能

### 2. 验收标准

| 验收项 | 标准 |
|--------|------|
| Tauri 窗口 | 无边框、透明、圆角、置顶 |
| 全局快捷键 | Alt+Space 全局响应 |
| QuickJS | VM 实例化成功 |
| WASM | 模块加载成功 |
| 搜索性能 | < 50ms (1000 条数据) |

---

## 执行流程

1. **阅读规格说明书** → [MVP-POC-SPEC.md](MVP-POC-SPEC.md)
2. **执行任务** → [checklist.md](checklist.md)
3. **记录问题和解决方案**
4. **验收** → [acceptance.md](acceptance.md)
5. **编写原型报告**

---

## 下一阶段

POC 验证通过后，进入下一阶段： [MVP-CORE-FRAMEWORK](../../plans/mvp/MVP-CORE-FRAMEWORK.md)

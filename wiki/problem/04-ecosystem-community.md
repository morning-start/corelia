# 生态与社区层面问题

## 4.1 插件开发文档

**决策**：按建议方案

| 问题 | 决策 | 说明 |
|------|------|------|
| 文档形式 | **完整 Guide** | 详细插件开发文档 |
| SDK 层级 | **TypeScript SDK** | 提供类型提示和 API 封装 |
| VSCode 插件 | **MVP 后再支持** | 降低初期工作量 |
| 插件模板 | **提供基础模板** | 快速上手 |

**文档结构规划**：
```
docs/
├── getting-started.md       # 快速开始
├── plugin-anatomy.md        # 插件结构
├── api-reference.md         # API 参考
├── examples/                # 示例代码
│   ├── quickjs-basic.md     # QuickJS 基础示例
│   └── webview-advanced.md  # Webview 高级示例
└── troubleshooting.md       # 常见问题
```

**已确定** ✅：
- [x] 文档形式 → **完整 Guide**
- [x] SDK 层级 → **TypeScript SDK**
- [x] VSCode 插件 → **MVP 后再支持**
- [x] 插件模板 → **提供基础模板**

---

## 4.2 插件分发机制

**决策**：选择 **A 方案（GitHub Release）**

| 问题 | 决策 | 说明 |
|------|------|------|
| 分发平台 | **GitHub Release** | 简单直接 |
| 插件包格式 | **.zip** | 打包插件文件夹 |
| 安装方式 | **拖拽安装** | 用户拖入窗口即可 |
| 版本管理 | **SemVer** | 遵循语义化版本 |
| 残留数据 | **提示用户手动删除** | 避免误删 |

**分发流程**：
```
开发者
    │
    ▼
GitHub Release 发布 .zip
    │
    ▼
用户下载 .zip
    │
    ▼
拖入 Corelia 窗口安装
    │
    ▼
自动解压到 plugins/ 目录
```

**已确定** ✅：
- [x] 分发平台 → **GitHub Release**
- [x] 插件包格式 → **.zip**
- [x] 安装方式 → **拖拽安装**

---

## 4.3 跨平台适配计划

**决策**：按建议方案

| 问题 | 决策 | 说明 |
|------|------|------|
| macOS/Linux 适配 | **MVP 后再适配** | 聚焦 Windows MVP |
| 技术方案 | **统一架构** | 逐步适配各平台 |
| 多平台 CI/CD | **后续配置** | MVP 后再配置 |

**跨平台路线图**：
```
Phase 1（MVP）: Windows 10/11 x64
    │
    ▼
Phase 2: macOS 11+ (Intel + Apple Silicon)
    │
    ▼
Phase 3: Linux (Ubuntu 20.04+, Fedora 35+)
```

**已确定** ✅：
- [x] macOS/Linux → **MVP 后再适配**
- [x] 技术方案 → **统一架构，逐步适配**

---

## 4.4 AI 集成方案

**决策**：**暂不集成到插件系统**（但 WASM 补丁层保留 AI 能力）

| 问题 | 决策 | 说明 |
|------|------|------|
| AI 插件系统 | **暂不集成** | 降低 MVP 复杂度 |
| WASM AI 补丁 | **保留** | 截图 OCR 需要 |
| 未来策略 | **后续再定** | 看 MVP 发展情况 |

**说明**：
- MVP 截图增强插件需要 WASM AI 补丁进行 OCR
- 但插件系统层面不提供通用 AI 调用接口
- 后续可根据用户需求再决定是否开放 AI 能力

**已确定** ✅：
- [x] AI 插件系统 → **暂不集成**
- [x] WASM AI 补丁 → **MVP 保留（截图 OCR 用）**

---

## 4.5 社区运营计划

**决策**：按建议方案

| 问题 | 决策 | 说明 |
|------|------|------|
| 社区平台 | **GitHub Discussions** | 零成本，生态好 |
| 插件 Showcase | **MVP 后再做** | 后续再展示 |
| 激励机制 | **先做基础** | 后期再考虑 |

**已确定** ✅：
- [x] 社区平台 → **GitHub Discussions**
- [x] 插件 Showcase → **MVP 后再做**

---

## 4.6 官方插件开发计划

**决策**：MVP 自带 7 个 QuickJS 插件 + 1 个 Webview 插件（共 8 个）

| 插件 | 类型 | 优先级 |
|------|------|--------|
| 快捷命令 | QuickJS | 🔴 必须 |
| 剪贴板增强 | QuickJS | 🔴 必须 |
| 计算器 | QuickJS | 🔴 必须 |
| 二维码 | QuickJS + WASM crypto | 🔴 必须 |
| 文件搜索 + 应用搜索 | QuickJS | 🔴 必须 |
| 截图增强 | QuickJS + WASM AI | 🔴 必须 |
| 云端同步配置 | Webview | 🔴 必须 |

**已确定** ✅（第一轮已确定）：
- [x] MVP 插件清单 → **8 个插件（7 QuickJS + 1 Webview）**

---

## 讨论方向建议

~~1. **先规划插件开发文档结构**：好的文档是社区基础~~
~~2. **确定分发机制**：影响插件开发者的发布流程~~
~~3. **评估 AI 集成优先级**：如果 WASM 补丁延后，AI 功能如何实现？~~

✅ **4.1 插件开发文档** - 完整 Guide + TypeScript SDK
✅ **4.2 插件分发机制** - GitHub Release + .zip + 拖拽安装
✅ **4.3 跨平台适配** - MVP 后再适配 macOS/Linux
✅ **4.4 AI 集成** - 暂不集成插件系统，WASM AI 补丁保留
✅ **4.5 社区运营** - GitHub Discussions
✅ **4.6 官方插件** - MVP 7 个插件

---

### 四轮讨论全部完成
# MVP-CORE-FRAMEWORK 核心框架实现计划

## 概述

本计划用于指导 MVP 核心框架实现阶段，包括基础架构、窗口管理、系统集成等核心功能。

---

## 基本信息

| 字段 | 内容 |
|------|------|
| **计划 ID** | MVP-CORE-FRAMEWORK |
| **计划名称** | 核心框架实现 |
| **所属阶段** | MVP Phase 1 |
| **前置计划** | MVP-POC |
| **预计工期** | 3-4 周 |
| **状态** | 待开始 |

---

## 目标与验收标准

### 目标

实现 Corelia 的核心框架，包括：

1. **窗口管理系统**：无边框窗口、置顶、透明背景、主题支持
2. **全局快捷键系统**：快捷键注册、响应、自定义配置
3. **系统集成**：文件系统、剪贴板、Shell 执行
4. **数据持久化**：用户配置、插件数据的存储和读取
5. **主题系统**：CSS Custom Properties + 3 个内置主题

### 验收标准

| 序号 | 验收标准 | 验证方法 | 通过标准 |
|------|---------|---------|---------|
| 1 | Alt+Space 可唤起/隐藏窗口 | 手动测试 | 响应正常 |
| 2 | 窗口置顶且不抢占焦点 | 手动测试 | 其他应用可正常操作 |
| 3 | 主题切换（深/浅/跟随）正常 | 手动测试 | 主题正确应用 |
| 4 | 快捷键可自定义且生效 | 手动测试 | 自定义键位正常 |
| 5 | 剪贴板读写功能正常 | 运行测试 | 数据正确读写 |
| 6 | Shell 命令执行正常 | 运行测试 | 命令正确执行 |
| 7 | 用户配置持久化 | 重启测试 | 配置不丢失 |

---

## 任务拆解

### 任务列表

| 任务 ID | 任务名称 | 预计工时 | 依赖 | 优先级 |
|---------|---------|---------|------|--------|
| CORE-01 | 项目结构搭建 | 8h | POC 完成 | P0 |
| CORE-02 | 窗口管理器实现 | 16h | CORE-01 | P0 |
| CORE-03 | 全局快捷键系统 | 16h | CORE-01 | P0 |
| CORE-04 | 主题系统实现 | 8h | CORE-01 | P0 |
| CORE-05 | 设置面板 UI | 12h | CORE-03, CORE-04 | P0 |
| CORE-06 | 剪贴板服务 | 8h | CORE-01 | P0 |
| CORE-07 | Shell 服务 | 8h | CORE-01 | P0 |
| CORE-08 | 数据存储服务 | 12h | CORE-01 | P0 |
| CORE-09 | 主界面布局 | 16h | CORE-02, CORE-04 | P0 |
| CORE-10 | 搜索组件 | 16h | CORE-09 | P0 |
| CORE-11 | 集成测试 | 8h | CORE-01~10 | P0 |
| CORE-12 | 文档更新 | 4h | CORE-11 | P1 |

### 详细任务说明

#### CORE-01 项目结构搭建

**任务描述**：搭建符合 ARCHITECTURE.md 的项目目录结构

**执行步骤**：
1. 创建前端目录结构
2. 创建 Rust 后端目录结构
3. 配置路径别名
4. 配置模块导出

**目录结构**：
```
src/
├── lib/
│   ├── components/     # UI 组件
│   ├── stores/        # 状态管理
│   ├── services/     # 服务层
│   ├── quickjs/      # QuickJS 运行时
│   └── utils/        # 工具函数
src-tauri/
├── src/
│   ├── commands/     # Tauri Commands
│   ├── plugins/      # 插件系统
│   └── patches/      # WASM 补丁
```

#### CORE-02 窗口管理器实现

**任务描述**：实现窗口的创建、显示/隐藏、置顶等管理功能

**执行步骤**：
1. 实现 WindowManager Rust 模块
2. 实现窗口创建配置
3. 实现显示/隐藏切换
4. 实现置顶/取消置顶
5. 实现窗口位置记忆
6. 实现多显示器支持

**Rust 模块**：`src-tauri/src/commands/window.rs`

#### CORE-03 全局快捷键系统

**任务描述**：实现全局快捷键的注册、响应和管理

**执行步骤**：
1. 集成 tauri-plugin-global-shortcut
2. 实现 ShortcutManager Rust 模块
3. 实现快捷键注册接口
4. 实现快捷键冲突检测
5. 实现自定义快捷键配置

**Rust 模块**：`src-tauri/src/commands/shortcut.rs`

#### CORE-04 主题系统实现

**任务描述**：实现基于 CSS Custom Properties 的主题系统

**执行步骤**：
1. 定义主题 CSS 变量
2. 实现深色/浅色/跟随系统主题
3. 实现主题切换逻辑
4. 实现主题持久化

**CSS 文件**：`src/lib/styles/themes.css`

#### CORE-05 设置面板 UI

**任务描述**：实现设置面板界面

**执行步骤**：
1. 创建 SettingPanel.svelte 组件
2. 实现快捷键设置 UI
3. 实现主题切换 UI
4. 实现开机自启动 UI
5. 实现关于页面 UI

**Svelte 组件**：`src/lib/components/SettingPanel.svelte`

#### CORE-06 剪贴板服务

**任务描述**：实现剪贴板读写服务

**执行步骤**：
1. 集成 tauri-plugin-clipboard
2. 实现 clipboard Rust 模块
3. 实现文本读写接口
4. 实现图片读写接口（P1）
5. 实现文件路径读取（P1）

**Rust 模块**：`src-tauri/src/commands/clipboard.rs`

#### CORE-07 Shell 服务

**任务描述**：实现 Shell 命令执行服务

**执行步骤**：
1. 集成 tauri-plugin-shell
2. 实现 shell Rust 模块
3. 实现命令执行接口
4. 实现 URL 打开
5. 实现终端打开
6. 实现应用启动

**Rust 模块**：`src-tauri/src/commands/shell.rs`

#### CORE-08 数据存储服务

**任务描述**：实现用户配置和插件数据的存储

**执行步骤**：
1. 集成 tauri-plugin-store
2. 实现 DataStore Rust 模块
3. 实现用户配置存储
4. 实现插件数据存储
5. 实现数据迁移机制

**Rust 模块**：`src-tauri/src/commands/store.rs`

#### CORE-09 主界面布局

**任务描述**：实现主界面布局和组件

**执行步骤**：
1. 实现主页面路由
2. 实现 SearchBox 组件
3. 实现 ResultList 组件
4. 实现分类标签栏
5. 实现快捷键提示

**Svelte 组件**：
- `src/lib/components/SearchBox.svelte`
- `src/lib/components/ResultList.svelte`

#### CORE-10 搜索组件

**任务描述**：实现搜索功能和结果展示

**执行步骤**：
1. 实现搜索状态管理
2. 集成 tiny-fuzzy
3. 实现搜索结果高亮
4. 实现键盘导航
5. 实现搜索历史

**Svelte Store**：`src/lib/stores/search.ts`

#### CORE-11 集成测试

**任务描述**：对核心框架进行集成测试

**执行步骤**：
1. 编写集成测试用例
2. 测试窗口管理
3. 测试快捷键
4. 测试主题切换
5. 测试搜索功能
6. 修复发现的问题

#### CORE-12 文档更新

**任务描述**：更新 SRS.md 和 ARCHITECTURE.md

**执行步骤**：
1. 更新实现细节
2. 更新目录结构
3. 更新模块设计

---

## 风险评估

| 风险描述 | 可能性 | 影响程度 | 应对措施 |
|---------|-------|---------|---------|
| Tauri 2.x API 变更 | 中 | 高 | 封装抽象层，锁定版本 |
| 主题系统兼容性 | 低 | 中 | 使用标准 CSS 变量 |
| 快捷键冲突 | 中 | 低 | 提供冲突提示 |

---

## 资源需求

| 资源类型 | 资源名称 | 状态 |
|---------|---------|------|
| Tauri 插件 | tauri-plugin-global-shortcut | 待集成 |
| Tauri 插件 | tauri-plugin-clipboard | 待集成 |
| Tauri 插件 | tauri-plugin-shell | 待集成 |
| Tauri 插件 | tauri-plugin-store | 待集成 |
| 前端库 | tiny-fuzzy | 待集成 |

---

## 执行 Checkpoint

| Checkpoint | 计划日期 | 实际日期 | 状态 |
|-----------|---------|---------|------|
| 启动 | | | ❌ |
| 窗口管理系统完成 | | | ❌ |
| 快捷键系统完成 | | | ❌ |
| 主题系统完成 | | | ❌ |
| 搜索功能完成 | | | ❌ |
| 集成测试通过 | | | ❌ |

---

## 参考资料

- [SRS 功能需求规格说明书](../../wiki/SRS.md)
- [系统架构设计](../../wiki/ARCHITECTURE.md)
- [MVP-POC 技术原型验证计划](MVP-POC.md)

# Corelia MVP 开发指南

> **版本**: 1.0  
> **最后更新**: 2026-05-01  
> **目标读者**: Corelia 核心开发者  
> **适用阶段**: MVP 开发周期

---

## 目录

1. [MVP 目标与范围](#1-mvp-目标与范围)
2. [技术栈与开发环境](#2-技术栈与开发环境)
3. [项目架构与模块说明](#3-项目架构与模块说明)
4. [MVP 开发路线图](#4-mvp-开发路线图)
5. [关键模块开发指南](#5-关键模块开发指南)
6. [性能优化重点](#6-性能优化重点)
7. [测试与质量保证](#7-测试与质量保证)
8. [常见问题解答](#8-常见问题解答)

---

## 1. MVP 目标与范围

### 1.1 MVP 核心目标

MVP (Minimum Viable Product) 的核心目标是：

| 目标 | 说明 | 衡量标准 |
|------|------|----------|
| **功能完整** | 实现核心效率工具功能 | 7个QuickJS插件 + 1个Webview插件可工作 |
| **性能达标** | 保证用户体验流畅 | 冷启动<2s，搜索响应<50ms |
| **架构解耦** | 为未来优化留足空间 | 模块边界清晰，易于扩展 |
| **可交付** | 可以发布给早期用户使用 | 文档完整，安装流程顺畅 |

### 1.2 MVP 功能范围

#### ✅ 包含的功能

| 类别 | 功能项 | 状态 |
|------|--------|------|
| **核心功能** | 全局快捷键唤起（Alt+Space） | ✅ 已实现 |
| | 搜索框与结果列表 | ✅ 已实现 |
| | 系统功能搜索（应用/文件/URL） | ✅ 已实现 |
| | 设置面板 | ✅ 已实现 |
| **插件系统** | QuickJS插件加载与执行 | ✅ 已实现 |
| | 插件元数据扫描与懒加载 | ✅ 已实现 |
| | 插件API兼容（uTools API） | ✅ 已实现 |
| **系统集成** | 剪贴板读写 | ✅ 已实现 |
| | 文件系统操作 | ✅ 已实现 |
| | Shell命令执行 | ✅ 已实现 |
| | 窗口管理 | ✅ 已实现 |
| **配置管理** | 分层配置（System/User/App） | ✅ 已实现 |
| | 配置持久化 | ✅ 已实现 |
| **主题系统** | 深色/浅色/跟随系统 | ✅ 已实现 |

#### 📦 MVP 插件清单

| 序号 | 插件名称 | 类型 | 优先级 | 状态 |
|------|----------|------|--------|------|
| 1 | 快捷命令 | QuickJS | P0 | ⏳ 待开发 |
| 2 | 剪贴板增强 | QuickJS | P0 | ⏳ 待开发 |
| 3 | 计算器 | QuickJS | P0 | ⏳ 待开发 |
| 4 | 二维码工具 | QuickJS + WASM | P1 | ⏳ 待开发 |
| 5 | 文件搜索 | QuickJS | P0 | ⏳ 待开发 |
| 6 | 应用启动器 | QuickJS | P0 | ⏳ 待开发 |
| 7 | 截图增强 | QuickJS + WASM | P1 | ⏳ 待开发 |
| 8 | 云端同步 | Webview | P2 | ⏳ 待开发 |

#### ❌ 不包含在 MVP 中的功能

- ❌ macOS/Linux 跨平台支持（仅Windows）
- ❌ 插件市场与在线分发
- ❌ 完整的测试覆盖（仅核心功能测试）
- ❌ AI 深度集成（仅预留WASM补丁层）
- ❌ 插件热重载（MVP后优化）
- ❌ 多语言国际化

---

## 2. 技术栈与开发环境

### 2.1 技术栈详细说明

#### 前端技术栈

| 技术 | 版本 | 用途 | 注意事项 |
|------|------|------|----------|
| Svelte | 5.x | UI框架 | 优先使用Runes语法 |
| SvelteKit | 2.x | 应用框架 | 使用adapter-static |
| TypeScript | ~5.6.2 | 类型安全 | 严格模式 |
| Vite | 6.x | 构建工具 | 配合Bun使用 |
| Bun | 1.x+ | 包管理与运行时 | 替代npm |

#### 后端技术栈

| 技术 | 版本 | 用途 | 注意事项 |
|------|------|------|----------|
| Rust | 1.75+ | 系统编程 | 稳定版 |
| Tauri | 2.x | 桌面框架 | 注意2.x的API变化 |
| rquickjs | 0.11.0 | QuickJS绑定 | 关注Promise支持 |

### 2.2 开发环境配置

#### 前置要求

```bash
# Windows 10/11 x64
# Node.js 18+ (建议直接用Bun)
# Rust 1.75+
# Bun 1.x+
```

#### 环境配置步骤

```bash
# 1. 安装 Bun (如果未安装)
powershell -c "irm bun.sh/install.ps1 | iex"

# 2. 安装 Rust (如果未安装)
# 下载并运行 rustup-init.exe

# 3. 克隆项目
git clone <repository-url>
cd corelia

# 4. 安装依赖
bun install

# 5. 验证安装
bun run check          # 前端类型检查
cd src-tauri && cargo check  # 后端编译检查
```

#### 开发工作流命令

```bash
# 前端开发服务器 (仅前端)
bun run dev

# Tauri 完整开发模式 (推荐)
bun run tauri dev

# 类型检查
bun run check

# 构建生产版本
bun run tauri build
```

---

## 3. 项目架构与模块说明

### 3.1 项目目录结构

```
corelia/
├── src/                          # 前端源码
│   ├── lib/
│   │   ├── components/          # UI组件
│   │   │   ├── SearchBox.svelte
│   │   │   ├── ResultList.svelte
│   │   │   └── SettingPanel.svelte
│   │   ├── stores/              # 状态管理 (Svelte 5 Runes)
│   │   │   ├── search/          # 搜索状态（已拆分）
│   │   │   │   ├── index.ts    # 搜索编排器
│   │   │   │   ├── system.ts   # 系统搜索
│   │   │   │   ├── plugin.ts   # 插件搜索
│   │   │   │   └── merger.ts   # 结果合并
│   │   │   ├── theme.ts        # 主题状态
│   │   │   ├── history.ts      # 历史记录
│   │   │   └── ...
│   │   ├── services/            # 业务逻辑服务
│   │   │   ├── executor/       # 结果执行器（已拆分）
│   │   │   │   ├── index.ts    # 执行编排
│   │   │   │   ├── system.ts   # 系统功能执行
│   │   │   │   ├── setting.ts  # 设置项执行
│   │   │   │   └── plugin.ts   # 插件执行
│   │   │   ├── plugin.ts       # 插件服务（无VM缓存）
│   │   │   ├── window.ts       # 窗口服务
│   │   │   └── ...
│   │   ├── plugins/             # 插件相关
│   │   │   ├── service.ts      # 插件服务（纯代理）
│   │   │   ├── store.ts        # 插件状态
│   │   │   └── types.ts        # 类型定义
│   │   ├── search/              # 搜索相关
│   │   │   └── fuzzy.ts        # 模糊搜索（含拼音优化）
│   │   └── ...
│   └── routes/
│       ├── +page.svelte        # 主页面
│       └── +layout.ts          # 布局
├── src-tauri/                   # Rust后端
│   ├── src/
│   │   ├── commands/           # Tauri Commands层
│   │   │   ├── plugin.rs       # 插件命令
│   │   │   ├── window.rs       # 窗口命令
│   │   │   ├── clipboard.rs    # 剪贴板命令
│   │   │   └── ...
│   │   ├── services/           # 业务逻辑层
│   │   │   ├── plugin_service.rs
│   │   │   ├── window_service.rs
│   │   │   └── ...
│   │   ├── plugins/            # 插件系统核心
│   │   │   ├── quickjs_runtime.rs  # QuickJS运行时
│   │   │   ├── loader.rs       # 插件加载器
│   │   │   ├── registry.rs     # 插件注册表
│   │   │   ├── api_bridge.rs   # API桥接（兼容uTools）
│   │   │   ├── wasm_bridge.rs  # WASM桥接（含退避策略）
│   │   │   └── mod.rs
│   │   └── main.rs
│   └── Cargo.toml
├── plugins/                     # 插件目录
│   ├── hello-world/            # 示例插件
│   ├── calc/                   # 计算器插件
│   └── ...
├── patches/                     # WASM补丁目录
│   └── crypto/                 # 加密补丁示例
├── docs/                        # 文档目录
│   ├── PROJECT_ANALYSIS_REPORT.md  # 架构分析报告
│   ├── MVP_DEVELOPMENT_GUIDE.md    # 本文档
│   └── ...
└── wiki/                        # 旧文档（参考用）
```

### 3.2 前后端职责边界

这是 MVP 阶段最重要的架构原则！

#### 前端 (Svelte) 负责

| 职责 | 说明 |
|------|------|
| UI渲染 | 所有用户界面展示 |
| 状态管理 | 使用Svelte 5 Runes管理响应式状态 |
| 用户输入处理 | 键盘、鼠标事件处理 |
| 搜索逻辑 | 前端模糊搜索、结果排序展示 |
| 错误展示 | 用户友好的错误提示（Toast、文字） |
| WASM执行 | Webview中的WASM代码执行 |

#### 前端 **不** 应该做

| ❌ 禁止 | 说明 |
|---------|------|
| 管理VM | 插件VM生命周期完全由后端管理 |
| 持久化数据 | 通过后端API读写配置 |
| 直接文件系统 | 通过后端Commands操作 |

#### 后端 (Rust) 负责

| 职责 | 说明 |
|------|------|
| 系统能力封装 | 剪贴板、窗口、文件系统等 |
| 插件生命周期 | VM创建/销毁、加载/卸载 |
| 配置持久化 | 读写配置文件 |
| 错误记录与上报 | 日志记录，不直接展示给用户 |
| 资源池管理 | VM池、WASM实例池等 |

#### 后端 **不** 应该做

| ❌ 禁止 | 说明 |
|---------|------|
| UI逻辑 | 不包含任何界面展示逻辑 |
| 直接操作Webview | 通过事件与前端通信 |

---

## 4. MVP 开发路线图

### 4.1 开发阶段划分

| 阶段 | 时间 | 主要任务 | 交付物 |
|------|------|----------|--------|
| **Phase 1** | Week 1 | 架构优化与Bug修复 | 优化后的核心代码 |
| **Phase 2** | Week 2-3 | MVP插件开发 | 7个QuickJS插件 |
| **Phase 3** | Week 4 | Webview插件 + 测试 | 1个Webview插件 + 测试 |
| **Phase 4** | Week 5 | 文档 + 发布准备 | MVP可发布版本 |

### 4.2 Phase 1: 架构优化与Bug修复 (Week 1)

**目标**: 解决架构问题，为插件开发打好基础

| 任务 | 优先级 | 预计工时 | 负责人 |
|------|--------|----------|--------|
| 修复22个Svelte警告 | P0 | 2h | 前端开发 |
| 插件搜索并行化 | P0 | 2h | 前端开发 |
| 移除前端VM缓存 | P0 | 4h | 前端/后端 |
| WASM轮询退避策略 | P1 | 2h | 后端开发 |
| 拼音搜索短路优化 | P1 | 1h | 前端开发 |
| 文档重组与统一 | P1 | 8h | 全体 |

#### 任务详情：移除前端VM缓存

**问题**: 前端 `src/lib/plugins/service.ts` 中有VM缓存，与后端重复。

**解决方案**:
```typescript
// 修改前：有缓存
private vmCache: Map<string, VmCacheEntry> = new Map();

// 修改后：纯代理，直接调用后端
// 移除所有 vmCache 相关代码
// 所有VM操作直接通过 invoke 调用后端
```

**涉及文件**:
- `src/lib/plugins/service.ts` - 移除缓存逻辑
- `src/lib/components/PluginManager.svelte` - 更新状态管理

---

### 4.3 Phase 2-3: MVP插件开发 (Week 2-4)

#### P0优先级插件（优先开发）

1. **快捷命令** - 用户自定义命令脚本
2. **剪贴板增强** - 剪贴板历史、搜索
3. **计算器** - 基础数学运算
4. **文件搜索** - 本地文件快速定位
5. **应用启动器** - 快速启动已安装应用

#### P1优先级插件

6. **二维码工具** - 生成/解析二维码（需要crypto WASM补丁）
7. **截图增强** - 截图 + OCR（需要AI WASM补丁，MVP可能简化）

#### P2优先级插件

8. **云端同步** - Webview插件（MVP最后做或简化）

### 4.4 Phase 4: 文档与发布准备 (Week 5)

- 完善用户文档
- 完善插件开发者文档
- 最终测试
- 打包发布

---

## 5. 关键模块开发指南

### 5.1 QuickJS 插件开发

#### 插件结构

```
plugins/my-plugin/
├── plugin.json              # 插件元数据
└── index.js                 # 插件入口（可选）
```

#### plugin.json 格式

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "type": "quickjs",
  "logo": "logo.png",
  "description": "插件描述",
  "patches": ["crypto"],  // 依赖的WASM补丁
  "features": [
    {
      "code": "feature-1",
      "label": "功能名称",
      "explain": "功能说明",
      "cmd": ["mycmd", "/^mycmd\\s+/"]
    }
  ]
}
```

#### index.js 插件代码示例

```javascript
// 插件入口代码
window.onPluginEnter = ({ code, type, payload }) => {
  console.log('插件进入', code, payload);
};

// 搜索回调（可选）
window.onSearch = ({ query }) => {
  return [
    {
      title: '搜索结果1',
      description: '描述',
      icon: 'icon.png',
      action: () => {
        console.log('执行操作');
      }
    }
  ];
};

// 使用uTools兼容API
window.utools.dbStorage.setItem('key', 'value');
window.utools.hideMainWindow();
```

### 5.2 Webview 插件开发（MVP后期）

Webview插件开发文档将在MVP后期完善。

### 5.3 WASM 补丁开发

WASM补丁开发较为复杂，MVP阶段主要使用已有的crypto补丁。

---

## 6. 性能优化重点

### 6.1 MVP阶段必须做的性能优化

| 优化项 | 预期收益 | 优先级 |
|--------|----------|--------|
| 插件搜索并行化 | 搜索响应从O(n)降到O(1) | P0 |
| 拼音搜索短路 | 英文搜索跳过拼音计算 | P1 |
| 移除前端VM缓存 | 减少内存占用，避免状态不一致 | P0 |
| WASM轮询退避 | 减少CPU占用 | P1 |

### 6.2 性能监控指标

开发过程中需要关注：

| 指标 | 目标 | 测量方法 |
|------|------|----------|
| 冷启动时间 | < 2s | 从点击图标到窗口出现 |
| 搜索响应 | < 50ms | 输入到结果展示 |
| 插件加载 | QuickJS <10ms | 首次使用插件的加载时间 |
| 内存占用（空载） | < 100MB | 任务管理器查看 |

### 6.3 性能优化技巧

#### 前端优化

1. **使用Svelte 5 Runes** - 更精细的响应式控制
2. **组件懒加载** - 非关键组件按需加载
3. **避免不必要的响应式** - 不需要响应的状态不用$state
4. **搜索防抖** - 快速输入时减少搜索次数

#### 后端优化

1. **避免阻塞操作** - 文件IO等耗时操作用async
2. **资源池化** - VM、连接等复用
3. **懒加载** - 插件仅在使用时完整加载
4. **缓存策略** - 合理使用缓存，避免双重缓存

---

## 7. 测试与质量保证

### 7.1 MVP测试策略

MVP阶段不追求100%测试覆盖，重点保证：

| 测试类型 | 范围 | 目标 |
|----------|------|------|
| **手动测试** | 核心功能 | 保证可用 |
| **边界测试** | 错误处理 | 保证不崩溃 |
| **性能测试** | 关键路径 | 保证性能达标 |

### 7.2 代码质量检查

```bash
# 前端类型检查
bun run check

# Rust编译检查
cd src-tauri && cargo check

# Rust Clippy检查（重要！）
cd src-tauri && cargo clippy
```

### 7.3 MVP发布检查清单

- [ ] 所有P0插件开发完成
- [ ] 所有P0架构优化完成
- [ ] 无编译错误
- [ ] 无Clippy警告
- [ ] 无Svelte警告
- [ ] 性能测试达标
- [ ] 核心功能手动测试通过
- [ ] 文档完整
- [ ] 安装流程顺畅

---

## 8. 常见问题解答

### 8.1 开发环境问题

**Q: Bun 在 Windows 上执行策略错误怎么办？**

A: 以管理员身份运行PowerShell：
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

---

**Q: Rust 编译依赖下载很慢怎么办？**

A: 配置国内镜像源，在 `~/.cargo/config.toml` 添加：
```toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"
```

### 8.2 架构设计问题

**Q: 为什么要移除前端VM缓存？不是会更慢吗？**

A: 
1. **状态一致性** - 双重缓存容易导致状态不一致
2. **简化逻辑** - 减少出错可能
3. **性能影响极小** - IPC通信很快，用户感知不到
4. **单一职责** - VM管理完全由后端负责更清晰

---

**Q: MVP后可以大规模重构吗？**

A: 当然！MVP阶段我们特意保持了解耦设计，为未来优化留足空间。

---

## 附录

### A. 相关文档索引

- [PROJECT_ANALYSIS_REPORT.md](./PROJECT_ANALYSIS_REPORT.md) - 项目架构分析报告
- [ARCHITECTURE_OPTIMIZATION_REPORT.md](./analysis/ARCHITECTURE_OPTIMIZATION_REPORT.md) - 详细优化分析
- [components/](./components/) - 模块详细文档
- [wiki/SRS.md](../wiki/SRS.md) - 需求规格说明

### B. 联系与支持

- 项目问题: GitHub Issues
- 技术讨论: GitHub Discussions

---

*祝MVP开发顺利！🚀*

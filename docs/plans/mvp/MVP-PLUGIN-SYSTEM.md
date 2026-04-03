# MVP-PLUGIN-SYSTEM 插件系统实现计划

## 概述

本计划用于指导 MVP 插件系统实现阶段，包括 QuickJS 插件运行时、Webview 插件支持、WASM 补丁层。

---

## 基本信息

| 字段 | 内容 |
|------|------|
| **计划 ID** | MVP-PLUGIN-SYSTEM |
| **计划名称** | 插件系统实现 |
| **所属阶段** | MVP Phase 2 |
| **前置计划** | MVP-CORE-FRAMEWORK |
| **预计工期** | 4-5 周 |
| **状态** | 待开始 |

---

## 目标与验收标准

### 目标

实现 Corelia 的插件系统，包括：

1. **插件基础设施**：插件目录扫描、元数据解析、生命周期管理
2. **QuickJS 运行时**：VM 实例化、JSON 配置驱动、API 模拟
3. **Webview 插件支持**：iframe 嵌入、iframe 通信
4. **插件懒加载**：元数据预加载、按需加载、缓存管理
5. **插件 API**：window.utools 兼容、window.corelia 扩展
6. **WASM 补丁层**：crypto 和 AI 补丁集成

### 验收标准

| 序号 | 验收标准 | 验证方法 | 通过标准 |
|------|---------|---------|---------|
| 1 | 插件目录扫描正常 | 运行测试 | 插件正确识别 |
| 2 | QuickJS 插件加载 < 10ms | 性能测试 | 满足性能目标 |
| 3 | Webview 插件 iframe 正常 | 手动测试 | 页面正常显示 |
| 4 | Tauri Event 通信正常 | 运行测试 | 数据正确传递 |
| 5 | 插件懒加载机制正常 | 手动测试 | 按需加载 |
| 6 | window.utools API 可用 | 运行测试 | API 正常调用 |
| 7 | WASM crypto 补丁正常 | 运行测试 | 加密功能正常 |
| 8 | WASM AI 补丁正常 | 运行测试 | OCR 功能正常 |

---

## 任务拆解

### 任务列表

| 任务 ID | 任务名称 | 预计工时 | 依赖 | 优先级 |
|---------|---------|---------|------|--------|
| PLG-01 | 插件基础设施 | 16h | CORE-FRAMEWORK 完成 | P0 |
| PLG-02 | QuickJS 运行时 | 24h | PLG-01 | P0 |
| PLG-03 | Webview 插件支持 | 16h | PLG-01 | P0 |
| PLG-04 | 插件懒加载机制 | 16h | PLG-02, PLG-03 | P0 |
| PLG-05 | 插件 API 实现 | 24h | PLG-02, PLG-03 | P0 |
| PLG-06 | WASM 补丁层 | 24h | PLG-01 | P0 |
| PLG-07 | 插件管理 UI | 12h | PLG-01 | P0 |
| PLG-08 | 插件热重载 | 8h | PLG-01 | P1 |
| PLG-09 | 集成测试 | 16h | PLG-01~08 | P0 |
| PLG-10 | 文档更新 | 8h | PLG-09 | P1 |

### 详细任务说明

#### PLG-01 插件基础设施

**任务描述**：实现插件的基础加载和管理功能

**执行步骤**：
1. 实现插件目录扫描
2. 实现 plugin.json 解析
3. 实现插件元数据结构
4. 实现插件注册表
5. 实现插件生命周期管理

**Rust 模块**：
- `src-tauri/src/plugins/loader.rs`
- `src-tauri/src/plugins/registry.rs`

#### PLG-02 QuickJS 运行时

**任务描述**：实现 QuickJS 引擎集成和插件运行

**执行步骤**：
1. 引入 QuickJS 库
2. 实现 VM 实例化
3. 实现 JSON 配置驱动
4. 实现 window.utools API 模拟
5. 实现插件代码执行
6. 实现插件隔离

**Rust 模块**：`src-tauri/src/plugins/quickjs.rs`

**前端模块**：`src/lib/quickjs/vm.ts`

#### PLG-03 Webview 插件支持

**任务描述**：实现 Webview 插件的 iframe 嵌入和通信

**执行步骤**：
1. 实现 iframe 创建和管理
2. 实现 iframe 通信（Tauri Event）
3. 实现 iframe 生命周期
4. 实现 iframe 样式隔离
5. 实现多窗口预留接口

#### PLG-04 插件懒加载机制

**任务描述**：实现插件的懒加载和缓存管理

**执行步骤**：
1. 实现元数据预加载（启动时）
2. 实现按需加载触发
3. 实现插件缓存池
4. 实现内存管理策略
5. 实现卸载机制

**缓存状态机**：
| 状态 | 说明 | 触发条件 |
|------|------|----------|
| MetaLoaded | 仅元数据已加载 | 应用启动 |
| Loading | 正在加载完整插件 | 用户输入匹配到前缀 |
| Ready | 插件已就绪 | 资源加载完成 |
| Cached | 插件已缓存 | 加载完成后自动进入 |
| Unloaded | 插件已卸载 | 内存压力或手动卸载 |

#### PLG-05 插件 API 实现

**任务描述**：实现插件调用的 API 层

**执行步骤**：
1. 实现 window.utools 兼容 API
   - dbStorage
   - clipboard
   - shell
   - createBrowserWindow
2. 实现 window.corelia 扩展 API
   - wasm.call()
   - sync.config()
   - ai.chat()

**前端模块**：`src/lib/services/plugin-api.ts`

#### PLG-06 WASM 补丁层

**任务描述**：实现 WASM 补丁的加载和调用

**执行步骤**：
1. 创建 crypto WASM 补丁
   - Base64 编解码
   - MD5/SHA 哈希
   - AES 加密解密
2. 创建 AI WASM 补丁
   - OCR 图像识别
   - 图像处理
3. 实现补丁加载器
4. 实现补丁版本管理
5. 实现补丁缓存

**Rust 模块**：`src-tauri/src/patches/pool.rs`

**WASM 项目**：
- `patches/crypto-v1/`
- `patches/ai-v1/`

#### PLG-07 插件管理 UI

**任务描述**：实现插件管理界面

**执行步骤**：
1. 创建插件列表视图
2. 实现插件启用/禁用
3. 实现插件信息展示
4. 实现插件卸载
5. 实现插件数据清理

**Svelte 组件**：`src/lib/components/PluginManager.svelte`

#### PLG-08 插件热重载

**任务描述**：实现插件修改后的热重载

**执行步骤**：
1. 实现文件监听
2. 实现插件重新加载
3. 实现状态同步
4. 实现 UI 刷新

#### PLG-09 集成测试

**任务描述**：对插件系统进行集成测试

**执行步骤**：
1. 编写集成测试用例
2. 测试 QuickJS 插件
3. 测试 Webview 插件
4. 测试 WASM 补丁
5. 测试懒加载
6. 测试热重载
7. 修复发现的问题

#### PLG-10 文档更新

**任务描述**：更新插件开发相关文档

**执行步骤**：
1. 更新 SRS.md
2. 更新 ARCHITECTURE.md
3. 编写插件开发指南

---

## 插件 API 详细设计

### window.utools 兼容 API

```typescript
interface UtoolsAPI {
  dbStorage: {
    setItem(key: string, value: string): void;
    getItem(key: string): string | null;
    removeItem(key: string): void;
    clear(): void;
  };
  clipboard: {
    readText(): Promise<string>;
    writeText(text: string): Promise<void>;
  };
  shell: {
    openPath(path: string): Promise<void>;
    openExternal(url: string): Promise<void>;
    exec(command: string): Promise<string>;
  };
  createBrowserWindow(url: string, options?: object): void;
}
```

### window.corelia 扩展 API

```typescript
interface CoreliaAPI {
  wasm: {
    call(patch: string, api: string, ...args: any[]): Promise<any>;
  };
  sync: {
    config(): Promise<SyncConfig>;
  };
  ai: {
    chat(prompt: string, stream?: boolean): Promise<string>;
    ocr(image: string): Promise<string>;
  };
}
```

---

## 风险评估

| 风险描述 | 可能性 | 影响程度 | 应对措施 |
|---------|-------|---------|---------|
| QuickJS 内存泄漏 | 中 | 高 | 实现插件超时机制 |
| iframe 通信延迟 | 低 | 中 | 使用高效的 Tauri Event |
| WASM 加载慢 | 中 | 低 | AOT 预编译，缓存复用 |
| 插件版本冲突 | 低 | 中 | 版本兼容检查 |

---

## 资源需求

| 资源类型 | 资源名称 | 状态 |
|---------|---------|------|
| QuickJS | quickjs-emscripten | 待集成 |
| WASM | wasm-pack | 待配置 |
| Tauri | 2.x | 已安装 |

---

## 执行 Checkpoint

| Checkpoint | 计划日期 | 实际日期 | 状态 |
|-----------|---------|---------|------|
| 启动 | | | ❌ |
| 插件基础设施完成 | | | ❌ |
| QuickJS 运行时完成 | | | ❌ |
| Webview 支持完成 | | | ❌ |
| WASM 补丁层完成 | | | ❌ |
| 集成测试通过 | | | ❌ |

---

## 参考资料

- [SRS 功能需求规格说明书](../../wiki/SRS.md)
- [系统架构设计](../../wiki/ARCHITECTURE.md)
- [MVP-CORE-FRAMEWORK 核心框架实现计划](MVP-CORE-FRAMEWORK.md)
- [MVP-POC 技术原型验证计划](MVP-POC.md)

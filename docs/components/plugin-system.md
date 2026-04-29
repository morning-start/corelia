# 插件系统

> Corelia 插件系统提供完整的插件加载、运行、管理能力

## 概述

插件系统由前端和后端协同工作，支持 QuickJS 沙箱运行和 WASM Patch 加速的双层架构。

## 目录结构

```
src-tauri/src/plugins/
├── quickjs_runtime.rs    # QuickJS运行时
├── api_bridge/           # utools API桥接
│   ├── mod.rs
│   ├── clipboard.rs
│   ├── db_storage.rs
│   ├── dialog.rs
│   ├── fetch.rs
│   ├── fs.rs
│   ├── notification.rs
│   ├── path.rs
│   ├── process.rs
│   ├── shell.rs
│   ├── wasm.rs
│   ├── window.rs
│   └── context.rs
├── loader/               # 插件加载器
│   ├── mod.rs
│   ├── scanner.rs        # 插件扫描
│   ├── manifest.rs       # 元数据处理
│   ├── lifecycle.rs      # 生命周期管理
│   ├── instance.rs       # 实例管理
│   ├── state.rs          # 状态机
│   └── health.rs         # 健康检查
├── registry.rs           # 插件注册表
├── wasm_bridge.rs        # WASM调用桥接
└── mod.rs
```

## QuickJS 运行时

**职责**：创建和管理 QuickJS 虚拟机实例，提供沙箱执行环境

**核心功能**：
- VM 实例创建和销毁
- VM 池化管理
- 代码执行和隔离
- 内存限制和超时控制
- 闲置 VM 回收

**关键参数**：
- 单个 VM 最大内存：50MB
- 执行超时：5秒
- 最大并发 VM：10个
- 闲置超时：5分钟

## ApiBridge

**职责**：将 Rust 系统能力注入到 QuickJS VM，提供 utools 兼容 API

**核心 API**：
- `dbStorage` - 插件隔离存储
- `clipboard` - 剪贴板操作
- `shell` - Shell 操作、文件打开
- `window` - 窗口控制
- `getPath` - 系统路径获取
- `fs` - 文件系统操作
- `fetch` - HTTP 请求
- `dialog` - 对话框
- `process` - 子进程、设备信息
- `notification` - 系统通知
- `wasm` - WASM 调用
- `context` - 上下文管理

## 插件加载器

**职责**：管理插件的完整生命周期

**状态机**：
```
MetaLoaded → Loading → Ready
    ↑              ↓
    └── Unloaded ←── Cached / Error
```

**核心流程**：
1. **扫描** - 解析 plugins/ 目录下所有 plugin.json
2. **加载** - 创建 VM、注入 API、执行插件代码
3. **缓存** - 闲置插件进入缓存状态
4. **卸载** - 销毁 VM，清理资源

## 插件注册表

**职责**：管理所有插件的索引和状态

**双重索引**：
- ID 精确查找
- 前缀模糊搜索

## WasmBridge

**职责**：桥接 QuickJS VM 和 WebView 之间的 WASM 调用

**调用流程**：
1. 插件调用 `utools.wasm.__wasm_call`
2. 后端生成唯一 requestId
3. 触发 `wasm-call` 事件到前端
4. 前端执行 WASM 函数
5. 前端调用 `wasm_store_call_result` 保存结果
6. 插件通过 `utools.wasm.__wasm_get_result` 获取结果

## Commands

**暴露给前端的命令**：
- `scan_plugins` - 扫描插件
- `load_plugin` - 加载插件
- `unload_plugin` - 卸载插件
- `find_plugins_by_prefix` - 前缀搜索插件
- `quickjs_*` - VM 管理命令
- `wasm_*` - WASM 桥接命令

## 相关文档

- [插件开发](../wiki/PLUGIN_SYSTEM.md)
- [API参考](../wiki/API.md)

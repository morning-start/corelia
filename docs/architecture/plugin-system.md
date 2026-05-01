# 插件系统架构

> Corelia 三层插件架构设计

---

## 三层插件架构

Corelia 采用三层插件架构，支持不同复杂度的插件场景：

```
┌─────────────────────────────────────────────────────────────┐
│                    Corelia 插件生态                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────────────────────────────────────────────────┐  │
│   │              插件应用层 (Plugin Application)           │  │
│   │                                                       │  │
│   │  ┌─────────────────┐     ┌─────────────────────┐   │  │
│   │  │   QuickJS 插件   │     │    Webview 插件      │   │  │
│   │  │                 │     │                      │   │  │
│   │  │ • 轻量快速       │     │ • Vue/React/Svelte  │   │  │
│   │  │ • JSON 配置驱动  │     │ • 自定义复杂 UI      │   │  │
│   │  │ • 标准模板       │     │ • HTML/CSS/JS 完整包 │   │  │
│   │  │                 │     │                      │   │  │
│   │  │  懒加载 ✅       │     │  懒加载 ✅           │   │  │
│   │  └─────────────────┘     └─────────────────────┘   │  │
│   └─────────────────────────────────────────────────────┘  │
│                              │                              │
│                              ▼                              │
│   ┌─────────────────────────────────────────────────────┐  │
│   │              WASM 补丁层 (WASM Patches)                │  │
│   │                                                       │  │
│   │   ┌──────────┐  ┌──────────┐  ┌──────────┐        │  │
│   │   │   AI     │  │  Crypto  │  │  Base64  │  ...   │  │
│   │   │ 补丁-v1   │  │ 补丁-v1   │  │  补丁-v1  │        │  │
│   │   └──────────┘  └──────────┘  └──────────┘        │  │
│   │                                                       │  │
│   │   按需加载 ✅  •  版本兼容检查  •  缓存复用          │  │
│   └─────────────────────────────────────────────────────┘  │
│                              │                              │
│                              ▼                              │
│   ┌─────────────────────────────────────────────────────┐  │
│   │              Rust 核心 API 层 (常驻内存)                 │  │
│   │   crypto • network • fs • clipboard • shell          │  │
│   │   window • db • shortcut                             │  │
│   └─────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 插件类型对比

| 特性 | QuickJS 插件 | Webview 插件 | WASM 补丁 |
|------|-------------|--------------|-----------|
| **加载方式** | 懒加载 | 懒加载 | 按需加载 |
| **响应时间** | ~7ms | ~200-350ms | ~200-400ms |
| **内存占用** | ~1-2MB | ~30-50MB | ~5-20MB |
| **适用场景** | 简单功能 | 复杂UI | 高性能计算 |
| **开发难度** | 低 | 中 | 高 |

---

## QuickJS 插件

### 特点

- **轻量快速** - 启动仅 ~7ms
- **JSON 配置驱动** - 无需复杂代码
- **沙箱隔离** - 安全可控
- **兼容 uTools API** - 降低迁移成本

### 插件结构

```
plugins/my-plugin/
├── plugin.json     # 插件元数据
└── index.js        # 插件入口（可选）
```

### plugin.json 格式

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "type": "quickjs",
  "logo": "logo.png",
  "description": "插件描述",
  "patches": ["crypto"],
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

### 生命周期

| 状态 | 说明 | 触发条件 |
|------|------|----------|
| `MetaLoaded` | 仅元数据已加载 | 应用启动扫描 |
| `Loading` | 正在加载完整插件 | 用户匹配到前缀 |
| `Ready` | 插件已就绪 | 加载完成 |
| `Cached` | 插件已缓存 | 自动进入 |
| `Unloaded` | 插件已卸载 | 内存压力或手动卸载 |

---

## Webview 插件

### 特点

- **完整 UI 能力** - 支持 Vue/React/Svelte
- **复杂交互** - 表单、可视化等
- **独立环境** - iframe 隔离

### 插件结构

```
plugins/my-webview-plugin/
├── plugin.json
├── index.html
├── assets/
│   ├── app.js
│   └── styles.css
└── public/
    └── logo.png
```

### MVP 状态

MVP 阶段 Webview 插件支持简化，主要实现：
- 基础 iframe 嵌入
- API 桥接
- 通信机制

---

## WASM 补丁

### 特点

- **高性能** - 原生代码执行
- **安全隔离** - 沙箱环境
- **按需加载** - 节省资源

### 补丁结构

```
patches/crypto/
├── patch.json
├── crypto.wasm
└── api.d.ts        # 类型定义（可选）
```

### patch.json 格式

```json
{
  "name": "crypto-v1",
  "version": "1.0.0",
  "description": "加密算法补丁",
  "apis": ["encrypt", "decrypt", "hash"],
  "size": "2.5MB"
}
```

### WASM 桥接机制

由于 QuickJS 无法直接运行 WASM，采用桥接模式：

```
QuickJS 插件
    │
    │ 调用 window.utools.wasm.call()
    ▼
Rust ApiBridge
    │
    │ Tauri Event
    ▼
Webview (WASM 执行)
    │
    │ 返回结果
    ▼
Rust WasmBridge
    │
    │ 存储结果
    ▼
QuickJS 获取结果
```

---

## API 兼容性

### uTools 兼容 API

Corelia 兼容 uTools 核心 API：

```typescript
// 存储
window.utools.dbStorage.setItem(key, value)
window.utools.dbStorage.getItem(key)

// 剪贴板
window.utools.clipboard.readText()
window.utools.clipboard.writeText(text)

// 窗口
window.utools.hideMainWindow()
window.utools.showMainWindow()

// Shell
window.utools.shellOpenExternal(url)
```

### Corelia 扩展 API

```typescript
// WASM 调用
window.corelia.wasm.call(patch, api, ...args)

// 同步配置
window.corelia.sync.config()
```

---

## 核心模块

### QuickJS 运行时

位置: `src-tauri/src/plugins/quickjs_runtime.rs`

功能：
- VM 实例创建/销毁
- VM 池管理（最大10个，闲置300秒回收）
- JS 代码执行

### 插件加载器

位置: `src-tauri/src/plugins/loader.rs`

功能：
- 插件扫描
- 生命周期管理
- VM 关联

### API 桥接

位置: `src-tauri/src/plugins/api_bridge.rs`

功能：
- uTools API 注入
- 系统能力暴露
- WASM 桥接

---

## 竞品对比

| 特性 | Corelia | uTools | ZTools |
|------|---------|--------|--------|
| **插件架构** | 三层架构 | Webview为主 | Webview + DLL |
| **JS引擎** | QuickJS (沙箱) | Node.js | 自研 |
| **性能** | 轻量快速 | 较重 | 中等 |
| **兼容性** | 兼容uTools API | 自有API | 自有API |
| **扩展性** | WASM补丁层 | Node.js生态 | DLL插件 |

---

## 相关文档

- [性能分析](./performance.md)
- [安全性分析](./security.md)
- [插件开发指南](../development/plugin-development.md)

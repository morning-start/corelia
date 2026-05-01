# 竞品分析

> uTools 与 ZTools 对比分析

---

## 竞品概览

| 产品 | 定位 | 用户量 | 插件数量 |
|------|------|--------|----------|
| **uTools** | 效率工具平台 | 500万+ | 500+ |
| **ZTools** | 免费效率工具 | 1万+ | 20+ |
| **Corelia** | 轻量效率工具 | 开发中 | MVP阶段 |

---

## uTools 分析

### 核心特点

- **一切皆插件** - 核心功能也是插件实现
- **跨平台** - Windows/macOS/Linux
- **AI 集成** - 内置 AI 助手，支持 MCP 协议
- **成熟生态** - 500+ 插件

### 技术栈

| 层级 | 技术 |
|------|------|
| 框架 | Electron |
| 前端 | Vue/React 等 |
| 运行时 | Node.js |

### 插件 API

```javascript
// 存储
window.utools.dbStorage.setItem(key, value)

// 剪贴板
window.utools.clipboard.readText()
window.utools.clipboard.writeText(text)

// 窗口
window.utools.hideMainWindow()
window.utools.showMainWindow()

// Shell
window.utools.shellOpenExternal(url)
```

### 优势

1. 用户基数大，生态成熟
2. 插件丰富，覆盖多种场景
3. AI 布局早，支持 MCP
4. 开发体验好，有官方工具

### 劣势

1. Electron 较重，内存占用高
2. 部分高级功能收费

---

## ZTools 分析

### 核心特点

- **完全免费** - 无广告，永久免费
- **DLL 插件** - 支持原生扩展
- **PouchDB** - 完整数据库支持
- **AI 内置** - 内置 AI 模型调用

### 技术栈

| 层级 | 技术 |
|------|------|
| 框架 | Delphi + CEF |
| 前端 | HTML/CSS/JS |
| 扩展 | DLL 插件 |

### 插件 API

```javascript
// 数据库
ztools.db.put(doc)
ztools.db.get(id)

// AI
ztools.ai({ prompt: '你好' })

// 剪贴板历史
ztools.clipboard.getHistory(page, pageSize)
```

### 优势

1. 完全免费无广告
2. DLL 插件支持，扩展性强
3. 完整数据库支持
4. 输入模拟 API

### 劣势

1. 用户量较小
2. 插件数量少
3. 跨平台支持较弱

---

## Corelia 对比

### 技术对比

| 维度 | Corelia | uTools | ZTools |
|------|---------|--------|--------|
| **框架** | Tauri | Electron | Delphi+CEF |
| **内存占用** | ~40-100MB | ~150MB+ | ~180MB |
| **启动速度** | 快 | 较慢 | 中等 |
| **插件架构** | 三层架构 | Webview | Webview+DLL |
| **JS引擎** | QuickJS | Node.js | 自研 |

### 功能对比

| 功能 | Corelia | uTools | ZTools |
|------|---------|--------|--------|
| 全局快捷键 | ✅ | ✅ | ✅ |
| 模糊搜索 | ✅ | ✅ | ✅ |
| 插件系统 | ✅ | ✅ | ✅ |
| WASM 扩展 | ✅ | ❌ | ❌ |
| DLL 扩展 | ❌ | ❌ | ✅ |
| AI 集成 | 规划中 | ✅ | ✅ |
| 跨平台 | Windows | 全平台 | 全平台 |

### Corelia 优势

1. **轻量快速** - Tauri 比 Electron 更轻
2. **三层架构** - QuickJS + Webview + WASM
3. **安全沙箱** - 插件隔离更安全
4. **兼容性好** - 兼容 uTools API

### Corelia 差异化

1. **QuickJS 插件** - 7ms 响应，比 Webview 快 30 倍
2. **WASM 补丁层** - 高性能原生扩展
3. **开源免费** - MIT 协议

---

## 借鉴要点

### 从 uTools 学习

1. **插件 API 设计** - 简洁易用
2. **开发者工具** - 官方调试工具
3. **插件分发** - GitHub Release + 拖拽安装

### 从 ZTools 学习

1. **数据库支持** - PouchDB 完整方案
2. **剪贴板历史** - 完整的剪贴板管理
3. **主搜索推送** - onMainPush 机制

---

## 相关文档

- [插件系统架构](../architecture/plugin-system.md)
- [技术栈分析](../architecture/tech-stack.md)

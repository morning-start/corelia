# 开发文档

> Corelia 开发指南

---

## 文档列表

| 文档 | 说明 |
|------|------|
| [环境配置](./environment.md) | 开发环境搭建 |
| [MVP 指南](./mvp-guide.md) | MVP 开发核心指南 |
| [插件开发](./plugin-development.md) | QuickJS/Webview 插件开发 |
| [代码规范](./conventions.md) | 代码风格与规范 |

---

## 快速开始

### 安装依赖

```bash
bun install
```

### 开发模式

```bash
bun run tauri dev
```

### 类型检查

```bash
bun run check
```

---

## 项目结构

```
corelia/
├── src/                    # 前端源码
│   ├── lib/
│   │   ├── components/    # UI 组件
│   │   ├── stores/        # 状态管理
│   │   └── services/      # 业务逻辑
│   └── routes/            # 页面路由
├── src-tauri/             # Rust 后端
│   └── src/
│       ├── commands/      # Tauri Commands
│       ├── services/      # 业务服务
│       └── plugins/       # 插件系统
├── plugins/               # 插件目录
└── docs/                  # 文档目录
```

---

## 相关文档

- [架构文档](../architecture/) - 架构设计详解
- [竞品分析](../reference/competitors.md) - uTools/ZTools 对比

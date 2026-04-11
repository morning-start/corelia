# Corelia Wiki 索引

欢迎使用 Corelia Wiki！这里包含了 Corelia 快速启动器的完整文档。

## 📚 文档导航

### 入门文档

| 文档 | 说明 | 适合人群 |
|------|------|----------|
| [README.md](README.md) | 项目介绍和快速开始 | 所有用户 |
| [DEVELOPMENT.md](DEVELOPMENT.md) | 开发环境搭建和开发指南 | 开发者 |
| [DEPLOYMENT.md](DEPLOYMENT.md) | 构建和部署指南 | 开发者/运维 |

### 技术文档

| 文档 | 说明 | 适合人群 |
|------|------|----------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | 系统架构设计详解 | 开发者/架构师 |
| [API.md](API.md) | Tauri Commands API 文档 | 前端开发者 |

### 专项文档

| 文档 | 说明 | 适合人群 |
|------|------|----------|
| [DPI 缩放指南](../dev/dpi-scaling-guide.md) | DPI 适配和窗口尺寸处理 | 开发者 |

## 🚀 快速开始

### 新用户

1. 阅读 [README.md](README.md) 了解项目
2. 下载安装包并安装
3. 按下 `Alt + Space` 呼出窗口
4. 开始搜索和使用

### 新开发者

1. 阅读 [DEVELOPMENT.md](DEVELOPMENT.md) 搭建环境
2. 克隆项目并安装依赖
3. 启动开发服务器
4. 开始贡献代码

### 运维人员

1. 阅读 [DEPLOYMENT.md](DEPLOYMENT.md)
2. 选择目标平台
3. 执行构建命令
4. 分发安装包

## 📖 文档结构

```
docs/wiki/
├── INDEX.md                  # 本文件 - 索引
├── README.md                 # 项目介绍
├── ARCHITECTURE.md           # 系统架构
├── DEVELOPMENT.md            # 开发指南
├── DEPLOYMENT.md             # 部署指南
└── API.md                    # API 文档
```

## 🎯 按主题查找

### 项目概览

- [什么是 Corelia？](README.md#核心功能)
- [技术栈是什么？](README.md#技术架构)
- [有哪些特性？](README.md#特性)

### 安装使用

- [如何安装？](README.md#安装)
- [如何使用？](README.md#快速开始)
- [如何配置？](README.md#配置说明)

### 开发相关

- [如何搭建开发环境？](DEVELOPMENT.md#开发环境搭建)
- [如何添加新功能？](DEVELOPMENT.md#添加新功能)
- [如何调试？](DEVELOPMENT.md#调试技巧)

### 架构设计

- [整体架构是什么？](ARCHITECTURE.md#整体架构)
- [前端架构如何？](ARCHITECTURE.md#前端架构)
- [后端架构如何？](ARCHITECTURE.md#后端架构)

### API 参考

- [如何调用 Tauri 命令？](API.md#调用方式)
- [窗口控制 API](API.md#窗口控制-api)
- [配置管理 API](API.md#配置管理-api)

### 部署发布

- [如何构建？](DEPLOYMENT.md#生产构建)
- [如何部署到 Windows？](DEPLOYMENT.md#31-windows)
- [如何部署到 macOS？](DEPLOYMENT.md#32-macos)
- [如何部署到 Linux？](DEPLOYMENT.md#33-linux)

## 🔧 常用命令

### 开发

```bash
# 启动开发服务器
bun run tauri dev

# 类型检查
bun run check

# 代码格式化
bun run format
```

### 构建

```bash
# 生产构建
bun run tauri build

# 清理构建产物
cargo clean
rm -rf node_modules/.vite
```

### 测试

```bash
# 运行测试
bun run test

# 检查代码质量
cargo clippy
```

## 📞 获取帮助

### 遇到问题？

1. 查看 [常见问题](DEVELOPMENT.md#常见问题)
2. 查看 [故障排除](DEPLOYMENT.md#故障排除)
3. 提交 [Issue](https://github.com/your-org/corelia/issues)

### 贡献代码

1. Fork 项目
2. 创建特性分支
3. 提交更改
4. 推送并创建 Pull Request

详见 [贡献指南](../CONTRIBUTING.md)

## 🔗 相关链接

### 官方资源

- [项目主页](https://github.com/your-org/corelia)
- [问题追踪](https://github.com/your-org/corelia/issues)
- [发布版本](https://github.com/your-org/corelia/releases)

### 技术文档

- [Tauri v2 文档](https://v2.tauri.app)
- [Svelte 5 文档](https://svelte.dev/docs/svelte)
- [Rust 文档](https://doc.rust-lang.org)

### 社区

- [Tauri Discord](https://discord.com/invite/tauri)
- [Svelte Discord](https://svelte.dev/chat)
- [Rust Forum](https://users.rust-lang.org)

## 📝 文档更新记录

| 日期 | 文档 | 更新内容 |
|------|------|----------|
| 2026-04-10 | 所有文档 | 初始版本创建 |

## 🎓 学习路径

### 初级开发者

1. ✅ 阅读 README.md
2. ✅ 搭建开发环境
3. ✅ 运行示例项目
4. ✅ 阅读 API.md 了解基本用法

### 中级开发者

1. ✅ 阅读 ARCHITECTURE.md 理解架构
2. ✅ 添加简单功能
3. ✅ 阅读 DEVELOPMENT.md 学习最佳实践
4. ✅ 贡献代码

### 高级开发者

1. ✅ 深入理解 ARCHITECTURE.md
2. ✅ 优化性能
3. ✅ 设计新功能
4. ✅ 指导其他开发者

## 📊 文档状态

| 文档 | 状态 | 完成度 |
|------|------|--------|
| README.md | ✅ 完成 | 100% |
| ARCHITECTURE.md | ✅ 完成 | 100% |
| DEVELOPMENT.md | ✅ 完成 | 100% |
| DEPLOYMENT.md | ✅ 完成 | 100% |
| API.md | ✅ 完成 | 100% |

## 💡 提示

- 使用 `Ctrl + F` 快速搜索文档内容
- 文档中的代码示例可以直接复制使用
- 遇到问题先查看常见问题章节
- 欢迎提交 Issue 改进文档

---

**最后更新**: 2026-04-10  
**维护者**: Corelia Team  
**版本**: v0.1.0

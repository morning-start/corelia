# Corelia

> 一个基于 Tauri v2 + Svelte 5 + Rust 的高性能快速启动器

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2.0-24C8DB.svg)](https://tauri.app)
[![Svelte](https://img.shields.io/badge/Svelte-v5.0-FF3E00.svg)](https://svelte.dev)
[![Rust](https://img.shields.io/badge/Rust-v1.94.0-dea584.svg)](https://www.rust-lang.org)

## ✨ 特性

- 🚀 **快速启动** - 全局快捷键秒开，无需等待
- 🎨 **现代设计** - 参考 Raycast/Alfred 的专业设计
- 🔍 **智能搜索** - 模糊匹配，快速定位
- 🖥️ **跨平台** - Windows、macOS、Linux 全支持
- 📦 **轻量级** - 二进制文件小，内存占用低
- 🛡️ **安全** - 基于 Rust，类型安全
- 🎯 **DPI 适配** - 高 DPI 屏幕完美显示

## 📦 安装

### 从源码构建

**环境要求**:
- Bun >= 1.3.0
- Rust >= 1.94.0
- Node.js >= 18.0.0

```bash
# 克隆项目
git clone https://github.com/your-org/corelia.git
cd corelia

# 安装依赖
bun install

# 开发模式
bun run tauri dev

# 生产构建
bun run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`

## 🚀 快速开始

1. **启动应用**: 双击运行或使用命令行启动
2. **呼出窗口**: 按下 `Alt + Space` (可自定义)
3. **搜索内容**: 输入应用名、命令或文件
4. **执行操作**: 使用方向键选择，Enter 执行

## 📖 功能

### 核心功能

- **应用启动**: 快速搜索并启动系统应用
- **文件搜索**: 模糊匹配文件名和路径
- **命令执行**: 支持系统命令和自定义命令
- **URL 打开**: 快速访问网页链接
- **搜索历史**: 记录最近搜索，方便重复使用
- **分类筛选**: 按系统/插件/历史分类浏览

### 高级功能

- **全局快捷键**: 自定义快捷键呼出窗口
- **自动隐藏**: 失焦后自动隐藏窗口
- **系统托盘**: 托盘图标快速控制
- **开机自启**: 支持开机自动启动
- **主题切换**: 支持深色/浅色/系统主题

## 🏗️ 技术架构

```
┌─────────────────────────────────────────┐
│           Frontend (Svelte 5)           │
│  ┌──────────┬──────────┬────────────┐  │
│  │Components│  Stores  │  Services  │  │
│  └──────────┴──────────┴────────────┘  │
└─────────────────┬───────────────────────┘
                  │ Tauri Commands
┌─────────────────▼───────────────────────┐
│           Backend (Rust)                │
│  ┌──────────┬──────────┬────────────┐  │
│  │Commands  │ Services │  Plugins   │  │
│  └────────────────────┴────────────  │
└─────────────────────────────────────────┘
```

**技术栈**:
- **前端**: Svelte 5 + TypeScript + Vite
- **后端**: Rust + Tauri v2
- **状态管理**: Svelte Stores + Rust AtomicBool
- **搜索算法**: Fuzzy 模糊匹配
- **数据存储**: tauri-plugin-store

## 📂 项目结构

```
corelia/
├── src/                    # 前端源码
│   ├── routes/            # 页面路由
│   ├── lib/               # 核心库
│   │   ├── components/    # Svelte 组件
│   │   ├── services/      # 前端服务
│   │   ├── stores/        # 状态管理
│   │   ├── search/        # 搜索算法
│   │   ├── utils/         # 工具函数
│   │   └── styles/        # 样式
│   └── app.html           # HTML 模板
│
├── src-tauri/             # 后端源码
│   ├── src/
│   │   ├── commands/      # Tauri 命令层
│   │   ├── services/      # 业务服务层
│   │   ├── lib.rs         # 应用入口
│   │   └── main.rs        # Rust 入口
│   ├── tauri.conf.json    # Tauri 配置
│   └── Cargo.toml         # Rust 依赖
│
├── docs/                  # 文档
│   ├── wiki/             # Wiki 文档
│   └── spec/             # 规格说明
│
└── package.json          # 前端依赖
```

## 🎯 配置说明

### 窗口配置

```typescript
// src/lib/config.ts
export const WINDOW_CONFIG = {
  WIDTH: 600,      // 窗口宽度（逻辑像素）
  HEIGHT: 420,     // 窗口高度（逻辑像素）
  MIN_WIDTH: 200,  // 最小宽度
  MIN_HEIGHT: 300, // 最小高度
  MAX_WIDTH: 1200, // 最大宽度
  MAX_HEIGHT: 900, // 最大高度
}
```

### 搜索配置

```typescript
export const SEARCH_CONFIG = {
  MAX_HISTORY_ITEMS: 5,     // 搜索历史最大条目数
  DEBOUNCE_DELAY: 150,      // 搜索防抖延迟（毫秒）
}
```

### 性能配置

```typescript
export const PERFORMANCE_CONFIG = {
  DELAY_UNPIN_MS: 100,      // 延迟取消置顶时间（毫秒）
}
```

## 🔧 开发指南

### 添加新命令

1. 在 `src-tauri/src/commands/` 创建命令文件
2. 在 `src-tauri/src/services/` 实现业务逻辑
3. 在 `src-tauri/src/lib.rs` 注册命令
4. 在前端使用 `invoke()` 调用

### 添加新组件

1. 在 `src/lib/components/` 创建 `.svelte` 文件
2. 使用 Svelte 5 Runes 语法 (`$state`, `$props`)
3. 在需要的地方导入使用

### 修改配置

1. 在 `src/lib/config.ts` 添加配置项
2. 在 `src-tauri/tauri.conf.json` 修改 Tauri 配置
3. 重启开发服务器生效

## 📝 常用命令

```bash
# 开发
bun run tauri dev

# 构建
bun run tauri build

# 类型检查
bun run check

# 代码格式化
bun run format

# 清理构建产物
cargo clean
rm -rf node_modules/.vite
```

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 提交 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [Tauri](https://tauri.app) - 桌面应用框架
- [Svelte](https://svelte.dev) - 前端框架
- [Fuzzy](https://github.com/simple-icons/simple-icons) - 模糊搜索库
- [Raycast](https://www.raycast.com) - 设计灵感

## 📞 联系方式

- 项目主页：[GitHub](https://github.com/your-org/corelia)
- 问题反馈：[Issues](https://github.com/your-org/corelia/issues)

---

**最后更新**: 2026-04-10  
**版本**: v0.1.0

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-01

### Added

#### Phase 1: 架构优化
- ✅ 拼音搜索短路优化 - 英文搜索跳过拼音转换
- ✅ 移除前端 VM 缓存 - 改为纯代理模式，避免双重缓存
- ✅ WASM 轮询退避策略 - 10ms→20ms→40ms→100ms 指数退避

#### Phase 2: MVP 插件开发 (P0)
- ✅ **Shortcuts** (快捷命令插件) - 保存和执行自定义命令
  - 前缀: `sc`
  - 支持命令分类、数据持久化
  - 默认命令: 打开终端、文件浏览器、任务管理器

- ✅ **Clipboard** (剪贴板增强插件) - 剪贴板历史和格式转换
  - 前缀: `cb`
  - 历史记录最多 50 条
  - 支持大小写转换、Base64 编解码

- ✅ **Calculator** (计算器插件) - 数学计算和单位换算
  - 前缀: `calc`
  - 支持数学表达式求值
  - 支持长度/重量/温度/存储单位换算
  - 支持百分比计算

- ✅ **File Search** (文件搜索插件) - 本地文件快速定位
  - 已有插件，功能完善

- ✅ **App Launcher** (应用启动器插件) - 快速打开常用应用
  - 前缀: `app`
  - 支持最近使用记录
  - 默认应用列表: Chrome、VS Code、Terminal 等

#### Phase 3: P1 插件开发
- ✅ **QR Code** (二维码工具插件) - 生成和解析二维码
  - 前缀: `qr`
  - 支持文本、URL、WiFi、联系人格式
  - 剪贴板快速生成

- ✅ **Screenshot** (截图增强插件) - 截图和历史管理
  - 前缀: `ss`
  - 支持区域截图、全屏截图
  - 历史记录最多 20 条

### Changed
- 优化拼音搜索性能，英文搜索直接跳过拼音转换
- 简化前端插件服务层，移除 VM 缓存逻辑
- 更新 README 文档，反映 MVP 完成状态

### Fixed
- 无 Svelte 警告
- 构建成功无错误

### Technical Details

#### 性能优化
| 优化项 | 优化前 | 优化后 |
|--------|--------|--------|
| 英文搜索 | 执行拼音转换 | 跳过转换，直接匹配 |
| VM 缓存 | 前后端双重缓存 | 后端统一管理 |
| WASM 轮询 | 固定间隔 | 指数退避 10-100ms |

#### 插件统计
- QuickJS 插件: 10 个
- P0 优先级: 5 个
- P1 优先级: 2 个
- 其他: 3 个

---

## [0.0.1] - 2026-04-XX

### Added
- 初始化项目结构
- Tauri 2.x + Svelte 5 架构搭建
- QuickJS 运行时集成
- 基础插件系统实现

# 前端架构

> 前端基于 SvelteKit，负责 UI 展示、状态管理和搜索逻辑

## 目录结构

```
src/
├── lib/
│   ├── components/       # UI 组件
│   ├── plugins/          # 插件前端层
│   ├── stores/           # 状态管理
│   ├── services/         # 前端服务
│   ├── search/           # 搜索系统
│   └── api.ts            # Tauri API 代理
├── routes/
│   ├── +page.svelte      # 主页面
│   └── +layout.ts        # 布局
└── app.html
```

## UI 组件

### 核心组件

| 组件 | 职责 |
|------|------|
| SearchBox | 搜索输入框、关键词监听、焦点管理 |
| ResultList | 搜索结果渲染、交互处理、结果选择 |
| SettingPanel | 设置面板、用户偏好配置 |
| PluginManager | 插件管理界面 |
| CategoryTabs | 分类标签、结果分类切换 |
| TitleBar | 标题栏、窗口控制 |
| ShortcutRecorder | 快捷键录制、配置 |
| HighlightedText | 文本高亮展示 |

## 状态管理

基于 Svelte 5 Runes 的响应式状态管理。

### 核心 Stores

| Store | 职责 |
|-------|------|
| searchStore | 搜索查询、结果状态 |
| theme | 主题配置（深色/浅色/系统） |
| system | 系统配置（快捷键/启动项） |
| user | 用户配置（行为/外观） |
| searchHistory | 搜索历史记录 |
| app | 应用全局状态 |

## 搜索系统

### 搜索流程

1. 用户输入关键词
2. 触发搜索请求
3. 多源搜索（系统搜索、插件搜索、历史记录）
4. 结果合并和排序
5. UI 渲染展示

### 模糊匹配

- 前缀匹配优先
- 模糊匹配支持
- 结果排序算法

## 插件前端层

### PatchLoader

职责：
- 监听 `wasm-load-patch` 事件加载 WASM
- 监听 `wasm-call` 事件执行 WASM 函数
- 调用 `wasm_register_functions` 注册函数
- 调用 `wasm_store_call_result` 保存结果

### PluginService

职责：
- 初始化插件系统
- 管理插件生命周期
- 提供插件搜索功能
- 同步插件状态

## 前端服务

### 核心服务

| 服务 | 职责 |
|------|------|
| executor | 结果执行器，处理用户选择 |
| clipboard | 剪贴板操作 |
| shell | Shell 操作、文件打开 |
| startup | 自启动管理 |
| store | 数据存储 |

## API 代理

`api.ts` 提供统一的 Tauri Command 调用封装，简化前端调用。

## 样式系统

使用 CSS 变量实现主题切换，支持：
- 深色主题
- 浅色主题
- 系统跟随

关键配置：
- `body` 背景透明（无边框窗口必需）
- 主题变量统一管理

## 相关文件

- `src/lib/components/` - 组件目录
- `src/lib/stores/` - 状态管理
- `src/lib/plugins/` - 插件前端层
- `src/lib/search/` - 搜索系统

# MVP-ECOSYSTEM 生态建设计划

## 概述

本计划用于指导 MVP 生态建设阶段，包括插件分发、文档完善、社区运营等。

---

## 基本信息

| 字段 | 内容 |
|------|------|
| **计划 ID** | MVP-ECOSYSTEM |
| **计划名称** | 生态建设 |
| **所属阶段** | MVP Phase 5 |
| **前置计划** | MVP-UX |
| **预计工期** | 2-3 周 |
| **状态** | 待开始 |

---

## 目标与验收标准

### 目标

建设 Corelia 生态系统，包括：

1. **插件分发机制**：GitHub Release + .zip + 拖拽安装
2. **插件开发文档**：完整 Guide + TypeScript SDK
3. **插件模板**：快速上手的基础模板
4. **GitHub 仓库**：开源仓库初始化
5. **社区运营**：GitHub Discussions

### 验收标准

| 序号 | 验收标准 | 验证方法 | 通过标准 |
|------|---------|---------|---------|
| 1 | 插件可通过拖拽安装 | 手动测试 | 安装正常 |
| 2 | 插件可正确解压到目录 | 手动测试 | 目录正确 |
| 3 | 开发文档完整可用 | 阅读测试 | 文档可读 |
| 4 | TypeScript SDK 可用 | 运行测试 | 类型提示正常 |
| 5 | GitHub 仓库结构完整 | 检查 | 结构正确 |
| 6 | Discussions 可用 | 访问测试 | 页面正常 |

---

## 任务拆解

### 任务列表

| 任务 ID | 任务名称 | 预计工时 | 依赖 | 优先级 |
|---------|---------|---------|------|--------|
| ECO-01 | GitHub 仓库初始化 | 8h | - | P0 |
| ECO-02 | 插件分发实现 | 16h | PLUGIN-SYSTEM 完成 | P0 |
| ECO-03 | 插件模板仓库 | 16h | ECO-01 | P0 |
| ECO-04 | TypeScript SDK | 24h | PLUGIN-SYSTEM 完成 | P0 |
| ECO-05 | 插件开发文档 | 16h | ECO-03, ECO-04 | P0 |
| ECO-06 | GitHub Actions CI/CD | 16h | ECO-01 | P0 |
| ECO-07 | GitHub Discussions | 4h | ECO-01 | P1 |
| ECO-08 | README 完善 | 4h | 所有 MVP 计划 | P1 |
| ECO-09 | 版本发布准备 | 8h | 所有 MVP 计划 | P0 |
| ECO-10 | 生态测试 | 8h | ECO-01~09 | P0 |

### 详细任务说明

#### ECO-01 GitHub 仓库初始化

**任务描述**：初始化 GitHub 开源仓库

**执行步骤**：
1. 创建 GitHub 仓库
2. 配置仓库设置
3. 添加 LICENSE 文件（MIT）
4. 添加 .gitignore 文件
5. 配置分支保护规则
6. 配置 CODEOWNERS

**仓库结构**：
```
.github/
├── ISSUE_TEMPLATE/
├── PULL_REQUEST_TEMPLATE.md
└── workflows/
docs/                 # 插件开发文档
plugins/              # 官方插件
patches/              # WASM 补丁
website/              # 官网/文档站
```

#### ECO-02 插件分发实现

**任务描述**：实现插件的拖拽安装和 GitHub Release 分发

**分发流程**：
```
开发者
    │
    ▼
GitHub Release 发布 .zip
    │
    ▼
用户下载 .zip
    │
    ▼
拖入 Corelia 窗口安装
    │
    ▼
自动解压到 plugins/ 目录
```

**执行步骤**：
1. 实现拖拽安装 UI
2. 实现 .zip 解压功能
3. 实现插件目录验证
4. 实现安装冲突处理
5. 实现卸载功能
6. 实现残留数据清理提示

#### ECO-03 插件模板仓库

**任务描述**：创建插件开发模板

**QuickJS 插件模板**：
```
template-quickjs/
├── plugin.json
├── index.js
├── README.md
└── package.json
```

**Webview 插件模板**：
```
template-webview/
├── plugin.json
├── index.html
├── app.js
├── styles.css
├── README.md
└── package.json
```

**执行步骤**：
1. 创建 QuickJS 插件模板
2. 创建 Webview 插件模板
3. 编写模板使用文档
4. 发布模板到 GitHub

#### ECO-04 TypeScript SDK

**任务描述**：开发 TypeScript SDK 提供类型提示和 API 封装

**SDK 结构**：
```
@corelia/sdk/
├── src/
│   ├── index.ts          # 导出
│   ├── utools.ts        # window.utools 类型
│   ├── corelia.ts       # window.corelia 类型
│   └── utils.ts         # 工具函数
├── package.json
└── README.md
```

**SDK 功能**：
- TypeScript 类型定义
- API 封装
- 错误处理
- 类型推断

**执行步骤**：
1. 定义 TypeScript 类型
2. 实现 API 封装
3. 编写使用文档
4. 发布到 npm（可选）
5. 配置 IDE 支持

#### ECO-05 插件开发文档

**任务描述**：编写完整的插件开发文档

**文档结构**：
```
docs/
├── getting-started.md       # 快速开始
├── plugin-anatomy.md        # 插件结构
├── plugin-json.md           # plugin.json 详解
├── api-reference.md         # API 参考
├── quickjs-guide.md         # QuickJS 插件开发
├── webview-guide.md         # Webview 插件开发
├── wasm-patches.md          # WASM 补丁开发
├── distribution.md          # 插件分发
├── troubleshooting.md       # 常见问题
└── examples/                # 示例代码
    ├── quickjs-basic.md     # QuickJS 基础示例
    ├── quickjs-advanced.md  # QuickJS 高级示例
    └── webview-advanced.md  # Webview 高级示例
```

**执行步骤**：
1. 编写快速开始指南
2. 编写插件结构文档
3. 编写 API 参考
4. 编写各类型插件开发指南
5. 编写常见问题
6. 编写示例代码

#### ECO-06 GitHub Actions CI/CD

**任务描述**：配置 GitHub Actions CI/CD

**CI 流程**：
1. 代码检查（ESLint、Rust fmt）
2. 单元测试
3. 集成测试
4. 构建

**CD 流程**：
1. 构建发布版本
2. 生成 Release
3. 上传构建产物

**工作流文件**：
```
.github/
└── workflows/
    ├── ci.yml     # 持续集成
    └── cd.yml     # 持续部署
```

#### ECO-07 GitHub Discussions

**任务描述**：配置 GitHub Discussions 社区

**配置内容**：
- 讨论分类
  - 公告
  - 插件分享
  - 问题求助
  - 功能建议
  - 闲聊
- 社区指南
- 管理员配置

#### ECO-08 README 完善

**任务描述**：完善 README.md

**内容**：
1. 项目介绍
2. 核心特性
3. 技术栈
4. 快速开始
5. 插件系统介绍
6. MVP 插件清单
7. 文档链接
8. 路线图
9. 贡献指南
10. License

#### ECO-09 版本发布准备

**任务描述**：准备 MVP 版本发布

**执行步骤**：
1. 确定版本号（v0.1.0-alpha）
2. 编写 CHANGELOG
3. 创建 GitHub Release
4. 打包发布版本
5. 编写发布说明
6. 宣传准备

#### ECO-10 生态测试

**任务描述**：对整个生态系统进行测试

**测试内容**：
1. 插件开发流程测试
2. 插件分发测试
3. 文档完整性测试
4. CI/CD 流程测试
5. 社区功能测试

---

## 插件分发机制

### 插件包格式

**格式**：.zip 压缩包

**结构**：
```
plugin-name-v1.0.0.zip
└── plugin-name/
    ├── plugin.json
    ├── index.js       # QuickJS 插件
    ├── index.html     # Webview 插件
    └── assets/        # 静态资源
```

### 安装流程

**用户操作**：
1. 下载插件 .zip
2. 拖入 Corelia 窗口
3. 确认安装

**系统操作**：
1. 验证 plugin.json
2. 解压到 plugins/ 目录
3. 扫描并注册插件
4. 显示成功提示

### 卸载流程

**用户操作**：
1. 打开插件管理
2. 选择插件
3. 点击卸载

**系统操作**：
1. 停止插件
2. 删除插件目录
3. 提示清理残留数据

---

## TypeScript SDK 设计

### SDK 类型定义

```typescript
// window.utools 类型
declare global {
  interface Window {
    utools: {
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
    };
  }
}

// window.corelia 类型
declare global {
  interface Window {
    corelia: {
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
    };
  }
}

export {};
```

---

## 风险评估

| 风险描述 | 可能性 | 影响程度 | 应对措施 |
|---------|-------|---------|---------|
| 插件审核缺失 | 高 | 中 | 建立社区规范 |
| 恶意插件 | 中 | 高 | 插件签名机制（P2） |
| SDK 维护成本 | 中 | 低 | 最小化 API 范围 |

---

## 执行 Checkpoint

| Checkpoint | 计划日期 | 实际日期 | 状态 |
|-----------|---------|---------|------|
| 启动 | | | ❌ |
| GitHub 仓库初始化完成 | | | ❌ |
| 插件分发实现完成 | | | ❌ |
| SDK 和文档完成 | | | ❌ |
| CI/CD 配置完成 | | | ❌ |
| 版本发布准备完成 | | | ❌ |

---

## 参考资料

- [SRS 功能需求规格说明书](../../wiki/SRS.md)
- [系统架构设计](../../wiki/ARCHITECTURE.md)
- [问题讨论记录 - 生态社区](../../wiki/problem/04-ecosystem-community.md)

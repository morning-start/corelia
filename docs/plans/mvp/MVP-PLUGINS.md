# MVP-PLUGINS MVP 插件开发计划

## 概述

本计划用于指导 MVP 插件开发，实现 7 个 QuickJS 插件 + 1 个 Webview 插件。

---

## 基本信息

| 字段 | 内容 |
|------|------|
| **计划 ID** | MVP-PLUGINS |
| **计划名称** | MVP 插件开发 |
| **所属阶段** | MVP Phase 3 |
| **前置计划** | MVP-PLUGIN-SYSTEM |
| **预计工期** | 3-4 周 |
| **状态** | 待开始 |

---

## 插件清单

| 序号 | 插件名称 | 类型 | WASM 依赖 | 优先级 |
|------|----------|------|-----------|--------|
| 1 | 快捷命令 | QuickJS | - | P0 |
| 2 | 剪贴板增强 | QuickJS | - | P0 |
| 3 | 计算器 | QuickJS | - | P0 |
| 4 | 二维码 | QuickJS | crypto | P0 |
| 5 | 文件搜索 + 应用搜索 | QuickJS | - | P0 |
| 6 | 截图增强 | QuickJS | AI | P0 |
| 7 | 云端同步配置 | Webview | - | P0 |

---

## 任务拆解

### 任务列表

| 任务 ID | 任务名称 | 预计工时 | 依赖 | 优先级 |
|---------|---------|---------|------|--------|
| PLG-DEV-01 | 快捷命令插件 | 16h | PLUGIN-SYSTEM 完成 | P0 |
| PLG-DEV-02 | 剪贴板增强插件 | 16h | PLUGIN-SYSTEM 完成 | P0 |
| PLG-DEV-03 | 计算器插件 | 8h | PLUGIN-SYSTEM 完成 | P0 |
| PLG-DEV-04 | 二维码插件 | 12h | PLG-DEV-01, WASM crypto | P0 |
| PLG-DEV-05 | 文件搜索插件 | 24h | PLUGIN-SYSTEM 完成 | P0 |
| PLG-DEV-06 | 截图增强插件 | 20h | PLG-DEV-01, WASM AI | P0 |
| PLG-DEV-07 | 云端同步插件 | 24h | PLUGIN-SYSTEM 完成 | P0 |
| PLG-DEV-08 | 插件打包测试 | 8h | PLG-DEV-01~07 | P0 |
| PLG-DEV-09 | 插件文档编写 | 8h | PLG-DEV-01~08 | P1 |

### 详细任务说明

#### PLG-DEV-01 快捷命令插件

**插件描述**：用户自定义命令脚本，快速执行常用操作

**功能列表**：
- 命令列表展示
- 命令添加/编辑/删除
- 命令分组
- 命令执行
- 命令搜索

**plugin.json 设计**：
```json
{
  "name": "快捷命令",
  "version": "1.0.0",
  "type": "quickjs",
  "patches": [],
  "features": [
    {
      "code": "cmd",
      "label": "命令列表",
      "type": "list",
      "items": []
    }
  ]
}
```

**执行步骤**：
1. 创建插件目录结构
2. 编写 plugin.json
3. 实现命令管理功能
4. 实现命令执行
5. 实现数据持久化
6. 测试

#### PLG-DEV-02 剪贴板增强插件

**插件描述**：剪贴板历史、搜索、快捷粘贴

**功能列表**：
- 剪贴板历史记录
- 历史记录搜索
- 快速粘贴
- 固定常用内容
- 历史记录清理

**plugin.json 设计**：
```json
{
  "name": "剪贴板增强",
  "version": "1.0.0",
  "type": "quickjs",
  "patches": [],
  "features": [
    {
      "code": "clip",
      "label": "剪贴板历史",
      "type": "list",
      "items": []
    }
  ]
}
```

**执行步骤**：
1. 创建插件目录结构
2. 编写 plugin.json
3. 实现剪贴板监听
4. 实现历史记录管理
5. 实现搜索功能
6. 实现快捷粘贴
7. 测试

#### PLG-DEV-03 计算器插件

**插件描述**：基础数学运算

**功能列表**：
- 基本运算（+、-、×、÷）
- 百分数计算
- 幂运算
- 平方根
- 计算历史

**plugin.json 设计**：
```json
{
  "name": "计算器",
  "version": "1.0.0",
  "type": "quickjs",
  "patches": [],
  "features": [
    {
      "code": "calc",
      "label": "计算",
      "type": "input",
      "placeholder": "输入表达式，如 1+2*3"
    }
  ]
}
```

**执行步骤**：
1. 创建插件目录结构
2. 编写 plugin.json
3. 实现表达式解析
4. 实现计算逻辑
5. 实现结果展示
6. 测试

#### PLG-DEV-04 二维码插件

**插件描述**：生成和解析二维码

**功能列表**：
- 文本生成二维码
- URL 生成二维码
- 二维码解析
- 二维码保存
- 扫描历史

**WASM 依赖**：crypto 补丁（Base64 编解码）

**plugin.json 设计**：
```json
{
  "name": "二维码",
  "version": "1.0.0",
  "type": "quickjs",
  "patches": ["crypto"],
  "features": [
    {
      "code": "qr",
      "label": "生成二维码",
      "type": "input",
      "placeholder": "输入文本或 URL"
    }
  ]
}
```

**执行步骤**：
1. 创建插件目录结构
2. 编写 plugin.json
3. 集成二维码库
4. 实现生成功能
5. 实现解析功能
6. 测试

#### PLG-DEV-05 文件搜索 + 应用搜索插件

**插件描述**：本地文件定位和应用快速启动

**功能列表**：
- 文件搜索
- 文件夹快速打开
- 应用搜索
- 应用快速启动
- 搜索历史
- 模糊匹配

**plugin.json 设计**：
```json
{
  "name": "文件搜索",
  "version": "1.0.0",
  "type": "quickjs",
  "patches": [],
  "features": [
    {
      "code": "file",
      "label": "搜索文件",
      "type": "input",
      "placeholder": "输入文件名"
    },
    {
      "code": "app",
      "label": "搜索应用",
      "type": "input",
      "placeholder": "输入应用名称"
    }
  ]
}
```

**应用搜索技术实现**：
1. 索引 Windows "开始菜单" 快捷方式
2. 索引 "桌面" 快捷方式
3. 使用 tiny-fuzzy 模糊匹配
4. 调用系统 shell 打开应用

**执行步骤**：
1. 创建插件目录结构
2. 编写 plugin.json
3. 实现文件索引
4. 实现应用索引
5. 实现搜索功能
6. 实现模糊匹配
7. 实现打开功能
8. 测试

#### PLG-DEV-06 截图增强插件

**插件描述**：截图 + OCR + 标注

**功能列表**：
- 屏幕截图
- OCR 文字识别
- 截图标注
- 截图保存
- 截图历史

**WASM 依赖**：AI 补丁（OCR）

**plugin.json 设计**：
```json
{
  "name": "截图增强",
  "version": "1.0.0",
  "type": "quickjs",
  "patches": ["ai"],
  "features": [
    {
      "code": "shot",
      "label": "截图",
      "type": "action",
      "action": "capture"
    }
  ]
}
```

**执行步骤**：
1. 创建插件目录结构
2. 编写 plugin.json
3. 集成截图功能
4. 集成 OCR 功能
5. 实现标注工具
6. 测试

#### PLG-DEV-07 云端同步配置插件

**插件描述**：配置跨设备同步

**功能列表**：
- 配置导出
- 配置导入
- 云端同步
- 同步状态显示
- 冲突处理

**技术实现**：Webview 插件

**plugin.json 设计**：
```json
{
  "name": "云端同步",
  "version": "1.0.0",
  "type": "webview",
  "main": "index.html",
  "patches": []
}
```

**Webview 结构**：
```
plugins/cloud-sync/
├── plugin.json
├── index.html
├── app.js        # Vue/React 应用
└── styles.css
```

**执行步骤**：
1. 创建插件目录结构
2. 编写 plugin.json
3. 实现 Webview 前端
4. 实现同步逻辑
5. 测试

#### PLG-DEV-08 插件打包测试

**任务描述**：对所有插件进行打包和测试

**执行步骤**：
1. 打包所有插件为 .zip
2. 测试拖拽安装
3. 测试插件功能
4. 测试插件卸载
5. 测试残留数据清理

#### PLG-DEV-09 插件文档编写

**任务描述**：编写插件开发文档

**执行步骤**：
1. 编写插件开发指南
2. 编写 API 参考文档
3. 编写示例代码
4. 整理常见问题

---

## 插件开发规范

### 目录结构

**QuickJS 插件结构**：
```
plugin-id/
├── plugin.json
└── index.js       # 可选，扩展默认行为
```

**Webview 插件结构**：
```
plugin-id/
├── plugin.json
├── index.html
├── app.js
└── styles.css
```

### plugin.json 规范

```json
{
  "name": "插件名称",
  "version": "1.0.0",
  "type": "quickjs | webview",
  "logo": "logo.png",
  "patches": ["crypto", "ai"],
  "features": [
    {
      "code": "feature-code",
      "label": "功能名称",
      "type": "list | input | action",
      "placeholder": "输入提示",
      "items": []
    }
  ]
}
```

---

## 风险评估

| 风险描述 | 可能性 | 影响程度 | 应对措施 |
|---------|-------|---------|---------|
| 应用搜索索引慢 | 中 | 中 | 后台索引，按需更新 |
| OCR 准确率低 | 中 | 中 | 使用成熟 OCR 库 |
| 云同步冲突 | 低 | 中 | 提供冲突处理 UI |

---

## 执行 Checkpoint

| Checkpoint | 计划日期 | 实际日期 | 状态 |
|-----------|---------|---------|------|
| 启动 | | | ❌ |
| QuickJS 插件完成 | | | ❌ |
| Webview 插件完成 | | | ❌ |
| 打包测试通过 | | | ❌ |
| 文档编写完成 | | | ❌ |

---

## 参考资料

- [SRS 功能需求规格说明书](../../wiki/SRS.md)
- [系统架构设计](../../wiki/ARCHITECTURE.md)
- [MVP-PLUGIN-SYSTEM 插件系统实现计划](MVP-PLUGIN-SYSTEM.md)

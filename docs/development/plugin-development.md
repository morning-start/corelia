# 插件开发指南

> QuickJS 和 Webview 插件开发

---

## QuickJS 插件

### 插件结构

```
plugins/my-plugin/
├── plugin.json     # 插件元数据（必需）
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

### 字段说明

| 字段 | 必需 | 说明 |
|------|------|------|
| name | ✅ | 插件唯一标识 |
| version | ✅ | 语义化版本 |
| type | ✅ | 固定为 `quickjs` |
| logo | ❌ | 插件图标 |
| description | ❌ | 插件描述 |
| patches | ❌ | 依赖的 WASM 补丁 |
| features | ✅ | 功能列表 |

### index.js 示例

```javascript
// 插件进入回调
window.onPluginEnter = ({ code, type, payload }) => {
  console.log('插件进入', code, payload);
};

// 搜索回调（可选）
window.onSearch = ({ query }) => {
  return [
    {
      title: '搜索结果',
      description: '描述',
      icon: 'icon.png',
      action: () => {
        console.log('执行操作');
      }
    }
  ];
};

// 插件退出回调
window.onPluginOut = () => {
  console.log('插件退出');
};
```

---

## API 参考

### 存储 API

```javascript
// 设置数据
window.utools.dbStorage.setItem('key', 'value');

// 获取数据
const value = window.utools.dbStorage.getItem('key');

// 删除数据
window.utools.dbStorage.removeItem('key');
```

### 剪贴板 API

```javascript
// 读取文本
const text = window.utools.clipboard.readText();

// 写入文本
window.utools.clipboard.writeText('文本内容');

// 写入图片
window.utools.clipboard.writeImage(base64String);
```

### 窗口 API

```javascript
// 隐藏主窗口
window.utools.hideMainWindow();

// 显示主窗口
window.utools.showMainWindow();

// 显示通知
window.utools.showNotification('通知内容');
```

### Shell API

```javascript
// 打开 URL
window.utools.shellOpenExternal('https://example.com');

// 打开文件所在目录
window.utools.shellShowItemInFolder('/path/to/file');
```

### 路径 API

```javascript
// 获取应用数据目录
const dataPath = window.utools.getPath('appData');

// 获取用户目录
const homePath = window.utools.getPath('home');
```

---

## Webview 插件

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

### plugin.json 格式

```json
{
  "name": "my-webview-plugin",
  "version": "1.0.0",
  "type": "webview",
  "main": "index.html",
  "logo": "logo.png"
}
```

### MVP 状态

MVP 阶段 Webview 插件支持简化：
- 基础 iframe 嵌入
- API 桥接
- 通信机制

---

## WASM 补丁

### 使用 WASM 补丁

```javascript
// 调用 WASM 函数
const result = await window.corelia.wasm.call(
  'crypto',    // 补丁名称
  'encrypt',   // API 名称
  data         // 参数
);
```

### MVP 可用补丁

| 补丁 | API | 说明 |
|------|-----|------|
| crypto | encrypt, decrypt, hash | 加密算法 |

---

## 调试技巧

### 查看日志

```javascript
console.log('调试信息');
```

日志会输出到 Tauri 控制台。

### 测试插件

1. 将插件放入 `plugins/` 目录
2. 运行 `bun run tauri dev`
3. 在搜索框输入插件前缀测试

---

## 相关文档

- [插件系统架构](../architecture/plugin-system.md)
- [MVP 指南](./mvp-guide.md)
- [API 文档](../reference/api.md)

# Corelia 插件开发快速上手

> 从零创建一个 Corelia 插件。完整规范见 [插件系统](wiki/PLUGIN_SYSTEM.md)，完整 API 见 [API 参考](wiki/API.md)。

## 创建最小插件

```bash
mkdir -p plugins/my-plugin
```

**`plugins/my-plugin/plugin.json`**
```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "type": "quickjs",
  "prefix": "mp",
  "description": "我的第一个插件",
  "author": "Your Name",
  "main": "index.js",
  "patches": []
}
```

**`plugins/my-plugin/index.js`**
```javascript
function pluginInit() {
  console.log('[my-plugin] 已加载');
}

function onSearch(query) {
  if (!query || query === '') return [
    { title: 'Hello', description: '点击执行', icon: '👋', action: 'hello' }
  ];
  return [];
}

function onAction(action) {
  switch(action) {
    case 'hello':
      return { type: 'text', message: '👋 Hello from my plugin!' };
    default:
      return { type: 'error', message: 'Unknown action' };
  }
}

if (typeof module !== 'undefined') {
  module.exports = { pluginInit, onSearch, onAction };
}
```

启动 Corelia 后，在搜索框输入 `mp` 即可看到插件。

## 更多参考

| 需要了解 | 看这里 |
|---------|-------|
| plugin.json 完整字段 | [插件系统](wiki/PLUGIN_SYSTEM.md) |
| utools API（存储/剪切板/Shell等） | [API 参考](wiki/API.md) |
| 插件生命周期与状态机 | [插件系统 → 插件生命周期](wiki/PLUGIN_SYSTEM.md) |
| WASM Patch 扩展 | [插件系统 → WASM Patch](wiki/PLUGIN_SYSTEM.md) |

## 最佳实践

- **懒加载**: 只在 `onSearch` 中做必要初始化
- **限制返回数**: `onSearch` 返回不超过 10 条
- **缓存结果**: 利用 `dbStorage` 缓存高频查询
- **错误处理**: `onAction` 用 try-catch 包裹，返回 `{ type: 'error', message: ... }`
- **前缀设计**: 2-4 个字符，简短有意义，避免与现有插件冲突

## 调试技巧

```javascript
// 插件日志输出到 Tauri 终端（RUST_LOG=debug 更详细）
console.log('[my-plugin] 信息');
console.error('[my-plugin] 错误');
```

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 插件不被发现 | plugin.json 缺少必填字段 | 检查 name/version/type |
| utools is not defined | API 未注入 | 确保通过 load_plugin 正确加载 |
| 搜索无匹配 | 前缀未配置或不匹配 | 检查 prefix 配置 |
| VM 池满 | 超过 10 个活跃 VM | 卸载不需要的插件 |
| 存储写入失败 | 超出 10MB 配额 | 清理旧数据 |
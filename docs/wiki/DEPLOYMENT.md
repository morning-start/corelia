# 构建与部署

> Corelia 的生产构建与故障排查。代码签名、自动更新等功能将在 Beta 阶段补充。

## 构建前检查

```bash
bun run check && cargo check --release
```

## 生产构建

```bash
bun run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/msi/`（`.msi` 安装包）。

## 关键配置

### 窗口配置 (`tauri.conf.json`)

```json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "Corelia",
        "width": 600,
        "height": 400,
        "decorations": false,
        "transparent": true,
        "resizable": false,
        "alwaysOnTop": true,
        "center": true
      }
    ]
  }
}
```

### 权限配置 (`capabilities/default.json`)

```json
{
  "permissions": [
    "core:default",
    "shell:allow-open",
    "clipboard:allow-read",
    "clipboard:allow-write",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "store:allow-get",
    "store:allow-set",
    "store:allow-delete",
    "dialog:allow-open",
    "dialog:allow-save",
    "autostart:allow-enable",
    "autostart:allow-disable"
  ]
}
```

## 清理与重置

```bash
rm -rf node_modules/.vite              # 清理 Vite 缓存
cd src-tauri && cargo clean            # 清理 Rust 编译缓存
rm -rf node_modules bun.lock           # 完全重置
rm -rf src-tauri/target
bun install
```

## 故障排查

### 构建失败

| 错误 | 原因 | 解决 |
|------|------|------|
| `link.exe not found` | MSVC 工具链缺失 | 安装 Visual Studio Build Tools |
| `rustup not found` | Rust 未安装 | 运行 rustup-init.exe |
| `wasm-pack not found` | WASM 工具缺失 | `cargo install wasm-pack` |
| `bun not found` | Bun 未安装 | 访问 bun.sh 安装 |

### 运行时问题

| 问题 | 原因 | 解决 |
|------|------|------|
| 窗口不透明 | 主题 CSS 未设置 transparent | 检查 `body { background: transparent }` |
| 快捷键不响应 | 权限未声明 | 检查 `capabilities/default.json` |
| 插件加载失败 | plugin.json 格式错误 | 检查 JSON 语法和必填字段 |
| WASM 加载失败 | 文件路径不正确 | 检查 patches 目录结构和文件名 |
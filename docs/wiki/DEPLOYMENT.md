# 构建与部署

> Corelia 的生产构建、打包分发与发布流程。

## 构建前检查

```bash
# 前端类型检查
bun run check

# Rust 编译检查
cd src-tauri && cargo check --release

# 一键检查 + 构建
bun run check && cargo check --release && bun run tauri build
```

## 生产构建

```bash
bun run tauri build
```

构建产物位置：

| 平台 | 产物路径 | 格式 |
|------|----------|------|
| Windows | `src-tauri/target/release/bundle/msi/` | `.msi` 安装包 |
| Windows | `src-tauri/target/release/bundle/nsis/` | `.exe` 安装包 |
| macOS | `src-tauri/target/release/bundle/dmg/` | `.dmg` 镜像 |
| Linux | `src-tauri/target/release/bundle/deb/` | `.deb` 包 |
| Linux | `src-tauri/target/release/bundle/appimage/` | `.AppImage` |

## 构建配置

### Tauri 配置 (`tauri.conf.json`)

关键构建相关字段：

```json
{
  "version": "0.1.0",
  "identifier": "com.corelia.app",
  "build": {
    "beforeBuildCommand": "bun run build",
    "beforeDevCommand": "bun run dev",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../build"
  },
  "bundle": {
    "active": true,
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.ico"],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    }
  }
}
```

### 窗口配置

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

确保构建前权限声明完整：

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
# 清理 Vite 缓存
rm -rf node_modules/.vite

# 清理 Rust 编译缓存
cd src-tauri && cargo clean

# 完全重置
rm -rf node_modules bun.lock
rm -rf src-tauri/target
bun install
```

## 代码签名

### Windows

1. 获取代码签名证书（.pfx 文件）
2. 配置 `tauri.conf.json`：

```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_THUMBPRINT",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

3. 构建时签名：

```bash
TAURI_SIGNING_PRIVATE_KEY=path/to/key.pem bun run tauri build
```

### macOS

```bash
# Apple Developer 证书签名
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
bun run tauri build
```

## 自动更新

Tauri 支持自动更新功能，配置步骤：

1. 在 `Cargo.toml` 添加依赖：

```toml
[dependencies]
tauri-plugin-updater = "2"
```

2. 在 `lib.rs` 注册插件：

```rust
.plugin(tauri_plugin_updater::Builder::new().build())
```

3. 配置更新服务器 URL 在 `tauri.conf.json`

## 版本发布流程

```
1. 更新版本号
   ├── package.json: "version": "x.y.z"
   ├── src-tauri/Cargo.toml: version = "x.y.z"
   └── src-tauri/tauri.conf.json: "version": "x.y.z"

2. 更新 CHANGELOG.md

3. 构建与测试
   └── bun run check && cargo check --release && bun run tauri build

4. 打 tag
   └── git tag vx.y.z

5. 上传构建产物到发布平台
```

## 性能优化

### 减小包体积

- **前端**：Vite 自动 tree-shaking，确保未使用代码被移除
- **Rust**：`cargo check --release` 自动优化
- **WASM**：使用 `wasm-opt` 进一步优化

```bash
# 安装 wasm-opt
cargo install wasm-opt

# 优化 WASM 文件
wasm-opt -Oz -o output.wasm input.wasm
```

### 启动速度

- 插件懒加载（仅 `scan_plugins` 扫描元数据，按需 `load_plugin`）
- QuickJS VM 池化（避免重复创建/销毁开销）
- 闲置 VM 自动回收（5 分钟超时）

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

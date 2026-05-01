# 开发环境配置

> Corelia 开发环境搭建指南

---

## 前置要求

| 工具 | 版本要求 | 说明 |
|------|----------|------|
| Windows | 10/11 x64 | MVP 仅支持 Windows |
| Rust | 1.75+ | 稳定版 |
| Bun | 1.x+ | 包管理与运行时 |
| Node.js | 18+ | 可选，Bun 可替代 |

---

## 安装步骤

### 1. 安装 Bun

```powershell
# Windows PowerShell
powershell -c "irm bun.sh/install.ps1 | iex"
```

**常见问题**: 如果遇到执行策略错误：

```powershell
# 以管理员身份运行
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### 2. 安装 Rust

1. 下载 [rustup-init.exe](https://rustup.rs/)
2. 运行安装程序
3. 选择默认安装

**验证安装**:

```bash
rustc --version
cargo --version
```

### 3. 克隆项目

```bash
git clone <repository-url>
cd corelia
```

### 4. 安装依赖

```bash
bun install
```

---

## 验证安装

### 前端检查

```bash
bun run check
```

预期输出：无错误（可能有警告）

### 后端检查

```bash
cd src-tauri
cargo check
cargo clippy
```

预期输出：无错误、无警告

---

## 开发命令

### 前端开发

```bash
# 仅前端开发服务器
bun run dev

# 类型检查
bun run check

# 构建前端
bun run build
```

### Tauri 开发

```bash
# 完整开发模式（推荐）
bun run tauri dev

# 构建生产版本
bun run tauri build
```

### Rust 开发

```bash
cd src-tauri

# 编译检查
cargo check

# Clippy 检查
cargo clippy

# 运行测试
cargo test
```

---

## IDE 配置

### VS Code 推荐扩展

```json
{
  "recommendations": [
    "svelte.svelte-vscode",
    "rust-lang.rust-analyzer",
    "tauri-apps.tauri-vscode",
    "dbaeumer.vscode-eslint",
    "esbenp.prettier-vscode"
  ]
}
```

### Rust Analyzer 配置

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all"
}
```

---

## 常见问题

### Bun 执行策略错误

**问题**: Windows 上 Bun 无法执行

**解决**: 以管理员身份运行 PowerShell，执行：
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Rust 编译慢

**问题**: 依赖下载和编译很慢

**解决**: 配置国内镜像源，在 `~/.cargo/config.toml` 添加：

```toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"
```

### Tauri 构建失败

**问题**: 缺少 Windows SDK 或构建工具

**解决**: 安装 Visual Studio Build Tools，确保包含：
- MSVC v143 生成工具
- Windows 10/11 SDK

---

## 相关文档

- [MVP 指南](./mvp-guide.md)
- [代码规范](./conventions.md)
- [架构文档](../architecture/)

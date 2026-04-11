# Corelia 部署指南

## 1. 构建前准备

### 1.1 环境检查

```bash
# 检查 Bun 版本
bun --version  # >= 1.3.0

# 检查 Rust 版本
rustc --version  # >= 1.94.0
cargo --version

# 检查 Node.js 版本
node --version  # >= 18.0.0
```

### 1.2 依赖安装

```bash
# 安装前端依赖
bun install

# 验证 Rust 依赖
cd src-tauri
cargo check
```

### 1.3 类型检查

```bash
# TypeScript 类型检查
bun run check

# Rust 类型检查
cd src-tauri
cargo check --release
```

### 1.4 代码格式化

```bash
# 前端代码格式化
bun run format

# Rust 代码格式化
cd src-tauri
cargo fmt

# 检查代码格式
cargo fmt -- --check
```

## 2. 生产构建

### 2.1 开发构建

```bash
# 开发模式构建（包含调试信息）
bun run tauri build
```

**产物位置**:
- Windows: `src-tauri/target/release/corelia.exe`
- macOS: `src-tauri/target/release/bundle/macos/corelia.app`
- Linux: `src-tauri/target/release/bundle/deb/corelia.deb`

### 2.2 发布构建

```bash
# 完全优化构建（无调试信息，体积更小）
cd src-tauri
cargo build --release

cd ..
bun run tauri build
```

### 2.3 构建配置

```json
// tauri.conf.json
{
  "build": {
    "beforeBuildCommand": "bun run build",
    "frontendDist": "../build"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

## 3. 各平台部署

### 3.1 Windows

#### 构建产物

```bash
bun run tauri build
```

**生成文件**:
- `src-tauri/target/release/bundle/msi/corelia_0.1.0_x64_en-US.msi` - MSI 安装包
- `src-tauri/target/release/bundle/nsis/corelia_0.1.0_x64-setup.exe` - NSIS 安装包
- `src-tauri/target/release/corelia.exe` - 独立可执行文件

#### 安装方式

**MSI 安装包**:
```bash
# 双击运行或使用命令行
msiexec /i corelia_0.1.0_x64_en-US.msi

# 静默安装
msiexec /i corelia_0.1.0_x64_en-US.msi /quiet

# 卸载
msiexec /x corelia_0.1.0_x64_en-US.msi
```

**NSIS 安装包**:
```bash
# 双击运行或使用命令行
corelia_0.1.0_x64-setup.exe /S  # 静默安装
```

**独立可执行文件**:
```bash
# 直接运行
corelia.exe

# 创建快捷方式
powershell "$WshShell = New-Object -comObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('corelia.lnk'); $Shortcut.TargetPath = 'corelia.exe'; $Shortcut.Save()"
```

#### 代码签名 (可选)

```json
// tauri.conf.json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "你的证书指纹",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

**获取证书**:
1. 从证书颁发机构购买代码签名证书
2. 安装证书到系统证书存储
3. 获取证书指纹
4. 配置到 `tauri.conf.json`

### 3.2 macOS

#### 构建产物

```bash
bun run tauri build
```

**生成文件**:
- `src-tauri/target/release/bundle/macos/corelia.app` - App 包
- `src-tauri/target/release/bundle/dmg/corelia_0.1.0_x64.dmg` - DMG 镜像
- `src-tauri/target/release/bundle/macos/corelia.pkg` - PKG 安装包

#### 安装方式

**DMG 镜像**:
```bash
# 挂载 DMG
hdiutil attach corelia_0.1.0_x64.dmg

# 拖拽到 Applications 文件夹
# 或使用命令行
cp -R /Volumes/corelia/corelia.app /Applications/

# 卸载 DMG
hdiutil detach /Volumes/corelia
```

**PKG 安装包**:
```bash
# 双击运行或使用命令行
sudo installer -pkg corelia.pkg -target /
```

**App 包**:
```bash
# 直接运行
open corelia.app

# 或移动到 Applications
cp -R corelia.app /Applications/
```

#### 代码签名 (必需)

```json
// tauri.conf.json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
      "entitlements": "entitlements.plist",
      "entitlementsValidation": true
    }
  }
}
```

**获取证书**:
1. 注册 Apple Developer 账号
2. 创建证书签名请求 (CSR)
3. 在开发者后台创建证书
4. 下载并安装证书
5. 获取签名身份名称

**entitlements.plist**:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
</dict>
</plist>
```

#### Notarization (推荐)

```bash
# 提交 notarization
xcrun notarytool submit corelia.app \
  --apple-id "your@apple.id" \
  --password "app-specific-password" \
  --team-id "TEAM_ID" \
  --wait

# 绑定 notarization 凭证
xcrun stapler staple corelia.app
```

### 3.3 Linux

#### 构建产物

```bash
bun run tauri build
```

**生成文件**:
- `src-tauri/target/release/bundle/deb/corelia_0.1.0_amd64.deb` - DEB 包
- `src-tauri/target/release/bundle/appimage/corelia_0.1.0_amd64.AppImage` - AppImage
- `src-tauri/target/release/bundle/rpm/corelia-0.1.0-1.x86_64.rpm` - RPM 包

#### 安装方式

**DEB 包 (Debian/Ubuntu)**:
```bash
# 安装
sudo dpkg -i corelia_0.1.0_amd64.deb

# 或使用 apt
sudo apt install ./corelia_0.1.0_amd64.deb

# 卸载
sudo dpkg -r corelia
```

**RPM 包 (Fedora/RHEL)**:
```bash
# 安装
sudo rpm -ivh corelia-0.1.0-1.x86_64.rpm

# 或使用 dnf
sudo dnf install corelia-0.1.0-1.x86_64.rpm

# 卸载
sudo rpm -e corelia
```

**AppImage (通用)**:
```bash
# 添加执行权限
chmod +x corelia_0.1.0_amd64.AppImage

# 运行
./corelia_0.1.0_amd64.AppImage

# 集成到系统 (可选)
./corelia_0.1.0_amd64.AppImage --appimage-extract-and-run
```

#### 创建桌面快捷方式

```bash
# 创建 .desktop 文件
sudo tee /usr/share/applications/corelia.desktop > /dev/null << 'EOF'
[Desktop Entry]
Name=Corelia
Comment=快速启动器
Exec=/usr/bin/corelia
Icon=corelia
Type=Application
Categories=Utility;
Terminal=false
EOF
```

## 4. 自动更新

### 4.1 配置更新服务器

```json
// tauri.conf.json
{
  "bundle": {
    "updater": {
      "active": true,
      "dialog": true,
      "endpoints": [
        "https://your-domain.com/update/{{target}}/{{arch}}/{{current_version}}"
      ],
      "pubkey": "你的公钥"
    }
  }
}
```

### 4.2 生成更新信息

```json
// update.json
{
  "version": "0.1.0",
  "notes": "初始版本发布",
  "pub_date": "2026-04-10T12:00:00Z",
  "platforms": {
    "windows": {
      "url": "https://your-domain.com/corelia_0.1.0_x64-setup.exe",
      "signature": "签名"
    },
    "macos": {
      "url": "https://your-domain.com/corelia_0.1.0_x64.dmg",
      "signature": "签名"
    },
    "linux": {
      "url": "https://your-domain.com/corelia_0.1.0_amd64.deb",
      "signature": "签名"
    }
  }
}
```

### 4.3 前端检查更新

```typescript
import { check } from '@tauri-apps/plugin-updater';

async function checkForUpdates() {
  const update = await check();
  
  if (update?.available) {
    console.log('可用更新:', update.version);
    
    if (confirm(`发现新版本 ${update.version}，是否更新？`)) {
      await update.downloadAndInstall();
    }
  }
}
```

## 5. 分发渠道

### 5.1 官网下载

**步骤**:
1. 构建所有平台的安装包
2. 上传到服务器
3. 提供下载链接
4. 配置自动更新

**示例页面**:
```html
<div class="download-section">
  <h2>下载 Corelia</h2>
  
  <a href="/downloads/corelia_0.1.0_x64-setup.exe" class="download-btn">
    <img src="/icons/windows.svg" alt="Windows">
    Windows 版
  </a>
  
  <a href="/downloads/corelia_0.1.0_x64.dmg" class="download-btn">
    <img src="/icons/apple.svg" alt="macOS">
    macOS 版
  </a>
  
  <a href="/downloads/corelia_0.1.0_amd64.deb" class="download-btn">
    <img src="/icons/linux.svg" alt="Linux">
    Linux 版
  </a>
</div>
```

### 5.2 应用商店

#### Microsoft Store

**要求**:
- 开发者账号 ($19 一次性)
- 应用符合商店政策
- MSIX 包格式

**步骤**:
1. 注册 Microsoft Partner Center
2. 创建应用提交
3. 上传 MSIX 包
4. 等待审核

#### Mac App Store

**要求**:
- Apple Developer 账号 ($99/年)
- 应用符合审核指南
- 使用 Mac 签名证书

**步骤**:
1. 注册 Apple Developer
2. 在 App Store Connect 创建应用
3. 使用 Xcode 提交
4. 等待审核

### 5.3 包管理器

#### Homebrew (macOS/Linux)

```ruby
# 创建 Formula
class Corelia < Formula
  desc "快速启动器"
  homepage "https://github.com/your-org/corelia"
  url "https://github.com/your-org/corelia/releases/download/v0.1.0/corelia_0.1.0_x64.dmg"
  sha256 "你的 SHA256"
  
  app "corelia.app"
end
```

#### Chocolatey (Windows)

```powershell
# 创建 nuspec 文件
choco new corelia

# 编辑 corelia.nuspec
# 上传到 Chocolatey
choco push corelia.0.1.0.nupkg
```

#### AUR (Arch Linux)

```bash
# 创建 PKGBUILD
pkgname=corelia
pkgver=0.1.0
pkgrel=1
pkgdesc="快速启动器"
arch=('x86_64')
url="https://github.com/your-org/corelia"
license=('MIT')
source=("$pkgname-$pkgver.tar.gz::https://github.com/your-org/corelia/archive/v$pkgver.tar.gz")
sha256sums=('你的 SHA256')

package() {
  cd "$pkgname-$pkgver"
  # 安装逻辑
}
```

## 6. 性能优化

### 6.1 二进制体积优化

```toml
# Cargo.toml
[profile.release]
lto = true           # 链接时优化
codegen-units = 1    # 单个代码生成单元
opt-level = 3        # 最高优化级别
strip = true         # 去除调试符号
panic = 'abort'      # 减小 panic 处理代码体积
```

**效果**:
- 无优化：~50MB
- LTO + 优化：~15MB
- 去除符号：~12MB

### 6.2 启动速度优化

```rust
// lib.rs
// 延迟加载非关键组件
lazy_static! {
    static ref HEAVY_COMPONENT: HeavyComponent = {
        std::thread::spawn(|| {
            HeavyComponent::init()
        }).join().unwrap()
    };
}
```

### 6.3 内存优化

```typescript
// 前端懒加载
const LazySettingPanel = lazy(() => import('./SettingPanel.svelte'));

// 虚拟滚动
{#each visibleResults as result}
  <ResultItem {result} />
{/each}
```

## 7. 质量检查

### 7.1 构建前检查清单

```bash
# ✅ 类型检查
bun run check

# ✅ Rust 检查
cd src-tauri && cargo check --release

# ✅ 代码格式化
bun run format && cargo fmt

# ✅ 运行测试
bun run test

# ✅ 检查图标文件
ls src-tauri/icons/

# ✅ 检查配置文件
cat tauri.conf.json
```

### 7.2 构建后测试

```bash
# ✅ 测试安装包
# Windows
msiexec /i corelia_0.1.0_x64_en-US.msi

# macOS
open corelia.app

# Linux
./corelia_0.1.0_amd64.AppImage

# ✅ 验证功能
# - 全局快捷键
# - 搜索功能
# - 配置保存
# - 自启动
# - 系统托盘

# ✅ 性能测试
# - 启动时间 < 0.5s
# - 内存占用 < 100MB
# - CPU 占用 < 5%
```

## 8. 故障排除

### Q1: 构建失败 - Rust 编译错误

**问题**: `error: could not compile corelia`

**解决**:
```bash
# 清理构建缓存
cd src-tauri
cargo clean

# 更新依赖
cargo update

# 重新构建
cargo build --release
```

### Q2: 前端构建失败

**问题**: `Module not found`

**解决**:
```bash
# 清理 node_modules
rm -rf node_modules
rm bun.lockb

# 重新安装
bun install

# 重新构建
bun run build
```

### Q3: 代码签名失败

**问题**: `codesign failed with exit code 1`

**解决**:
```bash
# macOS: 检查证书
security find-identity -v -p codesigning

# 确认证书名称正确
# 更新 tauri.conf.json 中的 signingIdentity
```

### Q4: 安装包体积过大

**问题**: 安装包超过 50MB

**解决**:
```toml
# Cargo.toml - 优化编译选项
[profile.release]
lto = true
codegen-units = 1
opt-level = 3
strip = true
```

### Q5: 应用无法启动

**问题**: 双击无反应

**解决**:
```bash
# 查看日志
# Windows: 事件查看器
# macOS: Console.app
# Linux: journalctl -f

# 检查依赖
ldd corelia  # Linux
otool -L corelia  # macOS
```

## 9. 持续集成

### GitHub Actions 示例

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        platform: [windows-latest, macos-latest, ubuntu-latest]
    
    runs-on: ${{ matrix.platform }}
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Bun
        uses: oven-sh/setup-bun@v1
        with:
          bun-version: latest
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install dependencies
        run: bun install
      
      - name: Build
        run: bun run tauri build
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: corelia-${{ matrix.platform }}
          path: src-tauri/target/release/bundle/
```

## 10. 发布清单

### 发布前检查

- [ ] 所有测试通过
- [ ] 类型检查通过
- [ ] 代码格式化完成
- [ ] 版本号已更新
- [ ] CHANGELOG 已更新
- [ ] 文档已更新
- [ ] 图标文件完整
- [ ] 配置文件正确

### 发布步骤

1. 更新版本号
2. 更新 CHANGELOG
3. 提交并打标签
4. 触发 CI 构建
5. 上传安装包
6. 更新官网下载链接
7. 发布更新公告

### 发布后验证

- [ ] 下载安装包
- [ ] 安装并测试
- [ ] 验证自动更新
- [ ] 检查用户反馈

---

**最后更新**: 2026-04-10  
**版本**: v0.1.0

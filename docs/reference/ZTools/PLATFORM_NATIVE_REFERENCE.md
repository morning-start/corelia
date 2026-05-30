# ZTools 平台原生能力层参考 —— 50+ 原生方法的 Rust 映射

> **覆盖文件:** `src/main/core/native/index.ts` (812 行), `utils/windowUtils.ts`, `utils/systemPaths.ts`, `utils/clipboardFiles.ts`, `utils/elevation.ts`, `utils/appleScriptHelper.ts`
> **核心价值:** ZTools 所有非 Web 能力（剪贴板监听、窗口追踪、鼠标钩子、截图、取色、UWP、输入模拟）都通过 C++ Node-API 原生模块实现

---

## 1. 架构概览

### 1.1 ZTools 原生层结构

```
resources/lib/
├── win/ztools_native.node   (374KB) — Windows C++ addon
└── mac/ztools_native.node   (78KB)  — macOS C++ addon

src/main/core/native/index.ts (812 行) — TypeScript 封装层
```

ZTools 的 C++ Node-API 模块暴露了 6 个管理器类 + 50+ 原生方法：

| 类 | 职责 | 平台 | 方法数 |
|----|------|------|--------|
| `ClipboardMonitor` | 剪贴板变化监听 | Win/Mac/Linux(轮询) | 4 |
| `WindowMonitor` | 前台窗口切换追踪 | Win/Mac | 3 |
| `WindowManager` | 窗口信息、输入模拟、鼠标控制 | Win/Mac | 12 |
| `MouseMonitor` | 鼠标按键监听（中键/右键等） | Win/Mac | 3 |
| `ScreenCapture` | 区域截图 | Win | 1 |
| `ColorPicker` | 屏幕取色 | Mac | 3 |
| `UwpManager` | UWP 应用管理 | Win | 2 |
| `IconExtractor` | 文件图标提取（PNG Buffer） | Win/Mac | 1 |
| `MuiResolver` | MUI 资源字符串解析 | Win | 1 |

### 1.2 加载机制

```typescript
// 按平台加载对应的 .node 文件
let addon: any = null
if (platform === 'darwin') {
  addon = require(resources/lib/mac/ztools_native.node)
} else if (platform === 'win32') {
  addon = require(resources/lib/win/ztools_native.node)
}
// Linux: 不加载原生模块，所有方法降级
```

每个管理器类对 `addon` 做一层 JS 封装，处理参数校验、平台降级、回调适配。Linux 上所有原生方法静默降级（返回 null/false 或轮询回退）。

---

## 2. NativeAddon 接口定义

```typescript
interface NativeAddon {
  // === 剪贴板 ===
  startMonitor: (callback: () => void) => void
  stopMonitor: () => void
  getClipboardFiles: () => ClipboardFile[]
  setClipboardFiles: (files: Array<string | { path: string }>) => boolean

  // === 窗口追踪 ===
  startWindowMonitor: (callback: (info: WindowInfo) => void) => void
  stopWindowMonitor: () => void
  getActiveWindow: () => ActiveWindowResult | null
  activateWindow: (identifier: string | number) => boolean
  getExplorerFolderPath: (hwnd: number) => string | null
  readBrowserWindowUrl: (browserName: string, hwnd: number, callback: (url: string|null) => void) => void

  // === 输入模拟 ===
  simulatePaste: () => boolean
  simulateKeyboardTap: (key: string, ...modifiers: string[]) => boolean
  simulateMouseMove: (x: number, y: number) => boolean
  simulateMouseClick: (x: number, y: number) => boolean
  simulateMouseDoubleClick: (x: number, y: number) => boolean
  simulateMouseRightClick: (x: number, y: number) => boolean
  unicodeType: (segment: string) => boolean

  // === 鼠标监听 ===
  startMouseMonitor: (buttonType: string, longPressMs: number, cb: () => any) => void
  stopMouseMonitor: () => void

  // === 截录取色 ===
  startRegionCapture: (callback: (result: any) => void) => void
  startColorPicker: (callback: (result: any) => void) => void
  stopColorPicker: () => void

  // === Windows 特有 ===
  getUwpApps: () => UwpAppInfo[]
  launchUwpApp: (appId: string) => boolean
  getFileIcon: (filePath: string) => Promise<Buffer>
  resolveMuiStrings: (refs: string[]) => { [ref: string]: string }
}
```

---

## 3. 逐类分析

### 3.1 ClipboardMonitor — 剪贴板监听

| 平台 | 实现方式 | 延迟 |
|------|---------|------|
| Windows | C++ `AddClipboardFormatListener` (事件驱动) | 实时 |
| macOS | C++ NSPasteboard `changeCount` 轮询 | ~50ms |
| Linux | JS 回退：`setInterval` 500ms 轮询 `electron.clipboard.readText()` | 500ms |

```typescript
// ZTools API
const monitor = new ClipboardMonitor()
monitor.start(() => {
  // 剪贴板变化了
  const files = ClipboardMonitor.getClipboardFiles()  // Windows only
})
monitor.stop()
```

**Corelia 映射：**
- 读写：`arboard` crate
- Windows 监听：`windows-rs` + `AddClipboardFormatListener` / `SetClipboardViewer`
- macOS 监听：`objc2` + NSPasteboard `changeCount` 轮询
- Linux 监听：`arboard` 轮询或 `wl-clipboard-rs`

### 3.2 WindowMonitor — 前台窗口追踪

```typescript
// ZTools — 实时获取用户正在操作的窗口
const monitor = new WindowMonitor()
monitor.start((windowInfo) => {
  // { app, bundleId?, pid?, title?, x?, y?, width?, height?, appPath?, className?, hwnd? }
  clipboardManager.setCurrentWindow(windowInfo)  // 告诉剪贴板管理器「当前在哪个窗口」
})
```

返回的 `WindowInfo` 结构：

```typescript
interface WindowInfo {
  app: string        // "Finder.app" / "Code.exe"
  bundleId?: string  // macOS: "com.apple.finder"
  pid?: number       // 进程 ID
  title?: string     // 窗口标题
  x, y, width, height?: number  // 窗口位置和尺寸
  appPath?: string   // 应用路径
  className?: string // Windows: "CabinetWClass" / "Progman" / "WorkerW"
  hwnd?: number      // Windows: 窗口句柄
}
```

**用途：**
1. 剪贴板历史记录来源应用
2. 窗口匹配搜索（在 ZTools 中搜索当前窗口的内容）
3. 超级面板的窗口屏蔽列表

**Corelia 映射：** Rust `windows-rs` (Win32 `GetForegroundWindow` + `GetWindowText` + `GetClassName`) 或 `objc2` (macOS `NSWorkspace.sharedWorkspace().frontmostApplication`)

### 3.3 WindowManager — 窗口控制 + 输入模拟

| 方法 | 平台 | 说明 | Corelia Rust crate |
|------|------|------|-------------------|
| `getActiveWindow()` | Win/Mac | 获取前台窗口标识 | `windows-rs` / `objc2` |
| `activateWindow(id)` | Win/Mac | 激活窗口 (Win: PID, Mac: bundleId) | `windows-rs` / `objc2` |
| `simulatePaste()` | Win/Mac | 模拟 Ctrl+V / Cmd+V | `enigo` crate |
| `simulateKeyboardTap(key, ...mods)` | Win/Mac | 模拟按键 | `enigo` crate |
| `unicodeType(segment)` | Win/Mac | Unicode 逐字符输入 | `enigo` |
| `simulateMouseMove(x, y)` | Win/Mac | 鼠标移动到屏幕位置 | `enigo` |
| `simulateMouseClick(x, y)` | Win/Mac | 鼠标左键单击 | `enigo` |
| `simulateMouseDoubleClick(x, y)` | Win/Mac | 鼠标左键双击 | `enigo` |
| `simulateMouseRightClick(x, y)` | Win/Mac | 鼠标右键单击 | `enigo` |
| `getExplorerFolderPath(hwnd)` | Win | 通过 COM 查询 Explorer 文件夹路径 | `windows-rs` (IShellWindows) |
| `readBrowserWindowUrl(name, hwnd)` | Win | 读取浏览器 URL | `windows-rs` (UI Automation) |

**推荐 Rust 替代：** `enigo` crate 覆盖了所有输入模拟需求（键盘 + 鼠标）。`windows-rs` 提供 Win32 API 访问。不需要自己写 C++ 模块。

### 3.4 MouseMonitor — 鼠标按键监听

超级面板的触发机制：

```typescript
// 监听鼠标中键点击（或长按右键）
MouseMonitor.start('middle', 0, () => {
  // 0 = 点击触发, >0 = 长按 N ms 后触发
  return { shouldBlock: true }  // true = 拦截原始事件，不传递给目标窗口
})
```

**触发选项：**
- `buttonType`: `'middle'` | `'right'` | `'back'` | `'forward'`
- `longPressMs`: `0` = 点击时触发 | `>0` = 按住 N ms 后触发
- 右键只支持长按（必须 `longPressMs > 0`）

**Corelia 映射：** 没有现成的 Rust 全局鼠标钩子 crate。需要：
- Windows: `windows-rs` + `SetWindowsHookEx(WH_MOUSE_LL)` — 低层键盘鼠标钩子
- macOS: `CGEventTap` 通过 `core-graphics` crate 或 `objc2`

### 3.5 ScreenCapture — 区域截图

```typescript
// Windows only — 启动区域选择界面
ScreenCapture.start((result) => {
  // { success: true, width: 800, height: 600, x: 100, y: 100 }
})
// macOS 暂不支持
```

**Corelia 映射：** Windows: `windows-rs` + `BitBlt` / `PrintWindow`。macOS: `screencapture` CLI 或 `AVFoundation`。或使用 `xcap` crate（跨平台截图库，支持窗口/屏幕/区域）。

### 3.6 ColorPicker — 屏幕取色

```typescript
// macOS only — 显示 9x9 放大镜网格
ColorPicker.start((result) => {
  // { success: true, hex: '#59636E' }
  // { success: false, hex: null }  // 用户按 ESC 取消
})
ColorPicker.stop()  // 手动取消
```

**Corelia 映射：** macOS: `objc2` + `NSColorSampler`。Windows: 无原生取色器，需要自绘放大镜窗口或使用第三方。

### 3.7 UwpManager — Windows UWP 应用

```typescript
// 枚举已安装的 UWP 应用
const apps = UwpManager.getUwpApps()
// [{ name: '计算器', appId: 'Microsoft.WindowsCalculator_8wekyb3d8bbwe!App', icon: '...', installLocation: '...' }]

// 启动 UWP 应用
UwpManager.launchUwpApp('Microsoft.WindowsCalculator_8wekyb3d8bbwe!App')
```

**Corelia 映射：** Rust 使用 `windows-rs` + `Windows.Management.Deployment.PackageManager` + `ShellExecuteEx`。

### 3.8 IconExtractor — 文件图标提取

```typescript
const iconBuffer: Buffer = await IconExtractor.getFileIcon('C:\\Windows\\notepad.exe')
// 返回 PNG 格式的 Buffer（可用于 web 显示或保存为文件）
```

**Corelia 映射：** Rust: Windows 使用 `windows-rs` + `SHGetFileInfoW` / `IExtractImage`。macOS: `NSWorkspace.icon(forFile:)` 通过 `objc2`。或使用 `icon` crate。

### 3.9 MuiResolver — MUI 资源字符串解析

```typescript
// Windows 特有 — 解析 "@%SystemRoot%\\system32\\shell32.dll,-22067" → "文件资源管理器"
const map = MuiResolver.resolve(['@%SystemRoot%\\system32\\shell32.dll,-22067'])
```

用于 `windowsScanner.ts`：获取系统快捷方式的本地化显示名称。

**Corelia 映射：** Rust `windows-rs` + `SHLoadIndirectString`。

---

## 4. 应用扫描子系统

### 4.1 架构

```
src/main/core/commandScanner/
├── types.ts               — Command 共享类型
├── utils.ts               — pLimit 并发控制
├── windowsScanner.ts      — Windows .lnk + .url 扫描 (372 行)
├── macScanner.ts          — macOS .app bundle 扫描 (400 行)
└── linuxScanner.ts        — Linux .desktop 文件扫描 (349 行)
```

每个平台导出统一的 `scanApplications(): Promise<Command[]>` 接口。

### 4.2 Windows 扫描流程

```
getWindowsScanPaths()
  → 读取注册表获取开始菜单路径（用户 + 公共）
  → 用户开始菜单 + 桌面 + 公共开始菜单 + 公共桌面

扫描每条路径：
  1. 读取 desktop.ini 获取本地化显示名称映射
     → 解析 [LocalizedFileNames] 段
     → 遇到 MUI 引用 (@dll,-id) 时批量交给 MuiResolver
  2. 递归扫描 .lnk 和 .url 文件
     → .lnk: shell.readShortcutLink() 获取目标
     → .url: 手动解析 URL= 和 IconFile= 字段
     → 跳过 http/https 链接，保留应用协议 (steam:// 等)
  3. 过滤: 按名称关键词 (uninstall/卸载/help/readme...)
  4. 去重: 按 name + targetPath 去重

返回 Command[]: { name, path, icon: "ztools-icon://...", acronym }
```

**关键细节：** `ztools-icon://` 自定义协议在 `iconProtocol.ts` 中注册，拦截 `ztools-icon://` 请求，调用 `IconExtractor.getFileIcon()` 返回 PNG。

### 4.3 macOS 扫描流程

```
扫描目录: /Applications, /System/Applications, ~/Applications
  → 过滤 .app 目录

读取 Info.plist:
  → CFBundleDisplayName, CFBundleName

获取本地化名称（优先 .lproj → .loctable）:
  1. lproj: Content/Resources/zh-Hans.lproj/InfoPlist.strings
     → 支持 binary plist / XML plist / UTF-16 文本
  2. loctable: Content/Resources/InfoPlist.loctable
     → 新版 macOS 系统应用使用

语言匹配逻辑:
  BCP 47 → lproj 目录名候选
  "zh-Hans-CN" → ["zh-Hans", "zh-Hans_CN", "zh_CN", "zh"]
  "en-US" → ["en_US", "en"]
  传统映射: ja → Japanese, ko → Korean, fr → French

并发限制: 50 个任务并行
```

### 4.4 Linux 扫描流程

```
扫描 XDG 路径:
  ~/.local/share/applications
  /usr/share/applications, /usr/local/share/applications

解析 .desktop 文件:
  → 过滤: Type=Application, NoDisplay!=true, Exec 存在
  → 本地化名称: Name[zh_CN] > Name[zh] > Name
  → Exec 清理: 移除 %f %u %F %U 等占位符

图标查找 (XDG 图标主题):
  hicolor → scalable/apps/ → 256x256 → ... → 32x32
  pixmaps 目录直查
  .png → .svg → .xpm

拼音支持:
  → 中文名提取拼音首字母: "微信" → "wx"
  → 作为搜索别名加入 aliases

并发限制: 30 个任务并行
```

### 4.5 Corelia 迁移要点

| 平台 | ZTools 方案 | Corelia Rust 方案 |
|------|------------|-------------------|
| Windows | `shell.readShortcutLink()` + `winreg` + `MuiResolver` | `windows-rs` `IShellLink` + `SHLoadIndirectString` |
| macOS | `simple-plist` npm + `app.getPreferredSystemLanguages()` | `plist` crate + `objc2` `NSBundle` |
| Linux | `fs/promises` + `pinyin-pro` | `std::fs` + XDG 路径硬编码 |
| 图标提取 | C++ `SHGetFileInfoW` / macOS `NSWorkspace` | `windows-rs` / `objc2` / `icon` crate |

---

## 5. 应用启动子系统

### 5.1 架构

```
src/main/core/commandLauncher/
├── types.ts                    — ConfirmDialogOptions, LaunchResult
├── index.ts                    — 统一入口，按平台分发
├── windowsLauncher.ts          — 203 行，多种启动方式
├── macLauncher.ts              — 41 行，open 命令
└── linuxLauncher.ts            — 133 行，spawn + wmctrl
```

### 5.2 Windows 启动策略（最复杂）

```typescript
launchApp(appPath, confirmDialog?)
  → 如果需要确认对话框，先显示

  → 协议分发:
    uwp:appId        → UwpManager.launchUwpApp(appId)
    ms-settings:     → shell.openExternal()
    steam://         → shell.openExternal()
    http/https       → shell.openExternal()

  → 命令分发:
    PowerShell.exe x → spawn(shell: true)
    rundll32 x       → execCommand via cmd.exe
    control.exe x    → execCommand
    msdt.exe x       → execCommand

  → 扩展名分发:
    .cpl → control.exe path
    .msc → cmd.exe /c mmc.exe path
    .lnk → shell.openPath()
    .exe (在 PATH) → shell.openPath()
    .exe (完整路径) → shell.openPath() → fallback shell.openExternal()
```

### 5.3 macOS & Linux

**macOS:** `exec(\`open "\${appPath}"\`)` — 最简单，因为 `open` 命令处理一切。

**Linux:** 
1. 拆解命令字符串（带引号处理）
2. 用 `wmctrl` 检查是否已有运行实例，有则激活窗口
3. `spawn(executable, args, { detached: true, stdio: 'ignore' })` + `unref()`

### 5.4 Corelia 迁移

Tauri 有 `tauri-plugin-shell` 提供 `shell.open` 和 `shell.execute`，但复杂的启动逻辑（确认对话框、uwp、协议分发）需要 Rust 自实现。

---

## 6. 窗口材质（windowUtils.ts）

```typescript
// 111 行 — Windows 11 窗口材质
function applyWindowMaterial(win: BrowserWindow, material: 'mica' | 'acrylic' | 'none'): void
function getDefaultWindowMaterial(): string
```

- **Mica:** Windows 11 22H2+ 的 `DwmSetWindowAttribute(DWMWA_MICA)` — 桌面壁纸混合
- **Acrylic:** Windows 11 的 `DwmSetWindowAttribute(DWMWA_USE_HOSTBACKDROPBRUSH)` — 丙烯酸模糊
- **macOS:** `windowConfig.vibrancy = 'fullscreen-ui'` — 毛玻璃效果

**Corelia 映射：** Rust `windows-rs` + `DwmSetWindowAttribute` + `DwmExtendFrameIntoClientArea`。

---

## 7. 平台路径（systemPaths.ts）

```typescript
function getWindowsScanPaths(): string[]  // 开始菜单 + 桌面路径
function getMacScanPaths(): string[]      // /Applications 等
```

Windows 路径从注册表读取：

```typescript
// 读取: HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders
// 关键值: {1e875f6a-a8d6-41b8-806b-9dfb6f14acc3} = "Common Start Menu"
//        {a77f5d77-2e2b-44c3-a6a2-aba601054a51} = "Common Programs"
//        {B4BFCC3A-DB2C-424C-B029-7FE99A87C641} = "Desktop"
```

---

## 8. 其他平台工具

| 文件 | 行数 | 功能 | Corelia |
|------|------|------|---------|
| `appleScriptHelper.ts` | 330 | macOS AppleScript 自动化 | `objc2` 或 `osa_execute` |
| `clipboardFiles.ts` | 98 | 剪贴板文件路径操作 | `arboard` 部分支持 |
| `elevation.ts` | 81 | Windows UAC 提权 | `windows-rs` + ShellExecute "runas" |

---

## 9. Corelia 原生能力实现策略

### 推荐 Rust crate 清单

| 原生能力 | Rust crate | 替代方案 |
|---------|-----------|---------|
| 输入模拟（键盘/鼠标） | `enigo` | 自建 `windows-rs`/`objc2` |
| 剪贴板读写 | `arboard` | — |
| 剪贴板监听（Win） | 自建 `windows-rs` + `AddClipboardFormatListener` | — |
| 剪贴板监听（Mac） | 自建 `objc2` + NSPasteboard 轮询 | — |
| 窗口追踪 | 自建 `windows-rs` / `objc2` | — |
| 鼠标全局钩子 | 自建 `windows-rs` + `SetWindowsHookEx` | — |
| 截图 | `xcap` | `windows-rs` BitBlt |
| 取色器 | 自建（macOS `NSColorSampler`） | — |
| UWP 枚举/启动 | `windows-rs` `PackageManager` | — |
| 文件图标 | 自建（Win `SHGetFileInfoW`, Mac `NSWorkspace`） | `icon` crate |
| MUI 字符串 | `windows-rs` `SHLoadIndirectString` | — |
| 注册表读取 | `windows-rs` | — |
| AppleScript | `osa_execute` crate | — |

**决策原则：** 能用 `enigo` 和 `arboard` 的尽量用，减少自建代码。平台特定功能（剪贴板监听、鼠标钩子、窗口追踪）需要 `windows-rs` / `objc2` 自建。

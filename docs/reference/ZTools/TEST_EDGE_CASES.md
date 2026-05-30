# ZTools Test Patterns & Edge Cases 参考

> **覆盖文件:** `tests/` 目录下 15 个 Vitest 测试文件，~2400 行
> **核心价值:** ZTools 的测试模式揭示了迁移到 Tauri/Rust 时必须处理的关键边界情况和架构决策

---

## 1. 测试框架配置

```typescript
// vitest.config.ts
vitest: {
  globals: true,
  environment: 'node',        // 非 jsdom — 主进程测试
  include: ['tests/**/*.test.ts'],
  alias: {
    '@renderer': 'src/renderer/src',
    '@shared': 'src/shared'
  }
}
```

**运行:** `vitest run` (CI), `vitest` (watch mode)

---

## 2. 关键测试分类

### 2.1 纯逻辑测试（无需 mock，直接翻译到 Rust）

这些测试没有 Electron/Node 依赖，可以直接用 Rust 的 `#[cfg(test)]` 翻译：

| 测试文件 | 测试内容 | 行数 | 迁移难度 |
|---------|---------|------|---------|
| `commandUtils.test.ts` | Command ID 生成、匹配分数计算 | 207 | ★☆☆ |
| `useSearchResults.test.ts` | 搜索结果去重 | 81 | ★☆☆ |
| `common.test.ts` | 首字母缩写提取 | 62 | ★☆☆ |
| `commandMatchers.test.ts` | 命令查找/过滤/匹配 | 289 | ★☆☆ |
| `pluginRuntimeNamespace.test.ts` | 命名空间 key 生成 | 62 | ★☆☆ |
| `lmdbUtils.test.ts` | LMDB 工具函数 | 183 | ★☆☆ |
| `internalPlugins.test.ts` | 权限配置 | 34 | ★☆☆ |
| `pluginDevelopmentRegistry.test.ts` | 插件开发注册表 CRUD | 383 | ★☆☆ |
| `windowsScanner.test.ts` | 扫描过滤/去重/图标 URL | 255 | ★☆☆ |
| `systemPluginOpenFolderRegex.test.ts` | 文件夹路径正则匹配 | 74 | ★☆☆ |
| `windowsExplorerCommands.test.ts` | Windows 路径处理 | 143 | ★☆☆ |

### 2.2 需要 Mock 的测试（需要 Rust 依赖注入）

| 测试文件 | 需要 Mock 的内容 | 方案 |
|---------|-----------------|------|
| `pluginRemovalCleanup.test.ts` | Electron、fs、LMDB、window、plugin manager | ✅ `mockall` crate + trait 接口 |
| `databasePluginIsolation.test.ts` | LMDB、Electron | ✅ `heed` trait + mock |
| `pluginFeatureRuntimeNamespace.test.ts` | LMDB、Electron | ✅ trait + mock |
| `pluginPreloadInternalApi.test.ts` | IPC Renderer | ✅ Tauri `invoke` mock |

---

## 3. 核心边界情况

### 3.1 Command ID 生成 (`commandUtils.test.ts`)

```typescript
// getCommandId 的稳定 ID 生成
// 格式: "${namespace}:${encodedPath}:${name}"
// namespace: "direct" | "app" | plugin.name

// 关键边界:
test('encodes spaces and special chars in path', () => {
  expect(getCommandId('app', 'C:\\Program Files\\App\\my app.exe', 'MyApp'))
    .toBe('app:C%3A%5CProgram%20Files%5CApp%5Cmy%20app.exe:MyApp')
})

test('normalizes backslashes to forward slashes', () => {
  expect(getCommandId('app', 'C:\\Windows\\notepad.exe', '记事本'))
    .toBe('app:C%3A/Windows/notepad.exe:记事本')   // 反斜杠 → 正斜杠
})

test('separates dev plugins via __dev suffix', () => {
  const devId = getCommandId('demo__dev', '/path/to/app', 'Demo')
  const prodId = getCommandId('demo', '/path/to/app', 'Demo')
  expect(devId).not.toBe(prodId)  // 开发版和生产版不同 ID
})
```

**Corelia 映射:** Rust `String` 处理 + `urlencoding` crate。ID 生成是纯函数。

### 3.2 搜索结果去重 (`useSearchResults.test.ts`)

```typescript
// 核心规则: "same path different name" 的场景必须保留
// 如 "原神" 和 "米哈游启动器" 都指向 launcher.exe

// 非插件去重 key: (name + path) 的组合
// 插件去重 key: (path + featureCode) 的组合
// 保留首次出现的顺序

test('preserves first occurrence order', () => {
  const results = [
    { name: 'A', path: '/same.exe', type: 'app' },
    { name: 'B', path: '/same.exe', type: 'app' },  // 不同名 → 保留
    { name: 'A', path: '/same.exe', type: 'app' },  // 同名 → 去重
  ]
  expect(deduplicate(results)).toHaveLength(2)
  expect(deduplicate(results)[0].name).toBe('A')     // 保留首次顺序
  expect(deduplicate(results)[1].name).toBe('B')
})

test('same path same name dedup works case-insensitively', () => {
  // Windows 路径大小写不敏感
  const results = [
    { name: 'Notepad', path: 'C:\\Windows\\NOTEPAD.EXE' },
    { name: 'Notepad', path: 'c:\\windows\\notepad.exe' },
  ]
  expect(deduplicate(results)).toHaveLength(1)
})
```

**Corelia 映射:** Rust `HashMap` 去重，key = `(name.to_lowercase(), path.to_lowercase())`，Windows 平台做大小写归一化。

### 3.3 缩写提取 (`common.test.ts`)

```typescript
describe('extractAcronym', () => {
  test('space-separated words', () => {
    expect(extractAcronym('Visual Studio Code')).toBe('vsc')
    expect(extractAcronym('Google Chrome')).toBe('gc')
  })

  test('camelCase words', () => {
    expect(extractAcronym('VisualStudioCode')).toBe('vsc')
    expect(extractAcronym('GitHub Desktop')).toBe('ghd')
  })

  test('mixed Chinese and English', () => {
    expect(extractAcronym('米哈游 Launcher')).toBe('米l')
    // 中文取第一个字符，英文取首字母
  })

  test('single word returns empty', () => {
    expect(extractAcronym('Notepad')).toBe('')
    expect(extractAcronym('微信')).toBe('')
  })

  test('numeric prefix', () => {
    expect(extractAcronym('7-Zip')).toBe('7z')
  })
})
```

### 3.4 命名空间隔离 (`pluginRuntimeNamespace.test.ts`)

```typescript
describe('pluginRuntimeNamespace', () => {
  describe('development plugin naming', () => {
    test('toDevPluginName adds __dev suffix', () => {
      expect(toDevPluginName('demo')).toBe('demo__dev')
    })

    // 关键设计决策: toDevPluginName 不是幂等的
    test('toDevPluginName is NOT idempotent', () => {
      expect(toDevPluginName('demo__dev')).toBe('demo__dev__dev')
    })

    test('fromDevPluginName strips __dev suffix', () => {
      expect(fromDevPluginName('demo__dev')).toBe('demo')
      expect(fromDevPluginName('demo')).toBe('demo')  // 无后缀不变
    })
  })

  describe('LMDB key prefix', () => {
    test('generates PLUGIN/<name>/ prefix', () => {
      expect(getDataKeyPrefix('demo')).toBe('PLUGIN/demo/')
      expect(getDataKeyPrefix('demo__dev')).toBe('PLUGIN/demo__dev/')
    })
  })

  describe('session partition', () => {
    test('generates persist:<name>', () => {
      expect(getSessionPartition('demo')).toBe('persist:demo')
    })
  })
})
```

### 3.5 插件级联清理 (`pluginRemovalCleanup.test.ts`)

```typescript
// 这是最复杂的测试，mock 了 10+ 个模块
// 验证删除开发插件时的级联清理:

test('removes dev project completely', async () => {
  // 1. 杀死插件进程
  expect(mockKillProcess).toHaveBeenCalled()
  // 2. 清除 LMDB 数据
  expect(mockClearPluginData).toHaveBeenCalledWith('demo__dev')
  expect(mockClearStorage).toHaveBeenCalledWith('demo__dev')
  // 3. 删除插件配置条目
  expect(mockDeleteInstalledPlugin).toHaveBeenCalled()
  // 4. 清理文件系统
  expect(mockRemovePluginDir).toHaveBeenCalled()
  // 5. 不删除无关插件
  expect(mockDeleteInstalledPlugin).not.toHaveBeenCalledWith('other-plugin')
  // 6. 触发清理完成事件
  expect(mockEmitCleanupEvent).toHaveBeenCalled()
})
```

**Corelia 映射:** 这是 Rust 中 trait-based 接口设计的典型案例。每个依赖项（LMDB、FS、进程管理、窗口管理）都应该是 trait，便于测试中 mock。

### 3.6 数据库隔离 (`databasePluginIsolation.test.ts`)

```typescript
describe('DatabaseAPI plugin isolation', () => {
  test('installed vs dev plugins have separate namespaces', () => {
    // doc keys
    const installedKeys = await db.getPluginDocKeys('demo')
    const devKeys = await db.getPluginDocKeys('demo__dev')
    expect(installedKeys).not.toEqual(devKeys)

    // attachment store 独立
    const installedAttach = db.getAttachmentDb('demo')
    const devAttach = db.getAttachmentDb('demo__dev')
    expect(installedAttach).not.toBe(devAttach)

    // metadata store 独立
    const installedMeta = db.getMetaDb('demo')
    const devMeta = db.getMetaDb('demo__dev')
    expect(installedMeta).not.toBe(devMeta)
  })

  test('clearPluginData only clears target namespace', async () => {
    // 向 demo 和 demo__dev 各写入数据
    // 清空 demo → demo__dev 的数据不受影响
    await db.clearPluginData('demo')
    const devKeys = await db.getPluginDocKeys('demo__dev')
    expect(devKeys.length).toBeGreaterThan(0)
  })

  test('getPluginDataStats provides aggregate with proper separation', async () => {
    const stats = await db.getPluginDataStats('demo__dev')
    expect(stats).toHaveProperty('docCount')
    expect(stats).toHaveProperty('attachmentCount')
    expect(stats).toHaveProperty('storageSize')
  })
})
```

### 3.7 Windows 路径处理 (`windowsExplorerCommands.test.ts`)

```typescript
describe('Windows explorer path resolution', () => {
  test('desktop window returns null', () => {
    expect(resolveExplorerPath('Progman')).toBeNull()
    expect(resolveExplorerPath('WorkerW')).toBeNull()
  })

  test('file:// URL conversion', () => {
    expect(convertFileUrl('file:///C:/Users/test'))
      .toBe('C:\\Users\\test')
    expect(convertFileUrl('file:///C:/Users/test%20folder'))
      .toBe('C:\\Users\\test folder')           // URL 解码
    expect(convertFileUrl('file:///C:/path/%23hash'))
      .toBe('C:\\path\\#hash')                  // %23 → #
  })

  describe('shell escaping', () => {
    test('PowerShell escaping: doubles single quotes', () => {
      expect(escapePowershell("It's a test"))
        .toBe("It''s a test")
    })
    test('CMD escaping: caret before double quotes', () => {
      expect(escapeCmd('path with "quotes"'))
        .toBe('path with ^"quotes^"')
    })
  })
})
```

### 3.8 正则匹配文件夹路径 (`systemPluginOpenFolderRegex.test.ts`)

```typescript
// 从 plugin.json 加载的正则:
const FOLDER_PATH_REGEX = new RegExp(pluginConfig.regex)

test('matches Windows absolute paths', () => {
  expect(FOLDER_PATH_REGEX.test('C:\\Users\\test\\Documents')).toBe(true)
  expect(FOLDER_PATH_REGEX.test('D:\\Projects\\my-app')).toBe(true)
})

test('matches Unix absolute paths', () => {
  expect(FOLDER_PATH_REGEX.test('/Users/test/Documents')).toBe(true)
  expect(FOLDER_PATH_REGEX.test('~/Downloads')).toBe(true)  // 家目录
})

test('rejects invalid Windows path characters', () => {
  expect(FOLDER_PATH_REGEX.test('C:\\path\\with\\*')).toBe(false)
  expect(FOLDER_PATH_REGEX.test('C:\\path\\with\\?')).toBe(false)
  expect(FOLDER_PATH_REGEX.test('C:\\path\\with\\"')).toBe(false)
  expect(FOLDER_PATH_REGEX.test('C:\\path\\with\\<')).toBe(false)
  expect(FOLDER_PATH_REGEX.test('C:\\path\\with\\|')).toBe(false)
})

test('rejects URLs', () => {
  expect(FOLDER_PATH_REGEX.test('https://example.com')).toBe(false)
  expect(FOLDER_PATH_REGEX.test('ftp://files')).toBe(false)
})

test('rejects forward-slash Windows paths', () => {
  // Windows 风格路径必须用反斜杠
  expect(FOLDER_PATH_REGEX.test('C:/Users/test')).toBe(false)
})
```

### 3.9 搜索匹配分数 (`commandUtils.test.ts`)

```typescript
describe('calculateMatchScore', () => {
  // 分数越高匹配越好
  test('exact match gets highest score', () => {
    const score = calculateMatchScore('VSCode', 'vscode')
    expect(score).toBeGreaterThan(90)
  })

  test('prefix match gets high score', () => {
    const score = calculateMatchScore('VSC', 'vscode')
    expect(score).toBeGreaterThan(70)
  })

  test('fuzzy match gets lower score', () => {
    const score = calculateMatchScore('vs', 'vscode')
    expect(score).toBeLessThan(70)
  })

  test('acronym match (vsc → visual studio code)', () => {
    const score = calculateMatchScore('vsc', 'Visual Studio Code')
    expect(score).toBeGreaterThan(0)
  })

  test('non-match returns 0', () => {
    const score = calculateMatchScore('xyz', 'vscode')
    expect(score).toBe(0)
  })
})
```

---

## 4. 迁移测试策略

### 4.1 Rust 测试工具选型

| 需求 | Rust 方案 |
|------|----------|
| 纯逻辑测试 | `#[cfg(test)]` + `#[test]` |
| Mock trait | `mockall` crate |
| 异步测试 | `#[tokio::test]` |
| 文件系统 mock | `tempfile` crate |
| LMDB mock | `heed` + trait |
| 依赖注入 | trait + `Box<dyn Trait>` |
| 属性测试 | `proptest` / `quickcheck` crate |

### 4.2 推荐测试结构

```
src-tauri/
├── src/
│   ├── commands/           # Tauri commands
│   ├── core/               # 核心逻辑
│   │   ├── scanner/        # 应用扫描（纯逻辑）
│   │   ├── launcher/       # 应用启动
│   │   ├── clipboard/      # 剪贴板
│   │   ├── window/         # 窗口管理
│   │   └── sync/           # 同步引擎
│   ├── plugin/             # 插件系统
│   │   ├── assembly/       # 插件装配（状态机）
│   │   ├── namespace/      # 命名空间
│   │   └── registry/       # 插件注册表
│   ├── db/                 # LMDB 封装
│   └── platform/           # 平台抽象 trait
└── tests/
    ├── common/             # 跨平台测试
    ├── windows/            # Windows 特有测试
    └── integration/        # 集成测试
```

### 4.3 关键迁移测试优先级

| 优先级 | 模块 | 原因 |
|--------|------|------|
| P0 | 命名空间隔离 (`namespace`) | 数据安全基础 |
| P0 | Command ID 生成 | 搜索关键路径 |
| P0 | 窗口去重逻辑 | UI 正确性 |
| P1 | 插件级联清理 | 防止资源泄漏 |
| P1 | LMDB 读写 | 数据持久化 |
| P1 | 扫描去重/过滤 | 搜索结果质量 |
| P2 | 匹配分数算法 | 搜索排序 |
| P2 | 路径正则 | 功能完整性 |

### 4.4 从 Vitest 到 Rust 的测试转换示例

```typescript
// ZTools TypeScript test
describe('extractAcronym', () => {
  test('space-separated words', () => {
    expect(extractAcronym('Visual Studio Code')).toBe('vsc')
  })
})
```

```rust
// Corelia Rust test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_acronym_space_separated() {
        assert_eq!(extract_acronym("Visual Studio Code"), "vsc");
    }
}
```

### 4.5 Mock 模式示例

```typescript
// ZTools: vi.mock
vi.mock('electron', () => ({
  ipcMain: { handle: vi.fn() },
  BrowserWindow: vi.fn(),
}))

vi.mock('../core/database', () => ({
  DatabaseAPI: {
    getPluginDocKeys: vi.fn(),
    clearPluginData: vi.fn(),
  }
}))
```

```rust
// Corelia: mockall
use mockall::automock;

#[automock]
pub trait DatabaseApi {
    fn get_plugin_doc_keys(&self, name: &str) -> Result<Vec<String>, String>;
    fn clear_plugin_data(&self, name: &str) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_plugin_data() {
        let mut mock = MockDatabaseApi::new();
        mock.expect_clear_plugin_data()
            .with(eq("demo__dev"))
            .returning(|_| Ok(()));
        
        let result = mock.clear_plugin_data("demo__dev");
        assert!(result.is_ok());
    }
}
```

---

## 5. 测试覆盖总结

| 测试维度 | ZTools 已有测试 | Corelia 需要补充 |
|---------|---------------|-----------------|
| 命名空间隔离 | ✅ `pluginRuntimeNamespace`, `databasePluginIsolation` | Rust `heed` 命名空间 |
| 搜索去重 | ✅ `useSearchResults`, `commandMatchers` | 搜索排序算法 |
| 路径处理 | ✅ `windowsExplorerCommands`, `windowsScanner` | 跨平台路径 |
| 插件生命周期 | ✅ `pluginDevelopmentRegistry`, `pluginRemovalCleanup` | Rust 状态机 |
| 权限系统 | ✅ `internalPlugins` | Tauri capabilities |
| LMDB 工具 | ✅ `lmdbUtils` | `heed` 封装 |
| 缩写提取 | ✅ `common` | 拼音支持 |
| 正则匹配 | ✅ `systemPluginOpenFolderRegex` | 相同正则 |
| IPC 通信 | ✅ `pluginPreloadInternalApi` | Tauri commands |
| **未覆盖:** 窗口管理 | ❌ 无 | 需要新写 |
| **未覆盖:** 剪贴板操作 | ❌ 无 | 需要新写 |
| **未覆盖:** 同步引擎 | ❌ 无 | 需要新写 |
| **未覆盖:** 翻译引擎 | ❌ 无 | 需要新写 |

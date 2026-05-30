# Corelia Rust 数据层规格说明书

> **基于 ZTools LMDB 架构分析** | 目标: sled + Tauri 2.x 集成
> **设计原则:** 纯 Rust、零 unsafe、编译期类型安全、无 C 依赖
> **日期:** 2026-05-30

---

## 1. 架构总览

### 1.1 三层存储架构

```mermaid
flowchart TD
    subgraph "Application Layer"
        CMD[Tauri Commands]
        PM[PluginManager]
        CM[ClipboardManager]
        SM[SyncEngine]
        MCP[MCP Server]
    end
    
    subgraph "Data Layer (CoreliaDb)"
        MAIN[Tree: main\nJSON 文档]
        META[Tree: meta\n同步元数据]
        ATTACH[Tree: attachment\n二进制附件]
        PLUGIN[Tree: plugin_{name}\n插件独立空间]
    end
    
    subgraph "Storage Engine"
        SLED[sled::Db\nACID / MVCC / 单写多读]
    end
    
    CMD --> MAIN
    CMD --> META
    PM --> PLUGIN
    CM --> ATTACH
    SM --> META
    MAIN --> SLED
    META --> SLED
    ATTACH --> SLED
    PLUGIN --> SLED
```

### 1.2 核心设计决策

| 决策 | 选择 | 替代方案 | 理由 |
|------|------|---------|------|
| 存储引擎 | **sled** | heed (lmdb-rs) / rusqlite | 纯 Rust、无 C 依赖、原生 Tree |
| 序列化 | **serde_json** | bincode / msgpack / protobuf | 人类可读、调试友好、Tauri IPC 原生支持 |
| ID生成 | **UUID v4** | 自增 / snowflake | 分布式友好、无需协调 |
| 并发控制 | **sled 内置 MVCC** | 自定义 RwLock | sled 已处理、简化代码 |
| 错误处理 | **sled::Error + thiserror** | Box<dyn Error> | 类型化错误、调用方可匹配 |
| 异步 | **tokio + tokio::task::spawn_blocking** | 直接同步调用 | sled 是同步的，用 spawn_blocking 不阻塞主线程 |

---

## 2. 核心结构体

### 2.1 CoreliaDb

```rust
// src-tauri/src/core/db/database.rs
use sled::{Db, Tree, IVec};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

pub type DbResult<T> = Result<T, DbError>;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Storage error: {0}")]
    Storage(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),
}

/// 三棵树 + 插件命名空间管理
pub struct CoreliaDb {
    db: Db,
}

impl CoreliaDb {
    /// 打开（或创建）数据库
    pub fn open(path: impl Into<PathBuf>) -> DbResult<Self> {
        let db = sled::open(path.into())?;
        
        // 初始化三棵树（如果不存在则创建）
        db.open_tree("main")?;
        db.open_tree("meta")?;
        db.open_tree("attachment")?;
        
        Ok(CoreliaDb { db })
    }
    
    // === Tree 访问器 ===
    
    /// 主数据树：存储结构化 JSON 文档
    pub fn main(&self) -> Tree { self.db.open_tree("main").unwrap() }
    
    /// 元数据树：存储 `_rev`、`_lastModified` 等同步元数据
    pub fn meta(&self) -> Tree { self.db.open_tree("meta").unwrap() }
    
    /// 附件树：存储二进制数据（图片、文件）
    pub fn attachment(&self) -> Tree { self.db.open_tree("attachment").unwrap() }
    
    /// 插件的命名空间树
    pub fn plugin_tree(&self, name: &str) -> DbResult<Tree> {
        Ok(self.db.open_tree(format!("plugin_{}", name))?)
    }
    
    /// 删除插件及所有数据
    pub fn drop_plugin(&self, name: &str) -> DbResult<()> {
        self.db.drop_tree(format!("plugin_{}", name))?;
        Ok(())
    }
}
```

### 2.2 文档类型

```rust
// src-tauri/src/core/db/types.rs
use serde::{Serialize, Deserialize};

/// 通用文档结构（存储为 JSON）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document<T: Serialize> {
    pub id: String,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<DocMeta>,
}

/// 文档元数据（存储在 meta Tree）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMeta {
    pub rev: u64,              // 版本号，同步用
    pub last_modified: u64,    // 最后修改时间（Unix 毫秒时间戳）
    pub created_at: u64,       // 创建时间
    pub size: u64,             // 数据大小（字节）
}

/// 剪贴板条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: String,            // UUID
    pub content_type: ClipboardContentType,
    pub text: Option<String>,
    pub image: Option<String>, // base64 编码
    pub file_paths: Option<Vec<String>>,
    pub app_name: Option<String>, // 来源应用
    pub timestamp: u64,
    pub is_favorited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardContentType {
    Text,
    Image,
    File,
    RichText,
    Unknown,
}

/// 应用扫描结果（Command 来源之一）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub path: String,
    pub icon: Option<String>,    // base64 图标
    pub category: AppCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppCategory {
    Application,
    SystemSetting,
    Utility,
    Development,
    Browser,
}
```

---

## 3. 命名空间约定

### 3.1 Key 命名规则

```
前缀/类别/标识符
```

| 前缀 | 所属 | 示例 |
|------|------|------|
| `ZTOOLS/` | 主程序 | `ZTOOLS/settings/general` |
| `ZTOOLS/` | 主程序 | `ZTOOLS/clipboard/{uuid}` |
| `ZTOOLS/` | 主程序 | `ZTOOLS/apps/{app_name}` |
| `PLUGIN/{name}/` | 插件 | (通过 Tree 隔离，不需要 key 前缀) |
| `SYNC/` | 同步引擎 | `SYNC/config` |

### 3.2 主要 Key 映射

| Key | Tree | 值类型 | 说明 |
|-----|------|--------|------|
| `ZTOOLS/settings/{key}` | main | JSON | 通用设置 |
| `ZTOOLS/clipboard/{uuid}` | main | JSON(ClipboardEntry) | 剪贴板历史 |
| `ZTOOLS/clipboard/index` | main | JSON(数组) | 剪贴板索引（支持快速分页） |
| `ZTOOLS/apps/{app_name}` | main | JSON(AppEntry) | 扫描到的应用 |
| `ZTOOLS/commands/index` | main | JSON(Command数组) | 全量 Command 索引 |
| `ZTOOLS/history/{uuid}` | main | JSON | 使用历史记录 |
| `ZTOOLS/pinned/{uuid}` | main | JSON | 固定项 |
| `ZTOOLS/themes/{name}` | main | JSON | 主题配置 |
| `meta/{key}` | meta | JSON(DocMeta) | 文档同步元数据 |
| `attachments/{uuid}` | attachment | 二进制 | 剪贴板图片/文件 |
| `SYNC/config` | main | JSON | 同步配置 |
| `SYNC/status` | main | JSON | 同步状态 |
| (插件数据) | plugin_{name} | JSON | 插件自动隔离 |

### 3.3 代码引用

```rust
impl CoreliaDb {
    // === 设置 ===
    
    pub fn get_setting(&self, key: &str) -> DbResult<Option<String>> {
        let key = format!("ZTOOLS/settings/{}", key);
        match self.main().get(key.as_bytes())? {
            Some(v) => Ok(Some(String::from_utf8_lossy(&v).to_string())),
            None => Ok(None),
        }
    }
    
    pub fn set_setting(&self, key: &str, value: &str) -> DbResult<()> {
        let key = format!("ZTOOLS/settings/{}", key);
        self.main().insert(key.as_bytes(), value.as_bytes())?;
        Ok(())
    }
    
    // === 剪贴板历史 ===
    
    pub fn save_clipboard_entry(&self, entry: &ClipboardEntry) -> DbResult<()> {
        let key = format!("ZTOOLS/clipboard/{}", entry.id);
        let json = serde_json::to_string(entry)?;
        self.main().insert(key.as_bytes(), json.as_bytes())?;
        Ok(())
    }
    
    pub fn get_clipboard_history(&self, limit: usize) -> DbResult<Vec<ClipboardEntry>> {
        let prefix = "ZTOOLS/clipboard/";
        let mut entries = Vec::new();
        
        for result in self.main().scan_prefix(prefix.as_bytes()) {
            let (_, value) = result?;
            let entry: ClipboardEntry = serde_json::from_slice(&value)?;
            entries.push(entry);
            if entries.len() >= limit { break; }
        }
        
        Ok(entries)
    }
}
```

---

## 4. 同步元数据引擎

### 4.1 元数据自动更新

```rust
impl CoreliaDb {
    /// 写入文档并自动更新元数据
    pub fn put_with_meta(&self, key: &str, value: &[u8]) -> DbResult<()> {
        // 写入主数据
        self.main().insert(key.as_bytes(), value)?;
        
        // 写入/更新元数据
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let meta_key = format!("meta/{}", key);
        let existing_meta: Option<DocMeta> = self.meta()
            .get(meta_key.as_bytes())?
            .map(|v| serde_json::from_slice(&v).unwrap());
        
        let new_meta = DocMeta {
            rev: existing_meta.map(|m| m.rev + 1).unwrap_or(1),
            last_modified: now,
            created_at: existing_meta.map(|m| m.created_at).unwrap_or(now),
            size: value.len() as u64,
        };
        
        self.meta()
            .insert(meta_key.as_bytes(), serde_json::to_vec(&new_meta)?)?;
        
        Ok(())
    }
    
    /// 批量获取所有文档的元数据（给同步引擎用）
    pub fn get_all_meta(&self) -> DbResult<Vec<(String, DocMeta)>> {
        let mut results = Vec::new();
        for result in self.meta().scan_prefix(b"meta/") {
            let (key, value) = result?;
            let key_str = String::from_utf8_lossy(&key)
                .strip_prefix("meta/")
                .unwrap_or("")
                .to_string();
            let meta: DocMeta = serde_json::from_slice(&value)?;
            results.push((key_str, meta));
        }
        Ok(results)
    }
    
    /// 获取需要同步的变更（基于时间戳）
    pub fn get_changes_since(&self, since: u64) -> DbResult<Vec<(String, DocMeta)>> {
        Ok(self.get_all_meta()?
            .into_iter()
            .filter(|(_, meta)| meta.last_modified > since)
            .collect())
    }
}
```

### 4.2 同步冲突检测

```rust
/// LWW (Last-Write-Wins) 冲突解决
pub fn resolve_conflict(
    local: &DocMeta,
    remote: &DocMeta,
) -> ConflictResolution {
    // 比较版本号和修改时间
    match local.rev.cmp(&remote.rev) {
        std::cmp::Ordering::Greater => ConflictResolution::KeepLocal,
        std::cmp::Ordering::Less => ConflictResolution::UseRemote,
        std::cmp::Ordering::Equal => {
            // 版本号相同，比较时间戳
            match local.last_modified.cmp(&remote.last_modified) {
                std::cmp::Ordering::Greater => ConflictResolution::KeepLocal,
                std::cmp::Ordering::Less => ConflictResolution::UseRemote,
                std::cmp::Ordering::Equal => ConflictResolution::NoChange,
            }
        }
    }
}

pub enum ConflictResolution {
    KeepLocal,
    UseRemote,
    NoChange,
}
```

---

## 5. Tauri 集成

### 5.1 状态管理

```rust
// src-tauri/src/main.rs 或 lib.rs
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 初始化数据库
            let app_data_dir = app.path().app_data_dir()?;
            let db_path = app_data_dir.join("corelia.db");
            let db = CoreliaDb::open(db_path)?;
            
            // 将 db 注入为 Tauri State
            app.manage(db);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 数据 Commands
            commands::db_get_setting,
            commands::db_set_setting,
            commands::db_get_clipboard_history,
            commands::db_save_clipboard_entry,
            // 插件 Commands
            commands::plugin_db_get,
            commands::plugin_db_put,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 5.2 Commands 中使用数据库

```rust
// src-tauri/src/commands/settings.rs
use tauri::State;
use crate::core::db::CoreliaDb;

#[tauri::command]
pub fn get_setting(app: State<'_, CoreliaDb>, key: String) -> Result<Option<String>, String> {
    app.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(app: State<'_, CoreliaDb>, key: String, value: String) -> Result<(), String> {
    app.set_setting(&key, &value).map_err(|e| e.to_string())
}
```

### 5.3 插件命名空间命令

```rust
// src-tauri/src/commands/plugins.rs
#[tauri::command]
pub async fn plugin_db_get(
    app: State<'_, CoreliaDb>,
    plugin_name: String,
    key: String,
) -> Result<Option<String>, String> {
    let tree = app.plugin_tree(&plugin_name).map_err(|e| e.to_string())?;
    
    match tree.get(key.as_bytes()) {
        Ok(Some(value)) => Ok(Some(String::from_utf8_lossy(&value).to_string())),
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn plugin_db_put(
    app: State<'_, CoreliaDb>,
    plugin_name: String,
    key: String,
    value: String,
) -> Result<(), String> {
    let tree = app.plugin_tree(&plugin_name).map_err(|e| e.to_string())?;
    tree.insert(key.as_bytes(), value.as_bytes())
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn plugin_db_delete(
    app: State<'_, CoreliaDb>,
    plugin_name: String,
    key: String,
) -> Result<(), String> {
    let tree = app.plugin_tree(&plugin_name).map_err(|e| e.to_string())?;
    tree.remove(key.as_bytes())
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

---

## 6. Cargo.toml 依赖

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
tauri-plugin-global-shortcut = "2"
tauri-plugin-clipboard-manager = "2"
tauri-plugin-single-instance = "2"
tauri-plugin-updater = "2"

# 数据层
sled = "0.34"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"

# 剪贴板监听
arboard = "3"
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_DataExchange",
] }
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"

# 同步引擎
reqwest = { version = "0.12", features = ["json"] }
sha2 = "0.10"
notify = "7"

# MCP Server
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.6", features = ["cors"] }

# 工具
rand = "0.8"
log = "0.4"
```

---

## 7. 性能目标

| 操作 | 目标延迟 | 测量场景 |
|------|---------|---------|
| 单 key 读 (热) | < 0.01ms | 内存中已缓存的 sled page |
| 单 key 读 (冷) | < 0.1ms | 从磁盘 page 加载 |
| 单 key 写 (同步) | < 0.05ms | insert + 不等待刷盘 |
| 批量写 100 条 | < 2ms | 批量 insert + 刷盘 |
| 前缀扫描 1000 条 | < 1ms | 迭代 1000 个 key |
| 插件 Tree 创建 | < 0.1ms | open_tree 新名称 |
| 插件 Tree drop | < 1ms | 删除整棵树 + 数据 |

## 8. 备份与恢复

```rust
impl CoreliaDb {
    /// 导出整个数据库为 JSON（手动备份）
    pub fn export_json(&self) -> DbResult<String> {
        let mut export = serde_json::Map::new();
        
        // 导出 main tree
        let mut main_data = serde_json::Map::new();
        for result in self.main().iter() {
            let (key, value) = result?;
            let key_str = String::from_utf8_lossy(&key).to_string();
            let val_str = String::from_utf8_lossy(&value).to_string();
            main_data.insert(key_str, serde_json::Value::String(val_str));
        }
        export.insert("main".into(), serde_json::Value::Object(main_data));
        
        serde_json::to_string_pretty(&export).map_err(|e| e.into())
    }
    
    /// 从 JSON 恢复
    pub fn import_json(&self, json: &str) -> DbResult<()> {
        let export: serde_json::Value = serde_json::from_str(json)?;
        
        if let Some(main) = export.get("main").and_then(|v| v.as_object()) {
            for (key, value) in main {
                if let Some(val_str) = value.as_str() {
                    self.main().insert(key.as_bytes(), val_str.as_bytes())?;
                }
            }
        }
        
        self.db.flush()?;
        Ok(())
    }
    
    /// 刷新到磁盘
    pub fn flush(&self) -> DbResult<()> {
        self.db.flush()?;
        Ok(())
    }
}
```

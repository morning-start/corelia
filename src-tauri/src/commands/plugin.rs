// 插件数据隔离存储 Commands
// 为每个插件提供独立的数据空间，支持读写、删除、清空等操作
// 包含 10MB 配额限制，防止插件滥用存储空间

use serde_json::Value;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

/// 单个插件的最大数据存储限制：10MB
const MAX_PLUGIN_DATA_SIZE: u64 = 10 * 1024 * 1024;

/// 内部辅助函数：检查并强制执行配额限制
fn enforce_quota(app: &AppHandle, plugin_id: &str, additional_bytes: u64) -> Result<(), String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    let data_dir = app_dir.join("data").join(plugin_id);

    let current_size = if data_dir.exists() {
        // 简化版：只检查 storage.json 文件大小
        let storage_file = data_dir.join("storage.json");
        if storage_file.exists() {
            std::fs::metadata(&storage_file)
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    if current_size + additional_bytes > MAX_PLUGIN_DATA_SIZE {
        Err(format!(
            "Plugin '{}' exceeds quota limit ({} bytes). Current: {}, Additional: {}, Max: {}",
            plugin_id, current_size + additional_bytes, current_size, additional_bytes, MAX_PLUGIN_DATA_SIZE
        ))
    } else {
        Ok(())
    }
}

/// 获取插件的数据目录路径
/// 自动创建目录（如不存在）
#[tauri::command]
pub async fn get_plugin_data_path(app: AppHandle, plugin_id: String) -> Result<String, String> {
    // 1. 获取应用数据目录 ($APPDATA/com.morningstart.corelia/)
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    // 2. 构建插件数据目录: data/{plugin_id}/
    let data_dir = app_dir.join("data").join(&plugin_id);

    // 3. 自动创建目录（递归创建父目录）
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create plugin data dir: {}", e))?;

    // 4. 返回绝对路径字符串
    Ok(data_dir.to_string_lossy().to_string())
}

/// 读取插件私有数据
/// 从 data/{plugin_id}/storage.json 读取
#[tauri::command]
pub async fn read_plugin_data(
    app: AppHandle,
    plugin_id: String,
    key: String,
) -> Result<Value, String> {
    // 1. 构建存储文件名: data/{plugin_id}/storage.json
    let store_file = format!("data/{}/storage.json", plugin_id);

    // 2. 获取 Store 实例
    let store = app.store(&store_file)
        .map_err(|e| format!("Failed to open store: {}", e))?;

    // 3. 按 key 读取数据
    match store.get(&key) {
        Some(value) => Ok(value),
        None => Err(format!("Key '{}' not found in plugin '{}'", key, plugin_id)),
    }
}

/// 写入插件私有数据
/// 写入到 data/{plugin_id}/storage.json
#[tauri::command]
pub async fn write_plugin_data(
    app: AppHandle,
    plugin_id: String,
    key: String,
    value: Value,
) -> Result<(), String> {
    // 配额检查（粗略估算：value 序列化后的大小）
    let estimated_size = value.to_string().len() as u64;
    enforce_quota(&app, &plugin_id, estimated_size)?;

    // 1. 构建存储文件名
    let store_file = format!("data/{}/storage.json", plugin_id);

    // 2. 获取 Store 实例
    let store = app.store(&store_file)
        .map_err(|e| format!("Failed to open store: {}", e))?;

    // 3. 写入数据
    store.set(&key, value);

    // 4. 保存到磁盘
    store.save()
        .map_err(|e| format!("Failed to save store: {}", e))?;

    println!("[PluginStore] {} wrote key '{}'", plugin_id, key);
    Ok(())
}

/// 删除插件私有数据中的某个 key
#[tauri::command]
pub async fn delete_plugin_data(
    app: AppHandle,
    plugin_id: String,
    key: String,
) -> Result<(), String> {
    let store_file = format!("data/{}/storage.json", plugin_id);
    let store = app.store(&store_file)
        .map_err(|e| format!("Failed to open store: {}", e))?;

    store.delete(&key);
    store.save()
        .map_err(|e| format!("Failed to save store: {}", e))?;

    Ok(())
}

/// 清除插件的所有数据（危险操作）
#[tauri::command]
pub async fn clear_plugin_data(
    app: AppHandle,
    plugin_id: String,
) -> Result<(), String> {
    // 1. 获取数据目录
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let data_dir = app_dir.join("data").join(&plugin_id);

    // 2. 检查目录是否存在
    if !data_dir.exists() {
        return Err(format!("Plugin data directory does not exist: {}", plugin_id));
    }

    // 3. 删除整个目录及其内容
    std::fs::remove_dir_all(&data_dir)
        .map_err(|e| format!("Failed to clear plugin data: {}", e))?;

    println!("[PluginStore] Cleared all data for plugin '{}'", plugin_id);
    Ok(())
}

/// 获取插件数据大小（字节）
#[tauri::command]
pub async fn get_plugin_data_size(
    app: AppHandle,
    plugin_id: String,
) -> Result<u64, String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let data_dir = app_dir.join("data").join(&plugin_id);

    if !data_dir.exists() {
        return Ok(0); // 无数据
    }

    // 递归计算目录大小
    fn dir_size(dir: &PathBuf) -> u64 {
        match std::fs::read_dir(dir) {
            Ok(reader) => {
                reader
                    .filter_map(Result::ok)
                    .map(|entry| {
                        let path = entry.path();
                        if path.is_dir() {
                            dir_size(&path)
                        } else {
                            std::fs::metadata(&path)
                                .map(|m| m.len())
                                .unwrap_or(0)
                        }
                    })
                    .sum()
            }
            Err(e) => {
                eprintln!("[PluginStore] 无法读取目录 {}: {}", dir.display(), e);
                0
            }
        }
    }

    Ok(dir_size(&data_dir))
}

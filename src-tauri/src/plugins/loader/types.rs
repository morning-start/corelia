use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

/// 插件元数据（来自 plugin.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub plugin_type: String,
    pub logo: Option<String>,
    pub prefix: Option<String>,
    pub main: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(default)]
    pub patches: Vec<String>,
    pub features: Option<Vec<FeatureConfig>>,
}

/// 功能配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub code: String,
    pub label: String,
    #[serde(rename = "type")]
    pub feature_type: String,
    pub items: Option<Vec<FeatureItem>>,
}

/// 功能子项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureItem {
    pub label: String,
    pub action: String,
    pub icon: Option<String>,
}

/// 动态注册的功能（运行时由插件调用 registerPluginFeature 注册）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredFeature {
    pub code: String,
    pub label: String,
    #[serde(rename = "type")]
    pub feature_type: String,
}

/// 插件运行时状态（状态机：MetaLoaded → Loading → Ready/Cached → Unloaded）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PluginState {
    MetaLoaded,
    Loading,
    Ready,
    Cached,
    Unloaded,
    Error(String),
}

impl std::fmt::Display for PluginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginState::MetaLoaded => write!(f, "MetaLoaded"),
            PluginState::Loading => write!(f, "Loading"),
            PluginState::Ready => write!(f, "Ready"),
            PluginState::Cached => write!(f, "Cached"),
            PluginState::Unloaded => write!(f, "Unloaded"),
            PluginState::Error(msg) => write!(f, "Error({})", msg),
        }
    }
}

/// 插件实例（运行时状态）
#[derive(Debug, Clone)]
pub struct PluginInstance {
    pub id: String,
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub vm_id: Option<String>,
    pub plugin_dir: PathBuf,
    pub loaded_at: Option<Instant>,
    pub last_used: Option<Instant>,
    pub registered_features: Vec<RegisteredFeature>,
    pub on_ready_callback: Option<String>,
    pub on_out_callback: Option<String>,
    pub load_error_count: u32,
    pub max_retries: u32,
    pub last_error: Option<String>,
    pub retry_after: Option<Instant>,
    pub retry_backoff_ms: u64,
}

/// 插件健康状态摘要（用于监控面板）
#[derive(Debug, Clone, serde::Serialize)]
pub struct PluginHealth {
    pub id: String,
    pub state: String,
    pub vm_id: Option<String>,
    pub loaded_at: Option<u64>,
    pub last_used: Option<u64>,
    pub error_count: u32,
    pub last_error: Option<String>,
}

/// 加载插件命令的返回值
#[derive(serde::Serialize)]
pub struct LoadResult {
    pub state: String,
    pub vm_id: Option<String>,
}
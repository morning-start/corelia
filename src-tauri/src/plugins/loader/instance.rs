use std::time::Instant;
use super::manifest::{PluginManifest, RegisteredFeature};
use super::state::PluginState;

#[derive(Debug, Clone)]
pub struct PluginInstance {
    pub id: String,
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub vm_id: Option<String>,
    pub plugin_dir: std::path::PathBuf,
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

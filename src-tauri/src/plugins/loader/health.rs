use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PluginHealth {
    pub id: String,
    pub state: String,
    pub vm_id: Option<String>,
    pub loaded_at: Option<u64>,
    pub last_used: Option<u64>,
    pub error_count: u32,
    pub last_error: Option<String>,
}

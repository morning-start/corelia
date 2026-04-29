use serde::{Deserialize, Serialize};

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

use crate::plugins::loader::types::{PluginInstance, PluginManifest, PluginState};
use crate::plugins::loader::PluginLoader;

impl PluginLoader {
    pub fn list_plugins(&self) -> Vec<&PluginInstance> {
        self.instances.values().collect()
    }

    pub fn list_manifests(&self) -> Vec<PluginManifest> {
        self.instances.values().map(|p| p.manifest.clone()).collect()
    }

    pub fn get_plugin(&self, id: &str) -> Option<&PluginInstance> {
        self.instances.get(id)
    }

    pub fn find_by_prefix(&self, prefix: &str) -> Vec<&PluginInstance> {
        let mut matches = Vec::new();
        let prefix_lower = prefix.to_lowercase();

        for instance in self.instances.values() {
            if let Some(ref p) = instance.manifest.prefix {
                let p_lower = p.to_lowercase();
                if p_lower.starts_with(&prefix_lower) || prefix_lower.starts_with(&p_lower) {
                    matches.push(instance);
                }
            }
        }
        matches
    }

    pub fn loaded_count(&self) -> usize {
        self.instances.values()
            .filter(|p| matches!(p.state, PluginState::Ready | PluginState::Cached))
            .count()
    }

    pub fn total_count(&self) -> usize {
        self.instances.len()
    }
}
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub code: String,
    pub label: String,
    #[serde(rename = "type")]
    pub feature_type: String,
    pub items: Option<Vec<FeatureItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureItem {
    pub label: String,
    pub action: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredFeature {
    pub code: String,
    pub label: String,
    #[serde(rename = "type")]
    pub feature_type: String,
}

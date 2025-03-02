use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigResponse {
    #[serde(rename = "EXCLUDED_MULTI_FILES")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_multi_files: Vec<String>,
    #[serde(rename = "EXCLUDED_MULTI_PARTS_EXT")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_multi_parts_ext: Vec<String>,
    #[serde(rename = "EXCLUDED_MULTI_PARTS_FILES")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_multi_parts_files: Vec<String>,
    #[serde(rename = "EXCLUDED_PLATFORMS")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_platforms: Vec<String>,
    #[serde(rename = "EXCLUDED_SINGLE_EXT")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_single_ext: Vec<String>,
    #[serde(rename = "EXCLUDED_SINGLE_FILES")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_single_files: Vec<String>,
    #[serde(rename = "PLATFORMS_BINDING")]
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub platforms_binding: serde_json::Value,
    #[serde(rename = "PLATFORMS_VERSIONS")]
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub platforms_versions: serde_json::Value,
}
impl std::fmt::Display for ConfigResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

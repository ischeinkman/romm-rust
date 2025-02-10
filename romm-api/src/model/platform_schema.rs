use super::FirmwareSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_name: Option<serde_json::Value>,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family_name: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family_slug: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firmware: Option<Vec<FirmwareSchema>>,
    pub fs_slug: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<serde_json::Value>,
    pub id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub igdb_id: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_path: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moby_id: Option<serde_json::Value>,
    pub name: String,
    pub rom_count: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sgdb_id: Option<serde_json::Value>,
    pub slug: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_logo: Option<serde_json::Value>,
}
impl std::fmt::Display for PlatformSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

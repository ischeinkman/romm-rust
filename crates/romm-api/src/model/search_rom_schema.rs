use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchRomSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub igdb_id: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub igdb_url_cover: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moby_id: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moby_url_cover: Option<String>,
    pub name: String,
    pub platform_id: i64,
    pub slug: String,
    pub summary: String,
}
impl std::fmt::Display for SearchRomSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

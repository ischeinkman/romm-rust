use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectionSchema {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub description: String,
    pub has_cover: bool,
    pub id: i64,
    pub is_public: bool,
    pub name: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub path_cover_l: serde_json::Value,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub path_cover_s: serde_json::Value,
    pub rom_count: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roms: Vec<i64>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub url_cover: String,
    #[serde(rename = "user__username")]
    pub user_username: String,
    pub user_id: i64,
}
impl std::fmt::Display for CollectionSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

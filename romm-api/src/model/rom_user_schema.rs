use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RomUserSchema {
    pub backlogged: bool,
    pub completion: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub difficulty: i64,
    pub hidden: bool,
    pub id: i64,
    pub is_main_sibling: bool,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub last_played: serde_json::Value,
    pub note_is_public: bool,
    pub note_raw_markdown: String,
    pub now_playing: bool,
    pub rating: i64,
    pub rom_id: i64,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub status: serde_json::Value,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "user__username")]
    pub user_username: String,
    pub user_id: i64,
}
impl std::fmt::Display for RomUserSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

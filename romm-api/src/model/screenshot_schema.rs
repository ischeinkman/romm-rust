use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScreenshotSchema {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub download_path: String,
    pub file_extension: String,
    pub file_name: String,
    pub file_name_no_ext: String,
    pub file_name_no_tags: String,
    pub file_path: String,
    pub file_size_bytes: i64,
    pub full_path: String,
    pub id: i64,
    pub rom_id: i64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub user_id: i64,
}
impl std::fmt::Display for ScreenshotSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

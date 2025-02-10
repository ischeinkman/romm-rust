use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirmwareSchema {
    pub crc_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub file_extension: String,
    pub file_name: String,
    pub file_name_no_ext: String,
    pub file_name_no_tags: String,
    pub file_path: String,
    pub file_size_bytes: i64,
    pub full_path: String,
    pub id: i64,
    pub is_verified: bool,
    pub md5_hash: String,
    pub sha1_hash: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
impl std::fmt::Display for FirmwareSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

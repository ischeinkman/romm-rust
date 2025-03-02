use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RomFile {
    pub filename: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub last_modified: serde_json::Value,
    pub size: i64,
}
impl std::fmt::Display for RomFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

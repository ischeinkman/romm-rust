use super::SaveSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UploadedSavesResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub saves: Vec<SaveSchema>,
    pub uploaded: i64,
}
impl std::fmt::Display for UploadedSavesResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

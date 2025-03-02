use super::ScreenshotSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UploadedScreenshotsResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub merged_screenshots: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub screenshots: Vec<ScreenshotSchema>,
    pub uploaded: i64,
}
impl std::fmt::Display for UploadedScreenshotsResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

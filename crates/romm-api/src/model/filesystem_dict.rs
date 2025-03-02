use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilesystemDict {
    #[serde(rename = "FS_PLATFORMS")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fs_platforms: Vec<String>,
}
impl std::fmt::Display for FilesystemDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

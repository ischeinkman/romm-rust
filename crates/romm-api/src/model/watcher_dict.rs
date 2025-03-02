use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WatcherDict {
    #[serde(rename = "ENABLED")]
    pub enabled: bool,
    #[serde(rename = "MESSAGE")]
    pub message: String,
    #[serde(rename = "TITLE")]
    pub title: String,
}
impl std::fmt::Display for WatcherDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

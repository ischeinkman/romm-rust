use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatsReturn {
    #[serde(rename = "FILESIZE")]
    pub filesize: i64,
    #[serde(rename = "PLATFORMS")]
    pub platforms: i64,
    #[serde(rename = "ROMS")]
    pub roms: i64,
    #[serde(rename = "SAVES")]
    pub saves: i64,
    #[serde(rename = "SCREENSHOTS")]
    pub screenshots: i64,
    #[serde(rename = "STATES")]
    pub states: i64,
}
impl std::fmt::Display for StatsReturn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

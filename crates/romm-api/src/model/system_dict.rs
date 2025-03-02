use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemDict {
    #[serde(rename = "SHOW_SETUP_WIZARD")]
    pub show_setup_wizard: bool,
    #[serde(rename = "VERSION")]
    pub version: String,
}
impl std::fmt::Display for SystemDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

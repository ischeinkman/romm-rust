use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OidcDict {
    #[serde(rename = "ENABLED")]
    pub enabled: bool,
    #[serde(rename = "PROVIDER")]
    pub provider: String,
}
impl std::fmt::Display for OidcDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

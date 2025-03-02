use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontendDict {
    #[serde(rename = "DISABLE_USERPASS_LOGIN")]
    pub disable_userpass_login: bool,
    #[serde(rename = "UPLOAD_TIMEOUT")]
    pub upload_timeout: i64,
}
impl std::fmt::Display for FrontendDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

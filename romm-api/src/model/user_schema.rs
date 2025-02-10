use super::Role;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSchema {
    pub avatar_path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub enabled: bool,
    pub id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_active: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_login: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub oauth_scopes: Vec<String>,
    pub role: Role,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub username: String,
}
impl std::fmt::Display for UserSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

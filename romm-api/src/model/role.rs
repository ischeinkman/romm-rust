use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Role {
    #[serde(rename = "viewer")]
    Viewer,
    #[serde(rename = "editor")]
    Editor,
    #[serde(rename = "admin")]
    Admin,
}

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthOpenidApiOauthOpenidGetResponse(pub serde_json::Value);

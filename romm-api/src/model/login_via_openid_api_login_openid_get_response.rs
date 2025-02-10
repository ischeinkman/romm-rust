use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoginViaOpenidApiLoginOpenidGetResponse(pub serde_json::Value);

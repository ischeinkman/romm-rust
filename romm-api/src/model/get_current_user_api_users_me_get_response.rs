use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetCurrentUserApiUsersMeGetResponse(pub serde_json::Value);

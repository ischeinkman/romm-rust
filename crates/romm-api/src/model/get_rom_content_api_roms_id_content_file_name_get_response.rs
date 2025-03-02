use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetRomContentApiRomsIdContentFileNameGetResponse(pub serde_json::Value);

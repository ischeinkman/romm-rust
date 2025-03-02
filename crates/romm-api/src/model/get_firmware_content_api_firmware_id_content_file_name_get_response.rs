use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetFirmwareContentApiFirmwareIdContentFileNameGetResponse(pub serde_json::Value);

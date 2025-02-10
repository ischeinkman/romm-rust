use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetRawAssetApiRawAssetsPathGetResponse(pub serde_json::Value);

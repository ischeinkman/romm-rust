use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeadRawAssetApiRawAssetsPathHeadResponse(pub serde_json::Value);

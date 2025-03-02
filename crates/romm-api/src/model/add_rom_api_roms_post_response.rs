use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddRomApiRomsPostResponse(pub serde_json::Value);

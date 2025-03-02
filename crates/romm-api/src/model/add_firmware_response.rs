use super::FirmwareSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddFirmwareResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub firmware: Vec<FirmwareSchema>,
    pub uploaded: i64,
}
impl std::fmt::Display for AddFirmwareResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

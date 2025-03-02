use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmulationDict {
    #[serde(rename = "DISABLE_EMULATOR_JS")]
    pub disable_emulator_js: bool,
    #[serde(rename = "DISABLE_RUFFLE_RS")]
    pub disable_ruffle_rs: bool,
}
impl std::fmt::Display for EmulationDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

use super::TaskDict;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchedulerDict {
    #[serde(rename = "RESCAN")]
    pub rescan: TaskDict,
    #[serde(rename = "SWITCH_TITLEDB")]
    pub switch_titledb: TaskDict,
}
impl std::fmt::Display for SchedulerDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

use super::{
    EmulationDict, FilesystemDict, FrontendDict, MetadataSourcesDict, OidcDict, SchedulerDict,
    SystemDict, WatcherDict,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeartbeatResponse {
    #[serde(rename = "EMULATION")]
    pub emulation: EmulationDict,
    #[serde(rename = "FILESYSTEM")]
    pub filesystem: FilesystemDict,
    #[serde(rename = "FRONTEND")]
    pub frontend: FrontendDict,
    #[serde(rename = "METADATA_SOURCES")]
    pub metadata_sources: MetadataSourcesDict,
    #[serde(rename = "OIDC")]
    pub oidc: OidcDict,
    #[serde(rename = "SCHEDULER")]
    pub scheduler: SchedulerDict,
    #[serde(rename = "SYSTEM")]
    pub system: SystemDict,
    #[serde(rename = "WATCHER")]
    pub watcher: WatcherDict,
}
impl std::fmt::Display for HeartbeatResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

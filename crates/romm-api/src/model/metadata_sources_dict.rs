use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetadataSourcesDict {
    #[serde(rename = "ANY_SOURCE_ENABLED")]
    pub any_source_enabled: bool,
    #[serde(rename = "IGDB_API_ENABLED")]
    pub igdb_api_enabled: bool,
    #[serde(rename = "MOBY_API_ENABLED")]
    pub moby_api_enabled: bool,
    #[serde(rename = "STEAMGRIDDB_ENABLED")]
    pub steamgriddb_enabled: bool,
}
impl std::fmt::Display for MetadataSourcesDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

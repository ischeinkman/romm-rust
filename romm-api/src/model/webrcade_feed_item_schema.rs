use super::WebrcadeFeedItemPropsSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebrcadeFeedItemSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "longTitle")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub long_title: Option<String>,
    pub props: WebrcadeFeedItemPropsSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    pub title: String,
    #[serde(rename = "type")]
    pub type_: String,
}
impl std::fmt::Display for WebrcadeFeedItemSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserNotesSchema {
    pub note_raw_markdown: String,
    pub user_id: i64,
    pub username: String,
}
impl std::fmt::Display for UserNotesSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

use super::{
    CollectionSchema, RomFile, RomSchema, RomUserSchema, SaveSchema, ScreenshotSchema, StateSchema,
    UserNotesSchema,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetailedRomSchema {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub age_ratings: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternative_names: Vec<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub average_rating: serde_json::Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collections: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub companies: Vec<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub crc_hash: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub file_extension: String,
    pub file_name: String,
    pub file_name_no_ext: String,
    pub file_name_no_tags: String,
    pub file_path: String,
    pub file_size_bytes: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<RomFile>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub first_release_date: serde_json::Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub franchises: Vec<String>,
    pub full_path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub game_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<String>,
    pub has_cover: bool,
    pub id: i64,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub igdb_id: serde_json::Value,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub igdb_metadata: serde_json::Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub languages: Vec<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub md5_hash: serde_json::Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub merged_screenshots: Vec<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub moby_id: serde_json::Value,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub moby_metadata: serde_json::Value,
    pub multi: bool,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub name: serde_json::Value,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub path_cover_l: serde_json::Value,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub path_cover_s: serde_json::Value,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub platform_custom_name: serde_json::Value,
    pub platform_display_name: String,
    pub platform_fs_slug: String,
    pub platform_id: i64,
    pub platform_name: String,
    pub platform_slug: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub revision: serde_json::Value,
    pub rom_user: RomUserSchema,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub sgdb_id: serde_json::Value,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub sha1_hash: serde_json::Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sibling_roms: Vec<RomSchema>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub slug: serde_json::Value,
    pub sort_comparator: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub summary: serde_json::Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub url_cover: serde_json::Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user_collections: Vec<CollectionSchema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user_notes: Vec<UserNotesSchema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user_saves: Vec<SaveSchema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user_screenshots: Vec<ScreenshotSchema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user_states: Vec<StateSchema>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub youtube_video_id: serde_json::Value,
}
impl std::fmt::Display for DetailedRomSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

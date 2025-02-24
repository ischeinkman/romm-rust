use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::md5hash::{md5, Md5Hash};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SaveMeta {
    pub rom: String,
    pub name: String,
    pub emulator: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub hash: Md5Hash,
    pub size: u64,
}
impl SaveMeta {
    pub fn apply_format_variables(
        &mut self,
        mut variables: HashMap<String, String>,
    ) -> Result<(), anyhow::Error> {
        self.rom = variables
            .remove("$ROM")
            .unwrap_or(std::mem::take(&mut self.rom));
        self.name = variables
            .remove("$NAME")
            .unwrap_or(std::mem::take(&mut self.name));
        self.emulator = variables
            .remove("$EMULATOR")
            .or(std::mem::take(&mut self.emulator));
        let ts = variables
            .remove("$TIMESTAMP")
            .map(|ts| DateTime::parse_from_rfc3339(&ts))
            .transpose()?
            .map(|ts| ts.to_utc());
        self.created = variables
            .remove("$CREATED")
            .map(|ts| DateTime::parse_from_rfc3339(&ts))
            .transpose()?
            .map(|ts| ts.to_utc())
            .or(ts)
            .unwrap_or(self.created);
        self.updated = variables
            .remove("$UPDATED")
            .map(|ts| DateTime::parse_from_rfc3339(&ts))
            .transpose()?
            .map(|ts| ts.to_utc())
            .or(ts)
            .unwrap_or(self.updated);
        Ok(())
    }
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.created.max(self.updated)
    }
    pub fn new_empty(rom: String, name: String, emulator: Option<String>) -> Self {
        let hash = md5(std::io::Cursor::new([])).unwrap();
        let created = DateTime::from_timestamp_nanos(0);
        let updated = DateTime::from_timestamp_nanos(0);
        Self {
            rom,
            name,
            emulator,
            created,
            updated,
            hash,
            size: 0,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

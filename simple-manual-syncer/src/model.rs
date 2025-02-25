use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{
    md5hash::{md5, Md5Hash},
    path_format_strings::FormatString,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SaveMeta {
    pub rom: Option<String>,
    pub name: String,
    pub ext: String,
    pub emulator: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub hash: Md5Hash,
    pub size: u64,
}
impl SaveMeta {
    pub fn rom(&self) -> &str {
        self.rom.as_deref().unwrap_or(&self.name)
    }
    pub fn output_target(&self, format: &FormatString) -> String {
        let mut vars = HashMap::new();
        if let Some(rom) = self.rom.as_deref() {
            vars.insert("$ROM", rom.to_owned());
        }
        vars.insert("$NAME", self.name.clone());
        vars.insert("$EXT", self.ext.clone());
        vars.insert("$CREATED", self.created.to_rfc3339());
        vars.insert("$UPDATED", self.updated.to_rfc3339());
        vars.insert("$TIMESTAMP", self.timestamp().to_rfc3339());
        if let Some(emu) = self.emulator.clone() {
            vars.insert("$EMULATOR", emu);
        }
        let mut retvl = format.build_with_vars(&vars);
        if retvl.ends_with("/") {
            retvl.push_str(&self.name);
            if !self.ext.is_empty() {
                retvl.push('.');
                retvl.push_str(&self.ext);
            }
        }
        retvl
    }
    pub fn apply_format_variables(
        &mut self,
        mut variables: HashMap<String, String>,
    ) -> Result<(), anyhow::Error> {
        self.rom = variables
            .remove("$ROM")
            .or_else(|| std::mem::take(&mut self.rom));
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
    pub fn new_empty(rom: String, name: String, ext: String, emulator: Option<String>) -> Self {
        let hash = md5(std::io::Cursor::new([])).unwrap();
        let created = DateTime::from_timestamp_nanos(0);
        let updated = DateTime::from_timestamp_nanos(0);
        Self {
            rom: Some(rom),
            name,
            ext,
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

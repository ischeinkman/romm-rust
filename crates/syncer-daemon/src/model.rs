use std::collections::HashMap;

use chrono::{DateTime, Utc};

use syncer_model::path_format_strings::FormatString;

use crate::md5hash::{md5, Md5Hash};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SaveMeta {
    /// The name of the rom; if unset defaults to `name`.
    pub rom: Option<String>,
    /// The name of the save file itself, without any extension or folder
    /// prefix.
    pub name: String,
    /// The file extention of the same.
    pub ext: String,
    /// The emulator of the save file, if known.
    pub emulator: Option<String>,
    /// The time the save file was created, defaulting to the unix epoch if
    /// unknown.
    pub created: DateTime<Utc>,
    /// The time the save file was last modified, defaulting to the unix epoch
    /// if unknown.
    pub updated: DateTime<Utc>,
    /// The hash value of the save, for checking for equality.
    pub hash: Md5Hash,
    /// The size of the save, for checking for equality.
    pub size: u64,
}
impl SaveMeta {
    /// The name of the ROM this save is for.
    pub fn rom(&self) -> &str {
        self.rom.as_deref().unwrap_or(&self.name)
    }
    /// Where the save file should be placed given the current metadata
    /// variables and the given format string template.
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
    /// Applies values extracted from a format string template to this save
    /// metadata.
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

    /// The effective timestamp of this save file.
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.created.max(self.updated)
    }

    /// Builds a new sentinel [`SaveMeta`] instance to represent a save file
    /// that doesn't exist yet.
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

    /// Checks whether or not this [SaveMeta] represents a non-existent save
    /// file.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Returns whether this metadata and the other metadata represent the same
    /// underlying save file, based on hash & size information.
    pub fn same_file(&self, other: &Self) -> bool {
        self.size == other.size && self.hash == other.hash
    }
}

use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{env, fmt::Debug};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

mod loading;
use loading::{FlattenedList, ParseableDuration};

use crate::path_format_strings::FormatString;
pub mod save_finding;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Configuration for dealing with the local system.
    pub system: SystemConfig,

    /// Configuration for dealing with the remote ROMM server.
    pub romm: RommConfig,
}

impl Config {
    pub fn join(self, other: Self) -> Self {
        Self {
            system: self.system.join(other.system),
            romm: self.romm.join(other.romm),
        }
    }
    pub fn load(files: impl Iterator<Item = impl AsRef<Path>>) -> Result<Self, anyhow::Error> {
        let mut retvl = Self::default();
        for file in files {
            let file = file.as_ref();
            let mut fh =
                File::open(file).with_context(|| format!("Error opening config file {file:?}."))?;
            let ext = file.extension().map(|s| s.to_string_lossy());
            let ext = ext.as_ref().map(|s| s.as_ref());
            let parsed = match ext {
                Some("toml") => {
                    let mut data = String::new();
                    fh.read_to_string(&mut data)
                        .with_context(|| format!("Error reading data from TOML file {file:?}."))?;
                    toml::from_str(&data)
                        .with_context(|| format!("Error parsing TOML file {file:?}."))?
                }
                Some("json") => serde_json::from_reader(fh)
                    .with_context(|| format!("Error parsing JSON file {file:?}."))?,
                _ => {
                    let mut data = String::new();
                    fh.read_to_string(&mut data)
                        .with_context(|| format!("Error reading data from TOML file {file:?}."))?;
                    toml::from_str(&data)
                        .with_context(|| format!("Error parsing TOML file {file:?}."))?
                }
            };
            retvl = retvl.join(parsed);
        }
        let romm_env_config = RommConfig::from_env()?;
        retvl.romm = retvl.romm.join(romm_env_config);
        retvl.validate()?;
        Ok(retvl)
    }
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.romm.validate()?;
        self.system.validate()?;
        Ok(())
    }
}

/// Configuration for dealing with the remote ROMM server.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RommConfig {
    /// The URL of the remote ROMM server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    /// The authorization header to use when making API calls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// The format string used for reading & uploading file names to ROMM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<FormatString>,
}

impl Debug for RommConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RommConfig")
            .field("url", &self.url)
            .field(
                "api_key",
                &self.api_key.as_deref().map(|s| "*".repeat(s.len())),
            )
            .finish()
    }
}

fn default_true() -> bool {
    true
}

fn is_true(b: &bool) -> bool {
    *b
}

impl RommConfig {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let url = env::var_os("ROMM_URL")
            .map(|s| s.into_string())
            .transpose()
            .map_err(|e| anyhow::anyhow!("Could not parse {e:?} as valid UTF-8"))
            .and_then(|raw| {
                raw.map(|raw| Url::parse(&raw))
                    .transpose()
                    .map_err(From::from)
            })
            .context("Error parsing URL from ROMM_URL")?;
        let api_key = env::var_os("ROMM_API_KEY")
            .map(|s| s.into_string())
            .transpose()
            .map_err(|e| anyhow::anyhow!("Could not parse {e:?} as valid UTF-8"))
            .context("Error parsing api key from ROMM_API_KEY")?;
        Ok(Self {
            url,
            api_key,
            format: None,
        })
    }
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.url
            .as_ref()
            .ok_or(ConfigError::MissingField("romm.url"))?;
        self.api_key
            .as_ref()
            .ok_or(ConfigError::MissingField("romm.api_key"))?;
        Ok(())
    }
}

impl RommConfig {
    pub fn join(self, other: Self) -> Self {
        Self {
            url: other.url.or(self.url),
            api_key: other.api_key.or(self.api_key),
            format: other.format.or(self.format),
        }
    }
}

/// Configuration for dealing with the local system.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemConfig {
    /// The list of formatted strings denoting where in the filesystem to look
    /// for save files.
    #[serde(default, skip_serializing_if = "FlattenedList::is_empty")]
    pub saves: FlattenedList<FormatString>,
    /// Allowlist of specific files/directories to be kept in sync.
    ///
    /// If [`None`] then no allowlist will be applied; any file matching an
    /// entry in `saves` and no entry in the deny list will be matched. If
    /// `Some(vec![])`, then no files will be synced.
    ///
    /// This field is mainly interacted with via the UI; we don't expect it to
    /// be manipulated directly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<PathBuf>>,
    /// Denylist of specific files/directories to NOT be kept in sync.
    ///
    /// This field is mainly interacted with via the UI; we don't expect it to
    /// be manipulated directly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<PathBuf>,
    /// Whether or not we should ignore hidden files; defaults to `true`.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub skip_hidden: bool,
    /// Where to put the local sync database, used for checking for modification
    /// conflicts and keep a record of updates.
    pub database: Option<PathBuf>,

    /// How often the daemon should poll the host & server for changes.
    pub poll_interval : ParseableDuration, 
}

impl SystemConfig {
    pub fn join(self, other: Self) -> Self {
        let mut deny = other.deny;
        deny.extend(self.deny);
        let allow = match (self.allow, other.allow) {
            (None, None) => None,
            (None, Some(a)) | (Some(a), None) => Some(a),
            (Some(mut a), Some(b)) => {
                a.extend(b);
                Some(a)
            }
        };
        Self {
            saves: self.saves.join(other.saves),
            skip_hidden: self.skip_hidden || other.skip_hidden,
            database: other.database.or(self.database),
            deny,
            allow,
            poll_interval : other.poll_interval,
        }
    }
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.saves.is_empty() {
            return Err(ConfigError::MissingField("system.saves"));
        }
        if self.database.is_none() {
            return Err(ConfigError::MissingField("system.database"));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required field {0}")]
    MissingField(&'static str),
}

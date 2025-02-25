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
use loading::FlattenedList;

use crate::path_format_strings::FormatString;
pub mod save_finding;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RommConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
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

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemConfig {
    #[serde(default, skip_serializing_if = "FlattenedList::is_empty")]
    pub saves: FlattenedList<FormatString>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub skip_hidden: bool,
    pub database: Option<PathBuf>,
}

impl SystemConfig {
    pub fn join(self, other: Self) -> Self {
        Self {
            saves: self.saves.join(other.saves),
            skip_hidden: self.skip_hidden || other.skip_hidden,
            database: other.database.or(self.database),
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

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub system: SystemConfig,
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

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required field {0}")]
    MissingField(&'static str),
}

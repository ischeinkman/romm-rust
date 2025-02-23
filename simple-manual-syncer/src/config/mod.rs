use std::env;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use url::Url;

mod loading;
use loading::FlattenedList;
pub mod save_finding;
pub mod save_formats;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Default)]
pub struct RommConfig {
    pub url: Option<Url>,
    pub api_key: Option<String>,
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
        Ok(Self { url, api_key })
    }
}

impl RommConfig {
    pub fn join(self, other: Self) -> Self {
        Self {
            url: other.url.or(self.url),
            api_key: other.api_key.or(self.api_key),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct SystemConfig {
    pub saves: FlattenedList<String>,
}

impl SystemConfig {
    pub fn join(self, other: Self) -> Self {
        Self {
            saves: self.saves.join(other.saves),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
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
        Ok(retvl)
    }
}

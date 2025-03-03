use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Error as JsError;
use serde_json::Value as JsValue;
use thiserror::Error;

/// The version of the daemon's RPC API.
pub const VERSION: u32 = 1;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct DaemonCommand {
    pub version: u32,
    #[serde(flatten)]
    pub body: DaemonCommandBody,
}

impl DaemonCommand {
    pub const fn new(body: DaemonCommandBody) -> Self {
        Self {
            version: VERSION,
            body,
        }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum DaemonCommandBody {
    /// Perform a full sync of all saves.
    DoSync,

    /// Reloads the configuration from disk.
    ReloadConfig,
}

#[derive(Debug, Error)]
pub enum CommandParseError {
    #[error("API version mismatch: daemon is on version {expected}, but received command was version {actual}")]
    VersionMismatch { expected: u32, actual: i64 },
    #[error(transparent)]
    Other(#[from] JsError),
}

impl TryFrom<&[u8]> for DaemonCommand {
    type Error = CommandParseError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let base_err = match serde_json::from_slice(value) {
            Ok(retvl) => {
                return Ok(retvl);
            }
            Err(e) => e,
        };

        let Ok(raw_obj) = serde_json::from_slice::<JsValue>(value) else {
            return Err(base_err.into());
        };
        let Some(actual_version) = raw_obj.get("version").and_then(|n| n.as_i64()) else {
            return Err(base_err.into());
        };
        if actual_version != VERSION as i64 {
            return Err(CommandParseError::VersionMismatch {
                expected: VERSION,
                actual: actual_version,
            });
        }
        Err(base_err.into())
    }
}

impl FromStr for DaemonCommand {
    type Err = CommandParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.as_bytes().try_into()
    }
}

use chrono::{DateTime, Utc};
use md5hash::Md5Hash;
use url::Url;
mod database;
mod md5hash;
mod rommclient;
use rommclient::RawClient;

fn main() {
    let url = Url::parse("https://romm.k8s.ilans.dev/").unwrap();
    let auth = format!("Basic {}", std::env::var("ROMM_API_TOKEN").unwrap());
    let cl = RawClient::new(url, auth);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SaveMeta {
    pub rom: String,
    pub name: String,
    pub emulator: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub md5: Md5Hash,
}
impl SaveMeta {
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.created.max(self.updated)
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum SyncTarget {
    Device,
    Remote,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Default)]
pub enum SyncDecision {
    #[default]
    Noop,
    PushToRemote,
    PullToDevice,
}

pub fn decide_action(
    device_save: &SaveMeta,
    remote_save: &SaveMeta,
    in_db: &SaveMeta,
) -> Result<SyncDecision, anyhow::Error> {
    if device_save.md5 == remote_save.md5 {
        return Ok(SyncDecision::Noop);
    }
    let device_is_db = device_save.md5 == in_db.md5;
    let remote_is_db = remote_save.md5 == in_db.md5;

    match (device_is_db, remote_is_db) {
        (true, true) => Ok(SyncDecision::Noop),
        (false, false) => Err(anyhow::anyhow!("CONFLICT")),
        (true, false) => {
            if device_save.timestamp() < remote_save.timestamp() {
                Ok(SyncDecision::PullToDevice)
            } else {
                Err(anyhow::anyhow!(
                    "TIMESTAMP: device >= remote, but not expected."
                ))
            }
        }
        (false, true) => {
            if device_save.timestamp() > remote_save.timestamp() {
                Ok(SyncDecision::PushToRemote)
            } else {
                Err(anyhow::anyhow!(
                    "TIMESTAMP: device <= remote, but not expected."
                ))
            }
        }
    }
}

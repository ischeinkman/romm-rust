use std::io;

use chrono::{DateTime, Utc};
use deviceclient::DeviceMeta;
use futures::FutureExt;
use md5hash::{md5, Md5Hash};
use url::Url;
mod database;
mod md5hash;
mod rommclient;
use rommclient::{RawClient, RommClient};
mod deviceclient;

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async_main());
}

async fn async_main() {
    let url = Url::parse("https://romm.k8s.ilans.dev/").unwrap();
    let auth = format!("Basic {}", std::env::var("ROMM_API_TOKEN").unwrap());
    let cl = RommClient::new(url, auth);
    let args = std::env::args().collect::<Vec<_>>();
    let rom = &args[1];
    let save = &args[2];

    let device_meta = DeviceMeta::from_path(rom.clone(), &save).await.unwrap();
    let romm_meta = cl.find_save_matching(&device_meta.meta).await.unwrap();
    
    let action = decide_action(&device_meta.meta, &romm_meta.meta, &device_meta.meta).unwrap();
    println!("{:?}", action);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SaveMeta {
    pub rom: String,
    pub name: String,
    pub emulator: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub hash: Md5Hash,
    pub size : u64, 
}
impl SaveMeta {
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
            size : 0, 
        }
    }
    pub fn is_empty(&self) -> bool {
        self.size == 0

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
    if device_save.is_empty() && !remote_save.is_empty() {
        return Ok(SyncDecision::PullToDevice);
    }
    if remote_save.is_empty() && !device_save.is_empty() {
        return Ok(SyncDecision::PushToRemote);
    }
    if device_save.hash == remote_save.hash {
        return Ok(SyncDecision::Noop);
    }
    let device_is_db = device_save.hash == in_db.hash;
    let remote_is_db = remote_save.hash == in_db.hash;

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

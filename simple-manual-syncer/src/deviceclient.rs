use std::{
    io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use futures::stream;
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
};
use tracing::debug;

use crate::{md5hash::md5_stream, SaveMeta};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DeviceMeta {
    pub path: PathBuf,
    pub meta: SaveMeta,
}

impl DeviceMeta {
    pub fn new(path: PathBuf, meta: SaveMeta) -> Self {
        Self { path, meta }
    }
    #[tracing::instrument]
    pub async fn from_path(path: &Path) -> io::Result<Self> {
        debug!("Building device-level metadata for save at path {path:?}");
        let path = path.to_owned();
        let fs_meta = fs::metadata(&path).await?;
        let created = unwrap_timestamp(fs_meta.created())?;
        let updated = unwrap_timestamp(fs_meta.modified())?;
        let size = fs_meta.size();
        debug!("Retrieved metadata information. Now building md5 hash...");
        let byte_stream = stream::try_unfold(File::open(&path).await?, |mut fh| async move {
            let mut buf = vec![0; 4 * 1024 * 1024];
            match fh.read(&mut buf).await {
                Ok(0) => Ok(None),
                Ok(n) => {
                    buf.resize(n, 0);
                    Ok(Some((buf, fh)))
                }
                Err(e) => Err(e),
            }
        });
        let hash = md5_stream(byte_stream).await?;
        debug!("Finished retrieving device save information.");
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let ext = path
            .extension()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let meta = SaveMeta {
            rom: None,
            name,
            ext,
            created,
            updated,
            hash,
            size,
            emulator: None, //TODO: this
        };
        Ok(Self::new(path, meta))
    }
}

/// Helper to unwrap a filesystem timestamp, defaulting to the unix epoch on
/// filesystems that don't support timestamps.
fn unwrap_timestamp(raw: Result<SystemTime, io::Error>) -> Result<DateTime<Utc>, io::Error> {
    let systime = match raw {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::Unsupported => SystemTime::UNIX_EPOCH,
        Err(e) => {
            return Err(e);
        }
    };
    Ok(DateTime::from(systime))
}

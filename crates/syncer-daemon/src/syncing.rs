use std::path::Path;

use futures::{StreamExt, TryStreamExt};
use tracing::{info, trace, warn};

use syncer_model::config::Config;
use syncer_model::path_format_strings::FormatString;

use crate::{
    database::SaveMetaDatabase,
    deviceclient::DeviceMeta,
    model::SaveMeta,
    rommclient::{RommClient, RommError, RommSaveMeta},
};

pub async fn run_sync(
    cfg: &Config,
    cl: &RommClient,
    db: &SaveMetaDatabase,
) -> Result<(), anyhow::Error> {
    let results =
        cfg.possible_saves()
            .map_err(anyhow::Error::from)
            .and_then(|(save, fmt, vars)| async move {
                let mut device_meta = DeviceMeta::from_path(save.as_ref()).await?;
                device_meta.meta.apply_format_variables(vars)?;
                Ok((device_meta, fmt))
            })
            .and_then(|(device_meta, fmt)| {
                let cl = &cl;
                let db = &db;
                async move {
                    run_sync_for_save(&device_meta, fmt, cfg.romm.format.as_ref(), cl, db).await
                }
            });
    let mut errors = results
        .filter_map(|res| futures::future::ready(res.err()))
        .collect::<Vec<_>>()
        .await;

    // TODO: Do something with the rest of the errors
    errors.pop().map_or(Ok(()), Err)
}

pub async fn run_sync_for_save(
    device_meta: &DeviceMeta,
    device_format: &FormatString,
    romm_format: Option<&FormatString>,
    cl: &RommClient,
    db: &SaveMetaDatabase,
) -> Result<(), anyhow::Error> {
    trace!(
        "Starting decision making tree for path {}",
        device_meta.path.display()
    );
    let romm_meta = match cl.find_save_matching(&device_meta.meta, romm_format).await {
        Ok(data) => data,
        Err(RommError::RomNotFound(_)) => {
            warn!(
                "Missing rom in remote for local file {}",
                device_meta.meta.rom()
            );
            return Ok(());
        }
        Err(other) => {
            return Err(anyhow::anyhow!("Error finding save: {other:?}"));
        }
    };

    let db_data = db.query_metadata(
        device_meta.meta.rom(),
        &device_meta.meta.name,
        device_meta.meta.emulator.as_deref(),
    )?;
    let action = decide_action(&device_meta.meta, &romm_meta.meta, &db_data)?;
    perform_action(
        &action,
        device_meta,
        device_format,
        &romm_meta,
        romm_format,
        cl,
        db,
    )
    .await?;
    Ok(())
}

pub async fn perform_action(
    action: &SyncDecision,
    device_meta: &DeviceMeta,
    device_format: &FormatString,
    romm_meta: &RommSaveMeta,
    romm_format: Option<&FormatString>,
    cl: &RommClient,
    db: &SaveMetaDatabase,
) -> Result<(), anyhow::Error> {
    info!(
        "{:?} ({:?}, {:?}) => {:?}",
        device_meta.path, romm_meta.rom_id, romm_meta.save_id, action
    );
    let new_meta = match action.target() {
        Some(PushTarget::Device) => {
            let target = romm_meta.meta.output_target(device_format);
            cl.pull_save(Path::new(&target), romm_meta).await?;
            &romm_meta.meta
        }
        Some(PushTarget::Remote) => {
            let mut mapped_romm_meta = romm_meta.clone();
            mapped_romm_meta.meta.created = device_meta.meta.created;
            mapped_romm_meta.meta.updated = device_meta.meta.updated;
            mapped_romm_meta.meta.hash = device_meta.meta.hash;
            mapped_romm_meta.meta.size = device_meta.meta.size;
            mapped_romm_meta.meta.emulator = device_meta.meta.emulator.clone();
            cl.push_save(&device_meta.path, &mapped_romm_meta, romm_format)
                .await?;
            &device_meta.meta
        }
        None => {
            // Should be equivalent, default to device arbitrarily
            &device_meta.meta
        }
    };
    if action.needs_db_resync() {
        db.upsert_metadata(new_meta)?;
    }
    Ok(())
}

/// The full list of syncing decisions we could make between the local save
/// file, the sync history database, and the remote ROMM save.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Default)]
pub enum SyncDecision {
    /// Do nothing; everything is in sync.
    #[default]
    Noop,
    /// Push the save file to the remote ROMM server.
    ///
    /// Implies a database resync after the file is pulled.
    PushToRemote,
    /// Pull the save file from ROMM into the device.
    ///
    /// Implies a database resync after the save file is pulled.
    PullToDevice,
    /// The local save fiile and remote ROMM save are the same, but the local
    /// sync DB is out of sync; fix the DB without touching either file.
    ///
    /// Note that this state implies something weird is going on; the user might
    /// have manually uploaded/download a file, or a database corruption
    /// occured, or a remote pull failed before resyncing the database.
    ResyncDb,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum PushTarget {
    /// We need to pull the save FROM the remote romm server TO the local
    /// device.
    Device,
    /// We need to push the save FROM the local device TO the remote romm
    /// server.
    Remote,
}

impl SyncDecision {
    /// Where will the new save be pushed to, if that is needed?
    pub const fn target(&self) -> Option<PushTarget> {
        use SyncDecision::*;
        match self {
            Noop | ResyncDb => None,
            PushToRemote => Some(PushTarget::Remote),
            PullToDevice => Some(PushTarget::Device),
        }
    }
    /// Do we need to update the local sync database?
    pub const fn needs_db_resync(&self) -> bool {
        !matches!(self, SyncDecision::Noop)
    }
}

#[tracing::instrument]
pub fn decide_action(
    device_save: &SaveMeta,
    remote_save: &SaveMeta,
    in_db: &SaveMeta,
) -> Result<SyncDecision, anyhow::Error> {
    // New save on device; push.
    if remote_save.is_empty() && !device_save.is_empty() {
        return Ok(SyncDecision::PushToRemote);
    }

    // New save on remote; pull.
    if device_save.is_empty() && !remote_save.is_empty() {
        return Ok(SyncDecision::PullToDevice);
    }

    let device_is_db = device_save.same_file(in_db);
    let remote_is_db = remote_save.same_file(in_db);
    let device_is_remote = device_save.same_file(remote_save);

    match (device_is_db, remote_is_db) {
        // Everything is in sync; do nothing.
        (true, true) => Ok(SyncDecision::Noop),
        // The remote & device are in sync, but the database hasn't encountered
        // either file; this means the user probably did a manual sync at some
        // point. Update the database and move on.
        (false, false) if device_is_remote => Ok(SyncDecision::ResyncDb),
        // The database has seen the local file before, but not the remote; this
        // implies the remote SHOULD have been created later.
        //
        // If not, we error since that implies the user has been manually
        // messing with things.
        (true, false) => {
            if device_save.timestamp() < remote_save.timestamp() {
                Ok(SyncDecision::PullToDevice)
            } else {
                Err(anyhow::anyhow!(
                    "TIMESTAMP: device >= remote, but not expected."
                ))
            }
        }
        // The database has seen the remote file before, but not the local one; this
        // implies the local file SHOULD have been created later.
        //
        // If not, we error since that implies the user has been manually
        // messing with things.
        (false, true) => {
            if device_save.timestamp() > remote_save.timestamp() {
                Ok(SyncDecision::PushToRemote)
            } else {
                Err(anyhow::anyhow!(
                    "TIMESTAMP: device <= remote, but not expected."
                ))
            }
        }
        // None of the database, local file, or remote file are in sync; a
        // conflict has occured that will require manual intervention.
        (false, false) => Err(anyhow::anyhow!("CONFLICT")),
    }
}

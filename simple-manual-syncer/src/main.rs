use std::{env, path::Path};

use config::Config;
use database::SaveMetaDatabase;
use futures::TryStreamExt;
use tracing::{info, level_filters::LevelFilter, trace, warn};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter, FmtSubscriber};

mod database;
mod md5hash;
mod rommclient;
use rommclient::{RommClient, RommError};
mod deviceclient;
use deviceclient::DeviceMeta;
mod config;
mod model;
mod path_format_strings;
mod utils;
use model::SaveMeta;

fn main() {
    init_logger();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async_main());
}

fn init_logger() {
    let trace_env = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var("ROM_SYNC_LOG")
        .from_env()
        .unwrap();
    let mut subscriber = FmtSubscriber::builder()
        .with_env_filter(trace_env)
        .with_file(true)
        .with_line_number(true);
    let no_color = env::var_os("NO_COLOR").map_or(false, |s| !s.eq_ignore_ascii_case("0"));
    let json_log = env::var_os("ROM_SYNC_LOG_JSON").map_or(false, |s| !s.eq_ignore_ascii_case("0"));
    match (no_color, json_log) {
        (false, false) => {
            subscriber = subscriber.with_ansi(true);
        }
        (true, false) => {
            subscriber = subscriber.with_ansi(false);
        }
        (false, true) => {
            todo!()
        }
        (true, true) => {
            todo!()
        }
    }
    subscriber.finish().init();
}

#[tracing::instrument]
async fn async_main() {
    let args = std::env::args().collect::<Vec<_>>();
    let cfg = Config::load(args.into_iter().skip(1)).unwrap();
    let db = SaveMetaDatabase::open(cfg.system.database.as_deref().unwrap()).unwrap();
    info!("Starting with config: {cfg:?}");
    let cl = RommClient::new(
        cfg.romm.url.clone().unwrap(),
        cfg.romm.api_key.clone().unwrap(),
    );

    config::save_finding::possible_saves(&cfg)
        .try_for_each(|(save, fmt, vars)| {
            let cl = &cl;
            let db = &db;
            let cfg = &cfg;
            async move {
                trace!("Starting decision making tree for path {save:?}");
                let mut device_meta = DeviceMeta::from_path(save.as_ref()).await.unwrap();
                device_meta.meta.apply_format_variables(vars).unwrap();
                let romm_meta = match cl
                    .find_save_matching(&device_meta.meta, cfg.romm.format.as_ref())
                    .await
                {
                    Ok(data) => data,
                    Err(RommError::RomNotFound(_)) => {
                        warn!(
                            "Missing rom in remote for local file {}",
                            device_meta.meta.rom()
                        );
                        return Ok(());
                    }
                    Err(other) => {
                        panic!("Error finding save: {other:?}");
                    }
                };

                let db_data = db
                    .query_metadata(
                        device_meta.meta.rom(),
                        &device_meta.meta.name,
                        device_meta.meta.emulator.as_deref(),
                    )
                    .unwrap();
                let action = decide_action(&device_meta.meta, &romm_meta.meta, &db_data).unwrap();
                info!(
                    "{:?} ({:?}, {:?}) => {:?}",
                    device_meta.path, romm_meta.rom_id, romm_meta.save_id, action
                );
                let new_meta = match action {
                    SyncDecision::Noop => device_meta.meta,
                    SyncDecision::PullToDevice => {
                        let target = romm_meta.meta.output_target(fmt);
                        cl.pull_save(Path::new(&target), &romm_meta).await.unwrap();
                        romm_meta.meta
                    }
                    SyncDecision::PushToRemote => {
                        cl.push_save(&device_meta.path, &romm_meta, cfg.romm.format.as_ref())
                            .await
                            .unwrap();
                        device_meta.meta
                    }
                };
                db.upsert_metadata(&new_meta).unwrap();
                Ok(())
            }
        })
        .await
        .unwrap();
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

#[tracing::instrument]
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

use std::{env, path::Path};

use tracing::{debug, info, level_filters::LevelFilter};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter, FmtSubscriber};

use syncer_model::config::Config;

mod database;
use database::SaveMetaDatabase;
mod md5hash;
mod rommclient;
use rommclient::RommClient;
mod save_finding;
mod deviceclient;
mod model;
use model::SaveMeta;
mod syncing;
use syncing::run_sync;
mod ui;
mod utils;

const CONFIG_PATHS: &[&str] = &[
    // Miyoo mini -- next to the binary
    "sync_config.toml",
    // Linux
    "~/.config/simply-manual-syncer/config.toml",
];

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
    let no_color = env::var_os("NO_COLOR").is_some_and(|s| !s.eq_ignore_ascii_case("0"));
    let json_log = env::var_os("ROM_SYNC_LOG_JSON").is_some_and(|s| !s.eq_ignore_ascii_case("0"));
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

#[allow(unused)]
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

    run_sync(&cfg, &cl, &db).await.unwrap();
}


async fn do_sync() -> Result<(), anyhow::Error> {
    let cfg = Config::load(CONFIG_PATHS.iter())?;
    cfg.validate()?;
    let db = SaveMetaDatabase::open(cfg.system.database.as_deref().unwrap())?;
    info!("Performing sync.");
    debug!("Performing sync with config: {cfg:?}");
    let cl = RommClient::new(
        cfg.romm.url.clone().unwrap(),
        cfg.romm.api_key.clone().unwrap(),
    );

    run_sync(&cfg, &cl, &db).await?;
    Ok(())
}

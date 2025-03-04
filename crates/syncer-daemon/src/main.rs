use std::{env, time::Duration};

use futures::TryStreamExt;
use tokio::{io::AsyncReadExt, net::UnixListener, task::JoinHandle};
use tracing::{debug, error, info, level_filters::LevelFilter};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter, FmtSubscriber};

use syncer_model::{
    commands::{DaemonCommand, DaemonCommandBody},
    config::Config,
    platforms::Platform,
};

mod database;
use database::SaveMetaDatabase;
mod md5hash;
mod rommclient;
use rommclient::RommClient;
mod deviceclient;
mod model;
mod save_finding;
use model::SaveMeta;
mod syncing;
use syncing::run_sync;
use utils::{ConfigurableSleep, ConfigurableSleepSetter, EventTrigger};
mod utils;

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

pub struct DaemonState {
    sync_trigger: EventTrigger,
    sync_loop_sleep: ConfigurableSleepSetter,
    /// The thread responsible for triggering a sync every `poll_interval` time.
    _sync_loop_thread: JoinHandle<()>,
    /// The background task that performs full syncs whenever triggered, either
    /// by the [`_sync_loop_thread`] or from a call to
    /// [`DaemonCommand::DoSync`].
    _sync_actor_thread: JoinHandle<()>,
}

impl DaemonState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (sync_trigger, _sync_actor_thread) = build_sync_actor_thread();
        let (sync_loop_sleep, _sync_loop_thread) =
            build_sync_loop_thread(Duration::MAX, sync_trigger.clone());
        let retvl = Self {
            sync_trigger,
            sync_loop_sleep,
            _sync_actor_thread,
            _sync_loop_thread,
        };
        retvl.run_command(&DaemonCommand::new(DaemonCommandBody::ReloadConfig));
        retvl
    }
    pub fn run_command(&self, cmd: &DaemonCommand) {
        match cmd.body {
            DaemonCommandBody::DoSync => {
                self.sync_trigger.trigger();
            }
            DaemonCommandBody::ReloadConfig => {
                let cfg = match load_config() {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        error!("Error reloading config: {e:?}");
                        return;
                    }
                };
                self.sync_loop_sleep.set(*cfg.system.poll_interval);
            }
        }
    }
}

fn build_sync_loop_thread(
    initial_duration: Duration,
    sync_trigger: EventTrigger,
) -> (ConfigurableSleepSetter, JoinHandle<()>) {
    let (mut rcv, snd) = ConfigurableSleep::new(initial_duration);
    let task = async move {
        loop {
            rcv.sleep().await;
            sync_trigger.trigger();
        }
    };
    let thread = tokio::spawn(task);
    (snd, thread)
}
fn build_sync_actor_thread() -> (EventTrigger, JoinHandle<()>) {
    let (snd, mut trigger) = EventTrigger::new();
    let task = async move {
        loop {
            trigger.wait_and_reset().await;
            if let Err(e) = do_sync().await {
                error!("Error during sync: {e:?}");
            }
        }
    };
    let thread = tokio::spawn(task);
    (snd, thread)
}

fn load_config() -> Result<Config, anyhow::Error> {
    let cfg = Config::load(Platform::get().config_input_paths())?;
    cfg.validate()?;
    Ok(cfg)
}

async fn do_sync() -> Result<(), anyhow::Error> {
    let cfg = load_config()?;
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

pub async fn open_command_stream() -> Result<(), anyhow::Error> {
    let socket_pt = Platform::get().socket_path();
    let listener = UnixListener::bind(socket_pt)?;

    let connection_stream = futures::stream::poll_fn(move |cx| listener.poll_accept(cx).map(Some))
        .map_ok(|(con, _)| {});

    todo!()
}

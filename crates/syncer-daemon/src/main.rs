use std::{env, path::PathBuf, sync::Arc, time::Duration};

use futures::{future::Either, pin_mut, FutureExt};
use notify::{RecursiveMode, Watcher};
use socketproto::spawn_command_listen_thread;
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
use tracing::{debug, error, info, level_filters::LevelFilter, warn};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter, FmtSubscriber};

use syncer_model::{
    commands::{DaemonCommand, DaemonCommandBody},
    config::Config,
    platforms::Platform,
};

mod database;
mod socketproto;
use database::SaveMetaDatabase;
mod md5hash;
mod rommclient;
use rommclient::RommClient;
mod deviceclient;
mod model;
use model::SaveMeta;
mod syncing;
use syncing::run_sync;
use utils::{ConfigurableSleep, ConfigurableSleepSetter, EventTrigger};
mod utils;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.iter().any(|s| s == "--version") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return;
    }
    init_logger();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async_main());
    debug!("Caught CTRL-C. Waiting for work to finish...");
    rt.shutdown_timeout(Duration::from_millis(1000));
    debug!("Shutting down.");
}

fn init_logger() {
    let trace_env = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var("ROM_SYNC_LOG")
        .from_env()
        .unwrap();
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(trace_env)
        .with_file(true)
        .with_line_number(true);
    let no_color = env::var_os("NO_COLOR").is_some_and(|s| !s.eq_ignore_ascii_case("0"))
        || env::var_os("ROM_SYNC_NO_COLOR").is_some_and(|s| !s.eq_ignore_ascii_case("0"));
    let json_log = env::var_os("ROM_SYNC_LOG_JSON").is_some_and(|s| !s.eq_ignore_ascii_case("0"));
    let subscriber = subscriber.with_ansi(!no_color);
    if json_log {
        subscriber.json().finish().init();
    } else {
        subscriber.finish().init();
    }
}

#[allow(unused)]
#[tracing::instrument]
async fn async_main() {
    let cfg = load_config().await.unwrap();
    let db = SaveMetaDatabase::open(cfg.system.database.as_deref().unwrap()).unwrap();
    info!("Starting with config: {cfg:?}");
    let cl = RommClient::new(
        cfg.romm.url.clone().unwrap(),
        cfg.romm.api_key.clone().unwrap(),
    );

    let state = Arc::new(DaemonState::new());
    let _command_waiter = spawn_command_listen_thread(Arc::clone(&state)).unwrap();
    wait_for_death().await.unwrap();
    if let Err(e) = tokio::fs::remove_file(Platform::get().socket_path()).await {
        warn!("Error cleaning up daemon socket: {e:?}");
    }
    info!("Terminating because of sigterm.");
}

pub struct DaemonState {
    /// The setter for configuring the time the `_sync_loop_thread` should sleep between syncs.
    sync_loop_sleep: ConfigurableSleepSetter,
    /// The thread responsible for triggering a sync every `poll_interval` time.
    _sync_loop_thread: JoinHandle<()>,

    /// The trigger for starting a sync on the `_sync_actor_thread`.
    sync_trigger: EventTrigger,
    /// The background task that performs full syncs whenever triggered, either
    /// by the [`_sync_loop_thread`] or from a call to
    /// [`DaemonCommand::DoSync`].
    _sync_actor_thread: JoinHandle<()>,

    /// The list of paths to listen to for changes
    fs_watch_paths: watch::Sender<Vec<PathBuf>>,

    /// The background task that triggers a sync whenever a relevant path gets modified (if enabled)
    _fs_watch_thread: JoinHandle<()>,
}

impl DaemonState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (sync_trigger, _sync_actor_thread) = build_sync_actor_thread();
        let (sync_loop_sleep, _sync_loop_thread) =
            build_sync_loop_thread(Duration::MAX, sync_trigger.clone());
        let (fs_watch_paths, _fs_watch_thread) = build_fs_watch_thread(sync_trigger.clone());
        let retvl = Self {
            sync_loop_sleep,
            _sync_loop_thread,
            sync_trigger,
            _sync_actor_thread,
            fs_watch_paths,
            _fs_watch_thread,
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
                let sync_loop_sleep = self.sync_loop_sleep.clone();
                let fs_watch_paths = self.fs_watch_paths.clone();
                tokio::task::spawn(async move {
                    let cfg = match load_config().await {
                        Ok(cfg) => cfg,
                        Err(e) => {
                            error!("Error reloading config: {e:?}");
                            return;
                        }
                    };
                    sync_loop_sleep.set(*cfg.system.poll_interval);
                    let new_watch_paths = if cfg.system.sync_on_file_change {
                        cfg.save_roots().collect()
                    } else {
                        Vec::new()
                    };
                    fs_watch_paths.send_replace(new_watch_paths);
                });
            }
        }
    }
}

fn build_fs_watch_thread(
    sync_trigger: EventTrigger,
) -> (watch::Sender<Vec<PathBuf>>, JoinHandle<()>) {
    let (snd, mut rcv) = watch::channel(Vec::<PathBuf>::new());
    let task = async move {
        loop {
            // Wrap this in a loop so we build the watcher both on initial
            // creation and when the list of paths is changed
            let (evt_snd, mut evt_rcv) = mpsc::unbounded_channel();
            let watcher = notify::recommended_watcher(move |evt| {
                evt_snd.send(evt).ok();
            });
            let mut watcher = match watcher {
                Ok(w) => w,
                Err(e) => {
                    error!("Error starting fs watcher thread: {e:?}");
                    return;
                }
            };
            for path in rcv.borrow_and_update().iter() {
                if let Err(e) = watcher.watch(path, RecursiveMode::Recursive) {
                    error!("Error watching path {path:?}: {e:?}");
                }
            }

            loop {
                let new_paths = rcv.changed().map(|_| ());
                let new_events = evt_rcv.recv();
                pin_mut!(new_paths);
                pin_mut!(new_events);

                match futures::future::select(new_paths, new_events).await {
                    Either::Left(((), _)) | Either::Right((None, _)) => {
                        // We got a new list of paths; rebuild the watcher.
                        break;
                    }
                    Either::Right((Some(Err(e)), _)) => {
                        error!("Error in watcher thread: {e:?}");
                    }
                    Either::Right((Some(Ok(evt)), _)) => {
                        if !evt.kind.is_access() {
                            debug!(
                                "Got FS notification {:?} for paths {:?}; triggering sync.",
                                evt.kind, evt.paths
                            );
                            sync_trigger.trigger();
                        }
                    }
                }
            }
        }
    };
    (snd, tokio::task::spawn(task))
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

async fn load_config() -> Result<Config, anyhow::Error> {
    let cfg = Config::load(Platform::get().config_input_paths()).await?;
    cfg.validate()?;
    Ok(cfg)
}

async fn do_sync() -> Result<(), anyhow::Error> {
    let cfg = load_config().await?;
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

#[cfg(unix)]
async fn wait_for_death() -> Result<(), anyhow::Error> {
    use tokio::signal::unix;
    use tracing::trace;
    let mut sigint = unix::signal(unix::SignalKind::interrupt())?;
    let mut sigterm = unix::signal(unix::SignalKind::terminate())?;
    let sigint = sigint.recv();
    let sigterm = sigterm.recv();
    futures::pin_mut!(sigint);
    futures::pin_mut!(sigterm);
    trace!("Now waiting for SIGINT or SIGTERM.");
    tokio::select! {
        _ = sigint => {
            trace!("Encountered SIGINT. Now dying...");
            Ok(())
        }
        _ = sigterm => {
            trace!("Encountered SIGTERM. Now dying...");
            Ok(())
        }
    }
}

#[cfg(not(unix))]
async fn wait_for_death() -> Result<(), anyhow::Error> {
    todo!()
}

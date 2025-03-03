use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant, SystemTime};

use chrono::{DateTime, Utc};
use futures::future::Either;
use futures::pin_mut;
use futures::{Stream, TryStream, TryStreamExt};
use syncer_model::config::Config;
use thiserror::Error;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::error::{SendError, TrySendError};
use tokio::sync::mpsc::{self, Receiver};
use tokio::sync::watch;

pub fn async_walkdir(root: &Path) -> impl TryStream<Ok = PathBuf, Error = io::Error> {
    futures::stream::unfold(vec![Ok(root.to_path_buf())], |mut queue| async move {
        let nxt = match queue.pop()? {
            Ok(pt) => pt,
            Err(e) => {
                return Some((Err(e), queue));
            }
        };
        match fs::symlink_metadata(&nxt).await.map(|meta| meta.is_dir()) {
            Ok(false) => {
                return Some((Ok(nxt), queue));
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Some((Ok(nxt), queue));
            }
            Ok(true) => {}
            Err(e) => {
                queue.push(Err(e));
                return Some((Ok(nxt), queue));
            }
        }
        let mut rdr = match tokio::fs::read_dir(&nxt).await {
            Ok(rdr) => rdr,
            Err(e) => {
                queue.push(Err(e));
                return Some((Ok(nxt), queue));
            }
        };
        loop {
            match rdr.next_entry().await {
                Ok(None) => {
                    break;
                }
                Ok(Some(ent)) => {
                    queue.push(Ok(ent.path()));
                }
                Err(e) => {
                    queue.push(Err(e));
                }
            }
        }
        Some((Ok(nxt), queue))
    })
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EitherError<A, B> {
    #[error(transparent)]
    A(#[from] A),
    #[error(transparent)]
    B(B),
}

pub async fn download<S, B, E>(data: S, dst: &Path) -> Result<(), EitherError<io::Error, E>>
where
    S: Stream<Item = Result<B, E>>,
    B: AsRef<[u8]>,
{
    let tmp_fname = dst.with_extension(timestamp_now().to_rfc3339());
    let mut fh = File::create_new(&tmp_fname).await?;

    pin_mut!(data);
    while let Some(chunk) = data.try_next().await.map_err(EitherError::B)? {
        fh.write_all(chunk.as_ref()).await?;
    }
    fh.flush().await?;
    drop(fh);
    fs::rename(tmp_fname, dst).await?;
    Ok(())
}

pub fn timestamp_now() -> DateTime<Utc> {
    let dt = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as _;
    DateTime::from_timestamp_nanos(dt)
}

/// A tool for sleeping for a configurable amount of time, where the time to
/// sleep is possible to change externally while a sleep is ongoing.
///
/// The motivating usecase is a background thread that runs every `X` time, with
/// `X` being user-configurable. Each update to `X` can then apply immediately,
/// without needing to wait for the previous sleep call to finish.
pub struct ConfigurableSleep {
    configuration_cb: watch::Receiver<Duration>,
}

#[derive(Clone)]
pub struct ConfigurableSleepSetter {
    snd: watch::Sender<Duration>,
}

impl ConfigurableSleepSetter {
    pub fn current(&self) -> Duration {
        *self.snd.borrow()
    }
    pub fn set(&self, duration: Duration) {
        self.snd.send_replace(duration);
    }
}

impl ConfigurableSleep {
    pub fn new(duration: Duration) -> (ConfigurableSleep, ConfigurableSleepSetter) {
        let (snd, configuration_cb) = watch::channel(duration);
        let sleep = ConfigurableSleep { configuration_cb };
        let setter = ConfigurableSleepSetter { snd };
        (sleep, setter)
    }

    /// Sleeps for the configured amount of time.
    ///
    /// If the amount of time to sleep is changed externally (via an associated
    /// [`ConfigurableSleepSetter`]) before the [`sleep`] completes, the already
    /// elapsed time will be put towards the newly configured time. If the newly
    /// configured time is shorter than the already elapsed duration the
    /// [`sleep`] call will exit immediately.
    ///
    /// For example, if you have a [`ConfigurableSleep`] with an initial
    /// duration of 1 hour and then call [`ConfigurableSleep::sleep`], and then
    /// after 30 minutes of sleeping someone calls
    /// [`ConfigurableSleepSetter::set`] with a [`Duration`] of 20 minutes, the
    /// currently-active [`ConfigurableSleep::sleep`] future will finish
    /// immediately.
    pub async fn sleep(&mut self) {
        let start = Instant::now();
        loop {
            let dt = *self.configuration_cb.borrow_and_update();
            if dt >= start.elapsed() {
                return;
            }
            let change_fut = self.configuration_cb.changed();
            let sleep_fut = tokio::time::sleep(dt);
            futures::pin_mut!(change_fut);
            futures::pin_mut!(sleep_fut);
            match futures::future::select(sleep_fut, change_fut).await {
                Either::Left(((), _)) => {
                    return;
                }
                Either::Right((Ok(()), _)) => {
                    continue;
                }
                Either::Right((Err(_), rest)) => {
                    rest.await;
                    return;
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct EventTrigger(mpsc::Sender<()>);

impl EventTrigger {
    pub fn new() -> (EventTrigger, EventTriggerRecv) {
        let (snd, rcv) = mpsc::channel(1);
        (EventTrigger(snd), EventTriggerRecv(rcv))
    }

    pub fn is_triggered(&self) -> bool {
        self.0.try_reserve().is_err()
    }
    pub async fn wait_for_reset(&self) {
        self.0.reserve().await.ok();
    }
    pub fn trigger(&self) {
        match self.0.try_send(()) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {}
            Err(TrySendError::Closed(_)) => {}
        }
    }
}

pub struct EventTriggerRecv(mpsc::Receiver<()>);
impl EventTriggerRecv {
    pub fn is_triggered(&self) -> bool {
        self.0.capacity() != self.0.max_capacity()
    }
    pub async fn wait_and_reset(&mut self) {
        self.0.recv().await;
        self.reset();
    }
    pub fn reset(&mut self) {
        while self.0.try_recv().is_ok() {}
    }
}

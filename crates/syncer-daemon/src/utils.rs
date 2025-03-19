use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant, SystemTime};

use chrono::{DateTime, Utc};
use futures::future::Either;
use futures::pin_mut;
use futures::{Stream, TryStreamExt};
use thiserror::Error;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::watch;
use tracing::trace;

static INCREMENTING_ID: AtomicUsize = AtomicUsize::new(0xa0_00);

/// Returns a new ID to use for debugging purposes.
///
/// Guranteed to not repeat across multiple calls, even in async/multi-threaded
/// environments.
pub fn new_id() -> usize {
    INCREMENTING_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EitherError<A, B> {
    #[error(transparent)]
    A(#[from] A),
    #[error(transparent)]
    B(B),
}

/// Atomically writes a file from a stream to the given location.
///
/// We do this by first writing the data to a temporary file path with a
/// timestamp-derived name and moving it to the correct location only after the
/// data has been completely written to the file.
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
    id: usize,
    configuration_cb: watch::Receiver<Duration>,
}

#[derive(Clone)]
pub struct ConfigurableSleepSetter {
    id: usize,
    snd: watch::Sender<Duration>,
}

impl ConfigurableSleepSetter {
    #[expect(unused)]
    pub fn current(&self) -> Duration {
        *self.snd.borrow()
    }
    pub fn set(&self, duration: Duration) {
        trace!("Setting delay for sleep {}", self.id);
        self.snd.send_replace(duration);
    }
}

impl ConfigurableSleep {
    pub fn new(duration: Duration) -> (ConfigurableSleep, ConfigurableSleepSetter) {
        let id = new_id();
        let (snd, configuration_cb) = watch::channel(duration);
        let sleep = ConfigurableSleep {
            id,
            configuration_cb,
        };
        let setter = ConfigurableSleepSetter { id, snd };
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
        let dt = *self.configuration_cb.borrow_and_update();
        trace!("Starting sleep on ID {} ({} s)", self.id, dt.as_secs_f64());
        loop {
            let dt = *self.configuration_cb.borrow_and_update();
            if dt >= start.elapsed() {
                break;
            }
            let change_fut = self.configuration_cb.changed();
            let sleep_fut = tokio::time::sleep(dt);
            futures::pin_mut!(change_fut);
            futures::pin_mut!(sleep_fut);
            match futures::future::select(sleep_fut, change_fut).await {
                Either::Left(((), _)) => {
                    break;
                }
                Either::Right((Ok(()), _)) => {
                    continue;
                }
                // The `change_fut` can only return an error if all associated
                // `ConfigrableSleepSetter`s have been dropped; in that case the
                // sleep value can never be changed, so we can await directly on
                // the sleep future without worry.
                Either::Right((Err(_), rest)) => {
                    rest.await;
                    break;
                }
            }
        }
        trace!("Finished sleep on ID {}", self.id);
    }
}

/// A synchronization primitive for triggering an event.
///
/// If multiple calls to [`EventTrigger:trigger`] occur before the reciever has
/// a chance to call [`EventTriggerRecv::wait_and_reset`] then only one event
/// will occur.
///
/// This is useful for things like save syncing and configuration flushing,
/// since even if multiple changes occur before we can do the associated
/// syncing/flushing we only ever care about the final value.
#[derive(Clone)]
pub struct EventTrigger {
    id: usize,
    inner: watch::Sender<bool>,
}

impl EventTrigger {
    pub fn new() -> (EventTrigger, EventTriggerRecv) {
        let id = new_id();
        let (inner, _) = watch::channel(false);
        (
            EventTrigger {
                id,
                inner: inner.clone(),
            },
            EventTriggerRecv { id, inner },
        )
    }

    pub fn trigger(&self) {
        trace!("Triggering: {}", self.id);
        self.inner.send_replace(true);
    }
}

pub struct EventTriggerRecv {
    id: usize,
    inner: watch::Sender<bool>,
}
impl EventTriggerRecv {
    pub async fn wait_and_reset(&mut self) {
        trace!("Waiting on trigger: {}", self.id);
        if self.inner.subscribe().wait_for(|b| *b).await.is_ok() {
            trace!("Triggered: {}", self.id);
            self.inner.send_replace(false);
        } else {
            // If all senders have closed the event will never trigger
            // again.
            //
            // This should be impossible though since we ourselves hold a
            // sender on this channel.
            trace!("Trigger dropped: {}", self.id);
            futures::future::pending().await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt;
    #[test]
    fn test_event_trigger() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        rt.block_on(async {
            let (snd, mut rcv) = EventTrigger::new();

            // Make sure we see the first trigger
            snd.trigger();
            assert!(rcv.wait_and_reset().now_or_never().is_some());

            // Make sure the trigger is reset correctly
            assert!(rcv.wait_and_reset().now_or_never().is_none());

            // Make sure we can trigger the event a second time
            snd.trigger();
            assert!(rcv.wait_and_reset().now_or_never().is_some());
            assert!(rcv.wait_and_reset().now_or_never().is_none());

            // Multiple triggers before a `wait_and_reset` should only cause a
            // single `wait_and_reset` to complete
            snd.trigger();
            snd.trigger();
            assert!(rcv.wait_and_reset().now_or_never().is_some());
            assert!(rcv.wait_and_reset().now_or_never().is_none());
        })
    }
}

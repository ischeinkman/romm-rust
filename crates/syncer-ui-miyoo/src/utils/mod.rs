mod views;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::{
    sync::{RwLock, RwLockReadGuard},
    task::JoinHandle,
};
pub use views::*;

/// A task executing in the background that will be stopped when the
/// [`BackgroundTask`] handle is dropped.
///
/// This request is given to the underlying task via the passed-in
/// [`AbortFlag`], allowing the underlying task to exit gracefully.
pub struct BackgroundTask {
    inner: Arc<JoinHandle<()>>,
    flag: TaskShouldDie,
}

impl Clone for BackgroundTask {
    fn clone(&self) -> Self {
        BackgroundTask {
            inner: Arc::clone(&self.inner),
            flag: TaskShouldDie(Arc::clone(&self.flag.0)),
        }
    }
}

/// The flag passed to the task of a [`BackgroundTask`] handle used for
/// signaling that the task should be stopped because the wrapping handle has
/// been dropped.
pub struct TaskShouldDie(Arc<AtomicBool>);

impl TaskShouldDie {
    /// Returns `true` when the task should stop.
    pub fn should_stop(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

impl BackgroundTask {
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: FnOnce(TaskShouldDie) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let flag = TaskShouldDie(Arc::new(AtomicBool::new(false)));
        let flag2 = TaskShouldDie(Arc::clone(&flag.0));
        let task = f(flag2);
        let inner = Arc::new(tokio::spawn(task));
        Self { inner, flag }
    }
}

impl Drop for BackgroundTask {
    fn drop(&mut self) {
        let fake_task = Arc::new(tokio::spawn(async {}));
        if Arc::into_inner(std::mem::replace(&mut self.inner, fake_task)).is_some() {
            self.flag.0.store(true, Ordering::Relaxed);
        }
    }
}

/// A [RwLock] wrapper that allows for non-blocking reads at the expense of:
///
/// * Slower and less ergonomic writes
/// * Needing to keep multiple copies of the value around
pub struct QuickReadSlot<T: Clone> {
    lock1: RwLock<T>,
    lock2: RwLock<T>,
}

impl<T: Clone> QuickReadSlot<T> {
    pub fn new(inner: T) -> Self {
        Self {
            lock1: RwLock::new(inner.clone()),
            lock2: RwLock::new(inner),
        }
    }

    /// Modifies the wrapped value with the given closure.
    pub async fn modify_with<F, R>(&self, f: F) -> R
    where
        F: for<'a> AsyncFnOnce(&'a mut T) -> R,
    {
        let mut l1 = self.lock1.write().await;
        let ret = f(&mut l1).await;
        let l1 = l1.downgrade();
        let value = l1.clone();
        *self.lock2.write().await = value;
        drop(l1);
        ret
    }

    /// Acquires a read lock of the inner value.
    ///
    /// This will *never* block.
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        // We know that at least 1 of these locks will be free of `.write()`
        // calls at any given moment, so this should theoretically never loop
        // more than once or twice unless we have a very specific pathologic
        // case where an infinite number of writers is scheduled *PERFECTLY* to
        // starve our reads.
        loop {
            if let Ok(ret) = self.lock1.try_read() {
                break ret;
            }
            if let Ok(ret) = self.lock2.try_read() {
                break ret;
            }
        }
    }
}

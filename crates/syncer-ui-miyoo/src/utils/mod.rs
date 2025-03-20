use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::{
    sync::{RwLock, RwLockReadGuard, Semaphore},
    task::JoinHandle,
};

mod views;
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
    /// The first RwLock, which will remain write-locked during modifications.
    lock1: RwLock<T>,
    /// The second RwLock, which will remain free during modifications until
    /// after `lock1` has been updated.
    lock2: RwLock<T>,
    /// A semaphore to block writes before they try acquiring `lock1`, making
    /// sure they don't enter the lock queue until both `lock1` and `lock2`
    /// don't have any outstanding write locks in their queues.
    writes_permit: Semaphore,
}

impl<T: Clone> QuickReadSlot<T> {
    pub fn new(inner: T) -> Self {
        Self {
            lock1: RwLock::new(inner.clone()),
            lock2: RwLock::new(inner),
            writes_permit: Semaphore::new(1),
        }
    }

    /// Modifies the wrapped value with the given closure.
    pub async fn modify_with<F, R>(&self, f: F) -> R
    where
        F: for<'a> AsyncFnOnce(&'a mut T) -> R,
    {
        let permit = self.writes_permit.acquire().await.unwrap_or_else(|_| {
            // Should never happen since we never close the semaphore and always
            // have a handle to it through `self`
            unreachable!()
        });
        let mut l1 = self.lock1.write().await;
        let res = f(&mut l1).await;

        // Since writers can't enter the lock queue until we release our permit,
        // downgrading `l1` will allow readers to read from it while we update
        // l2.
        //
        // After this all future calls to `read()` will return the new value.
        let l1 = l1.downgrade();
        let mut l2 = self.lock2.write().await;
        *l2 = l1.clone();
        drop(l2);
        drop(l1);
        drop(permit);
        res
    }

    /// Acquires a read lock of the inner value.
    ///
    /// This will *never* block.
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        // We know that at least 1 of these locks will be free of `.write()`
        // calls at any given moment, so this should theoretically never loop
        // more than once or twice.
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

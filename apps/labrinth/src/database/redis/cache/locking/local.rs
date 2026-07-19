use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use tokio::sync::Notify;
use tokio::time::{Instant, timeout_at};

use crate::database::models::DatabaseError;

#[derive(Clone)]
pub(in crate::database::redis::cache) struct LocalLockManager {
    locks: Arc<DashMap<String, Arc<LocalLockState>>>,
}

impl LocalLockManager {
    pub(super) fn new() -> Self {
        Self {
            locks: Arc::new(DashMap::with_capacity(2048)),
        }
    }

    pub(super) fn acquire(&self, key: String) -> LocalLockAcquisition {
        match self.locks.entry(key.clone()) {
            Entry::Occupied(entry) => {
                LocalLockAcquisition::Waiting(LocalLockWaiter {
                    state: entry.get().clone(),
                })
            }
            Entry::Vacant(entry) => {
                let state = Arc::new(LocalLockState::new());
                entry.insert(state.clone());
                LocalLockAcquisition::Owned(LocalLockGuard {
                    locks: self.locks.clone(),
                    key,
                    state,
                    released: false,
                })
            }
        }
    }
}

pub(super) enum LocalLockAcquisition {
    Owned(LocalLockGuard),
    Waiting(LocalLockWaiter),
}

pub(in crate::database::redis::cache) struct LocalLockGuard {
    locks: Arc<DashMap<String, Arc<LocalLockState>>>,
    key: String,
    state: Arc<LocalLockState>,
    released: bool,
}

impl LocalLockGuard {
    pub(super) fn release(mut self) {
        self.release_inner();
    }

    fn release_inner(&mut self) {
        if self.released {
            return;
        }

        self.released = true;
        self.locks
            .remove_if(&self.key, |_, state| Arc::ptr_eq(state, &self.state));
        self.state.released.store(true, Ordering::Release);
        self.state.notify.notify_waiters();
    }
}

impl Drop for LocalLockGuard {
    fn drop(&mut self) {
        self.release_inner();
    }
}

pub(in crate::database::redis::cache) struct LocalLockWaiter {
    state: Arc<LocalLockState>,
}

impl LocalLockWaiter {
    pub(super) async fn wait(
        self,
        deadline: Instant,
    ) -> Result<(), DatabaseError> {
        loop {
            if self.state.released.load(Ordering::Acquire) {
                return Ok(());
            }

            let notified = self.state.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.state.released.load(Ordering::Acquire) {
                return Ok(());
            }

            timeout_at(deadline, notified)
                .await
                .map_err(|_| lock_timeout())?;
        }
    }
}

struct LocalLockState {
    released: AtomicBool,
    notify: Notify,
}

impl LocalLockState {
    fn new() -> Self {
        Self {
            released: AtomicBool::new(false),
            notify: Notify::new(),
        }
    }
}

fn lock_timeout() -> DatabaseError {
    DatabaseError::LocalCacheTimeout {
        released: 0,
        total: 1,
    }
}

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use tokio::sync::Notify;
use tokio::time::{Instant, timeout_at};

use crate::database::models::DatabaseError;

#[derive(Clone)]
pub(in crate::database::redis::cache) struct LockCoordinator {
    locks: Arc<DashMap<String, Arc<LockState>>>,
}

impl LockCoordinator {
    pub(in crate::database::redis::cache) fn new() -> Self {
        Self {
            locks: Arc::new(DashMap::with_capacity(2048)),
        }
    }

    pub(in crate::database::redis::cache) fn acquire(
        &self,
        key: String,
    ) -> LockAcquisition {
        match self.locks.entry(key.clone()) {
            Entry::Occupied(entry) => LockAcquisition::Waiting(LockWaiter {
                state: entry.get().clone(),
            }),
            Entry::Vacant(entry) => {
                let state = Arc::new(LockState::new());
                entry.insert(state.clone());
                LockAcquisition::Owned(OwnedLockGuard {
                    locks: self.locks.clone(),
                    key,
                    state,
                    released: false,
                })
            }
        }
    }
}

pub(in crate::database::redis::cache) enum LockAcquisition {
    Owned(OwnedLockGuard),
    Waiting(LockWaiter),
}

pub(in crate::database::redis::cache) struct OwnedLockGuard {
    locks: Arc<DashMap<String, Arc<LockState>>>,
    key: String,
    state: Arc<LockState>,
    released: bool,
}

impl OwnedLockGuard {
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

impl Drop for OwnedLockGuard {
    fn drop(&mut self) {
        self.release_inner();
    }
}

pub(in crate::database::redis::cache) struct LockWaiter {
    state: Arc<LockState>,
}

impl LockWaiter {
    pub(in crate::database::redis::cache) async fn wait(
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

struct LockState {
    released: AtomicBool,
    notify: Notify,
}

impl LockState {
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

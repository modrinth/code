mod distributed;
mod local;

use std::time::Duration;

use redis::aio::ConnectionLike;
use tokio::time::Instant;

use crate::database::models::DatabaseError;

use super::super::config::CacheLockingStrategy;
use super::super::connection::RedisBackend;

use self::distributed::{
    DistributedLockGuard, DistributedLockManager, DistributedLockWaiter,
};
use self::local::{
    LocalLockAcquisition, LocalLockGuard, LocalLockManager, LocalLockWaiter,
};

pub(super) const WAIT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy)]
pub(super) struct LockTiming {
    pub(super) lease: Duration,
    pub(super) renewal: Duration,
    pub(super) poll_min: Duration,
    pub(super) poll_max: Duration,
}

impl Default for LockTiming {
    fn default() -> Self {
        Self {
            lease: Duration::from_secs(5),
            renewal: Duration::from_secs(2),
            poll_min: Duration::from_millis(50),
            poll_max: Duration::from_millis(500),
        }
    }
}

#[derive(Clone)]
pub(super) enum LockCoordinator {
    Local(LocalLockManager),
    Distributed(DistributedLockManager),
}

impl LockCoordinator {
    pub(super) fn new(
        strategy: CacheLockingStrategy,
        backend: RedisBackend,
    ) -> Self {
        match strategy {
            CacheLockingStrategy::Local => Self::Local(LocalLockManager::new()),
            CacheLockingStrategy::Distributed => Self::Distributed(
                DistributedLockManager::new(backend, LockTiming::default()),
            ),
        }
    }

    pub(super) fn strategy(&self) -> CacheLockingStrategy {
        match self {
            Self::Local(_) => CacheLockingStrategy::Local,
            Self::Distributed(_) => CacheLockingStrategy::Distributed,
        }
    }

    pub(super) async fn acquire(
        &self,
        key: String,
        deadline: Instant,
    ) -> Result<LockAcquisition, DatabaseError> {
        match self {
            Self::Local(manager) => Ok(manager.acquire(key).into()),
            Self::Distributed(manager) => manager.acquire(key, deadline).await,
        }
    }
}

pub(super) enum LockAcquisition {
    Owned(OwnedLockGuard),
    Waiting(LockWaiter),
}

impl From<LocalLockAcquisition> for LockAcquisition {
    fn from(acquisition: LocalLockAcquisition) -> Self {
        match acquisition {
            LocalLockAcquisition::Owned(guard) => {
                Self::Owned(OwnedLockGuard::Local(guard))
            }
            LocalLockAcquisition::Waiting(waiter) => {
                Self::Waiting(LockWaiter::Local(waiter))
            }
        }
    }
}

pub(super) enum OwnedLockGuard {
    Local(LocalLockGuard),
    Distributed(DistributedLockGuard),
}

impl OwnedLockGuard {
    pub(super) async fn validate_with_connection<C>(
        &self,
        connection: &mut C,
        deadline: Instant,
    ) -> Result<bool, DatabaseError>
    where
        C: ConnectionLike,
    {
        match self {
            Self::Local(_) => Ok(true),
            Self::Distributed(guard) => {
                guard.validate_with_connection(connection, deadline).await
            }
        }
    }

    pub(super) async fn release(
        self,
        deadline: Instant,
    ) -> Result<ReleaseOutcome, DatabaseError> {
        match self {
            Self::Local(guard) => {
                guard.release();
                Ok(ReleaseOutcome::Released)
            }
            Self::Distributed(guard) => guard.release(deadline).await,
        }
    }
}

pub(super) enum LockWaiter {
    Local(LocalLockWaiter),
    Distributed(DistributedLockWaiter),
}

impl LockWaiter {
    pub(super) async fn wait(
        self,
        deadline: Instant,
    ) -> Result<(), DatabaseError> {
        match self {
            Self::Local(waiter) => waiter.wait(deadline).await,
            Self::Distributed(waiter) => waiter.wait(deadline).await,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReleaseOutcome {
    Released,
    NotOwner,
}

/// Normalize only the requested lookup form's case. Raw IDs and aliases remain
/// distinct lock identities and may therefore fill concurrently.
pub(super) fn normalize_key(key: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        key.to_owned()
    } else {
        key.to_lowercase()
    }
}

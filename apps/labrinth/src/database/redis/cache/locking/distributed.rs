use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use redis::aio::ConnectionLike;
use tokio::runtime::Handle;
use tokio::time::{Instant, sleep, timeout_at};
use tokio_util::sync::CancellationToken;
use tracing::warn;

use crate::database::models::DatabaseError;

use super::super::super::commands;
use super::super::super::connection::RedisBackend;
use super::{
    LockAcquisition, LockTiming, LockWaiter, OwnedLockGuard, ReleaseOutcome,
};

#[derive(Clone)]
pub struct DistributedLockManager {
    backend: RedisBackend,
    timing: LockTiming,
}

impl DistributedLockManager {
    pub(super) fn new(backend: RedisBackend, timing: LockTiming) -> Self {
        Self { backend, timing }
    }

    pub(super) async fn acquire(
        &self,
        key: String,
    ) -> Result<LockAcquisition, DatabaseError> {
        let lock_key = cache_lock_key(&key);
        let acquired = self.try_acquire(&lock_key).await?;

        if !acquired {
            return Ok(LockAcquisition::Waiting(LockWaiter::Distributed(
                DistributedLockWaiter {
                    manager: self.clone(),
                    lock_key,
                },
            )));
        }

        let state = Arc::new(LeaseState::new());
        let renewal_cancellation_token = CancellationToken::new();
        self.spawn_renewal(
            lock_key.clone(),
            state.clone(),
            renewal_cancellation_token.clone(),
        );
        Ok(LockAcquisition::Owned(OwnedLockGuard::Distributed(
            DistributedLockGuard {
                manager: self.clone(),
                lock_key,
                state,
                renewal_cancellation_token,
                cleanup_complete: false,
            },
        )))
    }

    async fn try_acquire(&self, lock_key: &str) -> Result<bool, DatabaseError> {
        let mut connection = self.backend.connect().await?;
        commands::acquire_lock(
            &mut connection,
            lock_key,
            duration_millis(self.timing.lease),
        )
        .await
    }

    async fn renew(&self, lock_key: &str) -> Result<bool, DatabaseError> {
        let mut connection = self.backend.connect().await?;
        commands::renew_lock(
            &mut connection,
            lock_key,
            duration_millis(self.timing.lease),
        )
        .await
    }

    async fn renew_with_connection<C>(
        &self,
        connection: &mut C,
        lock_key: &str,
    ) -> Result<bool, DatabaseError>
    where
        C: ConnectionLike,
    {
        commands::renew_lock(
            connection,
            lock_key,
            duration_millis(self.timing.lease),
        )
        .await
    }

    async fn release(&self, lock_key: &str) -> Result<bool, DatabaseError> {
        let mut connection = self.backend.connect().await?;
        commands::release_lock(&mut connection, lock_key).await
    }

    async fn exists(&self, lock_key: &str) -> Result<bool, DatabaseError> {
        let mut connection = self.backend.connect().await?;
        commands::lock_exists(&mut connection, lock_key).await
    }

    fn spawn_renewal(
        &self,
        lock_key: String,
        state: Arc<LeaseState>,
        cancellation_token: CancellationToken,
    ) {
        let manager = self.clone();
        tokio::spawn(async move {
            loop {
                let result = tokio::select! {
                    biased;
                    _ = cancellation_token.cancelled() => break,
                    result = async {
                        sleep(manager.timing.renewal).await;
                        manager.renew(&lock_key).await
                    } => result,
                };

                match result {
                    Ok(true) => {}
                    Ok(false) => {
                        state.mark_lost();
                        break;
                    }
                    Err(_) => {
                        state.mark_lost();
                        warn!("failed to renew distributed cache lease");
                        break;
                    }
                }
            }
        });
    }

    fn spawn_cleanup(&self, lock_key: String) {
        let Ok(handle) = Handle::try_current() else {
            return;
        };
        let manager = self.clone();
        handle.spawn(async move {
            if manager.release(&lock_key).await.is_err() {
                warn!("failed to clean up distributed cache lease");
            }
        });
    }
}

pub struct DistributedLockGuard {
    manager: DistributedLockManager,
    lock_key: String,
    state: Arc<LeaseState>,
    renewal_cancellation_token: CancellationToken,
    cleanup_complete: bool,
}

impl DistributedLockGuard {
    pub(super) async fn validate_with_connection<C>(
        &self,
        connection: &mut C,
    ) -> Result<bool, DatabaseError>
    where
        C: ConnectionLike,
    {
        if !self.state.owned.load(Ordering::Acquire) {
            return Ok(false);
        }

        let result = self
            .manager
            .renew_with_connection(connection, &self.lock_key)
            .await;
        self.handle_validation_result(result)
    }

    fn handle_validation_result(
        &self,
        result: Result<bool, DatabaseError>,
    ) -> Result<bool, DatabaseError> {
        match result {
            Ok(true) => Ok(self.state.owned.load(Ordering::Acquire)),
            Ok(false) => {
                self.state.mark_lost();
                Ok(false)
            }
            Err(error) => {
                self.state.mark_lost();
                Err(error)
            }
        }
    }

    pub(super) async fn release(
        mut self,
    ) -> Result<ReleaseOutcome, DatabaseError> {
        self.stop_renewal();
        match self.manager.release(&self.lock_key).await {
            Ok(true) => {
                self.cleanup_complete = true;
                self.state.owned.store(false, Ordering::Release);
                Ok(ReleaseOutcome::Released)
            }
            Ok(false) => {
                self.cleanup_complete = true;
                self.state.mark_lost();
                Ok(ReleaseOutcome::NotOwner)
            }
            Err(error) => Err(error),
        }
    }

    fn stop_renewal(&self) {
        self.renewal_cancellation_token.cancel();
    }
}

impl Drop for DistributedLockGuard {
    fn drop(&mut self) {
        self.stop_renewal();
        if self.cleanup_complete {
            return;
        }

        self.manager.spawn_cleanup(self.lock_key.clone());
    }
}

pub struct DistributedLockWaiter {
    manager: DistributedLockManager,
    lock_key: String,
}

impl DistributedLockWaiter {
    pub(super) async fn wait(
        self,
        deadline: Instant,
    ) -> Result<(), DatabaseError> {
        let mut attempt = 0;
        loop {
            if !self.manager.exists(&self.lock_key).await? {
                return Ok(());
            }

            let delay = poll_delay(self.manager.timing, attempt);

            timeout_at(deadline, sleep(delay))
                .await
                .map_err(|_| lock_timeout())?;
            attempt = attempt.saturating_add(1);
        }
    }
}

struct LeaseState {
    owned: AtomicBool,
}

impl LeaseState {
    fn new() -> Self {
        Self {
            owned: AtomicBool::new(true),
        }
    }

    fn mark_lost(&self) {
        self.owned.store(false, Ordering::Release);
    }
}

fn cache_lock_key(key: &str) -> String {
    format!("{key}/lock")
}

fn poll_delay(timing: LockTiming, attempt: u32) -> Duration {
    // With the defaults, delays are 50, 100, 200, 250 then 500 ms per poll.
    timing
        .poll_min
        .saturating_mul(2_u32.saturating_pow(attempt))
        .min(timing.poll_max)
}

fn duration_millis(duration: Duration) -> u64 {
    duration.as_millis().min(u64::MAX as u128) as u64
}

fn lock_timeout() -> DatabaseError {
    DatabaseError::CacheTimeout {
        locks_released: 0,
        locks_waiting: 1,
        time_spent_pool_wait_ms: 0,
        time_spent_total_ms: 0,
    }
}

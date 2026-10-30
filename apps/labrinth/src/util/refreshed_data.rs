use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use eyre::Result;
use rand::RngExt;
use tokio::sync::{Mutex, RwLock};

/// Cached data with explicit refreshes, jittered expiry, and stale-on-error fallback.
/// Refreshes are serialized within this process; jitter spreads them across replicas.
pub struct RefreshedData<T> {
    data: RwLock<T>,
    fetch: Box<
        dyn Fn() -> Pin<Box<dyn Future<Output = Result<T>> + Send>>
            + Send
            + Sync,
    >,
    refresh_lock: Mutex<()>,
    clock_origin: Instant,
    refresh_at: AtomicSeconds,
    expiry: Duration,
    jitter: Duration,
}

const REFRESH_RETRY_DELAY: Duration = Duration::from_secs(60);
const REFRESH_RETRY_JITTER: Duration = Duration::from_secs(30);

/// An atomic duration stored with whole-second precision. Subseconds are truncated.
pub struct AtomicSeconds(AtomicU64);

impl AtomicSeconds {
    pub const fn new(value: Duration) -> Self {
        Self(AtomicU64::new(value.as_secs()))
    }

    pub fn load(&self, ordering: Ordering) -> Duration {
        Duration::from_secs(self.0.load(ordering))
    }

    pub fn store(&self, value: Duration, ordering: Ordering) {
        self.0.store(value.as_secs(), ordering);
    }
}

impl<T: Clone> RefreshedData<T> {
    /// Fetches the initial value and retains the callback for later refreshes.
    pub async fn new<F, Fut>(
        expiry: Duration,
        jitter: Duration,
        fetch: F,
    ) -> Result<Self>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T>> + Send + 'static,
    {
        let data = fetch().await?;
        Ok(Self {
            data: RwLock::new(data),
            fetch: Box::new(move || Box::pin(fetch())),
            refresh_lock: Mutex::new(()),
            clock_origin: Instant::now(),
            refresh_at: AtomicSeconds::new(Self::jittered_duration(
                expiry, jitter,
            )),
            expiry,
            jitter,
        })
    }

    /// Returns the cached value without fetching new data.
    pub async fn get(&self) -> T {
        self.data.read().await.clone()
    }

    /// Refreshes when due. Failures retain stale data and schedule a retry.
    pub async fn refresh(&self) {
        if !self.is_expired() {
            return;
        }

        let _guard = self.refresh_lock.lock().await;
        if !self.is_expired() {
            return;
        }

        match (self.fetch)().await {
            Ok(data) => {
                *self.data.write().await = data;
                self.schedule_refresh(self.expiry, self.jitter);
            }
            Err(error) => {
                self.schedule_refresh(
                    REFRESH_RETRY_DELAY,
                    REFRESH_RETRY_JITTER,
                );
                tracing::warn!(
                    ?error,
                    "failed to refresh data; serving cached value"
                );

            }
        }
    }

    fn is_expired(&self) -> bool {
        self.clock_origin.elapsed() >= self.refresh_at.load(Ordering::Acquire)
    }

    fn schedule_refresh(&self, delay: Duration, jitter: Duration) {
        self.refresh_at.store(
            self.clock_origin
                .elapsed()
                .saturating_add(Self::jittered_duration(delay, jitter)),
            Ordering::Release,
        );
    }

    fn jittered_duration(delay: Duration, jitter: Duration) -> Duration {
        let minimum = delay.saturating_sub(jitter);
        let maximum = delay.saturating_add(jitter);
        rand::rng().random_range(minimum..=maximum)
    }
}

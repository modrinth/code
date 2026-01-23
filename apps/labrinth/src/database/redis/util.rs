use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use deadpool_redis::PoolError;
use derive_more::{Deref, DerefMut};
use redis::{FromRedisValue, RedisResult, ToRedisArgs};
use tokio::sync::Notify;
use tokio::time::{Duration, timeout};
use tracing::{Instrument, info_span};

use crate::database::models::DatabaseError;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct InstrumentedPool {
    inner: deadpool_redis::Pool,
}

impl InstrumentedPool {
    pub fn new(inner: deadpool_redis::Pool) -> Self {
        Self { inner }
    }

    pub async fn get(&self) -> Result<deadpool_redis::Connection, PoolError> {
        self.inner
            .get()
            .instrument(info_span!("get redis connection"))
            .await
    }
}

pub fn redis_pipe() -> InstrumentedPipeline {
    InstrumentedPipeline {
        inner: redis::pipe(),
    }
}

#[derive(Clone, Deref, DerefMut)]
pub struct InstrumentedPipeline {
    #[deref]
    #[deref_mut]
    inner: redis::Pipeline,
}

impl InstrumentedPipeline {
    pub fn atomic(&mut self) -> &mut Self {
        self.inner.atomic();
        self
    }

    #[inline]
    pub async fn query_async<T: FromRedisValue>(
        &self,
        con: &mut impl redis::aio::ConnectionLike,
    ) -> RedisResult<T> {
        self.inner
            .query_async(con)
            .instrument(info_span!("execute redis pipeline"))
            .await
    }
}

pub fn cmd(name: &str) -> InstrumentedCmd {
    InstrumentedCmd {
        inner: redis::cmd(name),
        name: name.to_string(),
        args: Vec::new(),
    }
}

pub struct InstrumentedCmd {
    inner: redis::Cmd,
    name: String,
    args: Vec<String>,
}

impl InstrumentedCmd {
    #[inline]
    pub fn arg<T: ToRedisArgs + Debug>(&mut self, arg: T) -> &mut Self {
        self.args.push(format!("{arg:?}"));
        self.inner.arg(arg);
        self
    }

    #[inline]
    pub async fn query_async<T: FromRedisValue>(
        &self,
        con: &mut impl redis::aio::ConnectionLike,
    ) -> RedisResult<T> {
        let span = info_span!(
            "query_async",
            // <https://opentelemetry.io/docs/specs/semconv/db/redis/>
            db.system.name = "redis",
            db.operation.name = self.name,
            db.query.text = format!("{} {}", self.name, self.args.join(" ")),
        );
        self.inner.query_async(con).instrument(span).await
    }
}

pub fn cache() -> (CacheWriter, CacheSubscriber) {
    let shared = Arc::new(Shared::new());
    (
        CacheWriter {
            shared: shared.clone(),
        },
        CacheSubscriber { shared },
    )
}

pub struct CacheWriter {
    shared: Arc<Shared>,
}

impl CacheWriter {
    pub fn write(&self) {
        self.shared.make_ready();
    }
}

#[derive(Clone)]
pub struct CacheSubscriber {
    shared: Arc<Shared>,
}

impl CacheSubscriber {
    pub async fn wait_timeout(
        self,
        duration: Duration,
    ) -> Result<(), DatabaseError> {
        timeout(duration, self.shared.wait()).await.map_err(|_| {
            DatabaseError::LocalCacheTimeout {
                released: 0,
                total: 1,
            }
        })
    }
}

struct Shared {
    ready: AtomicBool,
    // With this implementation's intrusive linked lists, the waiters are stored inline in the future
    // so there's no heap allocation per waiter.
    wakers: Notify,
}

impl Shared {
    fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
            wakers: Notify::new(),
        }
    }

    fn make_ready(&self) {
        self.ready.store(true, Ordering::Release);
        self.wakers.notify_waiters();
    }

    async fn wait(&self) {
        let ready = self.ready.load(Ordering::Acquire);

        if ready {
            return;
        }

        let notification = self.wakers.notified();
        // Don't need to call `enable` as we use notify_waiters

        // Prevent race where the writer set the ready bit and notified waiters between the load and registering the waiter
        let ready = self.ready.load(Ordering::SeqCst);

        if ready {
            return;
        }

        notification.await;
    }
}

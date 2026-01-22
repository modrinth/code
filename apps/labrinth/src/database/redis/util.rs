use std::fmt::Debug;

use deadpool_redis::PoolError;
use derive_more::{Deref, DerefMut};
use redis::{FromRedisValue, RedisResult, ToRedisArgs};
use tracing::{Instrument, info_span};

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

use redis::{FromRedisValue, RedisResult, ToRedisArgs};
use tracing::{Instrument, info_span};

pub(crate) fn cmd(name: &str) -> InstrumentedCmd {
    InstrumentedCmd {
        inner: redis::cmd(name),
        name: name.to_string(),
    }
}

pub(crate) struct InstrumentedCmd {
    inner: redis::Cmd,
    name: String,
}

impl InstrumentedCmd {
    #[inline]
    pub fn arg<T: ToRedisArgs>(&mut self, arg: T) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    #[inline]
    pub async fn query_async<T: FromRedisValue>(
        &self,
        con: &mut impl redis::aio::ConnectionLike,
    ) -> RedisResult<T> {
        let span = info_span!(
            "cmd.query_async",
            // <https://opentelemetry.io/docs/specs/semconv/db/redis/>
            db.system.name = "redis",
            db.operation.name = self.name,
            db.query.text = self.name,
        );
        self.inner.query_async(con).instrument(span).await
    }
}

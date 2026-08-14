use std::time::Duration;

use eyre::{Result, WrapErr, bail};
use prometheus::Registry;

use super::RedisPool;
use super::config::{RedisConfig, RedisTopology};
use super::metrics::{
    LogicalPoolStatus, LogicalPoolStatusProvider,
    register_blocking_pool_metrics,
};

const POOL_RETAIN_INTERVAL: Duration = Duration::from_secs(30);
const MAX_IDLE_CONNECTION_AGE: Duration = Duration::from_secs(5 * 60);
const MAX_STANDALONE_CONNECTION_AGE: Duration = Duration::from_secs(120);

/// A pool of Redis connections used for blocking operations.
#[derive(Clone)]
pub(super) struct RedisBlockingPool {
    inner: RedisBlockingPoolInner,
}

#[derive(Clone)]
enum RedisBlockingPoolInner {
    Standalone(deadpool_redis::Pool),
    Cluster(deadpool_redis::cluster::Pool),
}

impl RedisBlockingPool {
    pub(super) async fn new(config: &RedisConfig) -> Result<Self> {
        let pool_size = config.blocking_pool_size();
        let inner = match config.topology() {
            RedisTopology::Standalone => {
                let connection_config = redis::AsyncConnectionConfig::new()
                    .set_connection_timeout(None)
                    .set_response_timeout(None);
                let manager = deadpool_redis::Manager::new_with_config(
                    config.seed_urls()[0].clone(),
                    connection_config,
                )
                .wrap_err("configuring standalone blocking Redis client")?;
                let pool = deadpool_redis::Pool::builder(manager)
                    .max_size(pool_size.max())
                    .wait_timeout(Some(Duration::from_millis(
                        config.wait_timeout_ms(),
                    )))
                    .runtime(deadpool_redis::Runtime::Tokio1)
                    .build()
                    .wrap_err("building standalone blocking Redis pool")?;
                retain_standalone_pool(pool.clone());
                RedisBlockingPoolInner::Standalone(pool)
            }
            RedisTopology::Cluster => {
                let manager = deadpool_redis::cluster::Manager::new(
                    config.seed_urls().to_vec(),
                    false,
                )
                .wrap_err("configuring clustered blocking Redis client")?;
                let pool = deadpool_redis::cluster::Pool::builder(manager)
                    .max_size(pool_size.max())
                    .wait_timeout(Some(Duration::from_millis(
                        config.wait_timeout_ms(),
                    )))
                    .runtime(deadpool_redis::Runtime::Tokio1)
                    .build()
                    .wrap_err("building clustered blocking Redis pool")?;
                retain_cluster_pool(pool.clone());
                RedisBlockingPoolInner::Cluster(pool)
            }
        };

        Ok(Self { inner })
    }

    pub(super) fn register_metrics(&self, registry: &Registry) -> Result<()> {
        register_blocking_pool_metrics(registry, self.clone())
    }

    async fn brpop(
        &self,
        key: &str,
        timeout: Duration,
    ) -> Result<Option<[Vec<u8>; 2]>> {
        if timeout.is_zero() {
            bail!("redis blocking timeout must be greater than zero");
        }

        let mut command = redis::cmd("BRPOP");
        command.arg(key).arg(timeout.as_secs_f64());

        let response: Option<(Vec<u8>, Vec<u8>)> =
            match &self.inner {
                RedisBlockingPoolInner::Standalone(pool) => {
                    let mut connection = pool.get().await.wrap_err(
                        "fetching standalone blocking Redis connection",
                    )?;
                    command.query_async(&mut connection).await.wrap_err(
                        "reading from standalone Redis blocking queue",
                    )?
                }
                RedisBlockingPoolInner::Cluster(pool) => {
                    let mut connection = pool.get().await.wrap_err(
                        "fetching clustered blocking Redis connection",
                    )?;
                    command.query_async(&mut connection).await.wrap_err(
                        "reading from clustered Redis blocking queue",
                    )?
                }
            };

        Ok(response.map(|(key, value)| [key, value]))
    }
}

impl LogicalPoolStatusProvider for RedisBlockingPool {
    fn logical_pool_status(&self) -> LogicalPoolStatus {
        match &self.inner {
            RedisBlockingPoolInner::Standalone(pool) => {
                LogicalPoolStatus::from_deadpool(pool.status())
            }
            RedisBlockingPoolInner::Cluster(pool) => {
                LogicalPoolStatus::from_deadpool(pool.status())
            }
        }
    }
}

impl RedisPool {
    pub async fn brpop(
        &self,
        key: &str,
        timeout: Duration,
    ) -> Result<Option<[Vec<u8>; 2]>> {
        self.blocking.brpop(key, timeout).await
    }
}

fn retain_standalone_pool(pool: deadpool_redis::Pool) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(POOL_RETAIN_INTERVAL).await;
            pool.retain(|_, metrics| {
                metrics.last_used() < MAX_IDLE_CONNECTION_AGE
                    && metrics.created.elapsed() < MAX_STANDALONE_CONNECTION_AGE
            });
        }
    });
}

fn retain_cluster_pool(pool: deadpool_redis::cluster::Pool) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(POOL_RETAIN_INTERVAL).await;
            pool.retain(|_, metrics| {
                metrics.last_used() < MAX_IDLE_CONNECTION_AGE
            });
        }
    });
}

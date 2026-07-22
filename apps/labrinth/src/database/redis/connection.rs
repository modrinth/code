use std::time::Duration;

use futures::future::try_join_all;
use prometheus::Registry;
use redis::aio::ConnectionLike;
use redis::cluster_read_routing::{
    RandomReplicaStrategy, RoundRobinReplicaStrategy,
};
use thiserror::Error;
use tracing::warn;

use crate::database::redis::ReadReplicaStrategy;

use super::config::{RedisBackendConfig, RedisConfig, RedisPoolSize};
use super::metrics::{
    LogicalPoolStatus, LogicalPoolStatusProvider, register_command_pool_metrics,
};

const POOL_RETAIN_INTERVAL: Duration = Duration::from_secs(30);
const MAX_IDLE_CONNECTION_AGE: Duration = Duration::from_secs(5 * 60);
const MAX_STANDALONE_CONNECTION_AGE: Duration = Duration::from_secs(120);

/// The primary backing "connection provider" for a Redis backend implementation.
#[derive(Clone)]
pub(super) enum RedisBackend {
    StandalonePooled(deadpool_redis::Pool),
    ClusterPooled(deadpool_redis::cluster::Pool),
    ClusterMultiplexed(redis::cluster_async::ClusterConnection),
}

#[derive(Debug, Error)]
pub(super) enum RedisBackendBuildError {
    #[error("failed to configure Redis client: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("failed to build Redis pool: {0}")]
    PoolBuild(#[from] deadpool_redis::BuildError),
    #[error("failed to establish initial Redis pool connections: {0}")]
    Pool(#[from] deadpool_redis::PoolError),
}

pub(super) struct RedisConnection {
    inner: RedisConnectionInner,
}

enum RedisConnectionInner {
    StandalonePooled(deadpool_redis::Connection),
    ClusterPooled(deadpool_redis::cluster::Connection),
    ClusterMultiplexed(redis::cluster_async::ClusterConnection),
}

impl RedisBackend {
    pub(super) async fn new(
        config: &RedisConfig,
    ) -> Result<Self, RedisBackendBuildError> {
        match config.backend() {
            RedisBackendConfig::StandalonePooled(pool_size) => {
                Self::standalone_pooled(config, pool_size).await
            }
            RedisBackendConfig::ClusterPooled(pool_size) => {
                Self::cluster_pooled(config, pool_size).await
            }
            RedisBackendConfig::ClusterMultiplexed => {
                Self::cluster_multiplexed(config).await
            }
        }
    }

    async fn standalone_pooled(
        config: &RedisConfig,
        pool_size: RedisPoolSize,
    ) -> Result<Self, RedisBackendBuildError> {
        let connection_config = redis::AsyncConnectionConfig::new()
            .set_connection_timeout(None)
            .set_response_timeout(None);
        let manager = deadpool_redis::Manager::new_with_config(
            config.seed_urls()[0].clone(),
            connection_config,
        )?;
        let pool = deadpool_redis::Pool::builder(manager)
            .max_size(pool_size.max())
            .wait_timeout(Some(Duration::from_millis(config.wait_timeout_ms())))
            .runtime(deadpool_redis::Runtime::Tokio1)
            .build()?;

        warm_standalone_pool(&pool, pool_size.min()).await?;
        retain_standalone_pool(pool.clone());

        Ok(Self::StandalonePooled(pool))
    }

    async fn cluster_pooled(
        config: &RedisConfig,
        pool_size: RedisPoolSize,
    ) -> Result<Self, RedisBackendBuildError> {
        let manager = deadpool_redis::cluster::Manager::new(
            config.seed_urls().to_vec(),
            false,
        )?;
        let pool = deadpool_redis::cluster::Pool::builder(manager)
            .max_size(pool_size.max())
            .wait_timeout(Some(Duration::from_millis(config.wait_timeout_ms())))
            .runtime(deadpool_redis::Runtime::Tokio1)
            .build()?;

        if config.read_replica_strategy() != ReadReplicaStrategy::Primary {
            warn!(
                "Cannot respect read replica strategy when using cluster pooled backend"
            );
        }

        warm_cluster_pool(&pool, pool_size.min()).await?;
        retain_cluster_pool(pool.clone());

        Ok(Self::ClusterPooled(pool))
    }

    async fn cluster_multiplexed(
        config: &RedisConfig,
    ) -> Result<Self, RedisBackendBuildError> {
        let mut builder = redis::cluster::ClusterClientBuilder::new(
            config.seed_urls().iter().map(String::as_str),
        );

        match config.read_replica_strategy() {
            ReadReplicaStrategy::Primary => {}
            ReadReplicaStrategy::RoundRobinReplica => {
                builder = builder
                    .read_routing_strategy(RoundRobinReplicaStrategy::new());
            }
            ReadReplicaStrategy::RandomReplica => {
                builder = builder.read_routing_strategy(RandomReplicaStrategy);
            }
        }

        let client = builder.build()?;
        let connection = client.get_async_connection().await?;

        Ok(Self::ClusterMultiplexed(connection))
    }

    pub(super) async fn connect(
        &self,
    ) -> Result<RedisConnection, deadpool_redis::PoolError> {
        let inner = match self {
            Self::StandalonePooled(pool) => {
                RedisConnectionInner::StandalonePooled(pool.get().await?)
            }
            Self::ClusterPooled(pool) => {
                RedisConnectionInner::ClusterPooled(pool.get().await?)
            }
            Self::ClusterMultiplexed(connection) => {
                RedisConnectionInner::ClusterMultiplexed(connection.clone())
            }
        };

        Ok(RedisConnection { inner })
    }

    pub(super) fn register_metrics(
        &self,
        registry: &Registry,
    ) -> Result<(), prometheus::Error> {
        register_command_pool_metrics(registry, self.clone())
    }
}

impl LogicalPoolStatusProvider for RedisBackend {
    fn logical_pool_status(&self) -> LogicalPoolStatus {
        match self {
            Self::StandalonePooled(pool) => {
                LogicalPoolStatus::from_deadpool(pool.status())
            }
            Self::ClusterPooled(pool) => {
                LogicalPoolStatus::from_deadpool(pool.status())
            }
            Self::ClusterMultiplexed(_) => {
                LogicalPoolStatus::shared_multiplexed()
            }
        }
    }
}

impl ConnectionLike for RedisConnectionInner {
    fn req_packed_command<'a>(
        &'a mut self,
        cmd: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        match self {
            Self::StandalonePooled(connection) => {
                connection.req_packed_command(cmd)
            }
            Self::ClusterPooled(connection) => {
                connection.req_packed_command(cmd)
            }
            Self::ClusterMultiplexed(connection) => {
                connection.req_packed_command(cmd)
            }
        }
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        match self {
            Self::StandalonePooled(connection) => {
                connection.req_packed_commands(cmd, offset, count)
            }
            Self::ClusterPooled(connection) => {
                connection.req_packed_commands(cmd, offset, count)
            }
            Self::ClusterMultiplexed(connection) => {
                connection.req_packed_commands(cmd, offset, count)
            }
        }
    }

    fn get_db(&self) -> i64 {
        match self {
            Self::StandalonePooled(connection) => connection.get_db(),
            Self::ClusterPooled(connection) => connection.get_db(),
            Self::ClusterMultiplexed(connection) => connection.get_db(),
        }
    }
}

impl ConnectionLike for RedisConnection {
    fn req_packed_command<'a>(
        &'a mut self,
        cmd: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        self.inner.req_packed_command(cmd)
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        self.inner.req_packed_commands(cmd, offset, count)
    }

    fn get_db(&self) -> i64 {
        self.inner.get_db()
    }
}

async fn warm_standalone_pool(
    pool: &deadpool_redis::Pool,
    min: usize,
) -> Result<(), deadpool_redis::PoolError> {
    let connections = try_join_all((0..min).map(|_| pool.get())).await?;
    drop(connections);
    Ok(())
}

async fn warm_cluster_pool(
    pool: &deadpool_redis::cluster::Pool,
    min: usize,
) -> Result<(), deadpool_redis::PoolError> {
    let connections = try_join_all((0..min).map(|_| pool.get())).await?;
    drop(connections);
    Ok(())
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

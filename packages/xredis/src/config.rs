use std::{fmt, str::FromStr};

use thiserror::Error;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CacheLockingStrategy {
    #[default]
    Local,
    Distributed,
}

impl CacheLockingStrategy {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Distributed => "distributed",
        }
    }
}

impl fmt::Display for CacheLockingStrategy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Error)]
#[error("invalid cache locking strategy; expected `local` or `distributed`")]
pub struct InvalidCacheLockingStrategy;

impl FromStr for CacheLockingStrategy {
    type Err = InvalidCacheLockingStrategy;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "local" => Ok(Self::Local),
            "distributed" => Ok(Self::Distributed),
            _ => Err(InvalidCacheLockingStrategy),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedisTopology {
    Standalone,
    Cluster,
}

#[derive(Debug, Error)]
#[error("invalid Redis topology; expected `standalone` or `cluster`")]
pub struct InvalidRedisMode;

impl FromStr for RedisTopology {
    type Err = InvalidRedisMode;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "standalone" => Ok(Self::Standalone),
            "cluster" => Ok(Self::Cluster),
            _ => Err(InvalidRedisMode),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedisConnectionType {
    Pooled,
    Multiplexed,
}

#[derive(Debug, Error)]
#[error("invalid Redis connection type; expected `pooled` or `multiplexed`")]
pub struct InvalidRedisConnectionType;

impl FromStr for RedisConnectionType {
    type Err = InvalidRedisConnectionType;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pooled" => Ok(Self::Pooled),
            "multiplexed" => Ok(Self::Multiplexed),
            _ => Err(InvalidRedisConnectionType),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RedisPoolSize {
    max: usize,
    min: usize,
}

impl RedisPoolSize {
    fn new(
        name: &'static str,
        max: usize,
        min: usize,
    ) -> Result<Self, RedisConfigError> {
        if max == 0 || min > max {
            return Err(RedisConfigError::InvalidPoolSize { name, max, min });
        }

        Ok(Self { max, min })
    }

    pub(crate) fn max(self) -> usize {
        self.max
    }

    pub(crate) fn min(self) -> usize {
        self.min
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RedisBackendConfig {
    StandalonePooled(RedisPoolSize),
    ClusterPooled(RedisPoolSize),
    ClusterMultiplexed,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    mode: RedisTopology,
    backend: RedisBackendConfig,
    seed_urls: Vec<String>,
    wait_timeout_ms: u64,
    blocking_pool_size: RedisPoolSize,
    cache_locking_strategy: CacheLockingStrategy,
    read_replica_strategy: ReadReplicaStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadReplicaStrategy {
    Primary,
    RoundRobinReplica,
    RandomReplica,
}

#[derive(Debug, Error)]
#[error(
    "invalid Redis read replica strategy; expected `primary`, `round_robin`, or `random`"
)]
pub struct InvalidRedisReadReplicaStrategy;

impl FromStr for ReadReplicaStrategy {
    type Err = InvalidRedisReadReplicaStrategy;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "primary" => Ok(Self::Primary),
            "round_robin" => Ok(Self::RoundRobinReplica),
            "random" => Ok(Self::RandomReplica),
            _ => Err(InvalidRedisReadReplicaStrategy),
        }
    }
}

#[derive(Debug, Error)]
pub enum RedisConfigError {
    #[error("Redis configuration must contain at least one URL")]
    MissingUrl,
    #[error("standalone Redis mode requires exactly one URL")]
    MultipleStandaloneUrls,
    #[error(
        "unsupported Redis configuration: `{mode:?}` mode with `{connection_type:?}` connections"
    )]
    UnsupportedConnectionType {
        mode: RedisTopology,
        connection_type: RedisConnectionType,
    },
    #[error(
        "invalid {name} Redis pool size: minimum {min} must not exceed nonzero maximum {max}"
    )]
    InvalidPoolSize {
        name: &'static str,
        max: usize,
        min: usize,
    },
    #[error("unsupported Redis cache locking strategy `{strategy}`")]
    UnsupportedCacheLockingStrategy { strategy: CacheLockingStrategy },
}

impl RedisConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mode: RedisTopology,
        connection_type: RedisConnectionType,
        raw_urls: &str,
        wait_timeout_ms: u64,
        standalone_pool_size: (usize, usize),
        cluster_pool_size: (usize, usize),
        blocking_pool_size: (usize, usize),
        cache_locking_strategy: CacheLockingStrategy,
        read_replica_strategy: ReadReplicaStrategy,
    ) -> Result<Self, RedisConfigError> {
        if cache_locking_strategy == CacheLockingStrategy::Distributed {
            return Err(RedisConfigError::UnsupportedCacheLockingStrategy {
                strategy: cache_locking_strategy,
            });
        }

        let seed_urls = raw_urls
            .split(',')
            .map(str::trim)
            .filter(|url| !url.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        if seed_urls.is_empty() {
            return Err(RedisConfigError::MissingUrl);
        }

        let backend = match (mode, connection_type) {
            (RedisTopology::Standalone, RedisConnectionType::Pooled) => {
                if seed_urls.len() != 1 {
                    return Err(RedisConfigError::MultipleStandaloneUrls);
                }
                RedisBackendConfig::StandalonePooled(RedisPoolSize::new(
                    "standalone",
                    standalone_pool_size.0,
                    standalone_pool_size.1,
                )?)
            }
            (RedisTopology::Cluster, RedisConnectionType::Pooled) => {
                RedisBackendConfig::ClusterPooled(RedisPoolSize::new(
                    "cluster",
                    cluster_pool_size.0,
                    cluster_pool_size.1,
                )?)
            }
            (RedisTopology::Cluster, RedisConnectionType::Multiplexed) => {
                RedisBackendConfig::ClusterMultiplexed
            }
            (mode, connection_type) => {
                return Err(RedisConfigError::UnsupportedConnectionType {
                    mode,
                    connection_type,
                });
            }
        };

        Ok(Self {
            mode,
            backend,
            seed_urls,
            wait_timeout_ms,
            blocking_pool_size: RedisPoolSize::new(
                "blocking",
                blocking_pool_size.0,
                blocking_pool_size.1,
            )?,
            cache_locking_strategy,
            read_replica_strategy,
        })
    }

    pub fn topology(&self) -> RedisTopology {
        self.mode
    }

    pub(crate) fn backend(&self) -> RedisBackendConfig {
        self.backend
    }

    pub fn seed_urls(&self) -> &[String] {
        &self.seed_urls
    }

    pub fn wait_timeout_ms(&self) -> u64 {
        self.wait_timeout_ms
    }

    pub(crate) fn blocking_pool_size(&self) -> RedisPoolSize {
        self.blocking_pool_size
    }

    pub fn read_replica_strategy(&self) -> ReadReplicaStrategy {
        self.read_replica_strategy
    }

    pub fn cache_locking_strategy(&self) -> CacheLockingStrategy {
        self.cache_locking_strategy
    }
}

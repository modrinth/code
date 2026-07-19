use std::str::FromStr;

use crate::env::ENV;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedisMode {
    Standalone,
    Cluster,
}

#[derive(Debug, Error)]
#[error("invalid Redis mode; expected `standalone` or `cluster`")]
pub struct InvalidRedisMode;

impl FromStr for RedisMode {
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
pub(super) struct RedisPoolSize {
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

    pub(super) fn max(self) -> usize {
        self.max
    }

    pub(super) fn min(self) -> usize {
        self.min
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum RedisBackendConfig {
    StandalonePooled(RedisPoolSize),
    ClusterPooled(RedisPoolSize),
    ClusterMultiplexed,
}

#[derive(Debug, Clone)]
pub(super) struct RedisConfig {
    mode: RedisMode,
    backend: RedisBackendConfig,
    seed_urls: Vec<String>,
    wait_timeout_ms: u64,
    blocking_pool_size: RedisPoolSize,
}

#[derive(Debug, Error)]
pub(super) enum RedisConfigError {
    #[error("`REDIS_URL` must contain at least one Redis URL")]
    MissingUrl,
    #[error("standalone Redis mode requires exactly one URL")]
    MultipleStandaloneUrls,
    #[error(
        "unsupported Redis configuration: `{mode:?}` mode with `{connection_type:?}` connections"
    )]
    UnsupportedConnectionType {
        mode: RedisMode,
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
}

impl RedisConfig {
    pub(super) fn from_env() -> Result<Self, RedisConfigError> {
        Self::new(
            ENV.REDIS_MODE,
            ENV.REDIS_CONNECTION_TYPE,
            &ENV.REDIS_URL,
            ENV.REDIS_WAIT_TIMEOUT_MS,
            (
                ENV.REDIS_MAX_CONNECTIONS as usize,
                ENV.REDIS_MIN_CONNECTIONS,
            ),
            (
                ENV.REDIS_CLUSTER_MAX_CONNECTIONS as usize,
                ENV.REDIS_CLUSTER_MIN_CONNECTIONS,
            ),
            RedisPoolSize::new(
                "blocking",
                ENV.REDIS_BLOCKING_MAX_CONNECTIONS as usize,
                0,
            )?,
        )
    }

    fn new(
        mode: RedisMode,
        connection_type: RedisConnectionType,
        raw_urls: &str,
        wait_timeout_ms: u64,
        standalone_pool_size: (usize, usize),
        cluster_pool_size: (usize, usize),
        blocking_pool_size: RedisPoolSize,
    ) -> Result<Self, RedisConfigError> {
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
            (RedisMode::Standalone, RedisConnectionType::Pooled) => {
                if seed_urls.len() != 1 {
                    return Err(RedisConfigError::MultipleStandaloneUrls);
                }
                RedisBackendConfig::StandalonePooled(RedisPoolSize::new(
                    "standalone",
                    standalone_pool_size.0,
                    standalone_pool_size.1,
                )?)
            }
            (RedisMode::Cluster, RedisConnectionType::Pooled) => {
                RedisBackendConfig::ClusterPooled(RedisPoolSize::new(
                    "cluster",
                    cluster_pool_size.0,
                    cluster_pool_size.1,
                )?)
            }
            (RedisMode::Cluster, RedisConnectionType::Multiplexed) => {
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
            blocking_pool_size,
        })
    }

    pub(super) fn mode(&self) -> RedisMode {
        self.mode
    }

    pub(super) fn backend(&self) -> RedisBackendConfig {
        self.backend
    }

    pub(super) fn seed_urls(&self) -> &[String] {
        &self.seed_urls
    }

    pub(super) fn wait_timeout_ms(&self) -> u64 {
        self.wait_timeout_ms
    }

    pub(super) fn blocking_pool_size(&self) -> RedisPoolSize {
        self.blocking_pool_size
    }
}

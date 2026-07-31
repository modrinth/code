use std::sync::Arc;

use crate::env::ENV;

struct RedisConfig {
    inner: xredis::RedisConfig,
    cache_settings: xredis::CacheSettings,
}

impl RedisConfig {
    fn from_env() -> Result<Self, xredis::RedisConfigError> {
        let inner = xredis::RedisConfig::new(
            ENV.REDIS_TOPOLOGY,
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
            (ENV.REDIS_BLOCKING_MAX_CONNECTIONS as usize, 0),
            ENV.REDIS_CACHE_LOCKING_STRATEGY,
            ENV.REDIS_READ_REPLICA_STRATEGY,
        )?;
        let cache_settings = xredis::CacheSettings {
            default_expiry: ENV.REDIS_DEFAULT_EXPIRY,
            actual_expiry: ENV.REDIS_ACTUAL_EXPIRY,
            version_default_expiry: ENV.REDIS_VERSION_DEFAULT_EXPIRY,
            version_actual_expiry: ENV.REDIS_VERSION_ACTUAL_EXPIRY,
            encoding_format: ENV.REDIS_ENCODING_FORMAT,
            compression_algorithm: ENV.REDIS_COMPRESSION_ALGORITHM,
            compression_level: ENV.REDIS_COMPRESSION_LEVEL,
            compression_threshold_bytes: ENV.REDIS_COMPRESSION_THRESHOLD_BYTES,
            compression_min_savings_ratio: ENV
                .REDIS_COMPRESSION_MIN_SAVINGS_RATIO,
        };

        Ok(Self {
            inner,
            cache_settings,
        })
    }
}

pub async fn from_env(
    meta_namespace: impl Into<Arc<str>>,
) -> xredis::RedisPool {
    let config = RedisConfig::from_env().expect("invalid Redis configuration");
    xredis::RedisPool::new(meta_namespace, config.inner, config.cache_settings)
        .await
        .expect("failed to initialize Redis connections")
}

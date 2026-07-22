#![recursion_limit = "256"]

use std::fmt::{Debug, Display};
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;

use dashmap::DashMap;
use prometheus::Registry;
use redis::aio::ConnectionLike;
use redis::{FromRedisValue, ToRedisArgs};
use serde::Serialize;
use serde::de::DeserializeOwned;

mod blocking;
mod cache;
mod commands;
mod config;
mod connection;
mod key;
mod metrics;
mod pubsub;
mod util;

use cache::{CacheManager, ConnectionProvider};
pub use cache::{
    CacheSettings, Codec, EncodingFormat, InvalidCodec, InvalidEncodingFormat,
    RedisValue,
};
pub use config::{
    CacheLockingStrategy, InvalidCacheLockingStrategy,
    InvalidRedisConnectionType, InvalidRedisMode,
    InvalidRedisReadReplicaStrategy, ReadReplicaStrategy, RedisConfig,
    RedisConfigError, RedisConnectionType, RedisTopology,
};
use connection::RedisBackend;
pub use connection::RedisBackendBuildError;
pub use key::KeyBuilder;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("error while interacting with Redis: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Redis pool error: {0}")]
    Pool(#[from] deadpool_redis::PoolError),
    #[error("error while serializing a Redis cache value: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Redis blocking timeout must be greater than zero")]
    InvalidBlockingTimeout,
    #[error(
        "timeout waiting on local cache lock ({released}/{total} released)"
    )]
    LocalCacheTimeout { released: usize, total: usize },
}

#[derive(Clone)]
pub struct RedisPool {
    backend: RedisBackend,
    blocking: blocking::RedisBlockingPool,
    cache: CacheManager,
    config: RedisConfig,
    key_builder: KeyBuilder,
}

pub struct RedisConnection {
    inner: connection::RedisConnection,
    key_builder: KeyBuilder,
    settings: CacheSettings,
}

impl RedisPool {
    pub async fn new(
        meta_namespace: impl Into<Arc<str>>,
        config: RedisConfig,
        cache_settings: CacheSettings,
    ) -> Result<Self, RedisBackendBuildError> {
        tracing::info!(
            strategy = %config.cache_locking_strategy(),
            "configured Redis cache locking"
        );

        let backend = RedisBackend::new(&config).await?;

        let blocking = blocking::RedisBlockingPool::new(&config).await?;
        let key_builder = KeyBuilder::new(meta_namespace, config.topology());
        let cache = CacheManager::new(key_builder.clone(), cache_settings);

        Ok(Self {
            backend,
            blocking,
            cache,
            config,
            key_builder,
        })
    }

    pub fn key(&self) -> &KeyBuilder {
        &self.key_builder
    }
}

impl RedisPool {
    pub async fn connect(&self) -> Result<RedisConnection, Error> {
        Ok(RedisConnection {
            inner: self.backend.connect().await?,
            key_builder: self.key_builder.clone(),
            settings: self.cache.settings().clone(),
        })
    }

    pub async fn register_and_set_metrics(
        &self,
        registry: &Registry,
    ) -> Result<(), prometheus::Error> {
        self.backend.register_metrics(registry)?;
        self.blocking.register_metrics(registry)
    }

    pub async fn get_cached_keys<F, Fut, T, K, E>(
        &self,
        namespace: &str,
        keys: &[K],
        closure: F,
    ) -> Result<Vec<T>, E>
    where
        F: FnOnce(Vec<K>) -> Fut,
        Fut: Future<Output = Result<DashMap<K, T>, E>>,
        E: From<Error>,
        T: Serialize + DeserializeOwned,
        K: Display
            + Hash
            + Eq
            + PartialEq
            + Clone
            + DeserializeOwned
            + Serialize
            + Debug,
    {
        self.cache
            .get_cached_keys(self, namespace, keys, closure)
            .await
    }

    pub async fn get_cached_keys_raw<F, Fut, T, K, E>(
        &self,
        namespace: &str,
        keys: &[K],
        closure: F,
    ) -> Result<std::collections::HashMap<K, T>, E>
    where
        F: FnOnce(Vec<K>) -> Fut,
        Fut: Future<Output = Result<DashMap<K, T>, E>>,
        E: From<Error>,
        T: Serialize + DeserializeOwned,
        K: Display
            + Hash
            + Eq
            + PartialEq
            + Clone
            + DeserializeOwned
            + Serialize
            + Debug,
    {
        self.cache
            .get_cached_keys_raw(self, namespace, keys, closure)
            .await
    }

    pub async fn get_cached_keys_with_slug<F, Fut, T, I, K, S, E>(
        &self,
        namespace: &str,
        slug_namespace: &str,
        case_sensitive: bool,
        keys: &[I],
        closure: F,
    ) -> Result<Vec<T>, E>
    where
        F: FnOnce(Vec<I>) -> Fut,
        Fut: Future<Output = Result<DashMap<K, (Option<S>, T)>, E>>,
        E: From<Error>,
        T: Serialize + DeserializeOwned,
        I: Display + Hash + Eq + PartialEq + Clone + Debug,
        K: Display
            + Hash
            + Eq
            + PartialEq
            + Clone
            + DeserializeOwned
            + Serialize,
        S: Display + Clone + DeserializeOwned + Serialize + Debug,
    {
        self.cache
            .get_cached_keys_with_slug(
                self,
                namespace,
                slug_namespace,
                case_sensitive,
                keys,
                closure,
            )
            .await
    }

    pub async fn get_cached_keys_raw_with_slug<F, Fut, T, I, K, S, E>(
        &self,
        namespace: &str,
        slug_namespace: Option<&str>,
        case_sensitive: bool,
        keys: &[I],
        closure: F,
    ) -> Result<std::collections::HashMap<K, T>, E>
    where
        F: FnOnce(Vec<I>) -> Fut,
        Fut: Future<Output = Result<DashMap<K, (Option<S>, T)>, E>>,
        E: From<Error>,
        T: Serialize + DeserializeOwned,
        I: Display + Hash + Eq + PartialEq + Clone + Debug,
        K: Display
            + Hash
            + Eq
            + PartialEq
            + Clone
            + DeserializeOwned
            + Serialize,
        S: Display + Clone + DeserializeOwned + Serialize + Debug,
    {
        self.cache
            .get_cached_keys_raw_with_slug(
                self,
                namespace,
                slug_namespace,
                case_sensitive,
                keys,
                closure,
            )
            .await
    }
}

impl ConnectionProvider for RedisPool {
    type Connection = RedisConnection;

    fn connect(
        &self,
    ) -> impl Future<Output = Result<Self::Connection, Error>> + Send {
        RedisPool::connect(self)
    }
}

impl RedisConnection {
    pub fn key(&self) -> &KeyBuilder {
        &self.key_builder
    }

    pub async fn set<D>(
        &mut self,
        key: &str,
        data: D,
        expiry: Option<i64>,
    ) -> Result<(), Error>
    where
        D: ToRedisArgs + Send + Sync + Debug,
    {
        commands::set(
            &mut self.inner,
            key,
            data,
            expiry.unwrap_or(self.settings.default_expiry),
        )
        .await
        .map_err(Error::from)
    }

    pub async fn set_serialized<D>(
        &mut self,
        key: &str,
        data: D,
        expiry: Option<i64>,
    ) -> Result<(), Error>
    where
        D: Serialize,
    {
        commands::set_serialized(
            &mut self.inner,
            key,
            data,
            expiry,
            &self.settings,
        )
        .await
        .map_err(Error::from)
    }

    pub async fn get(&mut self, key: &str) -> Result<Option<String>, Error> {
        commands::get(&mut self.inner, key)
            .await
            .map_err(Error::from)
    }

    pub async fn get_many(
        &mut self,
        keys: &[String],
    ) -> Result<Vec<Option<Vec<u8>>>, Error> {
        commands::get_many(&mut self.inner, keys)
            .await
            .map_err(Error::from)
    }

    pub async fn get_many_typed<R>(
        &mut self,
        keys: &[String],
    ) -> Result<Vec<Option<R>>, Error>
    where
        R: FromRedisValue,
    {
        commands::get_many_as(&mut self.inner, keys)
            .await
            .map_err(Error::from)
    }

    pub async fn get_deserialized<R>(
        &mut self,
        key: &str,
    ) -> Result<Option<R>, Error>
    where
        R: for<'a> serde::Deserialize<'a>,
    {
        commands::get_deserialized(&mut self.inner, key, &self.settings)
            .await
            .map_err(Error::from)
    }

    pub async fn get_many_deserialized<R>(
        &mut self,
        keys: &[String],
    ) -> Result<Vec<Option<R>>, Error>
    where
        R: for<'a> serde::Deserialize<'a>,
    {
        commands::get_many_deserialized(&mut self.inner, keys, &self.settings)
            .await
            .map_err(Error::from)
    }

    pub async fn delete(&mut self, key: &str) -> Result<(), Error> {
        commands::delete(&mut self.inner, key)
            .await
            .map_err(Error::from)
    }

    pub async fn delete_many(&mut self, keys: &[String]) -> Result<(), Error> {
        commands::delete_many(&mut self.inner, keys)
            .await
            .map_err(Error::from)
    }

    pub async fn lpush<D>(&mut self, key: &str, value: D) -> Result<(), Error>
    where
        D: ToRedisArgs + Send + Sync + Debug,
    {
        commands::lpush(&mut self.inner, key, value)
            .await
            .map_err(Error::from)
    }

    pub async fn incr(&mut self, key: &str) -> Result<Option<u64>, Error> {
        commands::incr(&mut self.inner, key)
            .await
            .map_err(Error::from)
    }
}

impl ConnectionLike for RedisConnection {
    fn req_packed_command<'a>(
        &'a mut self,
        command: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        self.inner.req_packed_command(command)
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        pipeline: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        self.inner.req_packed_commands(pipeline, offset, count)
    }

    fn get_db(&self) -> i64 {
        self.inner.get_db()
    }
}

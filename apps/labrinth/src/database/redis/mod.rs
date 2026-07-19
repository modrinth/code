use std::fmt::{Debug, Display};
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;

use dashmap::DashMap;
use prometheus::Registry;
use redis::ToRedisArgs;
use redis::aio::ConnectionLike;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::database::models::DatabaseError;
use crate::env::ENV;

mod blocking;
mod cache;
mod commands;
mod config;
mod connection;
mod key;
mod metrics;
mod pubsub;
mod util;

use cache::{CacheManager, CacheSettings, ConnectionProvider};
pub use cache::{Codec, EncodingFormat, RedisValue};
use config::RedisConfig;
pub use config::{RedisConnectionType, RedisMode};
use connection::RedisBackend;
pub use key::KeyBuilder;

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
    pub async fn new(meta_namespace: impl Into<Arc<str>>) -> Self {
        let config =
            RedisConfig::from_env().expect("invalid Redis configuration");
        let backend = RedisBackend::new(&config)
            .await
            .expect("failed to initialize Redis connections");
        let blocking = blocking::RedisBlockingPool::new(&config)
            .await
            .expect("failed to initialize blocking Redis connections");
        let key_builder = KeyBuilder::new(meta_namespace, config.mode());
        let cache = CacheManager::new(
            key_builder.clone(),
            CacheSettings {
                encoding_format: ENV.REDIS_ENCODING_FORMAT,
                compression_algorithm: ENV.REDIS_COMPRESSION_ALGORITHM,
                compression_level: ENV.REDIS_COMPRESSION_LEVEL,
                compression_threshold_bytes: ENV
                    .REDIS_COMPRESSION_THRESHOLD_BYTES,
                compression_min_savings_ratio: ENV
                    .REDIS_COMPRESSION_MIN_SAVINGS_RATIO,
            },
        );

        Self {
            backend,
            blocking,
            cache,
            config,
            key_builder,
        }
    }

    pub fn key(&self) -> &KeyBuilder {
        &self.key_builder
    }

    pub async fn connect(&self) -> Result<RedisConnection, DatabaseError> {
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

    pub async fn get_cached_keys<F, Fut, T, K>(
        &self,
        namespace: &str,
        keys: &[K],
        closure: F,
    ) -> Result<Vec<T>, DatabaseError>
    where
        F: FnOnce(Vec<K>) -> Fut,
        Fut: Future<Output = Result<DashMap<K, T>, DatabaseError>>,
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

    pub async fn get_cached_keys_raw<F, Fut, T, K>(
        &self,
        namespace: &str,
        keys: &[K],
        closure: F,
    ) -> Result<std::collections::HashMap<K, T>, DatabaseError>
    where
        F: FnOnce(Vec<K>) -> Fut,
        Fut: Future<Output = Result<DashMap<K, T>, DatabaseError>>,
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

    pub async fn get_cached_keys_with_slug<F, Fut, T, I, K, S>(
        &self,
        namespace: &str,
        slug_namespace: &str,
        case_sensitive: bool,
        keys: &[I],
        closure: F,
    ) -> Result<Vec<T>, DatabaseError>
    where
        F: FnOnce(Vec<I>) -> Fut,
        Fut: Future<Output = Result<DashMap<K, (Option<S>, T)>, DatabaseError>>,
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

    pub async fn get_cached_keys_raw_with_slug<F, Fut, T, I, K, S>(
        &self,
        namespace: &str,
        slug_namespace: Option<&str>,
        case_sensitive: bool,
        keys: &[I],
        closure: F,
    ) -> Result<std::collections::HashMap<K, T>, DatabaseError>
    where
        F: FnOnce(Vec<I>) -> Fut,
        Fut: Future<Output = Result<DashMap<K, (Option<S>, T)>, DatabaseError>>,
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
    ) -> impl Future<Output = Result<Self::Connection, DatabaseError>> + Send
    {
        RedisPool::connect(self)
    }
}

impl RedisConnection {
    pub fn keyspace(&self) -> &KeyBuilder {
        &self.key_builder
    }

    pub async fn set<D>(
        &mut self,
        key: &str,
        data: D,
        expiry: Option<i64>,
    ) -> Result<(), DatabaseError>
    where
        D: ToRedisArgs + Send + Sync + Debug,
    {
        commands::set(&mut self.inner, key, data, expiry).await
    }

    pub async fn set_serialized<D>(
        &mut self,
        key: &str,
        data: D,
        expiry: Option<i64>,
    ) -> Result<(), DatabaseError>
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
    }

    pub async fn get(
        &mut self,
        key: &str,
    ) -> Result<Option<String>, DatabaseError> {
        commands::get(&mut self.inner, key).await
    }

    pub async fn get_many(
        &mut self,
        keys: &[String],
    ) -> Result<Vec<Option<Vec<u8>>>, DatabaseError> {
        commands::get_many(&mut self.inner, keys).await
    }

    pub async fn get_deserialized<R>(
        &mut self,
        key: &str,
    ) -> Result<Option<R>, DatabaseError>
    where
        R: for<'a> serde::Deserialize<'a>,
    {
        commands::get_deserialized(&mut self.inner, key, &self.settings).await
    }

    pub async fn get_many_deserialized<R>(
        &mut self,
        keys: &[String],
    ) -> Result<Vec<Option<R>>, DatabaseError>
    where
        R: for<'a> serde::Deserialize<'a>,
    {
        commands::get_many_deserialized(&mut self.inner, keys, &self.settings)
            .await
    }

    pub async fn delete(&mut self, key: &str) -> Result<(), DatabaseError> {
        commands::delete(&mut self.inner, key).await
    }

    pub async fn delete_many(
        &mut self,
        keys: &[String],
    ) -> Result<(), DatabaseError> {
        commands::delete_many(&mut self.inner, keys).await
    }

    pub async fn lpush<D>(
        &mut self,
        key: &str,
        value: D,
    ) -> Result<(), DatabaseError>
    where
        D: ToRedisArgs + Send + Sync + Debug,
    {
        commands::lpush(&mut self.inner, key, value).await
    }

    pub async fn incr(
        &mut self,
        key: &str,
    ) -> Result<Option<u64>, DatabaseError> {
        commands::incr(&mut self.inner, key).await
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

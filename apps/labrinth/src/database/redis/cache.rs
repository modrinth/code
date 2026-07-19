use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::future::Future;
use std::hash::Hash;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use ariadne::ids::base62_impl::{parse_base62, to_base62};
use chrono::{TimeZone, Utc};
use dashmap::DashMap;
use futures::future::Either;
use futures::stream::{FuturesUnordered, StreamExt};
use redis::aio::ConnectionLike;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{Instrument, info_span};

use crate::database::models::DatabaseError;

use super::commands;
use super::key::KeyBuilder;
use super::util;

const ACTUAL_EXPIRY: i64 = 60 * 30;
const VERSION_DEFAULT_EXPIRY: i64 = 60 * 60 * 48;
const VERSION_ACTUAL_EXPIRY: i64 = 60 * 60 * 24;

pub(super) trait ConnectionProvider {
    type Connection: ConnectionLike;

    fn connect(
        &self,
    ) -> impl Future<Output = Result<Self::Connection, DatabaseError>> + Send;
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    Raw = 0,
    Lz4 = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingFormat {
    Json,
    Postcard,
}

#[derive(Debug, Error)]
#[error("invalid redis codec")]
pub struct InvalidCodec;

#[derive(Debug, Error)]
#[error("invalid redis encoding format")]
pub struct InvalidEncodingFormat;

impl TryFrom<u8> for Codec {
    type Error = InvalidCodec;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Raw),
            1 => Ok(Self::Lz4),
            _ => Err(InvalidCodec),
        }
    }
}

impl FromStr for Codec {
    type Err = InvalidCodec;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "lz4" => Ok(Self::Lz4),
            _ => Err(InvalidCodec),
        }
    }
}

impl FromStr for EncodingFormat {
    type Err = InvalidEncodingFormat;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "json" => Ok(Self::Json),
            "postcard" => Ok(Self::Postcard),
            _ => Err(InvalidEncodingFormat),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheSettings {
    pub encoding_format: EncodingFormat,
    pub compression_algorithm: Codec,
    pub compression_level: i32,
    pub compression_threshold_bytes: usize,
    pub compression_min_savings_ratio: f64,
}

impl CacheSettings {
    pub fn encode_value<T: Serialize>(
        &self,
        value: &T,
    ) -> Result<Vec<u8>, DatabaseError> {
        let mut value = match self.encoding_format {
            EncodingFormat::Json => serde_json::to_vec(value)?,
            EncodingFormat::Postcard => postcard::to_allocvec(value)?,
        };

        if self.compression_level > 0
            && self.compression_algorithm == Codec::Lz4
            && value.len() >= self.compression_threshold_bytes
        {
            let compressed = lz4_flex::block::compress_prepend_size(&value);
            let savings_ratio = value.len().saturating_sub(compressed.len())
                as f64
                / value.len().max(1) as f64
                * 100.0;

            if savings_ratio >= self.compression_min_savings_ratio {
                let mut encoded = Vec::with_capacity(compressed.len() + 1);
                encoded.push(Codec::Lz4 as u8);
                encoded.extend(compressed);
                return Ok(encoded);
            }
        }

        let mut encoded = Vec::with_capacity(value.len() + 1);
        encoded.push(Codec::Raw as u8);
        encoded.append(&mut value);
        Ok(encoded)
    }

    pub fn decode_value<T>(&self, value: &[u8]) -> Option<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        let (codec, value) = value.split_first()?;
        let value = match Codec::try_from(*codec).ok()? {
            Codec::Raw => Cow::Borrowed(value),
            Codec::Lz4 => Cow::Owned(
                lz4_flex::block::decompress_size_prepended(value).ok()?,
            ),
        };

        match self.encoding_format {
            EncodingFormat::Json => serde_json::from_slice(&value).ok(),
            EncodingFormat::Postcard => postcard::from_bytes(&value).ok(),
        }
    }
}

#[derive(Clone)]
pub struct CacheManager {
    key_builder: KeyBuilder,
    settings: CacheSettings,
    cache_list: Arc<DashMap<String, util::CacheSubscriber>>,
}

impl CacheManager {
    pub fn new(key_builder: KeyBuilder, settings: CacheSettings) -> Self {
        Self {
            key_builder,
            settings,
            cache_list: Arc::new(DashMap::with_capacity(2048)),
        }
    }

    pub fn settings(&self) -> &CacheSettings {
        &self.settings
    }

    #[tracing::instrument(skip(self, provider, closure))]
    pub async fn get_cached_keys<P, F, Fut, T, K>(
        &self,
        provider: &P,
        namespace: &str,
        keys: &[K],
        closure: F,
    ) -> Result<Vec<T>, DatabaseError>
    where
        P: ConnectionProvider,
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
        Ok(self
            .get_cached_keys_raw(provider, namespace, keys, closure)
            .await?
            .into_values()
            .collect())
    }

    #[tracing::instrument(skip(self, provider, closure))]
    pub async fn get_cached_keys_raw<P, F, Fut, T, K>(
        &self,
        provider: &P,
        namespace: &str,
        keys: &[K],
        closure: F,
    ) -> Result<HashMap<K, T>, DatabaseError>
    where
        P: ConnectionProvider,
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
        self.get_cached_keys_raw_with_slug(
            provider,
            namespace,
            None,
            false,
            keys,
            |ids| async move {
                Ok(closure(ids)
                    .await?
                    .into_iter()
                    .map(|(key, value)| (key, (None::<String>, value)))
                    .collect())
            },
        )
        .await
    }

    #[tracing::instrument(skip(self, provider, closure))]
    pub async fn get_cached_keys_with_slug<P, F, Fut, T, I, K, S>(
        &self,
        provider: &P,
        namespace: &str,
        slug_namespace: &str,
        case_sensitive: bool,
        keys: &[I],
        closure: F,
    ) -> Result<Vec<T>, DatabaseError>
    where
        P: ConnectionProvider,
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
        Ok(self
            .get_cached_keys_raw_with_slug(
                provider,
                namespace,
                Some(slug_namespace),
                case_sensitive,
                keys,
                closure,
            )
            .await?
            .into_values()
            .collect())
    }

    #[tracing::instrument(skip(self, provider, closure))]
    pub async fn get_cached_keys_raw_with_slug<P, F, Fut, T, I, K, S>(
        &self,
        provider: &P,
        namespace: &str,
        slug_namespace: Option<&str>,
        case_sensitive: bool,
        keys: &[I],
        closure: F,
    ) -> Result<HashMap<K, T>, DatabaseError>
    where
        P: ConnectionProvider,
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
        let ids = keys
            .iter()
            .map(|key| (key.to_string(), key.clone()))
            .collect::<DashMap<String, I>>();

        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let get_cached_values = |ids: DashMap<String, I>| {
            async move {
                let slug_ids = if let Some(slug_namespace) = slug_namespace {
                    async {
                        let keys = ids
                            .iter()
                            .map(|entry| {
                                let logical_key = if case_sensitive {
                                    entry.value().to_string()
                                } else {
                                    entry.value().to_string().to_lowercase()
                                };
                                self.key_builder
                                    .entity(slug_namespace, logical_key)
                            })
                            .collect::<Vec<_>>();
                        let mut connection = provider.connect().await?;
                        Ok::<_, DatabaseError>(
                            commands::get_many_strings(&mut connection, &keys)
                                .await?
                                .into_iter()
                                .flatten()
                                .collect::<Vec<_>>(),
                        )
                    }
                    .instrument(info_span!("get slug ids"))
                    .await?
                } else {
                    Vec::new()
                };

                let keys = ids
                    .iter()
                    .map(|entry| entry.value().to_string())
                    .chain(ids.iter().filter_map(|entry| {
                        parse_base62(&entry.value().to_string())
                            .ok()
                            .map(|value| value.to_string())
                    }))
                    .chain(slug_ids)
                    .map(|key| self.key_builder.entity(namespace, key))
                    .collect::<Vec<_>>();

                let mut connection = provider.connect().await?;
                let mut cached_values = HashMap::new();
                for value in commands::get_many(&mut connection, &keys).await? {
                    if let Some(value) = value.and_then(|value| {
                        self.settings
                            .decode_value::<RedisValue<T, K, S>>(&value)
                    }) {
                        cached_values.insert(value.key.clone(), value);
                    }
                }

                Ok::<_, DatabaseError>((cached_values, ids))
            }
            .instrument(info_span!("get cached values"))
        };

        let (default_expiry, actual_expiry) = cache_expiries(namespace);
        let current_time = Utc::now();
        let mut expired_values = HashMap::new();

        let (cached_values_raw, ids) = get_cached_values(ids).await?;
        let mut cached_values = cached_values_raw
            .into_iter()
            .filter_map(|(key, value)| {
                if Utc.timestamp_opt(value.iat + actual_expiry, 0).unwrap()
                    < current_time
                {
                    expired_values.insert(value.key.to_string(), value);
                    None
                } else {
                    remove_resolved_ids(&ids, &value);
                    Some((key, value))
                }
            })
            .collect::<HashMap<_, _>>();

        let subscribe_ids = DashMap::new();
        let mut cache_writers = HashMap::new();

        if !ids.is_empty() {
            let fetch_ids = ids
                .iter()
                .map(|entry| entry.key().clone())
                .collect::<Vec<_>>();

            for key in fetch_ids {
                let lock_key = self.key_builder.entity(
                    namespace,
                    if case_sensitive {
                        key.to_lowercase()
                    } else {
                        key.clone()
                    },
                );

                match self.acquire_lock(lock_key) {
                    Either::Left(sentinel) => {
                        cache_writers.insert(key, sentinel);
                    }
                    Either::Right(subscriber) => {
                        if let Some((key, raw_key)) = ids.remove(&key) {
                            if let Some(value) = expired_values.remove(&key) {
                                remove_resolved_ids(&ids, &value);
                                cached_values.insert(value.key.clone(), value);
                            } else {
                                subscribe_ids.insert(raw_key, subscriber);
                            }
                        }
                    }
                }
            }
        }

        let mut fetch_tasks = Vec::new();

        if !ids.is_empty() {
            fetch_tasks.push(Either::Left(async {
                let fetch_ids = ids
                    .iter()
                    .map(|entry| entry.value().clone())
                    .collect::<Vec<_>>();
                let values = closure(fetch_ids).await?;
                let mut return_values = HashMap::new();
                let mut connection = provider.connect().await?;

                for (key, (slug, value)) in values {
                    let value = RedisValue {
                        key: key.clone(),
                        iat: Utc::now().timestamp(),
                        val: value,
                        alias: slug.clone(),
                    };
                    let redis_key =
                        self.key_builder.entity(namespace, key.to_string());
                    commands::set(
                        &mut connection,
                        &redis_key,
                        self.settings.encode_value(&value)?,
                        Some(default_expiry),
                    )
                    .await?;

                    if let Some(slug) = slug {
                        ids.remove(&slug.to_string());
                        if let Some(slug_namespace) = slug_namespace {
                            let actual_slug = if case_sensitive {
                                slug.to_string()
                            } else {
                                slug.to_string().to_lowercase()
                            };
                            let slug_key = self
                                .key_builder
                                .entity(slug_namespace, actual_slug);
                            commands::set(
                                &mut connection,
                                &slug_key,
                                key.to_string(),
                                Some(default_expiry),
                            )
                            .await?;
                        }
                    }

                    let key_string = key.to_string();
                    ids.remove(&key_string);
                    if let Ok(value) = key_string.parse::<u64>() {
                        ids.remove(&to_base62(value));
                    }
                    return_values.insert(key, value);
                }

                drop(cache_writers);
                Result::<_, DatabaseError>::Ok(return_values)
            }));
        }

        if !subscribe_ids.is_empty() {
            fetch_tasks.push(Either::Right(async move {
                let mut futures = FuturesUnordered::new();
                let len = subscribe_ids.len();

                for (key, subscriber) in subscribe_ids {
                    futures.push(async move {
                        (
                            key,
                            subscriber
                                .wait_timeout(Duration::from_secs(5))
                                .await,
                        )
                    });
                }

                let fetch_ids = DashMap::with_capacity(len);
                while let Some((key, result)) = futures.next().await {
                    result?;
                    fetch_ids.insert(key.to_string(), key);
                }

                let (return_values, _) = get_cached_values(fetch_ids).await?;
                Ok(return_values)
            }));
        }

        if !fetch_tasks.is_empty() {
            for values in futures::future::try_join_all(fetch_tasks).await? {
                cached_values.extend(values);
            }
        }

        Ok(cached_values
            .into_iter()
            .map(|(key, value)| (key, value.val))
            .collect())
    }

    fn acquire_lock(
        &self,
        key: String,
    ) -> Either<LockSentinel, util::CacheSubscriber> {
        let mut writer = None;
        let subscriber =
            self.cache_list.entry(key.clone()).or_insert_with(|| {
                let (new_writer, subscriber) = util::cache();
                writer = Some(new_writer);
                subscriber
            });

        match writer {
            Some(writer) => Either::Left(LockSentinel {
                cache_list: self.cache_list.clone(),
                key,
                writer,
            }),
            None => Either::Right(subscriber.clone()),
        }
    }
}

fn cache_expiries(namespace: &str) -> (i64, i64) {
    match namespace
        .split_once(':')
        .map(|value| value.0)
        .unwrap_or(namespace)
    {
        "versions" | "versions_files" => {
            (VERSION_DEFAULT_EXPIRY, VERSION_ACTUAL_EXPIRY)
        }
        _ => (commands::DEFAULT_EXPIRY, ACTUAL_EXPIRY),
    }
}

fn remove_resolved_ids<I, T, K, S>(
    ids: &DashMap<String, I>,
    value: &RedisValue<T, K, S>,
) where
    K: Display,
    S: Display,
{
    let key = value.key.to_string();
    ids.remove(&key);
    if let Ok(value) = key.parse::<u64>() {
        ids.remove(&to_base62(value));
    }
    if let Some(alias) = &value.alias {
        ids.remove(&alias.to_string());
    }
}

struct LockSentinel {
    cache_list: Arc<DashMap<String, util::CacheSubscriber>>,
    key: String,
    writer: util::CacheWriter,
}

impl Drop for LockSentinel {
    fn drop(&mut self) {
        self.writer.write();
        self.cache_list.remove(&self.key);
    }
}

#[derive(Serialize, Deserialize)]
pub struct RedisValue<T, K, S> {
    key: K,
    alias: Option<S>,
    iat: i64,
    val: T,
}

impl<T, K, S> RedisValue<T, K, S> {
    pub fn value(&self) -> &T {
        &self.val
    }
}

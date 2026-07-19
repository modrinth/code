use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::future::Future;
use std::hash::Hash;
use std::str::FromStr;

use ariadne::ids::base62_impl::{parse_base62, to_base62};
use chrono::{TimeZone, Utc};
use dashmap::DashMap;
use futures::stream::{FuturesUnordered, StreamExt};
use redis::aio::ConnectionLike;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::{Instant, timeout_at};
use tracing::{Instrument, info_span};

use crate::database::models::DatabaseError;

use super::commands;
use super::config::CacheLockingStrategy;
use super::connection::RedisBackend;
use super::key::KeyBuilder;

mod locking;

use locking::{
    LockAcquisition, LockCoordinator, LockWaiter, OwnedLockGuard, WAIT_TIMEOUT,
    normalize_key,
};

const ACTUAL_EXPIRY: i64 = 60 * 30;
const FILL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
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
        }
    }
}

#[derive(Clone)]
pub struct CacheManager {
    key_builder: KeyBuilder,
    settings: CacheSettings,
    locking: LockCoordinator,
}

impl CacheManager {
    pub fn new(
        key_builder: KeyBuilder,
        settings: CacheSettings,
        locking_strategy: CacheLockingStrategy,
        backend: RedisBackend,
    ) -> Self {
        Self {
            locking: LockCoordinator::new(locking_strategy, backend),
            key_builder,
            settings,
        }
    }

    pub fn settings(&self) -> &CacheSettings {
        &self.settings
    }

    #[tracing::instrument(skip(self, provider, keys, closure))]
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

    #[tracing::instrument(skip(self, provider, keys, closure))]
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

    #[tracing::instrument(skip(self, provider, keys, closure))]
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

    #[tracing::instrument(skip(self, provider, keys, closure))]
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
            .map(|key| {
                (normalize_key(&key.to_string(), case_sensitive), key.clone())
            })
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
                                let logical_key = normalize_key(
                                    &entry.value().to_string(),
                                    case_sensitive,
                                );
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
        let mut expired_identities = HashMap::new();
        let deadline = Instant::now() + WAIT_TIMEOUT;

        let (cached_values_raw, ids) = get_cached_values(ids).await?;
        let mut cached_values = cached_values_raw
            .into_iter()
            .filter_map(|(key, value)| {
                if Utc.timestamp_opt(value.iat + actual_expiry, 0).unwrap()
                    < current_time
                {
                    let canonical_key = value.key.to_string();
                    for identity in value_identities(&value, case_sensitive) {
                        expired_identities
                            .insert(identity, canonical_key.clone());
                    }
                    expired_values.insert(canonical_key, value);
                    None
                } else {
                    remove_resolved_ids(&ids, &value, case_sensitive);
                    Some((key, value))
                }
            })
            .collect::<HashMap<_, _>>();

        let mut waiters = Vec::new();
        let mut owned_locks = HashMap::new();

        if !ids.is_empty() {
            let fetch_ids = ids
                .iter()
                .map(|entry| entry.key().clone())
                .collect::<Vec<_>>();
            for key in fetch_ids {
                if !ids.contains_key(&key) {
                    continue;
                }

                let lock_key = self.key_builder.entity(namespace, &key);
                let acquisition = match self.locking.acquire(lock_key).await {
                    Ok(acquisition) => acquisition,
                    Err(error) => {
                        release_owned_locks(owned_locks).await;
                        return Err(error);
                    }
                };

                match acquisition {
                    LockAcquisition::Owned(guard) => {
                        owned_locks.insert(key, guard);
                    }
                    LockAcquisition::Waiting(waiter) => {
                        if let Some(canonical_key) =
                            expired_identities.get(&key).cloned()
                            && let Some(value) =
                                expired_values.remove(&canonical_key)
                        {
                            remove_resolved_ids(&ids, &value, case_sensitive);
                            expired_identities.retain(|_, canonical| {
                                canonical != &canonical_key
                            });
                            cached_values.insert(value.key.clone(), value);
                        } else if let Some((_, raw_key)) = ids.remove(&key) {
                            waiters.push((raw_key, waiter));
                        }
                    }
                }
            }
        }

        let fill_result = if !ids.is_empty() {
            async {
                let fetch_ids = ids
                    .iter()
                    .map(|entry| entry.value().clone())
                    .collect::<Vec<_>>();

                let fill_deadline = Instant::now() + FILL_TIMEOUT;

                let values = timeout_at(fill_deadline, closure(fetch_ids))
                    .await
                    .map_err(|_| {
                        lock_timeout_error(
                            self.locking.strategy(),
                            0,
                            waiters.len(),
                        )
                    })??;

                let mut return_values = HashMap::new();
                let mut encoded_values = Vec::with_capacity(values.len());

                for (key, (slug, value)) in values {
                    let value = RedisValue {
                        key: key.clone(),
                        iat: Utc::now().timestamp(),
                        val: value,
                        alias: slug.clone(),
                    };
                    let encoded = self.settings.encode_value(&value)?;
                    encoded_values.push((key, slug, value, encoded));
                }

                let mut connection = provider.connect().await?;
                for (key, slug, _, encoded) in &encoded_values {
                    let redis_key =
                        self.key_builder.entity(namespace, key.to_string());
                    commands::set(
                        &mut connection,
                        &redis_key,
                        encoded,
                        Some(default_expiry),
                    )
                    .await?;
                    if let Some(slug) = slug
                        && let Some(slug_namespace) = slug_namespace
                    {
                        let canonical_key = key.to_string();
                        let actual_slug =
                            normalize_key(&slug.to_string(), case_sensitive);
                        let slug_key = self
                            .key_builder
                            .entity(slug_namespace, actual_slug);
                        commands::set(
                            &mut connection,
                            &slug_key,
                            canonical_key.as_bytes(),
                            Some(default_expiry),
                        )
                        .await?;
                    }
                }

                for (key, _, value, _) in encoded_values {
                    remove_resolved_ids(&ids, &value, case_sensitive);
                    return_values.insert(key, value);
                }

                Result::<_, DatabaseError>::Ok(return_values)
            }
            .await
        } else {
            Ok(HashMap::new())
        };

        release_owned_locks(owned_locks).await;

        let operation_result = match fill_result {
            Ok(mut values) => {
                if waiters.is_empty() {
                    Ok(values)
                } else {
                    match wait_for_locks(
                        self.locking.strategy(),
                        waiters,
                        deadline,
                    )
                    .await
                    {
                        Ok(released_ids) => {
                            let fetch_ids = released_ids
                                .into_iter()
                                .map(|key| {
                                    (
                                        normalize_key(
                                            &key.to_string(),
                                            case_sensitive,
                                        ),
                                        key,
                                    )
                                })
                                .collect::<DashMap<_, _>>();
                            match get_cached_values(fetch_ids).await {
                                Ok((released_values, _)) => {
                                    values.extend(released_values);
                                    Ok(values)
                                }
                                Err(error) => Err(error),
                            }
                        }
                        Err(error) => Err(error),
                    }
                }
            }
            Err(error) => Err(error),
        };
        cached_values.extend(operation_result?);

        Ok(cached_values
            .into_iter()
            .map(|(key, value)| (key, value.val))
            .collect())
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
    case_sensitive: bool,
) where
    K: Display,
    S: Display,
{
    for identity in value_identities(value, case_sensitive) {
        ids.remove(&normalize_key(&identity, case_sensitive));
    }
}

fn value_identities<T, K, S>(
    value: &RedisValue<T, K, S>,
    case_sensitive: bool,
) -> Vec<String>
where
    K: Display,
    S: Display,
{
    let mut identities = Vec::with_capacity(5);
    let canonical_key = value.key.to_string();

    push_identity(&mut identities, canonical_key.clone());
    if !case_sensitive {
        push_identity(&mut identities, canonical_key.to_lowercase());
    }

    if let Ok(decimal_id) = canonical_key.parse::<u64>() {
        let base62_id = to_base62(decimal_id);
        push_identity(&mut identities, base62_id.clone());

        if !case_sensitive {
            push_identity(&mut identities, base62_id.to_lowercase());
        }
    } else if let Ok(decimal_id) = parse_base62(&canonical_key) {
        push_identity(&mut identities, decimal_id.to_string());
    }

    if let Some(alias) = &value.alias {
        let alias = alias.to_string();
        push_identity(&mut identities, alias.clone());
        if !case_sensitive {
            push_identity(&mut identities, alias.to_lowercase());
        }
    }

    identities
}

fn push_identity(identities: &mut Vec<String>, identity: String) {
    if !identities.contains(&identity) {
        identities.push(identity);
    }
}

async fn wait_for_locks<I>(
    strategy: CacheLockingStrategy,
    waiters: Vec<(I, LockWaiter)>,
    deadline: Instant,
) -> Result<Vec<I>, DatabaseError> {
    let total = waiters.len();
    let mut released = Vec::with_capacity(total);
    let mut futures = FuturesUnordered::new();
    for (key, waiter) in waiters {
        futures.push(async move {
            let result = waiter.wait(deadline).await;
            (key, result)
        });
    }

    while let Some((key, result)) = futures.next().await {
        match result {
            Ok(()) => {
                released.push(key);
            }
            Err(error)
                if is_lock_timeout(&error) || Instant::now() >= deadline =>
            {
                return Err(lock_timeout_error(
                    strategy,
                    released.len(),
                    total,
                ));
            }
            Err(error) => return Err(error),
        }
    }
    Ok(released)
}

fn is_lock_timeout(error: &DatabaseError) -> bool {
    matches!(
        error,
        DatabaseError::CacheTimeout { .. }
            | DatabaseError::LocalCacheTimeout { .. }
    )
}

fn lock_timeout_error(
    strategy: CacheLockingStrategy,
    locks_released: usize,
    locks_waiting: usize,
) -> DatabaseError {
    match strategy {
        CacheLockingStrategy::Local => DatabaseError::LocalCacheTimeout {
            released: locks_released,
            total: locks_waiting,
        },
        CacheLockingStrategy::Distributed => DatabaseError::CacheTimeout {
            locks_released,
            locks_waiting,
            time_spent_pool_wait_ms: 0,
            time_spent_total_ms: WAIT_TIMEOUT.as_millis() as u64,
        },
    }
}

async fn release_owned_locks(owned_locks: HashMap<String, OwnedLockGuard>) {
    for guard in owned_locks.into_values() {
        if let Err(error) = guard.release().await {
            tracing::warn!(
                error = ?error,
                "failed to explicitly release cache lock",
            );
        }
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

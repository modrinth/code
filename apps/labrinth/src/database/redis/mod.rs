use super::models::DatabaseError;
use ariadne::ids::base62_impl::{parse_base62, to_base62};
use chrono::{TimeZone, Utc};
use dashmap::DashMap;
use deadpool_redis::{Config, Runtime};
use futures::future::Either;
use futures::stream::{FuturesUnordered, StreamExt};
use prometheus::{IntGauge, Registry};
use redis::ToRedisArgs;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::future::Future;
use std::hash::Hash;
use std::time::Duration;
use tracing::{Instrument, info_span};
use util::{cmd, redis_pipe};

pub mod util;

const DEFAULT_EXPIRY: i64 = 60 * 60 * 12; // 12 hours
const ACTUAL_EXPIRY: i64 = 60 * 30; // 30 minutes

#[derive(Clone)]
pub struct RedisPool {
    pub url: String,
    pub pool: util::InstrumentedPool,
    cache_list: DashMap<String, util::CacheSubscriber>,
    meta_namespace: String,
}

pub struct RedisConnection {
    pub connection: deadpool_redis::Connection,
    meta_namespace: String,
}

impl RedisPool {
    // initiate a new redis pool
    // testing pool uses a hashmap to mimic redis behaviour for very small data sizes (ie: tests)
    // PANICS: production pool will panic if redis url is not set
    pub fn new(meta_namespace: Option<String>) -> Self {
        let wait_timeout =
            dotenvy::var("REDIS_WAIT_TIMEOUT_MS").ok().map_or_else(
                || Duration::from_millis(15000),
                |x| {
                    Duration::from_millis(
                        x.parse::<u64>().expect(
                            "REDIS_WAIT_TIMEOUT_MS must be a valid u64",
                        ),
                    )
                },
            );

        let url = dotenvy::var("REDIS_URL").expect("Redis URL not set");
        let pool = Config::from_url(url.clone())
            .builder()
            .expect("Error building Redis pool")
            .max_size(
                dotenvy::var("REDIS_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|x| x.parse().ok())
                    .unwrap_or(10000),
            )
            .wait_timeout(Some(wait_timeout))
            .runtime(Runtime::Tokio1)
            .build()
            .expect("Redis connection failed");

        let pool = RedisPool {
            url,
            pool: util::InstrumentedPool::new(pool),
            cache_list: DashMap::with_capacity(2048),
            meta_namespace: meta_namespace.unwrap_or("".to_string()),
        };

        let interval = Duration::from_secs(30);
        let max_age = Duration::from_secs(5 * 60); // 5 minutes
        let pool_ref = pool.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                pool_ref
                    .pool
                    .retain(|_, metrics| metrics.last_used() < max_age);
            }
        });

        pool
    }

    pub async fn register_and_set_metrics(
        &self,
        registry: &Registry,
    ) -> Result<(), prometheus::Error> {
        let redis_max_size = IntGauge::new(
            "labrinth_redis_pool_max_size",
            "Maximum size of Redis pool",
        )?;
        let redis_size = IntGauge::new(
            "labrinth_redis_pool_size",
            "Current size of Redis pool",
        )?;
        let redis_available = IntGauge::new(
            "labrinth_redis_pool_available",
            "Available connections in Redis pool",
        )?;
        let redis_waiting = IntGauge::new(
            "labrinth_redis_pool_waiting",
            "Number of futures waiting for a Redis connection",
        )?;

        registry.register(Box::new(redis_max_size.clone()))?;
        registry.register(Box::new(redis_size.clone()))?;
        registry.register(Box::new(redis_available.clone()))?;
        registry.register(Box::new(redis_waiting.clone()))?;

        let redis_pool_ref = self.pool.clone();
        tokio::spawn(async move {
            loop {
                let status = redis_pool_ref.status();
                redis_max_size.set(status.max_size as i64);
                redis_size.set(status.size as i64);
                redis_available.set(status.available as i64);
                redis_waiting.set(status.waiting as i64);

                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn connect(&self) -> Result<RedisConnection, DatabaseError> {
        Ok(RedisConnection {
            connection: self.pool.get().await?,
            meta_namespace: self.meta_namespace.clone(),
        })
    }

    #[tracing::instrument(skip(self, closure))]
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
        Ok(self
            .get_cached_keys_raw(namespace, keys, closure)
            .await?
            .into_iter()
            .map(|x| x.1)
            .collect())
    }

    #[tracing::instrument(skip(self, closure))]
    pub async fn get_cached_keys_raw<F, Fut, T, K>(
        &self,
        namespace: &str,
        keys: &[K],
        closure: F,
    ) -> Result<HashMap<K, T>, DatabaseError>
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
        self.get_cached_keys_raw_with_slug(
            namespace,
            None,
            false,
            keys,
            |ids| async move {
                Ok(closure(ids)
                    .await?
                    .into_iter()
                    .map(|(key, val)| (key, (None::<String>, val)))
                    .collect())
            },
        )
        .await
    }

    #[tracing::instrument(skip(self, closure))]
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
        Ok(self
            .get_cached_keys_raw_with_slug(
                namespace,
                Some(slug_namespace),
                case_sensitive,
                keys,
                closure,
            )
            .await?
            .into_iter()
            .map(|x| x.1)
            .collect())
    }

    #[tracing::instrument(skip(self, closure))]
    pub async fn get_cached_keys_raw_with_slug<F, Fut, T, I, K, S>(
        &self,
        namespace: &str,
        slug_namespace: Option<&str>,
        case_sensitive: bool,
        keys: &[I],
        closure: F,
    ) -> Result<HashMap<K, T>, DatabaseError>
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
        let ids = keys
            .iter()
            .map(|x| (x.to_string(), x.clone()))
            .collect::<DashMap<String, I>>();

        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let get_cached_values = |ids: DashMap<String, I>| {
            async move {
                let slug_ids = if let Some(slug_namespace) = slug_namespace {
                    async {
                        let mut connection = self.pool.get().await?;

                        let args = ids
                            .iter()
                            .map(|x| {
                                format!(
                                    "{}_{slug_namespace}:{}",
                                    self.meta_namespace,
                                    if case_sensitive {
                                        x.value().to_string()
                                    } else {
                                        x.value().to_string().to_lowercase()
                                    }
                                )
                            })
                            .collect::<Vec<_>>();

                        let v = cmd("MGET")
                            .arg(&args)
                            .query_async::<Vec<Option<String>>>(&mut connection)
                            .await?
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>();
                        Ok::<_, DatabaseError>(v)
                    }
                    .instrument(info_span!("get slug ids"))
                    .await?
                } else {
                    Vec::new()
                };

                let mut connection = self.pool.get().await?;
                let args = ids
                    .iter()
                    .map(|x| x.value().to_string())
                    .chain(ids.iter().filter_map(|x| {
                        parse_base62(&x.value().to_string())
                            .ok()
                            .map(|x| x.to_string())
                    }))
                    .chain(slug_ids)
                    .map(|x| format!("{}_{namespace}:{x}", self.meta_namespace))
                    .collect::<Vec<_>>();

                let cached_values = cmd("MGET")
                    .arg(&args)
                    .query_async::<Vec<Option<String>>>(&mut connection)
                    .await?
                    .into_iter()
                    .filter_map(|x| {
                        x.and_then(|val| {
                            serde_json::from_str::<RedisValue<T, K, S>>(&val)
                                .ok()
                        })
                        .map(|val| (val.key.clone(), val))
                    })
                    .collect::<HashMap<_, _>>();

                Ok::<_, DatabaseError>((cached_values, ids))
            }
            .instrument(info_span!("get cached values"))
        };

        let current_time = Utc::now();
        let mut expired_values = HashMap::new();

        let (cached_values_raw, ids) = get_cached_values(ids).await?;
        let mut cached_values = cached_values_raw
            .into_iter()
            .filter_map(|(key, val)| {
                if Utc.timestamp_opt(val.iat + ACTUAL_EXPIRY, 0).unwrap()
                    < current_time
                {
                    expired_values.insert(val.key.to_string(), val);

                    None
                } else {
                    let key_str = val.key.to_string();
                    ids.remove(&key_str);

                    if let Ok(value) = key_str.parse::<u64>() {
                        let base62 = to_base62(value);
                        ids.remove(&base62);
                    }

                    if let Some(ref alias) = val.alias {
                        ids.remove(&alias.to_string());
                    }

                    Some((key, val))
                }
            })
            .collect::<HashMap<_, _>>();

        let subscribe_ids = DashMap::new();
        let mut cache_writers = HashMap::new();

        if !ids.is_empty() {
            let fetch_ids =
                ids.iter().map(|x| x.key().clone()).collect::<Vec<_>>();

            fetch_ids.into_iter().for_each(|key| {
                let ns_key_value = if case_sensitive {
                    key.to_lowercase()
                } else {
                    key.clone()
                };
                let namespaced_key = format!(
                    "{}_{namespace}:{ns_key_value}",
                    self.meta_namespace,
                );
                let either = self.acquire_lock(namespaced_key);

                match either {
                    Either::Left(sentinel) => {
                        cache_writers.insert(key, sentinel);
                    }

                    Either::Right(subscriber) => {
                        if let Some((key, raw_key)) = ids.remove(&key) {
                            if let Some(val) = expired_values.remove(&key) {
                                if let Some(ref alias) = val.alias {
                                    ids.remove(&alias.to_string());
                                }

                                if let Ok(value) =
                                    val.key.to_string().parse::<u64>()
                                {
                                    let base62 = to_base62(value);
                                    ids.remove(&base62);
                                }

                                cached_values.insert(val.key.clone(), val);
                            } else {
                                subscribe_ids.insert(raw_key, subscriber);
                            }
                        }
                    }
                }
            });
        }

        let mut fetch_tasks = Vec::new();

        if !ids.is_empty() {
            fetch_tasks.push(Either::Left(async {
                let fetch_ids =
                    ids.iter().map(|x| x.value().clone()).collect::<Vec<_>>();

                let vals = closure(fetch_ids).await?;
                let mut return_values = HashMap::new();

                let mut pipe = redis_pipe();
                // Doesn't need to be atomic

                if !vals.is_empty() {
                    for (key, (slug, value)) in vals {
                        let value = RedisValue {
                            key: key.clone(),
                            iat: Utc::now().timestamp(),
                            val: value,
                            alias: slug.clone(),
                        };

                        pipe.set_ex(
                            format!(
                                "{}_{namespace}:{key}",
                                self.meta_namespace
                            ),
                            serde_json::to_string(&value)?,
                            DEFAULT_EXPIRY as u64,
                        );

                        if let Some(slug) = slug {
                            ids.remove(&slug.to_string());

                            if let Some(slug_namespace) = slug_namespace {
                                let actual_slug = if case_sensitive {
                                    slug.to_string()
                                } else {
                                    slug.to_string().to_lowercase()
                                };

                                pipe.set_ex(
                                    format!(
                                        "{}_{slug_namespace}:{}",
                                        self.meta_namespace, actual_slug
                                    ),
                                    key.to_string(),
                                    DEFAULT_EXPIRY as u64,
                                );

                                /*
                                if let Some(_sentinel) =
                                    cache_writers.remove(&actual_slug)
                                {
                                    // drop it
                                }
                                */
                            }
                        }

                        let key_str = key.to_string();
                        ids.remove(&key_str);

                        /*
                        if let Some(_sentinel) = cache_writers.remove(&key_str)
                        {
                            // drop it
                        }
                        */

                        if let Ok(value) = key_str.parse::<u64>() {
                            let base62 = to_base62(value);
                            ids.remove(&base62);

                            /*
                            if let Some(_sentinel) =
                                cache_writers.remove(&base62)
                            {
                                // drop it
                            }
                            */
                        }

                        return_values.insert(key, value);
                    }
                }

                let mut connection = self.pool.get().await?;
                pipe.query_async::<()>(&mut connection).await?;

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
            for map in futures::future::try_join_all(fetch_tasks).await? {
                for (key, value) in map {
                    cached_values.insert(key, value);
                }
            }
        }

        Ok(cached_values.into_iter().map(|x| (x.0, x.1.val)).collect())
    }

    /// Acquire or create a cache lock onto the given key.
    fn acquire_lock(
        &self,
        key: String,
    ) -> Either<LockSentinel<'_>, util::CacheSubscriber> {
        let mut out_writer = None;
        let subscriber =
            self.cache_list.entry(key.clone()).or_insert_with(|| {
                let (writer, subscriber) = util::cache();
                out_writer = Some(writer);
                subscriber
            });

        match out_writer {
            Some(writer) => Either::Left(LockSentinel {
                pool: self,
                key,
                writer,
            }),
            None => Either::Right(subscriber.clone()),
        }
    }
}

struct LockSentinel<'a> {
    pool: &'a RedisPool,
    key: String,
    writer: util::CacheWriter,
}

impl<'a> Drop for LockSentinel<'a> {
    fn drop(&mut self) {
        self.writer.write();
        self.pool.cache_list.remove(&self.key);
    }
}

impl RedisConnection {
    #[tracing::instrument(skip(self))]
    pub async fn set(
        &mut self,
        namespace: &str,
        id: &str,
        data: &str,
        expiry: Option<i64>,
    ) -> Result<(), DatabaseError> {
        let mut cmd = cmd("SET");
        redis_args(
            &mut cmd,
            vec![
                format!("{}_{}:{}", self.meta_namespace, namespace, id),
                data.to_string(),
                "EX".to_string(),
                expiry.unwrap_or(DEFAULT_EXPIRY).to_string(),
            ]
            .as_slice(),
        );
        redis_execute::<()>(&mut cmd, &mut self.connection).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, id, data))]
    pub async fn set_serialized_to_json<Id, D>(
        &mut self,
        namespace: &str,
        id: Id,
        data: D,
        expiry: Option<i64>,
    ) -> Result<(), DatabaseError>
    where
        Id: Display,
        D: serde::Serialize,
    {
        self.set(
            namespace,
            &id.to_string(),
            &serde_json::to_string(&data)?,
            expiry,
        )
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get(
        &mut self,
        namespace: &str,
        id: &str,
    ) -> Result<Option<String>, DatabaseError> {
        let mut cmd = cmd("GET");
        redis_args(
            &mut cmd,
            vec![format!("{}_{}:{}", self.meta_namespace, namespace, id)]
                .as_slice(),
        );
        let res = redis_execute(&mut cmd, &mut self.connection).await?;
        Ok(res)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_many(
        &mut self,
        namespace: &str,
        ids: &[String],
    ) -> Result<Vec<Option<String>>, DatabaseError> {
        let mut cmd = cmd("MGET");
        redis_args(
            &mut cmd,
            ids.iter()
                .map(|x| format!("{}_{}:{}", self.meta_namespace, namespace, x))
                .collect::<Vec<_>>()
                .as_slice(),
        );
        let res = redis_execute(&mut cmd, &mut self.connection).await?;
        Ok(res)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_deserialized_from_json<R>(
        &mut self,
        namespace: &str,
        id: &str,
    ) -> Result<Option<R>, DatabaseError>
    where
        R: for<'a> serde::Deserialize<'a>,
    {
        Ok(self
            .get(namespace, id)
            .await?
            .and_then(|x| serde_json::from_str(&x).ok()))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_many_deserialized_from_json<R>(
        &mut self,
        namespace: &str,
        ids: &[String],
    ) -> Result<Vec<Option<R>>, DatabaseError>
    where
        R: for<'a> serde::Deserialize<'a>,
    {
        Ok(self
            .get_many(namespace, ids)
            .await?
            .into_iter()
            .map(|x| x.and_then(|val| serde_json::from_str::<R>(&val).ok()))
            .collect::<Vec<_>>())
    }

    #[tracing::instrument(skip(self, id))]
    pub async fn delete<T1>(
        &mut self,
        namespace: &str,
        id: T1,
    ) -> Result<(), DatabaseError>
    where
        T1: Display,
    {
        let mut cmd = cmd("DEL");
        redis_args(
            &mut cmd,
            vec![format!("{}_{}:{}", self.meta_namespace, namespace, id)]
                .as_slice(),
        );
        redis_execute::<()>(&mut cmd, &mut self.connection).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, iter))]
    pub async fn delete_many(
        &mut self,
        iter: impl IntoIterator<Item = (&str, Option<String>)>,
    ) -> Result<(), DatabaseError> {
        let mut cmd = cmd("DEL");
        let mut any = false;
        for (namespace, id) in iter {
            if let Some(id) = id {
                redis_args(
                    &mut cmd,
                    [format!("{}_{}:{}", self.meta_namespace, namespace, id)]
                        .as_slice(),
                );
                any = true;
            }
        }

        if any {
            redis_execute::<()>(&mut cmd, &mut self.connection).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, value))]
    pub async fn lpush(
        &mut self,
        namespace: &str,
        key: &str,
        value: impl ToRedisArgs + Send + Sync + Debug,
    ) -> Result<(), DatabaseError> {
        let key = format!("{}_{namespace}:{key}", self.meta_namespace);
        cmd("LPUSH")
            .arg(key)
            .arg(value)
            .query_async::<()>(&mut self.connection)
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn brpop(
        &mut self,
        namespace: &str,
        key: &str,
        timeout: Option<f64>,
    ) -> Result<Option<[String; 2]>, DatabaseError> {
        let key = format!("{}_{namespace}:{key}", self.meta_namespace);
        // a timeout of 0 is infinite
        let timeout = timeout.unwrap_or(0.0);
        let values = cmd("BRPOP")
            .arg(key)
            .arg(timeout)
            .query_async(&mut self.connection)
            .await?;
        Ok(values)
    }
}

#[derive(Serialize, Deserialize)]
pub struct RedisValue<T, K, S> {
    key: K,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<S>,
    iat: i64,
    val: T,
}

pub fn redis_args(cmd: &mut util::InstrumentedCmd, args: &[String]) {
    for arg in args {
        cmd.arg(arg);
    }
}

pub async fn redis_execute<T>(
    cmd: &mut util::InstrumentedCmd,
    redis: &mut deadpool_redis::Connection,
) -> Result<T, deadpool_redis::PoolError>
where
    T: redis::FromRedisValue,
{
    let res = cmd.query_async::<T>(redis).await?;
    Ok(res)
}

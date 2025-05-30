use super::models::DatabaseError;
use ariadne::ids::base62_impl::{parse_base62, to_base62};
use chrono::{TimeZone, Utc};
use dashmap::DashMap;
use deadpool_redis::{Config, Runtime};
use prometheus::{IntGauge, Registry};
use redis::{Cmd, ExistenceCheck, SetExpiry, SetOptions, cmd};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::time::Duration;

const DEFAULT_EXPIRY: i64 = 60 * 60 * 12; // 12 hours
const ACTUAL_EXPIRY: i64 = 60 * 30; // 30 minutes

#[derive(Clone)]
pub struct RedisPool {
    pub url: String,
    pub pool: deadpool_redis::Pool,
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
            .runtime(Runtime::Tokio1)
            .build()
            .expect("Redis connection failed");

        RedisPool {
            url,
            pool,
            meta_namespace: meta_namespace.unwrap_or("".to_string()),
        }
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

    pub async fn connect(&self) -> Result<RedisConnection, DatabaseError> {
        Ok(RedisConnection {
            connection: self.pool.get().await?,
            meta_namespace: self.meta_namespace.clone(),
        })
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
        Ok(self
            .get_cached_keys_raw(namespace, keys, closure)
            .await?
            .into_iter()
            .map(|x| x.1)
            .collect())
    }

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

        let get_cached_values = |ids: DashMap<String, I>| async move {
            let slug_ids = if let Some(slug_namespace) = slug_namespace {
                let mut connection = self.pool.get().await?;
                cmd("MGET")
                    .arg(
                        ids.iter()
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
                            .collect::<Vec<_>>(),
                    )
                    .query_async::<Vec<Option<String>>>(&mut connection)
                    .await?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };

            let mut connection = self.pool.get().await?;
            let cached_values = cmd("MGET")
                .arg(
                    ids.iter()
                        .map(|x| x.value().to_string())
                        .chain(ids.iter().filter_map(|x| {
                            parse_base62(&x.value().to_string())
                                .ok()
                                .map(|x| x.to_string())
                        }))
                        .chain(slug_ids)
                        .map(|x| {
                            format!("{}_{namespace}:{x}", self.meta_namespace)
                        })
                        .collect::<Vec<_>>(),
                )
                .query_async::<Vec<Option<String>>>(&mut connection)
                .await?
                .into_iter()
                .filter_map(|x| {
                    x.and_then(|val| {
                        serde_json::from_str::<RedisValue<T, K, S>>(&val).ok()
                    })
                    .map(|val| (val.key.clone(), val))
                })
                .collect::<HashMap<_, _>>();

            Ok::<_, DatabaseError>((cached_values, ids))
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

        if !ids.is_empty() {
            let mut pipe = redis::pipe();

            let fetch_ids =
                ids.iter().map(|x| x.key().clone()).collect::<Vec<_>>();

            fetch_ids.iter().for_each(|key| {
                pipe.atomic().set_options(
                    // We store locks in lowercase because they are case insensitive
                    format!(
                        "{}_{namespace}:{}/lock",
                        self.meta_namespace,
                        key.to_lowercase()
                    ),
                    100,
                    SetOptions::default()
                        .get(true)
                        .conditional_set(ExistenceCheck::NX)
                        .with_expiration(SetExpiry::EX(60)),
                );
            });
            let results = {
                let mut connection = self.pool.get().await?;

                pipe.query_async::<Vec<Option<i32>>>(&mut connection)
                    .await?
            };

            for (idx, key) in fetch_ids.into_iter().enumerate() {
                if let Some(locked) = results.get(idx) {
                    if locked.is_none() {
                        continue;
                    }
                }

                if let Some((key, raw_key)) = ids.remove(&key) {
                    if let Some(val) = expired_values.remove(&key) {
                        if let Some(ref alias) = val.alias {
                            ids.remove(&alias.to_string());
                        }

                        if let Ok(value) = val.key.to_string().parse::<u64>() {
                            let base62 = to_base62(value);
                            ids.remove(&base62);
                        }

                        cached_values.insert(val.key.clone(), val);
                    } else {
                        subscribe_ids.insert(key, raw_key);
                    }
                }
            }
        }

        #[allow(clippy::type_complexity)]
        let mut fetch_tasks: Vec<
            Pin<
                Box<
                    dyn Future<
                        Output = Result<
                            HashMap<K, RedisValue<T, K, S>>,
                            DatabaseError,
                        >,
                    >,
                >,
            >,
        > = Vec::new();

        if !ids.is_empty() {
            fetch_tasks.push(Box::pin(async {
                let fetch_ids =
                    ids.iter().map(|x| x.value().clone()).collect::<Vec<_>>();

                let vals = closure(fetch_ids).await?;
                let mut return_values = HashMap::new();

                let mut pipe = redis::pipe();
                if !vals.is_empty() {
                    for (key, (slug, value)) in vals {
                        let value = RedisValue {
                            key: key.clone(),
                            iat: Utc::now().timestamp(),
                            val: value,
                            alias: slug.clone(),
                        };

                        pipe.atomic().set_ex(
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

                                pipe.atomic().set_ex(
                                    format!(
                                        "{}_{slug_namespace}:{}",
                                        self.meta_namespace, actual_slug
                                    ),
                                    key.to_string(),
                                    DEFAULT_EXPIRY as u64,
                                );

                                pipe.atomic().del(format!(
                                    "{}_{namespace}:{}/lock",
                                    // Locks are stored in lowercase
                                    self.meta_namespace,
                                    actual_slug.to_lowercase()
                                ));
                            }
                        }

                        let key_str = key.to_string();
                        ids.remove(&key_str);

                        if let Ok(value) = key_str.parse::<u64>() {
                            let base62 = to_base62(value);
                            ids.remove(&base62);

                            pipe.atomic().del(format!(
                                "{}_{namespace}:{}/lock",
                                self.meta_namespace,
                                // Locks are stored in lowercase
                                base62.to_lowercase()
                            ));
                        }

                        pipe.atomic().del(format!(
                            "{}_{namespace}:{key}/lock",
                            self.meta_namespace
                        ));

                        return_values.insert(key, value);
                    }
                }

                for (key, _) in ids {
                    pipe.atomic().del(format!(
                        "{}_{namespace}:{}/lock",
                        self.meta_namespace,
                        key.to_lowercase()
                    ));
                    pipe.atomic().del(format!(
                        "{}_{namespace}:{key}/lock",
                        self.meta_namespace
                    ));
                }

                let mut connection = self.pool.get().await?;
                pipe.query_async::<()>(&mut connection).await?;

                Ok(return_values)
            }));
        }

        if !subscribe_ids.is_empty() {
            fetch_tasks.push(Box::pin(async {
                let mut interval =
                    tokio::time::interval(Duration::from_millis(100));
                let start = Utc::now();
                loop {
                    let results = {
                        let mut connection = self.pool.get().await?;
                        cmd("MGET")
                            .arg(
                                subscribe_ids
                                    .iter()
                                    .map(|x| {
                                        format!(
                                            "{}_{namespace}:{}/lock",
                                            self.meta_namespace,
                                            // We lowercase key because locks are stored in lowercase
                                            x.key().to_lowercase()
                                        )
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .query_async::<Vec<Option<String>>>(&mut connection)
                            .await?
                    };

                    if results.into_iter().all(|x| x.is_none()) {
                        break;
                    }

                    if (Utc::now() - start) > chrono::Duration::seconds(5) {
                        return Err(DatabaseError::CacheTimeout);
                    }

                    interval.tick().await;
                }

                let (return_values, _) =
                    get_cached_values(subscribe_ids).await?;

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
}

impl RedisConnection {
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
}

#[derive(Serialize, Deserialize)]
pub struct RedisValue<T, K, S> {
    key: K,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<S>,
    iat: i64,
    val: T,
}

pub fn redis_args(cmd: &mut Cmd, args: &[String]) {
    for arg in args {
        cmd.arg(arg);
    }
}

pub async fn redis_execute<T>(
    cmd: &mut Cmd,
    redis: &mut deadpool_redis::Connection,
) -> Result<T, deadpool_redis::PoolError>
where
    T: redis::FromRedisValue,
{
    let res = cmd.query_async::<T>(redis).await?;
    Ok(res)
}

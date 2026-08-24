use std::fmt::Debug;

use eyre::{Result, WrapErr};
use redis::aio::ConnectionLike;
use redis::{FromRedisValue, ToRedisArgs};

use super::cache::CacheSettings;
use super::connection::RoutableConnection;
use super::routing::primary_mget_routing;
use super::util::cmd;

pub const MGET_CHUNK_SIZE: usize = 32;

#[tracing::instrument(skip_all)]
pub async fn set<C, D>(
    connection: &mut C,
    key: &str,
    data: D,
    expiry: i64,
) -> Result<()>
where
    C: ConnectionLike,
    D: ToRedisArgs + Send + Sync + Debug,
{
    cmd("SET")
        .arg(key)
        .arg(data)
        .arg("EX")
        .arg(expiry)
        .query_async::<()>(connection)
        .await
        .wrap_err("writing to Redis")?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn set_serialized<C, D>(
    connection: &mut C,
    key: &str,
    data: D,
    expiry: Option<i64>,
    settings: &CacheSettings,
) -> Result<()>
where
    C: ConnectionLike,
    D: serde::Serialize,
{
    set(
        connection,
        key,
        settings
            .encode_value(&data)
            .wrap_err("serializing Redis value")?,
        expiry.unwrap_or(settings.default_expiry),
    )
    .await
}

#[tracing::instrument(skip_all)]
pub async fn get<C>(connection: &mut C, key: &str) -> Result<Option<String>>
where
    C: ConnectionLike,
{
    cmd("GET")
        .arg(key)
        .query_async(connection)
        .await
        .wrap_err("fetching from Redis")
}

/// Issues ordinary `MGET` commands in bounded chunks. Cluster routing and
/// result ordering remain redis-rs's responsibility; multiple chunks are not
/// an atomic snapshot.
#[tracing::instrument(skip_all)]
pub async fn get_many<C>(
    connection: &mut C,
    keys: &[String],
) -> Result<Vec<Option<Vec<u8>>>>
where
    C: ConnectionLike,
{
    get_many_as(connection, keys).await
}

#[tracing::instrument(skip_all)]
pub async fn get_many_strings<C>(
    connection: &mut C,
    keys: &[String],
) -> Result<Vec<Option<String>>>
where
    C: ConnectionLike,
{
    get_many_as(connection, keys).await
}

pub(super) async fn get_many_primary<C>(
    connection: &mut C,
    keys: &[String],
) -> Result<Vec<Option<Vec<u8>>>>
where
    C: RoutableConnection,
{
    get_many_primary_as(connection, keys).await
}

pub(super) async fn get_many_strings_primary<C>(
    connection: &mut C,
    keys: &[String],
) -> Result<Vec<Option<String>>>
where
    C: RoutableConnection,
{
    get_many_primary_as(connection, keys).await
}

pub(super) async fn get_many_as<C, T>(
    connection: &mut C,
    keys: &[String],
) -> Result<Vec<Option<T>>>
where
    C: ConnectionLike,
    T: FromRedisValue,
{
    let mut values = Vec::with_capacity(keys.len());
    for chunk in keys.chunks(MGET_CHUNK_SIZE) {
        let part = cmd("MGET")
            .arg(chunk)
            .query_async::<Vec<Option<T>>>(connection)
            .await
            .wrap_err("fetching multiple values from Redis")?;
        values.extend(part);
    }
    Ok(values)
}

async fn get_many_primary_as<C, T>(
    connection: &mut C,
    keys: &[String],
) -> Result<Vec<Option<T>>>
where
    C: RoutableConnection,
    T: FromRedisValue,
{
    let mut values = Vec::with_capacity(keys.len());
    for chunk in keys.chunks(MGET_CHUNK_SIZE) {
        let mut command = redis::cmd("MGET");
        command.arg(chunk);
        let value = connection
            .route_command(command, primary_mget_routing(chunk))
            .await
            .wrap_err("fetching multiple values from primary Redis nodes")?;
        let value = value
            .extract_error()
            .wrap_err("extracting Redis response")?;
        let part = redis::from_redis_value::<Vec<Option<T>>>(value)
            .map_err(redis::RedisError::from)
            .wrap_err("decoding Redis response")?;
        values.extend(part);
    }
    Ok(values)
}

#[tracing::instrument(skip_all)]
pub async fn get_deserialized<C, R>(
    connection: &mut C,
    key: &str,
    settings: &CacheSettings,
) -> Result<Option<R>>
where
    C: ConnectionLike,
    R: for<'a> serde::Deserialize<'a>,
{
    let value: Option<Vec<u8>> = cmd("GET")
        .arg(key)
        .query_async(connection)
        .await
        .wrap_err("fetching serialized value from Redis")?;
    Ok(value.and_then(|value| settings.decode_value(&value)))
}

#[tracing::instrument(skip_all)]
pub async fn get_many_deserialized<C, R>(
    connection: &mut C,
    keys: &[String],
    settings: &CacheSettings,
) -> Result<Vec<Option<R>>>
where
    C: ConnectionLike,
    R: for<'a> serde::Deserialize<'a>,
{
    Ok(get_many(connection, keys)
        .await
        .wrap_err("fetching serialized values from Redis")?
        .into_iter()
        .map(|value| value.and_then(|value| settings.decode_value(&value)))
        .collect())
}

#[tracing::instrument(skip_all)]
pub async fn delete<C>(connection: &mut C, key: &str) -> Result<()>
where
    C: ConnectionLike,
{
    cmd("DEL")
        .arg(key)
        .query_async::<()>(connection)
        .await
        .wrap_err("deleting from Redis")?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn delete_many<C>(connection: &mut C, keys: &[String]) -> Result<()>
where
    C: ConnectionLike,
{
    if !keys.is_empty() {
        cmd("DEL")
            .arg(keys)
            .query_async::<()>(connection)
            .await
            .wrap_err("deleting multiple values from Redis")?;
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn lpush<C, D>(connection: &mut C, key: &str, value: D) -> Result<()>
where
    C: ConnectionLike,
    D: ToRedisArgs + Send + Sync + Debug,
{
    cmd("LPUSH")
        .arg(key)
        .arg(value)
        .query_async::<()>(connection)
        .await
        .wrap_err("pushing to Redis list")?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn incr<C>(connection: &mut C, key: &str) -> Result<Option<u64>>
where
    C: ConnectionLike,
{
    cmd("INCR")
        .arg(key)
        .query_async(connection)
        .await
        .wrap_err("incrementing Redis value")
}

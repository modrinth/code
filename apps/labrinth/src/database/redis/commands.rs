use std::fmt::Debug;

use redis::aio::ConnectionLike;
use redis::{FromRedisValue, ToRedisArgs};

use crate::database::models::DatabaseError;

use super::cache::CacheSettings;
use super::util::cmd;

pub const DEFAULT_EXPIRY: i64 = 60 * 60 * 12;
pub const MGET_CHUNK_SIZE: usize = 32;

#[tracing::instrument(skip_all)]
pub async fn set<C, D>(
    connection: &mut C,
    key: &str,
    data: D,
    expiry: Option<i64>,
) -> Result<(), DatabaseError>
where
    C: ConnectionLike,
    D: ToRedisArgs + Send + Sync + Debug,
{
    cmd("SET")
        .arg(key)
        .arg(data)
        .arg("EX")
        .arg(expiry.unwrap_or(DEFAULT_EXPIRY))
        .query_async::<()>(connection)
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn set_serialized<C, D>(
    connection: &mut C,
    key: &str,
    data: D,
    expiry: Option<i64>,
    settings: &CacheSettings,
) -> Result<(), DatabaseError>
where
    C: ConnectionLike,
    D: serde::Serialize,
{
    set(connection, key, settings.encode_value(&data)?, expiry).await
}

#[tracing::instrument(skip_all)]
pub async fn get<C>(
    connection: &mut C,
    key: &str,
) -> Result<Option<String>, DatabaseError>
where
    C: ConnectionLike,
{
    Ok(cmd("GET").arg(key).query_async(connection).await?)
}

/// Issues ordinary `MGET` commands in bounded chunks. Cluster routing and
/// result ordering remain redis-rs's responsibility; multiple chunks are not
/// an atomic snapshot.
#[tracing::instrument(skip_all)]
pub async fn get_many<C>(
    connection: &mut C,
    keys: &[String],
) -> Result<Vec<Option<Vec<u8>>>, DatabaseError>
where
    C: ConnectionLike,
{
    get_many_as(connection, keys).await
}

#[tracing::instrument(skip_all)]
pub async fn get_many_strings<C>(
    connection: &mut C,
    keys: &[String],
) -> Result<Vec<Option<String>>, DatabaseError>
where
    C: ConnectionLike,
{
    get_many_as(connection, keys).await
}

pub(super) async fn get_many_as<C, T>(
    connection: &mut C,
    keys: &[String],
) -> Result<Vec<Option<T>>, DatabaseError>
where
    C: ConnectionLike,
    T: FromRedisValue,
{
    let mut values = Vec::with_capacity(keys.len());
    for chunk in keys.chunks(MGET_CHUNK_SIZE) {
        let part = cmd("MGET")
            .arg(chunk)
            .query_async::<Vec<Option<T>>>(connection)
            .await?;
        values.extend(part);
    }
    Ok(values)
}

#[tracing::instrument(skip_all)]
pub async fn get_deserialized<C, R>(
    connection: &mut C,
    key: &str,
    settings: &CacheSettings,
) -> Result<Option<R>, DatabaseError>
where
    C: ConnectionLike,
    R: for<'a> serde::Deserialize<'a>,
{
    let value: Option<Vec<u8>> =
        cmd("GET").arg(key).query_async(connection).await?;
    Ok(value.and_then(|value| settings.decode_value(&value)))
}

#[tracing::instrument(skip_all)]
pub async fn get_many_deserialized<C, R>(
    connection: &mut C,
    keys: &[String],
    settings: &CacheSettings,
) -> Result<Vec<Option<R>>, DatabaseError>
where
    C: ConnectionLike,
    R: for<'a> serde::Deserialize<'a>,
{
    Ok(get_many(connection, keys)
        .await?
        .into_iter()
        .map(|value| value.and_then(|value| settings.decode_value(&value)))
        .collect())
}

#[tracing::instrument(skip_all)]
pub async fn delete<C>(
    connection: &mut C,
    key: &str,
) -> Result<(), DatabaseError>
where
    C: ConnectionLike,
{
    cmd("DEL").arg(key).query_async::<()>(connection).await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn delete_many<C>(
    connection: &mut C,
    keys: &[String],
) -> Result<(), DatabaseError>
where
    C: ConnectionLike,
{
    if !keys.is_empty() {
        cmd("DEL").arg(keys).query_async::<()>(connection).await?;
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn lpush<C, D>(
    connection: &mut C,
    key: &str,
    value: D,
) -> Result<(), DatabaseError>
where
    C: ConnectionLike,
    D: ToRedisArgs + Send + Sync + Debug,
{
    cmd("LPUSH")
        .arg(key)
        .arg(value)
        .query_async::<()>(connection)
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn incr<C>(
    connection: &mut C,
    key: &str,
) -> Result<Option<u64>, DatabaseError>
where
    C: ConnectionLike,
{
    Ok(cmd("INCR").arg(key).query_async(connection).await?)
}

#[tracing::instrument(
    name = "redis.cache_lock.acquire",
    skip_all,
    fields(command = "SET {} {} NX PX {}")
)]
pub(super) async fn acquire_lock<C>(
    connection: &mut C,
    key: &str,
    lease_ms: u64,
) -> Result<bool, DatabaseError>
where
    C: ConnectionLike,
{
    let response = acquire_lock_command(key, lease_ms)
        .query_async::<Option<String>>(connection)
        .await?;
    Ok(response.is_some())
}

fn acquire_lock_command(key: &str, lease_ms: u64) -> redis::Cmd {
    let mut command = redis::cmd("SET");
    command.arg(key).arg("").arg("NX").arg("PX").arg(lease_ms);
    command
}

#[tracing::instrument(
    name = "redis.cache_lock.renew",
    skip_all,
    fields(command = "PEXPIRE {} {}")
)]
pub(super) async fn renew_lock<C>(
    connection: &mut C,
    key: &str,
    lease_ms: u64,
) -> Result<bool, DatabaseError>
where
    C: ConnectionLike,
{
    let renewed = renew_lock_command(key, lease_ms)
        .query_async::<i64>(connection)
        .await?;
    Ok(renewed == 1)
}

fn renew_lock_command(key: &str, lease_ms: u64) -> redis::Cmd {
    let mut command = redis::cmd("PEXPIRE");
    command.arg(key).arg(lease_ms);
    command
}

#[tracing::instrument(
    name = "redis.cache_lock.release",
    skip_all,
    fields(command = "DEL {}")
)]
pub(super) async fn release_lock<C>(
    connection: &mut C,
    key: &str,
) -> Result<bool, DatabaseError>
where
    C: ConnectionLike,
{
    let released = release_lock_command(key)
        .query_async::<i64>(connection)
        .await?;
    Ok(released == 1)
}

fn release_lock_command(key: &str) -> redis::Cmd {
    let mut command = redis::cmd("DEL");
    command.arg(key);
    command
}

#[tracing::instrument(
    name = "redis.cache_lock.exists",
    skip_all,
    fields(command = "EXISTS {}")
)]
pub(super) async fn lock_exists<C>(
    connection: &mut C,
    key: &str,
) -> Result<bool, DatabaseError>
where
    C: ConnectionLike,
{
    Ok(redis::cmd("EXISTS")
        .arg(key)
        .query_async::<u64>(connection)
        .await?
        != 0)
}

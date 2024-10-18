use redis::Cmd;

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

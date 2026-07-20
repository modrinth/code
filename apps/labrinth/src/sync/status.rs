use crate::database::redis::RedisPool;
use crate::queue::socket::ActiveSockets;
use ariadne::ids::UserId;
use ariadne::users::UserStatus;
use redis::AsyncCommands;

const EXPIRY_TIME_SECONDS: i64 = 60;

pub async fn get_user_status(
    user: UserId,
    local_sockets: &ActiveSockets,
    redis: &RedisPool,
) -> Option<UserStatus> {
    if let Some(friend_status) = local_sockets.get_status(user) {
        return Some(friend_status);
    }

    if let Ok(mut conn) = redis.pool.get().await
        && let Ok(mut statuses) =
            conn.sscan::<_, Vec<u8>>(get_field_name(user)).await
        && let Some(status) = statuses.next_item().await
    {
        return postcard::from_bytes::<UserStatus>(&status).ok();
    }

    None
}

pub async fn replace_user_status(
    old_status: Option<&UserStatus>,
    new_status: Option<&UserStatus>,
    redis: &RedisPool,
) -> Result<(), redis::RedisError> {
    let Some(user) = new_status.or(old_status).map(|x| x.user_id) else {
        return Ok(());
    };

    if let Ok(mut conn) = redis.pool.get().await {
        let field_name = get_field_name(user);
        let mut pipe = redis::pipe();
        pipe.atomic();
        if let Some(status) = old_status {
            pipe.srem(&field_name, postcard::to_allocvec(status).unwrap())
                .ignore();
        }
        if let Some(status) = new_status {
            pipe.sadd(&field_name, postcard::to_allocvec(status).unwrap())
                .ignore();
            pipe.expire(&field_name, EXPIRY_TIME_SECONDS).ignore();
        }
        return pipe.query_async(&mut conn).await;
    }

    Ok(())
}

pub async fn push_back_user_expiry(
    user: UserId,
    redis: &RedisPool,
) -> Result<(), redis::RedisError> {
    if let Ok(mut conn) = redis.pool.get().await {
        return conn.expire(get_field_name(user), EXPIRY_TIME_SECONDS).await;
    }
    Ok(())
}

fn get_field_name(user: UserId) -> String {
    format!("user_status:v1:{user}")
}

use crate::auth::get_user_from_headers;
use crate::database::PgPool;
use crate::database::models::DBUser;
use crate::database::models::blocked_user_item::DBBlockedUser;
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use actix_web::{HttpRequest, delete, post, web};
use eyre::eyre;
use xredis::RedisPool;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(block_user);
    cfg.service(unblock_user);
}

/// Block a user.
#[utoipa::path(tag = "blocked_users", responses((status = NO_CONTENT)))]
#[post("/block/{id}")]
pub async fn block_user(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_WRITE,
    )
    .await?
    .1;

    let user_id = info.into_inner().0;
    let Some(blocked) = DBUser::get(&user_id, &**pool, &redis).await? else {
        return Err(ApiError::NotFound);
    };

    if blocked.id == user.id.into() {
        return Err(ApiError::Request(eyre!("you cannot block yourself")));
    }

    DBBlockedUser {
        user_id: user.id.into(),
        blocked_id: blocked.id,
    }
    .insert(&**pool)
    .await?;

    Ok(())
}

/// Unblock a user.
#[utoipa::path(tag = "blocked_users", responses((status = NO_CONTENT)))]
#[delete("/block/{id}")]
pub async fn unblock_user(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_WRITE,
    )
    .await?
    .1;

    let user_id = info.into_inner().0;
    let Some(blocked) = DBUser::get(&user_id, &**pool, &redis).await? else {
        return Err(ApiError::NotFound);
    };

    DBBlockedUser::remove(user.id.into(), blocked.id, &**pool).await?;

    Ok(())
}

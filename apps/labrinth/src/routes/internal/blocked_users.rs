use crate::auth::get_user_from_headers;
use crate::database::PgPool;
use crate::database::models::DBUser;
use crate::database::models::DBUserId;
use crate::database::models::blocked_user_item::DBBlockedUser;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::guards::admin_key_guard;
use actix_web::{HttpRequest, delete, get, post, web};
use ariadne::ids::base62_impl::parse_base62;
use eyre::eyre;
use serde::Serialize;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(block_user);
    cfg.service(unblock_user);
    cfg.service(block_status);
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct BlockStatus {
    pub blocked: bool,
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

/// Check whether one user has blocked another.
#[utoipa::path(tag = "blocked_users", responses((status = OK, body = BlockStatus)))]
#[get("/block/{user_id}/{target_id}", guard = "admin_key_guard")]
pub async fn block_status(
    info: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
) -> Result<web::Json<BlockStatus>, ApiError> {
    let (user_id, target_id) = info.into_inner();

    let user_id = DBUserId(
        parse_base62(&user_id)
            .map_err(|_| ApiError::InvalidInput("invalid user_id".to_string()))?
            as i64,
    );
    let target_id = DBUserId(
        parse_base62(&target_id)
            .map_err(|_| ApiError::InvalidInput("invalid target_id".to_string()))?
            as i64,
    );

    let blocked =
        DBBlockedUser::is_blocked(user_id, target_id, &**pool).await?;

    Ok(web::Json(BlockStatus { blocked }))
}

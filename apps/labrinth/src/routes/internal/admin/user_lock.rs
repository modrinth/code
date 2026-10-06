use actix_web::{HttpRequest, delete, put, web};
use eyre::eyre;
use serde::Deserialize;
use xredis::RedisPool;

use crate::auth::get_user_from_headers;
use crate::database::PgPool;
use crate::database::models::DBUser;
use crate::database::models::session_item::DBSession;
use crate::database::models::user_lock_item::DBUserLock;
use crate::models::pats::Scopes;
use crate::models::users::Role;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context as _;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(lock_user).service(unlock_user);
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct LockUserRequest {
    pub reason: String,
}

/// Locks a user's account, preventing it from performing any write operations,
/// and revokes all of its sessions.
///
/// Locking an already locked user replaces the existing lock's reason.
#[utoipa::path(
	context_path = "/admin/user",
	tag = "moderation",
	request_body = LockUserRequest,
	responses((status = NO_CONTENT))
)]
#[put("/{id}/lock")]
pub async fn lock_user(
    req: HttpRequest,
    path: web::Path<(String,)>,
    body: web::Json<LockUserRequest>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    if !user.role.is_admin() {
        return Err(ApiError::Auth(eyre!("only admins can lock users")));
    }

    let reason = body.reason.trim();
    if reason.is_empty() {
        return Err(ApiError::Request(eyre!("lock reason must not be empty")));
    }

    let target = DBUser::get(&path.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?
        .wrap_not_found_err("user not found")?;

    if Role::from_string(&target.role).is_mod() {
        return Err(ApiError::Auth(eyre!("cannot lock a staff account")));
    }

    let mut txn = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    DBUserLock::upsert(target.id, user.id.into(), reason, &mut txn)
        .await
        .wrap_internal_err("locking user")?;

    let sessions = DBSession::remove_all_for_user(target.id, &mut txn)
        .await
        .wrap_internal_err("revoking user sessions")?;

    txn.commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    DBUser::clear_caches(&[(target.id, Some(target.username))], &redis)
        .await
        .wrap_internal_err("clearing user cache")?;

    DBSession::clear_user_sessions_cache(target.id, sessions, &redis)
        .await
        .wrap_internal_err("clearing session cache")?;

    Ok(())
}

/// Removes the lock from a user's account.
#[utoipa::path(
	context_path = "/admin/user",
	tag = "moderation",
	responses((status = NO_CONTENT))
)]
#[delete("/{id}/lock")]
pub async fn unlock_user(
    req: HttpRequest,
    path: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    if !user.role.is_admin() {
        return Err(ApiError::Auth(eyre!("only admins can unlock users")));
    }

    let target = DBUser::get(&path.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?
        .wrap_not_found_err("user not found")?;

    if !DBUserLock::delete(target.id, &**pool)
        .await
        .wrap_internal_err("unlocking user")?
    {
        return Err(ApiError::NotFound(eyre!("user is not locked")));
    }

    DBUser::clear_caches(&[(target.id, Some(target.username))], &redis)
        .await
        .wrap_internal_err("clearing user cache")?;

    Ok(())
}

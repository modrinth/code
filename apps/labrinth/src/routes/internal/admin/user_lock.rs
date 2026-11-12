use actix_web::{HttpRequest, delete, put, web};
use anyhow::anyhow;
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
    .auth_context("authenticating API request")?;

    if !user.scopes.contains(Scopes::ADMIN_WRITE) {
        return Err(ApiError::Auth(anyhow!("only admins can lock users")));
    }

    let reason = body.reason.trim();
    if reason.is_empty() {
        return Err(ApiError::Request(anyhow!(
            "lock reason must not be empty"
        )));
    }

    let target = DBUser::get(&path.into_inner().0, &**pool, &redis)
        .await
        .internal_context("fetching user from database")?
        .not_found_context("user not found")?;

    if matches!(
        Role::from_string(&target.role),
        Role::Moderator | Role::Admin
    ) {
        return Err(ApiError::Auth(anyhow!("cannot lock a staff account")));
    }

    let mut txn = pool
        .begin()
        .await
        .internal_context("starting database transaction")?;

    DBUserLock::upsert(target.id, user.user.id.into(), reason, &mut txn)
        .await
        .internal_context("locking user")?;

    let sessions = DBSession::remove_all_for_user(target.id, &mut txn)
        .await
        .internal_context("revoking user sessions")?;

    txn.commit()
        .await
        .internal_context("committing database transaction")?;

    DBUser::clear_caches(&[(target.id, Some(target.username))], &redis)
        .await
        .internal_context("clearing user cache")?;

    DBSession::clear_user_sessions_cache(target.id, sessions, &redis)
        .await
        .internal_context("clearing session cache")?;

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
    .auth_context("authenticating API request")?;

    if !user.scopes.contains(Scopes::ADMIN_WRITE) {
        return Err(ApiError::Auth(anyhow!("only admins can unlock users")));
    }

    let target = DBUser::get(&path.into_inner().0, &**pool, &redis)
        .await
        .internal_context("fetching user from database")?
        .not_found_context("user not found")?;

    if !DBUserLock::delete(target.id, &**pool)
        .await
        .internal_context("unlocking user")?
    {
        return Err(ApiError::NotFound(anyhow!("user is not locked")));
    }

    DBUser::clear_caches(&[(target.id, Some(target.username))], &redis)
        .await
        .internal_context("clearing user cache")?;

    Ok(())
}

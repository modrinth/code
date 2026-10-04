use actix_web::{HttpRequest, delete, put, web};
use eyre::eyre;
use serde::Deserialize;
use xredis::RedisPool;

use crate::auth::get_user_from_headers;
use crate::database::PgPool;
use crate::database::models::DBUser;
use crate::database::models::user_restriction_item::DBUserRestriction;
use crate::models::pats::Scopes;
use crate::models::users::{Role, User};
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context as _;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(restrict_user).service(unrestrict_user);
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct RestrictUserRequest {
    pub removed_perms: Scopes,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub private_reason: Option<String>,
}

async fn authenticate_moderator(
    req: &HttpRequest,
    pool: &PgPool,
    redis: &RedisPool,
    session_queue: &AuthQueue,
) -> Result<User, ApiError> {
    let user = get_user_from_headers(
        req,
        pool,
        redis,
        session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    if !user.role.is_mod() {
        return Err(ApiError::Auth(eyre!(
            "only moderators can manage user restrictions"
        )));
    }

    Ok(user)
}

async fn get_target(
    id: &str,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<DBUser, ApiError> {
    let target = DBUser::get(id, pool, redis)
        .await
        .wrap_internal_err("fetching user from database")?
        .wrap_not_found_err("user not found")?;

    if Role::from_string(&target.role).is_mod() {
        return Err(ApiError::Auth(eyre!("cannot restrict a staff account")));
    }

    Ok(target)
}

fn non_blank(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// Replaces a user's restriction, removing the given scopes from every token
/// the user authenticates with.
///
/// An empty `removed_perms` lifts the restriction.
#[utoipa::path(
	context_path = "/moderation",
	tag = "moderation",
	request_body = RestrictUserRequest,
	responses((status = NO_CONTENT))
)]
#[put("/user/{id}/restrictions")]
pub async fn restrict_user(
    req: HttpRequest,
    path: web::Path<(String,)>,
    body: web::Json<RestrictUserRequest>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    let user =
        authenticate_moderator(&req, &pool, &redis, &session_queue).await?;
    let target = get_target(&path.into_inner().0, &pool, &redis).await?;

    if !Scopes::removable().contains(body.removed_perms) {
        return Err(ApiError::Request(eyre!(
            "`removed_perms` contains scopes that cannot be removed"
        )));
    }

    if body.removed_perms.is_empty() {
        DBUserRestriction::delete(target.id, &**pool)
            .await
            .wrap_internal_err("lifting user restriction")?;
    } else {
        DBUserRestriction::upsert(
            target.id,
            body.removed_perms,
            non_blank(body.reason.as_deref()),
            non_blank(body.private_reason.as_deref()),
            user.id.into(),
            &**pool,
        )
        .await
        .wrap_internal_err("restricting user")?;
    }

    DBUser::clear_caches(&[(target.id, Some(target.username))], &redis)
        .await
        .wrap_internal_err("clearing user cache")?;

    Ok(())
}

/// Lifts a user's restriction.
#[utoipa::path(
	context_path = "/moderation",
	tag = "moderation",
	responses((status = NO_CONTENT))
)]
#[delete("/user/{id}/restrictions")]
pub async fn unrestrict_user(
    req: HttpRequest,
    path: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    authenticate_moderator(&req, &pool, &redis, &session_queue).await?;
    let target = get_target(&path.into_inner().0, &pool, &redis).await?;

    if !DBUserRestriction::delete(target.id, &**pool)
        .await
        .wrap_internal_err("lifting user restriction")?
    {
        return Err(ApiError::NotFound(eyre!("user is not restricted")));
    }

    DBUser::clear_caches(&[(target.id, Some(target.username))], &redis)
        .await
        .wrap_internal_err("clearing user cache")?;

    Ok(())
}

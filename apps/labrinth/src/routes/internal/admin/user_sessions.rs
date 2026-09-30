use actix_web::{HttpRequest, delete, web};
use eyre::eyre;
use xredis::RedisPool;

use crate::auth::get_user_from_headers;
use crate::database::PgPool;
use crate::database::models::DBUser;
use crate::database::models::pat_item::DBPersonalAccessToken;
use crate::database::models::session_item::DBSession;
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context as _;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(revoke_user_sessions);
}

/// Revokes all sessions and personal access tokens of a user, signing them out everywhere.
#[utoipa::path(
	context_path = "/admin/user",
	tag = "moderation",
	responses((status = NO_CONTENT))
)]
#[delete("/{id}/sessions")]
pub async fn revoke_user_sessions(
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
        return Err(ApiError::Auth(eyre!(
            "only admins can revoke user sessions"
        )));
    }

    let target = DBUser::get(&path.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?
        .wrap_not_found_err("user not found")?;

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    let sessions = DBSession::remove_all_for_user(target.id, &mut transaction)
        .await
        .wrap_internal_err("revoking user sessions")?;

    let pats =
        DBPersonalAccessToken::remove_all_for_user(target.id, &mut transaction)
            .await
            .wrap_internal_err("revoking user personal access tokens")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    DBSession::clear_user_sessions_cache(target.id, sessions, &redis)
        .await
        .wrap_internal_err("clearing session cache")?;

    DBPersonalAccessToken::clear_user_pats_cache(target.id, pats, &redis)
        .await
        .wrap_internal_err("clearing personal access token cache")?;

    Ok(())
}

use actix_web::{HttpRequest, delete, post, web};
use chrono::Duration;
use eyre::eyre;
use lettre::message::Mailbox;
use serde::Deserialize;
use xredis::RedisPool;

use crate::auth::get_user_from_headers;
use crate::database::PgPool;
use crate::database::models::DBUser;
use crate::database::models::flow_item::DBFlow;
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::session_item::DBSession;
use crate::models::notifications::NotificationBody;
use crate::models::pats::Scopes;
use crate::models::users::Role;
use crate::queue::email::EmailQueue;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::ApiContext as _;
use crate::util::error::Context as _;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(force_password_reset).service(reset_2fa);
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ForcePasswordResetRequest {
    /// Replaces the user's email address. Defaults to their current one.
    pub email: Option<String>,
}

/// Forces a password reset and revokes access.
#[utoipa::path(
	context_path = "/moderation/user-credentials",
	tag = "moderation",
	request_body = ForcePasswordResetRequest,
	responses((status = NO_CONTENT))
)]
#[post("/{id}/password-reset")]
pub async fn force_password_reset(
    req: HttpRequest,
    path: web::Path<(String,)>,
    body: web::Json<ForcePasswordResetRequest>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    email_queue: web::Data<EmailQueue>,
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
            "you must be an admin to reset a user's password"
        )));
    }

    let target = DBUser::get(&path.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?
        .wrap_not_found_err("user not found")?;

    if Role::from_string(&target.role).is_mod() {
        return Err(ApiError::Auth(eyre!(
            "cannot reset password of a staff account"
        )));
    }

    let address = body
        .into_inner()
        .email
        .map(|email| email.trim().to_owned())
        .filter(|email| !email.is_empty())
        .or(target.email)
        .wrap_request_err("user has no email address, one must be specified")?;

    let mailbox: Mailbox =
        address.parse().wrap_request_err("invalid email address")?;

    let email_taken = DBUser::get_by_case_insensitive_email(&address, &**pool)
        .await
        .wrap_internal_err("checking for existing email address")?
        .into_iter()
        .any(|id| id != target.id);

    if email_taken {
        return Err(ApiError::Request(eyre!(
            "email address is already registered to another account"
        )));
    }

    let mut txn = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    sqlx::query!(
        r#"
        UPDATE users
        SET
            password = NULL,
            email = $2::varchar,
            email_verified = email_verified AND LOWER(email) IS NOT DISTINCT FROM LOWER($2)
        WHERE id = $1
        "#,
        target.id as _,
        address,
    )
    .execute(&mut txn)
    .await
    .wrap_internal_err("clearing user password and setting email")?;

    let sessions = DBSession::remove_all_for_user(target.id, &mut txn)
        .await
        .wrap_internal_err("revoking user sessions")?;

    let flow = DBFlow::ForcedPasswordReset {
        user_id: target.id,
        email: address,
    }
    .insert(Duration::hours(24), &redis)
    .await
    .wrap_internal_err("inserting password reset flow")?;

    email_queue
        .send_one(
            &mut txn,
            NotificationBody::ResetPassword { flow },
            target.id,
            mailbox,
        )
        .await
        .wrap_api_err("sending password reset email")?
        .as_user_error()
        .wrap_api_err("validating email delivery status")?;

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

/// Removes two-factor authentication and backup codes from a user's account.
#[utoipa::path(
	context_path = "/moderation/user-credentials",
	tag = "moderation",
	responses((status = NO_CONTENT))
)]
#[delete("/{id}/2fa")]
pub async fn reset_2fa(
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
            "you must be an admin to reset a user's 2FA"
        )));
    }

    let target = DBUser::get(&path.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?
        .wrap_not_found_err("user not found")?;

    if Role::from_string(&target.role).is_mod() {
        return Err(ApiError::Auth(eyre!(
            "cannot reset 2FA of a staff account"
        )));
    }

    if target.totp_secret.is_none() {
        return Err(ApiError::Request(eyre!("user does not have 2FA enabled")));
    }

    let mut txn = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    DBUser::remove_2fa(target.id, &mut txn)
        .await
        .wrap_internal_err("removing 2FA")?;

    NotificationBuilder {
        body: NotificationBody::TwoFactorRemoved,
    }
    .insert(target.id, &mut txn, &redis)
    .await
    .wrap_internal_err("inserting 2FA removal notification")?;

    txn.commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    DBUser::clear_caches(&[(target.id, Some(target.username))], &redis)
        .await
        .wrap_internal_err("clearing user cache")?;

    Ok(())
}

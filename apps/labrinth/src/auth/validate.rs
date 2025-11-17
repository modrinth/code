use super::AuthProvider;
use crate::App;
use crate::auth::AuthenticationError;
use crate::database::models::{DBUser, user_item};
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::models::users::User;
use crate::queue::session::AuthQueue;
use crate::routes::internal::session::get_session_metadata;
use actix_web::HttpRequest;
use actix_web::http::header::{AUTHORIZATION, HeaderValue};
use chrono::Utc;

pub async fn get_maybe_user_from_headers<'a, E>(
    app: &App,
    req: &HttpRequest,
    executor: E,
    required_scopes: Scopes,
) -> Result<Option<(Scopes, User)>, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
{
    if !req.headers().contains_key(AUTHORIZATION) {
        return Ok(None);
    }

    // Fetch DB user record and minos user from headers
    let Some((scopes, db_user)) =
        get_user_record_from_bearer_token(app, req, None, executor).await?
    else {
        return Ok(None);
    };

    if !scopes.contains(required_scopes) {
        return Ok(None);
    }

    Ok(Some((scopes, User::from_full(db_user))))
}

pub async fn get_full_user_from_headers<'a, E>(
    app: &App,
    req: &HttpRequest,
    executor: E,
    required_scopes: Scopes,
) -> Result<(Scopes, DBUser), AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
{
    let (scopes, db_user) =
        get_user_record_from_bearer_token(app, req, None, executor)
            .await?
            .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

    if !scopes.contains(required_scopes) {
        return Err(AuthenticationError::InvalidCredentials);
    }

    Ok((scopes, db_user))
}

pub async fn get_user_from_headers<'a, E>(
    req: &HttpRequest,
    executor: E,
    _redis: &RedisPool,
    _session_queue: &AuthQueue,
    required_scopes: Scopes,
) -> Result<(Scopes, User), AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
{
    get_user_from_headers_v2(
        crate::APP.get().unwrap(),
        req,
        executor,
        required_scopes,
    )
    .await
}

pub async fn get_user_from_headers_v2<'a, E>(
    app: &App,
    req: &HttpRequest,
    executor: E,
    required_scopes: Scopes,
) -> Result<(Scopes, User), AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
{
    let (scopes, db_user) =
        get_full_user_from_headers(app, req, executor, required_scopes).await?;

    Ok((scopes, User::from_full(db_user)))
}

pub async fn get_user_record_from_bearer_token<'a, 'b, E>(
    app: &App,
    req: &HttpRequest,
    token: Option<&str>,
    executor: E,
) -> Result<Option<(Scopes, user_item::DBUser)>, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
{
    let token = if let Some(token) = token {
        token
    } else {
        extract_authorization_header(req)?
    };

    let possible_user = match token.split_once('_') {
        Some(("mrp", _)) => {
            let pat =
                crate::database::models::pat_item::DBPersonalAccessToken::get(
                    token,
                    executor,
                    &app.env.redis,
                )
                .await?
                .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

            if pat.expires < Utc::now() {
                return Err(AuthenticationError::InvalidCredentials);
            }

            let user = user_item::DBUser::get_id(
                pat.user_id,
                executor,
                &app.env.redis,
            )
            .await?;

            app.state.auth_queue.add_pat(pat.id).await;

            user.map(|x| (pat.scopes, x))
        }
        Some(("mra", _)) => {
            let session =
                crate::database::models::session_item::DBSession::get(
                    token,
                    executor,
                    &app.env.redis,
                )
                .await?
                .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

            if session.expires < Utc::now() {
                return Err(AuthenticationError::InvalidCredentials);
            }

            let user = user_item::DBUser::get_id(
                session.user_id,
                executor,
                &app.env.redis,
            )
            .await?;

            if req
                .headers()
                .get("x-ratelimit-key")
                .and_then(|x| x.to_str().ok())
                .is_none_or(|x| x != &app.env.rate_limit_ignore_key)
            {
                let metadata = get_session_metadata(req).await?;
                app.state.auth_queue.add_session(session.id, metadata).await;
            }

            user.map(|x| (Scopes::all(), x))
        }
        Some(("mro", _)) => {
            use crate::database::models::oauth_token_item::DBOAuthAccessToken;

            let hash = DBOAuthAccessToken::hash_token(token);
            let access_token =
                crate::database::models::oauth_token_item::DBOAuthAccessToken::get(hash, executor)
                    .await?
                    .ok_or(AuthenticationError::InvalidCredentials)?;

            if access_token.expires < Utc::now() {
                return Err(AuthenticationError::InvalidCredentials);
            }

            let user = user_item::DBUser::get_id(
                access_token.user_id,
                executor,
                &app.env.redis,
            )
            .await?;

            app.state
                .auth_queue
                .add_oauth_access_token(access_token.id)
                .await;

            user.map(|u| (access_token.scopes, u))
        }
        Some(("github" | "gho" | "ghp", _)) => {
            let user = AuthProvider::GitHub.get_user(&app.env, token).await?;
            let id =
                AuthProvider::GitHub.get_user_id(&user.id, executor).await?;

            let user = user_item::DBUser::get_id(
                id.ok_or_else(|| AuthenticationError::InvalidCredentials)?,
                executor,
                &app.env.redis,
            )
            .await?;

            user.map(|x| ((Scopes::all() ^ Scopes::restricted()), x))
        }
        _ => return Err(AuthenticationError::InvalidAuthMethod),
    };

    Ok(possible_user)
}

pub fn extract_authorization_header(
    req: &HttpRequest,
) -> Result<&str, AuthenticationError> {
    let headers = req.headers();
    let token_val: Option<&HeaderValue> = headers.get(AUTHORIZATION);
    let token_val = token_val
        .ok_or_else(|| AuthenticationError::InvalidAuthMethod)?
        .to_str()
        .map_err(|_| AuthenticationError::InvalidCredentials)?;
    Ok(if let Some(token) = token_val.strip_prefix("Bearer ") {
        token
    } else {
        token_val
    })
}

pub async fn check_is_moderator_from_headers<'a, 'b, E>(
    app: &App,
    req: &HttpRequest,
    executor: E,
    required_scopes: Scopes,
) -> Result<User, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
{
    let user = get_user_from_headers_v2(app, req, executor, required_scopes)
        .await?
        .1;

    if user.role.is_mod() {
        Ok(user)
    } else {
        Err(AuthenticationError::InvalidCredentials)
    }
}

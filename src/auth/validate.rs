use crate::auth::flows::AuthProvider;
use crate::auth::session::get_session_metadata;
use crate::auth::AuthenticationError;
use crate::database::models::user_item;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::models::users::{Role, User, UserId, UserPayoutData};
use crate::queue::session::AuthQueue;
use actix_web::HttpRequest;
use chrono::Utc;
use reqwest::header::{HeaderValue, AUTHORIZATION};

pub async fn get_user_from_headers<'a, E>(
    req: &HttpRequest,
    executor: E,
    redis: &RedisPool,
    session_queue: &AuthQueue,
    required_scopes: Option<&[Scopes]>,
) -> Result<(Scopes, User), AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
{
    // Fetch DB user record and minos user from headers
    let (scopes, db_user) =
        get_user_record_from_bearer_token(req, None, executor, redis, session_queue)
            .await?
            .ok_or_else(|| AuthenticationError::InvalidCredentials)?;
    let mut auth_providers = Vec::new();
    if db_user.github_id.is_some() {
        auth_providers.push(AuthProvider::GitHub)
    }
    if db_user.gitlab_id.is_some() {
        auth_providers.push(AuthProvider::GitLab)
    }
    if db_user.discord_id.is_some() {
        auth_providers.push(AuthProvider::Discord)
    }
    if db_user.google_id.is_some() {
        auth_providers.push(AuthProvider::Google)
    }
    if db_user.microsoft_id.is_some() {
        auth_providers.push(AuthProvider::Microsoft)
    }
    if db_user.steam_id.is_some() {
        auth_providers.push(AuthProvider::Steam)
    }

    let user = User {
        id: UserId::from(db_user.id),
        username: db_user.username,
        name: db_user.name,
        email: db_user.email,
        email_verified: Some(db_user.email_verified),
        avatar_url: db_user.avatar_url,
        bio: db_user.bio,
        created: db_user.created,
        role: Role::from_string(&db_user.role),
        badges: db_user.badges,
        auth_providers: Some(auth_providers),
        has_password: Some(db_user.password.is_some()),
        has_totp: Some(db_user.totp_secret.is_some()),
        github_id: None,
        payout_data: Some(UserPayoutData {
            balance: db_user.balance,
            trolley_id: db_user.trolley_id,
            trolley_status: db_user.trolley_account_status,
        }),
    };

    if let Some(required_scopes) = required_scopes {
        for scope in required_scopes {
            if !scopes.contains(*scope) {
                return Err(AuthenticationError::InvalidCredentials);
            }
        }
    }

    Ok((scopes, user))
}

pub async fn get_user_record_from_bearer_token<'a, 'b, E>(
    req: &HttpRequest,
    token: Option<&str>,
    executor: E,
    redis: &RedisPool,
    session_queue: &AuthQueue,
) -> Result<Option<(Scopes, user_item::User)>, AuthenticationError>
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
                crate::database::models::pat_item::PersonalAccessToken::get(token, executor, redis)
                    .await?
                    .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

            if pat.expires < Utc::now() {
                return Err(AuthenticationError::InvalidCredentials);
            }

            let user = user_item::User::get_id(pat.user_id, executor, redis).await?;

            session_queue.add_pat(pat.id).await;

            user.map(|x| (pat.scopes, x))
        }
        Some(("mra", _)) => {
            let session =
                crate::database::models::session_item::Session::get(token, executor, redis)
                    .await?
                    .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

            if session.expires < Utc::now() {
                return Err(AuthenticationError::InvalidCredentials);
            }

            let user = user_item::User::get_id(session.user_id, executor, redis).await?;

            let rate_limit_ignore = dotenvy::var("RATE_LIMIT_IGNORE_KEY")?;
            if !req
                .headers()
                .get("x-ratelimit-key")
                .and_then(|x| x.to_str().ok())
                .map(|x| x == rate_limit_ignore)
                .unwrap_or(false)
            {
                let metadata = get_session_metadata(req).await?;
                session_queue.add_session(session.id, metadata).await;
            }

            user.map(|x| (Scopes::all(), x))
        }
        Some(("mro", _)) => {
            use crate::database::models::oauth_token_item::OAuthAccessToken;

            let hash = OAuthAccessToken::hash_token(token);
            let access_token =
                crate::database::models::oauth_token_item::OAuthAccessToken::get(hash, executor)
                    .await?
                    .ok_or(AuthenticationError::InvalidCredentials)?;

            if access_token.expires < Utc::now() {
                return Err(AuthenticationError::InvalidCredentials);
            }

            let user = user_item::User::get_id(access_token.user_id, executor, redis).await?;

            session_queue.add_oauth_access_token(access_token.id).await;

            user.map(|u| (access_token.scopes, u))
        }
        Some(("github", _)) | Some(("gho", _)) | Some(("ghp", _)) => {
            let user = AuthProvider::GitHub.get_user(token).await?;
            let id = AuthProvider::GitHub.get_user_id(&user.id, executor).await?;

            let user = user_item::User::get_id(
                id.ok_or_else(|| AuthenticationError::InvalidCredentials)?,
                executor,
                redis,
            )
            .await?;

            user.map(|x| ((Scopes::all() ^ Scopes::restricted()), x))
        }
        _ => return Err(AuthenticationError::InvalidAuthMethod),
    };
    Ok(possible_user)
}

pub fn extract_authorization_header(req: &HttpRequest) -> Result<&str, AuthenticationError> {
    let headers = req.headers();
    let token_val: Option<&HeaderValue> = headers.get(AUTHORIZATION);
    token_val
        .ok_or_else(|| AuthenticationError::InvalidAuthMethod)?
        .to_str()
        .map_err(|_| AuthenticationError::InvalidCredentials)
}

pub async fn check_is_moderator_from_headers<'a, 'b, E>(
    req: &HttpRequest,
    executor: E,
    redis: &RedisPool,
    session_queue: &AuthQueue,
    required_scopes: Option<&[Scopes]>,
) -> Result<User, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
{
    let user = get_user_from_headers(req, executor, redis, session_queue, required_scopes)
        .await?
        .1;

    if user.role.is_mod() {
        Ok(user)
    } else {
        Err(AuthenticationError::InvalidCredentials)
    }
}

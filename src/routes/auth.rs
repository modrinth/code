/*!
This auth module is primarily for use within the main website. Applications interacting with the
authenticated API (a very small portion - notifications, private projects, editing/creating projects
and versions) should either retrieve the Modrinth GitHub token through the site, or create a personal
app token for use with Modrinth.

JUst as a summary: Don't implement this flow in your application! Instead, use a personal access token
or create your own GitHub OAuth2 application.

This system will be revisited and allow easier interaction with the authenticated API once we roll
out our own authentication system.
*/

use crate::database::models::{generate_state_id, User};
use crate::models::error::ApiError;
use crate::models::ids::base62_impl::{parse_base62, to_base62};
use crate::models::ids::DecodingError;
use crate::models::users::{Badges, Role};
use crate::parse_strings_from_var;
use crate::util::auth::get_github_user_from_token;
use actix_web::http::StatusCode;
use actix_web::web::{scope, Data, Query, ServiceConfig};
use actix_web::{get, HttpResponse};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use thiserror::Error;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(scope("auth").service(auth_callback).service(init));
}

#[derive(Error, Debug)]
pub enum AuthorizationError {
    #[error("Environment Error")]
    Env(#[from] dotenvy::Error),
    #[error("An unknown database error occured: {0}")]
    SqlxDatabase(#[from] sqlx::Error),
    #[error("Database Error: {0}")]
    Database(#[from] crate::database::models::DatabaseError),
    #[error("Error while parsing JSON: {0}")]
    SerDe(#[from] serde_json::Error),
    #[error("Error while communicating to GitHub OAuth2")]
    Github(#[from] reqwest::Error),
    #[error("Invalid Authentication credentials")]
    InvalidCredentials,
    #[error("Authentication Error: {0}")]
    Authentication(#[from] crate::util::auth::AuthenticationError),
    #[error("Error while decoding Base62")]
    Decoding(#[from] DecodingError),
    #[error("Invalid callback URL specified")]
    Url,
    #[error("User is not allowed to access Modrinth services")]
    Banned,
}
impl actix_web::ResponseError for AuthorizationError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthorizationError::Env(..) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthorizationError::SqlxDatabase(..) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AuthorizationError::Database(..) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AuthorizationError::SerDe(..) => StatusCode::BAD_REQUEST,
            AuthorizationError::Github(..) => StatusCode::FAILED_DEPENDENCY,
            AuthorizationError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthorizationError::Decoding(..) => StatusCode::BAD_REQUEST,
            AuthorizationError::Authentication(..) => StatusCode::UNAUTHORIZED,
            AuthorizationError::Url => StatusCode::BAD_REQUEST,
            AuthorizationError::Banned => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ApiError {
            error: match self {
                AuthorizationError::Env(..) => "environment_error",
                AuthorizationError::SqlxDatabase(..) => "database_error",
                AuthorizationError::Database(..) => "database_error",
                AuthorizationError::SerDe(..) => "invalid_input",
                AuthorizationError::Github(..) => "github_error",
                AuthorizationError::InvalidCredentials => "invalid_credentials",
                AuthorizationError::Decoding(..) => "decoding_error",
                AuthorizationError::Authentication(..) => {
                    "authentication_error"
                }
                AuthorizationError::Url => "url_error",
                AuthorizationError::Banned => "user_banned",
            },
            description: &self.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct AuthorizationInit {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Authorization {
    pub code: String,
    pub state: String,
}

#[derive(Serialize, Deserialize)]
pub struct AccessToken {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}

//http://localhost:8000/api/v1/auth/init?url=https%3A%2F%2Fmodrinth.com%2Fmods
#[get("init")]
pub async fn init(
    Query(info): Query<AuthorizationInit>,
    client: Data<PgPool>,
) -> Result<HttpResponse, AuthorizationError> {
    let url =
        url::Url::parse(&info.url).map_err(|_| AuthorizationError::Url)?;

    let allowed_callback_urls =
        parse_strings_from_var("ALLOWED_CALLBACK_URLS").unwrap_or_default();

    let domain = url.domain().ok_or(AuthorizationError::Url)?;
    if !allowed_callback_urls.iter().any(|x| domain.ends_with(x))
        && domain != "modrinth.com"
    {
        return Err(AuthorizationError::Url);
    }

    let mut transaction = client.begin().await?;

    let state = generate_state_id(&mut transaction).await?;

    sqlx::query!(
        "
            INSERT INTO states (id, url)
            VALUES ($1, $2)
            ",
        state.0,
        info.url
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    let client_id = dotenvy::var("GITHUB_CLIENT_ID")?;
    let url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&state={}&scope={}",
        client_id,
        to_base62(state.0 as u64),
        "read%3Auser"
    );

    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", &*url))
        .json(AuthorizationInit { url }))
}

#[get("callback")]
pub async fn auth_callback(
    Query(info): Query<Authorization>,
    client: Data<PgPool>,
) -> Result<HttpResponse, AuthorizationError> {
    let mut transaction = client.begin().await?;
    let state_id = parse_base62(&info.state)?;

    let result_option = sqlx::query!(
        "
            SELECT url, expires FROM states
            WHERE id = $1
            ",
        state_id as i64
    )
    .fetch_optional(&mut *transaction)
    .await?;

    if let Some(result) = result_option {
        let duration: chrono::Duration = result.expires - Utc::now();

        if duration.num_seconds() < 0 {
            return Err(AuthorizationError::InvalidCredentials);
        }

        sqlx::query!(
            "
            DELETE FROM states
            WHERE id = $1
            ",
            state_id as i64
        )
        .execute(&mut *transaction)
        .await?;

        let client_id = dotenvy::var("GITHUB_CLIENT_ID")?;
        let client_secret = dotenvy::var("GITHUB_CLIENT_SECRET")?;

        let url = format!(
            "https://github.com/login/oauth/access_token?client_id={}&client_secret={}&code={}",
            client_id, client_secret, info.code
        );

        let token: AccessToken = reqwest::Client::new()
            .post(&url)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await?
            .json()
            .await?;

        let user = get_github_user_from_token(&token.access_token).await?;

        let user_result =
            User::get_from_github_id(user.id, &mut *transaction).await?;
        match user_result {
            Some(_) => {}
            None => {
                let banned_user = sqlx::query!(
                    "SELECT user FROM banned_users WHERE github_id = $1",
                    user.id as i64
                )
                .fetch_optional(&mut *transaction)
                .await?;

                if banned_user.is_some() {
                    return Err(AuthorizationError::Banned);
                }

                let user_id =
                    crate::database::models::generate_user_id(&mut transaction)
                        .await?;

                let mut username_increment: i32 = 0;
                let mut username = None;

                while username.is_none() {
                    let test_username = format!(
                        "{}{}",
                        &user.login,
                        if username_increment > 0 {
                            username_increment.to_string()
                        } else {
                            "".to_string()
                        }
                    );

                    let new_id = crate::database::models::User::get_id_from_username_or_id(
                        &test_username,
                        &**client,
                    )
                    .await?;

                    if new_id.is_none() {
                        username = Some(test_username);
                    } else {
                        username_increment += 1;
                    }
                }

                if let Some(username) = username {
                    User {
                        id: user_id,
                        github_id: Some(user.id as i64),
                        username,
                        name: user.name,
                        email: user.email,
                        avatar_url: Some(user.avatar_url),
                        bio: user.bio,
                        created: Utc::now(),
                        role: Role::Developer.to_string(),
                        badges: Badges::default(),
                        balance: Decimal::ZERO,
                        payout_wallet: None,
                        payout_wallet_type: None,
                        payout_address: None,
                        flame_anvil_key: None,
                    }
                    .insert(&mut transaction)
                    .await?;
                }
            }
        }

        transaction.commit().await?;

        let redirect_url = if result.url.contains('?') {
            format!("{}&code={}", result.url, token.access_token)
        } else {
            format!("{}?code={}", result.url, token.access_token)
        };

        Ok(HttpResponse::TemporaryRedirect()
            .append_header(("Location", &*redirect_url))
            .json(AuthorizationInit { url: redirect_url }))
    } else {
        Err(AuthorizationError::InvalidCredentials)
    }
}

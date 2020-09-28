use crate::models::error::ApiError;
use log::{info};
use actix_web::web::{Query, ServiceConfig, scope, Data};
use actix_web::{get, HttpResponse};
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use serde_json::Value;
use crate::database::models::generate_state_id;
use sqlx::postgres::PgPool;
use crate::models::ids::base62_impl::{to_base62, parse_base62};
use chrono::Utc;
use crate::models::ids::{DecodingError};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/auth/")
            .service(auth_callback)
            .service(init)
    );
}

#[derive(Error, Debug)]
pub enum AuthorizationError {
    #[error("Environment Error")]
    EnvError(#[from] dotenv::Error),
    #[error("An unknown database error occured")]
    SqlxDatabaseError(#[from] sqlx::Error),
    #[error("Database Error: {0}")]
    DatabaseError(#[from] crate::database::models::DatabaseError),
    #[error("Error while parsing JSON: {0}")]
    SerDeError(#[from] serde_json::Error),
    #[error("Error while communicating to GitHub OAuth2")]
    GithubError(#[from] reqwest::Error),
    #[error("Invalid Authentication credentials")]
    InvalidCredentialsError,
    #[error("Error while decoding Base62")]
    DecodingError(#[from] DecodingError),
}
impl actix_web::ResponseError for AuthorizationError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthorizationError::EnvError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthorizationError::SqlxDatabaseError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthorizationError::DatabaseError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthorizationError::SerDeError(..) => StatusCode::BAD_REQUEST,
            AuthorizationError::GithubError(..) => StatusCode::FAILED_DEPENDENCY,
            AuthorizationError::InvalidCredentialsError => StatusCode::UNAUTHORIZED,
            AuthorizationError::DecodingError(..) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ApiError {
            error: match self {
                AuthorizationError::EnvError(..) => "environment_error",
                AuthorizationError::SqlxDatabaseError(..) => "database_error",
                AuthorizationError::DatabaseError(..) => "database_error",
                AuthorizationError::SerDeError(..) => "invalid_input",
                AuthorizationError::GithubError(..) => "github_error",
                AuthorizationError::InvalidCredentialsError => "invalid_credentials",
                AuthorizationError::DecodingError(..) => "decoding_error",
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

#[derive(Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: usize,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub bio: String,
}

//http://localhost:8000/api/v1/auth/init?url=https%3A%2F%2Fmodrinth.com%2Fmods
#[get("init")]
pub async fn init(Query(info): Query<AuthorizationInit>, client: Data<PgPool>) -> Result<HttpResponse, AuthorizationError> {
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

    let client_id = dotenv::var("GITHUB_CLIENT_ID")?;
    let url = format!("https://github.com/login/oauth/authorize?client_id={}&state={}&scope={}", client_id, to_base62(state.0 as u64), "%20repo%20read%3Aorg%20read%3Auser%20user%3Aemail");

    Ok(HttpResponse::PermanentRedirect()
        .header("Location", &*url)
        .json(AuthorizationInit {
            url,
        }))
}

#[get("callback")]
pub async fn auth_callback(Query(info): Query<Authorization>, client: Data<PgPool>) -> Result<HttpResponse, AuthorizationError> {
    let mut transaction = client.begin().await?;
    let state_id = parse_base62(&*info.state)?;

    let result = sqlx::query!(
            "
            SELECT url,expires FROM states
            WHERE id = $1
            ",
            state_id as i64
        )
        .fetch_one(&mut *transaction)
        .await?;

    let now = Utc::now();
    let duration = result.expires.signed_duration_since(now);

    info!("{:?}", duration.num_seconds());
    if duration.num_seconds() < 0 {
        return Err(AuthorizationError::InvalidCredentialsError);
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

    let client_id = dotenv::var("GITHUB_CLIENT_ID")?;
    let client_secret = dotenv::var("GITHUB_CLIENT_SECRET")?;

    let url = format!(
        "https://github.com/login/oauth/access_token?client_id={}&client_secret={}&code={}",
        client_id, client_secret, info.code
    );

    let client = reqwest::Client::new();

    let token : AccessToken = client
        .post(&url)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?
        .json()
        .await?;

    let user : Value = client
        .get("https://api.github.com/user")
        .header(reqwest::header::USER_AGENT, "Modrinth")
        .header(reqwest::header::AUTHORIZATION, format!("token {}", token.access_token))
        .send()
        .await?
        .json()
        .await?;

    transaction.commit().await?;

    let redirect_url = format!("{}?url={}", result.url, token.access_token);

    Ok(HttpResponse::PermanentRedirect()
        .header("Location", &*redirect_url)
        .json(AuthorizationInit {
            url: redirect_url,
        }))
}

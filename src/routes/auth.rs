use crate::models::error::ApiError;
use log::{info};
use actix_web::web::{Query, ServiceConfig, scope};
use actix_web::{get, HttpResponse};
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(auth_callback);
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
}
// "https://github.com/login/oauth/authorize?client_id=3acffb2e808d16d4b226&redirect_uri=http%3A%2F%2Flocalhost%3A8000%2Fapi%2Fv1%2Fauthcallback"
impl actix_web::ResponseError for AuthorizationError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthorizationError::EnvError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthorizationError::SqlxDatabaseError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthorizationError::DatabaseError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthorizationError::SerDeError(..) => StatusCode::BAD_REQUEST,
            AuthorizationError::GithubError(..) => StatusCode::FAILED_DEPENDENCY,
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
            },
            description: &self.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Authorization {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AccessToken {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}

#[get("authcallback")]
pub async fn auth_callback(Query(info): Query<Authorization>) -> Result<HttpResponse, AuthorizationError> {
    let client_id = dotenv::var("GITHUB_CLIENT_ID")?;
    let client_secret = dotenv::var("GITHUB_CLIENT_SECRET")?;

    let url = format!(
        "https://github.com/login/oauth/access_token?client_id={}&client_secret={}&code={}",
        client_id, client_secret, info.code
    );

    let token : AccessToken = reqwest::Client::new()
        .post(&url)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?
        .json()
        .await?;

    Ok(HttpResponse::Ok().json(token))
}

pub mod checks;
pub mod email;
pub mod oauth;
pub mod templates;
pub mod validate;
pub use crate::auth::email::send_email;
pub use checks::{
    filter_enlisted_projects_ids, filter_enlisted_version_ids,
    filter_visible_collections, filter_visible_project_ids,
    filter_visible_projects,
};
use serde::{Deserialize, Serialize};
// pub use pat::{generate_pat, PersonalAccessToken};
pub use validate::{check_is_moderator_from_headers, get_user_from_headers};

use crate::file_hosting::FileHostingError;
use crate::models::error::ApiError;
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Environment Error")]
    Env(#[from] dotenvy::Error),
    #[error("An unknown database error occurred: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Database Error: {0}")]
    Database(#[from] crate::database::models::DatabaseError),
    #[error("Error while parsing JSON: {0}")]
    SerDe(#[from] serde_json::Error),
    #[error("Error while communicating to external provider")]
    Reqwest(#[from] reqwest::Error),
    #[error("Error uploading user profile picture")]
    FileHosting(#[from] FileHostingError),
    #[error("Error while decoding PAT: {0}")]
    Decoding(#[from] ariadne::ids::DecodingError),
    #[error("{0}")]
    Mail(#[from] email::MailError),
    #[error("Invalid Authentication Credentials")]
    InvalidCredentials,
    #[error("Authentication method was not valid")]
    InvalidAuthMethod,
    #[error("GitHub Token from incorrect Client ID")]
    InvalidClientId,
    #[error("User email/account is already registered on Modrinth")]
    DuplicateUser,
    #[error("Invalid state sent, you probably need to get a new websocket")]
    SocketError,
    #[error("Invalid callback URL specified")]
    Url,
}

impl actix_web::ResponseError for AuthenticationError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthenticationError::Env(..) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthenticationError::Sqlx(..) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthenticationError::Database(..) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AuthenticationError::SerDe(..) => StatusCode::BAD_REQUEST,
            AuthenticationError::Reqwest(..) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AuthenticationError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthenticationError::Decoding(..) => StatusCode::BAD_REQUEST,
            AuthenticationError::Mail(..) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthenticationError::InvalidAuthMethod => StatusCode::UNAUTHORIZED,
            AuthenticationError::InvalidClientId => StatusCode::UNAUTHORIZED,
            AuthenticationError::Url => StatusCode::BAD_REQUEST,
            AuthenticationError::FileHosting(..) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AuthenticationError::DuplicateUser => StatusCode::BAD_REQUEST,
            AuthenticationError::SocketError => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ApiError {
            error: self.error_name(),
            description: self.to_string(),
        })
    }
}

impl AuthenticationError {
    pub fn error_name(&self) -> &'static str {
        match self {
            AuthenticationError::Env(..) => "environment_error",
            AuthenticationError::Sqlx(..) => "database_error",
            AuthenticationError::Database(..) => "database_error",
            AuthenticationError::SerDe(..) => "invalid_input",
            AuthenticationError::Reqwest(..) => "network_error",
            AuthenticationError::InvalidCredentials => "invalid_credentials",
            AuthenticationError::Decoding(..) => "decoding_error",
            AuthenticationError::Mail(..) => "mail_error",
            AuthenticationError::InvalidAuthMethod => "invalid_auth_method",
            AuthenticationError::InvalidClientId => "invalid_client_id",
            AuthenticationError::Url => "url_error",
            AuthenticationError::FileHosting(..) => "file_hosting",
            AuthenticationError::DuplicateUser => "duplicate_user",
            AuthenticationError::SocketError => "socket",
        }
    }
}

#[derive(
    Serialize, Deserialize, Default, Eq, PartialEq, Clone, Copy, Debug,
)]
#[serde(rename_all = "lowercase")]
pub enum AuthProvider {
    #[default]
    GitHub,
    Discord,
    Microsoft,
    GitLab,
    Google,
    Steam,
    PayPal,
}

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
use crate::labrinth_error_type;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use ariadne::i18n::I18nEnum;
use ariadne::i18n_enum;
use thiserror::Error;

// TODO add fields
#[derive(Error, Debug)]
pub enum AuthenticationError {
    // #[error("Environment Error")]
    Env(#[from] dotenvy::Error),

    // #[error("An unknown database error occurred: {0}")]
    Sqlx(#[from] sqlx::Error),

    // #[error("Database Error: {0}")]
    Database(#[from] crate::database::models::DatabaseError),

    // #[error("Error while parsing JSON: {0}")]
    SerDe(#[from] serde_json::Error),

    // #[error("Error while communicating to external provider")]
    Reqwest(#[from] reqwest::Error),

    // #[error("Error uploading user profile picture")]
    FileHosting(#[from] FileHostingError),

    // #[error("Error while decoding PAT: {0}")]
    Decoding(#[from] ariadne::ids::DecodingError),

    // #[error("{0}")]
    Mail(#[from] email::MailError),

    // #[error("Invalid Authentication Credentials")]
    InvalidCredentials,

    // #[error("Authentication method was not valid")]
    InvalidAuthMethod,

    // #[error("GitHub Token from incorrect Client ID")]
    InvalidClientId,

    // #[error(
    //     "User email is already registered on Modrinth. Try 'Forgot password' to access your account."
    // )]
    DuplicateUser,

    // #[error("Invalid state sent, you probably need to get a new websocket")]
    SocketError,

    // #[error("Invalid callback URL specified")]
    Url,
}

i18n_enum!(
    AuthenticationError,
    root_key: "error.unauthorized",
    Env(..) => "environment_error",
    Sqlx(cause) => "database_error.unknown",
    Database(cause) => "database_error",
    SerDe(cause) => "invalid_input",
    Reqwest(..) => "network_error",
    FileHosting(..) => "file_hosting",
    Decoding(cause) => "decoding_error",
    Mail(cause) => "mail_error",
    InvalidCredentials! => "invalid_credentials",
    InvalidAuthMethod! => "invalid_auth_method",
    InvalidClientId! => "invalid_client_id",
    DuplicateUser! => "duplicate_user",
    SocketError! => "socket",
    Url! => "url_error",
);

labrinth_error_type!(AuthenticationError);

impl ResponseError for AuthenticationError {
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
        HttpResponse::build(self.status_code()).json(self.as_api_error())
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

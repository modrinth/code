use crate::file_hosting::FileHostingError;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse};
use futures::FutureExt;

pub mod v2;
pub mod v3;

mod health;
mod index;
mod maven;
mod not_found;
mod updates;

pub use self::not_found::not_found;

pub fn root_config(cfg: &mut web::ServiceConfig) {
    cfg.service(index::index_get);
    cfg.service(health::health_get);
    cfg.service(web::scope("maven").configure(maven::config));
    cfg.service(web::scope("updates").configure(updates::config));
    cfg.service(
        web::scope("api/v1").wrap_fn(|req, _srv| {
            async {
                Ok(req.into_response(
                    HttpResponse::Gone()
                        .content_type("application/json")
                        .body(r#"{"error":"api_deprecated","description":"You are using an application that uses an outdated version of Modrinth's API. Please either update it or switch to another application. For developers: https://docs.modrinth.com/docs/migrations/v1-to-v2/"}"#)
                ))
            }.boxed_local()
        })
    );
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Environment Error")]
    Env(#[from] dotenvy::Error),
    #[error("Error while uploading file")]
    FileHosting(#[from] FileHostingError),
    #[error("Database Error: {0}")]
    Database(#[from] crate::database::models::DatabaseError),
    #[error("Database Error: {0}")]
    SqlxDatabase(#[from] sqlx::Error),
    #[error("Internal server error: {0}")]
    Xml(String),
    #[error("Deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Authentication Error: {0}")]
    Authentication(#[from] crate::util::auth::AuthenticationError),
    #[error("Authentication Error: {0}")]
    CustomAuthentication(String),
    #[error("Invalid Input: {0}")]
    InvalidInput(String),
    #[error("Error while validating input: {0}")]
    Validation(String),
    #[error("Search Error: {0}")]
    Search(#[from] meilisearch_sdk::errors::Error),
    #[error("Indexing Error: {0}")]
    Indexing(#[from] crate::search::indexing::IndexingError),
    #[error("Ariadne Error: {0}")]
    Analytics(String),
    #[error("Crypto Error: {0}")]
    Crypto(String),
    #[error("Payments Error: {0}")]
    Payments(String),
    #[error("Discord Error: {0}")]
    DiscordError(String),
    #[error("Error while decoding Base62: {0}")]
    Decoding(#[from] crate::models::ids::DecodingError),
    #[error("Image Parsing Error: {0}")]
    ImageError(#[from] image::ImageError),
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Env(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Database(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::SqlxDatabase(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Authentication(..) => StatusCode::UNAUTHORIZED,
            ApiError::CustomAuthentication(..) => StatusCode::UNAUTHORIZED,
            ApiError::Xml(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Json(..) => StatusCode::BAD_REQUEST,
            ApiError::Search(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Indexing(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::FileHosting(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InvalidInput(..) => StatusCode::BAD_REQUEST,
            ApiError::Validation(..) => StatusCode::BAD_REQUEST,
            ApiError::Analytics(..) => StatusCode::FAILED_DEPENDENCY,
            ApiError::Crypto(..) => StatusCode::FORBIDDEN,
            ApiError::Payments(..) => StatusCode::FAILED_DEPENDENCY,
            ApiError::DiscordError(..) => StatusCode::FAILED_DEPENDENCY,
            ApiError::Decoding(..) => StatusCode::BAD_REQUEST,
            ApiError::ImageError(..) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(
            crate::models::error::ApiError {
                error: match self {
                    ApiError::Env(..) => "environment_error",
                    ApiError::SqlxDatabase(..) => "database_error",
                    ApiError::Database(..) => "database_error",
                    ApiError::Authentication(..) => "unauthorized",
                    ApiError::CustomAuthentication(..) => "unauthorized",
                    ApiError::Xml(..) => "xml_error",
                    ApiError::Json(..) => "json_error",
                    ApiError::Search(..) => "search_error",
                    ApiError::Indexing(..) => "indexing_error",
                    ApiError::FileHosting(..) => "file_hosting_error",
                    ApiError::InvalidInput(..) => "invalid_input",
                    ApiError::Validation(..) => "invalid_input",
                    ApiError::Analytics(..) => "analytics_error",
                    ApiError::Crypto(..) => "crypto_error",
                    ApiError::Payments(..) => "payments_error",
                    ApiError::DiscordError(..) => "discord_error",
                    ApiError::Decoding(..) => "decoding_error",
                    ApiError::ImageError(..) => "invalid_image",
                },
                description: &self.to_string(),
            },
        )
    }
}

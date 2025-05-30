use crate::file_hosting::FileHostingError;
use crate::routes::analytics::{page_view_ingest, playtime_ingest};
use crate::util::cors::default_cors;
use crate::util::env::parse_strings_from_var;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, web};
use futures::FutureExt;

pub mod internal;
pub mod v2;
pub mod v3;

#[cfg(target_os = "linux")]
pub mod debug;

pub mod v2_reroute;

mod analytics;
mod index;
mod maven;
mod not_found;
mod updates;

pub use self::not_found::not_found;

pub fn root_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("maven")
            .wrap(default_cors())
            .configure(maven::config),
    );
    cfg.service(
        web::scope("updates")
            .wrap(default_cors())
            .configure(updates::config),
    );
    cfg.service(
        web::scope("analytics")
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|origin, _req_head| {
                        let allowed_origins =
                            parse_strings_from_var("ANALYTICS_ALLOWED_ORIGINS")
                                .unwrap_or_default();

                        allowed_origins.contains(&"*".to_string())
                            || allowed_origins.contains(
                                &origin
                                    .to_str()
                                    .unwrap_or_default()
                                    .to_string(),
                            )
                    })
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::ACCEPT,
                        actix_web::http::header::CONTENT_TYPE,
                    ])
                    .max_age(3600),
            )
            .service(page_view_ingest)
            .service(playtime_ingest),
    );
    cfg.service(
        web::scope("api/v1")
            .wrap(default_cors())
            .wrap_fn(|req, _srv| {
            async {
                Ok(req.into_response(
                    HttpResponse::Gone()
                        .content_type("application/json")
                        .body(r#"{"error":"api_deprecated","description":"You are using an application that uses an outdated version of Modrinth's API. Please either update it or switch to another application. For developers: https://docs.modrinth.com/api/#versioning"}"#)
                ))
            }.boxed_local()
        })
    );
    cfg.service(
        web::scope("")
            .wrap(default_cors())
            .service(index::index_get)
            .service(Files::new("/", "assets/")),
    );
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Environment Error")]
    Env(#[from] dotenvy::Error),
    #[error("Error while uploading file: {0}")]
    FileHosting(#[from] FileHostingError),
    #[error("Database Error: {0}")]
    Database(#[from] crate::database::models::DatabaseError),
    #[error("Database Error: {0}")]
    SqlxDatabase(#[from] sqlx::Error),
    #[error("Database Error: {0}")]
    RedisDatabase(#[from] redis::RedisError),
    #[error("Clickhouse Error: {0}")]
    Clickhouse(#[from] clickhouse::error::Error),
    #[error("Internal server error: {0}")]
    Xml(String),
    #[error("Deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Authentication Error: {0}")]
    Authentication(#[from] crate::auth::AuthenticationError),
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
    #[error("Payments Error: {0}")]
    Payments(String),
    #[error("Discord Error: {0}")]
    Discord(String),
    #[error("Captcha Error. Try resubmitting the form.")]
    Turnstile,
    #[error("Error while decoding Base62: {0}")]
    Decoding(#[from] ariadne::ids::DecodingError),
    #[error("Image Parsing Error: {0}")]
    ImageParse(#[from] image::ImageError),
    #[error("Password Hashing Error: {0}")]
    PasswordHashing(#[from] argon2::password_hash::Error),
    #[error("{0}")]
    Mail(#[from] crate::auth::email::MailError),
    #[error("Error while rerouting request: {0}")]
    Reroute(#[from] reqwest::Error),
    #[error("Unable to read Zip Archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Resource not found")]
    NotFound,
    #[error(
        "You are being rate-limited. Please wait {0} milliseconds. 0/{1} remaining."
    )]
    RateLimitError(u128, u32),
    #[error("Error while interacting with payment processor: {0}")]
    Stripe(#[from] stripe::StripeError),
}

impl ApiError {
    pub fn as_api_error<'a>(&self) -> crate::models::error::ApiError<'a> {
        crate::models::error::ApiError {
            error: match self {
                ApiError::Env(..) => "environment_error",
                ApiError::Database(..) => "database_error",
                ApiError::SqlxDatabase(..) => "database_error",
                ApiError::RedisDatabase(..) => "database_error",
                ApiError::Authentication(..) => "unauthorized",
                ApiError::CustomAuthentication(..) => "unauthorized",
                ApiError::Xml(..) => "xml_error",
                ApiError::Json(..) => "json_error",
                ApiError::Search(..) => "search_error",
                ApiError::Indexing(..) => "indexing_error",
                ApiError::FileHosting(..) => "file_hosting_error",
                ApiError::InvalidInput(..) => "invalid_input",
                ApiError::Validation(..) => "invalid_input",
                ApiError::Payments(..) => "payments_error",
                ApiError::Discord(..) => "discord_error",
                ApiError::Turnstile => "turnstile_error",
                ApiError::Decoding(..) => "decoding_error",
                ApiError::ImageParse(..) => "invalid_image",
                ApiError::PasswordHashing(..) => "password_hashing_error",
                ApiError::Mail(..) => "mail_error",
                ApiError::Clickhouse(..) => "clickhouse_error",
                ApiError::Reroute(..) => "reroute_error",
                ApiError::NotFound => "not_found",
                ApiError::Zip(..) => "zip_error",
                ApiError::Io(..) => "io_error",
                ApiError::RateLimitError(..) => "ratelimit_error",
                ApiError::Stripe(..) => "stripe_error",
            },
            description: self.to_string(),
        }
    }
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Env(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Database(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::SqlxDatabase(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::RedisDatabase(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Clickhouse(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Authentication(..) => StatusCode::UNAUTHORIZED,
            ApiError::CustomAuthentication(..) => StatusCode::UNAUTHORIZED,
            ApiError::Xml(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Json(..) => StatusCode::BAD_REQUEST,
            ApiError::Search(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Indexing(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::FileHosting(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InvalidInput(..) => StatusCode::BAD_REQUEST,
            ApiError::Validation(..) => StatusCode::BAD_REQUEST,
            ApiError::Payments(..) => StatusCode::FAILED_DEPENDENCY,
            ApiError::Discord(..) => StatusCode::FAILED_DEPENDENCY,
            ApiError::Turnstile => StatusCode::BAD_REQUEST,
            ApiError::Decoding(..) => StatusCode::BAD_REQUEST,
            ApiError::ImageParse(..) => StatusCode::BAD_REQUEST,
            ApiError::PasswordHashing(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Mail(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Reroute(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::Zip(..) => StatusCode::BAD_REQUEST,
            ApiError::Io(..) => StatusCode::BAD_REQUEST,
            ApiError::RateLimitError(..) => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Stripe(..) => StatusCode::FAILED_DEPENDENCY,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.as_api_error())
    }
}

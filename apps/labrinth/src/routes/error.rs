use crate::database::models::loader_fields::VersionFieldParseError;
use crate::file_hosting::FileHostingError;
use crate::models::error::AsApiError;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use ariadne::i18n_enum;

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

    // TODO: Use an I18nEnum instead of a String
    #[error("Internal server error: {0}")]
    Xml(String),

    #[error("Deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Authentication Error: {0}")]
    Authentication(#[from] crate::auth::AuthenticationError),

    // TODO: Use an I18nEnum instead of a String
    #[error("Authentication Error: {0}")]
    CustomAuthentication(String),

    // TODO: Use an I18nEnum instead of a String
    #[error("Invalid Input: {0}")]
    InvalidInput(String),

    #[error("Invalid Input: {0}")]
    InvalidLoaderField(#[from] VersionFieldParseError),

    // TODO: Perhaps remove this in favor of InvalidInput?
    #[error("Error while validating input: {0}")]
    Validation(String),

    #[error("Search Error: {0}")]
    Search(#[from] meilisearch_sdk::errors::Error),

    #[error("Indexing Error: {0}")]
    Indexing(#[from] crate::search::indexing::IndexingError),

    // TODO: Use an I18nEnum instead of a String
    #[error("Payments Error: {0}")]
    Payments(String),

    // TODO: Use an I18nEnum instead of a String
    #[error("Discord Error: {0}")]
    Discord(String),

    #[error("Slack Webhook Error: Error while sending projects webhook")]
    Slack,

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

    #[error("The requested route does not exist")]
    RouteNotFound,

    // TODO: Use an I18nEnum instead of a String
    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("External tax compliance API Error")]
    TaxComplianceApi,

    #[error(
        "You are being rate-limited. Please wait {0} milliseconds. 0/{1} remaining."
    )]
    RateLimitError(u128, u32),

    #[error("Error while interacting with payment processor: {0}")]
    Stripe(#[from] stripe::StripeError),
}

i18n_enum!(
    ApiError,
    root_key: "labrinth.error",
    Env(..) => "environment_error",
    FileHosting(cause) => "file_hosting_error",
    Database(cause) => "database_error",
    SqlxDatabase(cause) => "database_error",
    RedisDatabase(cause) => "database_error",
    Clickhouse(cause) => "clickhouse_error",
    Xml(cause) => "xml_error",
    Json(cause) => "json_error",
    Authentication(cause) => "unauthorized",
    CustomAuthentication(cause) => "unauthorized",
    InvalidInput(cause) => "invalid_input",
    InvalidLoaderField(cause) => "invalid_input",
    Validation(cause) => "invalid_input.validation",
    Search(cause) => "search_error",
    Indexing(cause) => "indexing_error",
    Payments(cause) => "payments_error",
    Discord(cause) => "discord_error",
    Slack! => "slack_error",
    Turnstile! => "turnstile_error",
    Decoding(cause) => "decoding_error",
    ImageParse(cause) => "invalid_image",
    PasswordHashing(cause) => "password_hashing_error",
    Mail(transparent cause) => "mail_error",
    Reroute(cause) => "reroute_error",
    Zip(cause) => "zip_error",
    Io(cause) => "io_error",
    NotFound! => "not_found",
    RouteNotFound! => "not_found.route",
    Conflict(cause) => "conflict",
    TaxComplianceApi! => "tax_compliance_api_error",
    RateLimitError(wait_ms, total_allowed_requests) => "ratelimit_error",
    Stripe(cause) => "stripe_error",
);

impl ResponseError for ApiError {
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
            ApiError::InvalidLoaderField(..) => StatusCode::BAD_REQUEST,
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
            ApiError::RouteNotFound => StatusCode::NOT_FOUND,
            ApiError::Conflict(..) => StatusCode::CONFLICT,
            ApiError::TaxComplianceApi => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Zip(..) => StatusCode::BAD_REQUEST,
            ApiError::Io(..) => StatusCode::BAD_REQUEST,
            ApiError::RateLimitError(..) => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Stripe(..) => StatusCode::FAILED_DEPENDENCY,
            ApiError::Slack => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.as_api_error())
    }
}

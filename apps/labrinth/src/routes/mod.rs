use crate::database::models::DelphiReportIssueDetailsId;
use crate::file_hosting::FileHostingError;
use crate::routes::analytics::{page_view_ingest, playtime_ingest};
use crate::util::cors::default_cors;
use crate::util::env::parse_strings_from_var;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, web};
use futures::FutureExt;
use serde_json::json;

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

/// Error when calling an HTTP endpoint.
#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    /// Error occurred on the server side, which the caller has no fault in.
    #[error(transparent)]
    Internal(eyre::Report),
    /// Caller made an invalid or malformed request.
    #[error(transparent)]
    Request(eyre::Report),
    /// Caller attempted a request which they are not allowed to make.
    #[error(transparent)]
    Auth(eyre::Report),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Environment error")]
    Env(#[from] dotenvy::Error),
    #[error("Error while uploading file: {0}")]
    FileHosting(#[from] FileHostingError),
    #[error("Database error: {0}")]
    Database(#[from] crate::database::models::DatabaseError),
    #[error("SQLx database error: {0}")]
    SqlxDatabase(#[from] sqlx::Error),
    #[error("Redis database error: {0}")]
    RedisDatabase(#[from] redis::RedisError),
    #[error("Clickhouse error: {0}")]
    Clickhouse(#[from] clickhouse::error::Error),
    #[error("XML error: {0}")]
    Xml(String),
    #[error("Deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Authentication error: {0}")]
    Authentication(#[from] crate::auth::AuthenticationError),
    #[error("Authentication error: {0}")]
    CustomAuthentication(String),
    #[error("Error while validating input: {0}")]
    Validation(String),
    #[error("Search error: {0}")]
    Search(#[from] meilisearch_sdk::errors::Error),
    #[error("Indexing error: {0}")]
    Indexing(#[from] crate::search::indexing::IndexingError),
    #[error("Payments error: {0}")]
    Payments(String),
    #[error("Discord error: {0}")]
    Discord(String),
    #[error("Slack webhook error: {0}")]
    Slack(String),
    #[error("Captcha error. Try resubmitting the form.")]
    Turnstile,
    #[error("Error while decoding Base62: {0}")]
    Decoding(#[from] ariadne::ids::DecodingError),
    #[error("Image parsing error: {0}")]
    ImageParse(#[from] image::ImageError),
    #[error("Password hashing error: {0}")]
    PasswordHashing(#[from] argon2::password_hash::Error),
    #[error("{0}")]
    Mail(#[from] crate::queue::email::MailError),
    #[error("Error while rerouting request: {0:?}")]
    Reroute(#[from] reqwest::Error),
    #[error("Unable to read zip archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Resource not found")]
    NotFound,
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("External tax compliance API error")]
    TaxComplianceApi,
    #[error(transparent)]
    TaxProcessor(#[from] crate::util::anrok::AnrokError),
    #[error(
        "You are being rate-limited. Please wait {0} milliseconds. 0/{1} remaining."
    )]
    RateLimitError(u128, u32),
    #[error("Error while interacting with payment processor: {0}")]
    Stripe(#[from] stripe::StripeError),
    #[error("Error while interacting with Delphi: {0}")]
    Delphi(reqwest::Error),
    #[error(transparent)]
    Mural(#[from] Box<muralpay::ApiError>),
    #[error("report still has {} issue details with no verdict", details.len())]
    TechReviewDetailsWithNoVerdict {
        details: Vec<DelphiReportIssueDetailsId>,
    },
}

impl ApiError {
    pub fn as_api_error<'a>(&self) -> crate::models::error::ApiError<'a> {
        crate::models::error::ApiError {
            error: match self {
                Self::Internal(..) => "internal_error",
                Self::Request(..) => "request_error",
                Self::Auth(..) => "auth_error",
                Self::Env(..) => "environment_error",
                Self::Database(..) => "database_error",
                Self::SqlxDatabase(..) => "database_error",
                Self::RedisDatabase(..) => "database_error",
                Self::Authentication(..) => "unauthorized",
                Self::CustomAuthentication(..) => "unauthorized",
                Self::Xml(..) => "xml_error",
                Self::Json(..) => "json_error",
                Self::Search(..) => "search_error",
                Self::Indexing(..) => "indexing_error",
                Self::FileHosting(..) => "file_hosting_error",
                Self::InvalidInput(..) => "invalid_input",
                Self::Validation(..) => "invalid_input",
                Self::Payments(..) => "payments_error",
                Self::Discord(..) => "discord_error",
                Self::Turnstile => "turnstile_error",
                Self::Decoding(..) => "decoding_error",
                Self::ImageParse(..) => "invalid_image",
                Self::PasswordHashing(..) => "password_hashing_error",
                Self::Mail(..) => "mail_error",
                Self::Clickhouse(..) => "clickhouse_error",
                Self::Reroute(..) => "reroute_error",
                Self::NotFound => "not_found",
                Self::Conflict(..) => "conflict",
                Self::TaxComplianceApi => "tax_compliance_api_error",
                Self::Zip(..) => "zip_error",
                Self::Io(..) => "io_error",
                Self::RateLimitError(..) => "ratelimit_error",
                Self::Stripe(..) => "stripe_error",
                Self::TaxProcessor(..) => "tax_processor_error",
                Self::Slack(..) => "slack_error",
                Self::Delphi(..) => "delphi_error",
                Self::Mural(..) => "mural_error",
                Self::TechReviewDetailsWithNoVerdict { .. } => {
                    "tech_review_issues_with_no_verdict"
                }
            },
            description: match self {
                Self::Internal(e) => format!("{e:#?}"),
                Self::Request(e) => format!("{e:#?}"),
                Self::Auth(e) => format!("{e:#?}"),
                _ => self.to_string(),
            },
            details: match self {
                Self::Mural(err) => serde_json::to_value(err.clone()).ok(),
                Self::TechReviewDetailsWithNoVerdict { details } => {
                    let details = serde_json::to_value(details)
                        .expect("details should never fail to serialize");
                    Some(json!({
                        "issue_details": details
                    }))
                }
                _ => None,
            },
        }
    }
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Internal(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Request(..) => StatusCode::BAD_REQUEST,
            Self::Auth(..) => StatusCode::UNAUTHORIZED,
            Self::InvalidInput(..) => StatusCode::BAD_REQUEST,
            Self::Env(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SqlxDatabase(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::RedisDatabase(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Clickhouse(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Authentication(..) => StatusCode::UNAUTHORIZED,
            Self::CustomAuthentication(..) => StatusCode::UNAUTHORIZED,
            Self::Xml(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Json(..) => StatusCode::BAD_REQUEST,
            Self::Search(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Indexing(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FileHosting(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation(..) => StatusCode::BAD_REQUEST,
            Self::Payments(..) => StatusCode::FAILED_DEPENDENCY,
            Self::Discord(..) => StatusCode::FAILED_DEPENDENCY,
            Self::Turnstile => StatusCode::BAD_REQUEST,
            Self::Decoding(..) => StatusCode::BAD_REQUEST,
            Self::ImageParse(..) => StatusCode::BAD_REQUEST,
            Self::PasswordHashing(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Mail(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Reroute(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict(..) => StatusCode::CONFLICT,
            Self::TaxComplianceApi => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Zip(..) => StatusCode::BAD_REQUEST,
            Self::Io(..) => StatusCode::BAD_REQUEST,
            Self::RateLimitError(..) => StatusCode::TOO_MANY_REQUESTS,
            Self::Stripe(..) => StatusCode::FAILED_DEPENDENCY,
            Self::TaxProcessor(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Slack(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Delphi(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Mural(..) => StatusCode::BAD_REQUEST,
            Self::TechReviewDetailsWithNoVerdict { .. } => {
                StatusCode::BAD_REQUEST
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.as_api_error())
    }
}

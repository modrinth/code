use crate::file_hosting::FileHostingError;
use crate::routes::analytics::{page_view_ingest, playtime_ingest};
use crate::util::cors::default_cors;
use crate::util::env::parse_strings_from_var;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::http::StatusCode;
use actix_web::http::header::{AcceptLanguage, Header, LanguageTag};
use actix_web::{HttpRequest, HttpResponse, ResponseError, web};
use ariadne::i18n::I18nEnum;
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

#[derive(thiserror::Error, I18nEnum, Debug)]
#[i18n_root_key("error")]
pub enum ApiError {
    #[translation_id("environment_error")]
    // #[error("Environment Error")]
    Env(#[from] dotenvy::Error),

    #[translation_id("file_hosting_error")]
    #[translate_fields(cause = translate(0))]
    // #[error("Error while uploading file: {0}")]
    FileHosting(#[from] FileHostingError),

    #[translation_id("database_error")]
    #[translate_fields(cause = translate(0))]
    // #[error("Database Error: {0}")]
    Database(#[from] crate::database::models::DatabaseError),

    #[translation_id("database_error")]
    #[translate_fields(cause = 0)]
    // #[error("Database Error: {0}")]
    SqlxDatabase(#[from] sqlx::Error),

    #[translation_id("database_error")]
    #[translate_fields(cause = 0)]
    // #[error("Database Error: {0}")]
    RedisDatabase(#[from] redis::RedisError),

    #[translation_id("clickhouse_error")]
    #[translate_fields(cause = 0)]
    // #[error("Clickhouse Error: {0}")]
    Clickhouse(#[from] clickhouse::error::Error),

    #[translation_id("xml_error")]
    #[translate_fields(cause = 0)]
    // TODO: Use an I18nEnum instead of a String
    // #[error("Internal server error: {0}")]
    Xml(String),

    #[translation_id("json_error")]
    #[translate_fields(cause = 0)]
    // #[error("Deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[translation_id("unauthorized")]
    #[translate_fields(cause = translate(0))]
    // #[error("Authentication Error: {0}")]
    Authentication(#[from] crate::auth::AuthenticationError),

    #[translation_id("unauthorized")]
    #[translate_fields(cause = 0)]
    // TODO: Use an I18nEnum instead of a String
    // #[error("Authentication Error: {0}")]
    CustomAuthentication(String),

    #[translation_id("invalid_input")]
    #[translate_fields(cause = 0)]
    // TODO: Use an I18nEnum instead of a String
    // #[error("Invalid Input: {0}")]
    InvalidInput(String),

    // TODO: Perhaps remove this in favor of InvalidInput?
    #[translation_id("invalid_input")]
    #[translate_fields(cause = 0)]
    // #[error("Error while validating input: {0}")]
    Validation(String),

    #[translation_id("search_error")]
    #[translate_fields(cause = 0)]
    // #[error("Search Error: {0}")]
    Search(#[from] meilisearch_sdk::errors::Error),

    #[translation_id("indexing_error")]
    #[translate_fields(cause = translate(0))]
    // #[error("Indexing Error: {0}")]
    Indexing(#[from] crate::search::indexing::IndexingError),

    #[translation_id("payments_error")]
    #[translate_fields(cause = 0)]
    // TODO: Use an I18nEnum instead of a String
    // #[error("Payments Error: {0}")]
    Payments(String),

    #[translation_id("discord_error")]
    #[translate_fields(cause = 0)]
    // TODO: Use an I18nEnum instead of a String
    // #[error("Discord Error: {0}")]
    Discord(String),

    #[translation_id("turnstile_error")]
    // #[error("Captcha Error. Try resubmitting the form.")]
    Turnstile,

    #[translation_id("decoding_error")]
    #[translate_fields(cause = translate(0))]
    // #[error("Error while decoding Base62: {0}")]
    Decoding(#[from] ariadne::ids::DecodingError),

    #[translation_id("invalid_image")]
    #[translate_fields(cause = 0)]
    // #[error("Image Parsing Error: {0}")]
    ImageParse(#[from] image::ImageError),

    #[translation_id("password_hashing_error")]
    #[translate_fields(cause = 0)]
    // #[error("Password Hashing Error: {0}")]
    PasswordHashing(#[from] argon2::password_hash::Error),

    #[translation_id("mail_error")]
    #[translate_fields(cause = translate(0))]
    // #[error("{0}")]
    Mail(#[from] crate::auth::email::MailError),

    #[translation_id("reroute_error")]
    #[translate_fields(cause = 0)]
    // #[error("Error while rerouting request: {0}")]
    Reroute(#[from] reqwest::Error),

    #[translation_id("zip_error")]
    #[translate_fields(cause = 0)]
    // #[error("Unable to read Zip Archive: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[translation_id("io_error")]
    #[translate_fields(cause = 0)]
    // #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    // TODO: Add route not found
    #[translation_id("not_found")]
    // #[error("Resource not found")]
    NotFound,

    #[translation_id("conflict")]
    #[translate_fields(cause = 0)]
    // TODO: Use an I18nEnum instead of a String
    // #[error("Conflict: {0}")]
    Conflict(String),

    #[translation_id("tax_compliance_api_error")]
    // #[error("External tax compliance API Error")]
    TaxComplianceApi,

    #[translation_id("ratelimit_error")]
    #[translate_fields(wait_ms = 0, total_allowed_requests = 1)]
    // #[error(
    //     "You are being rate-limited. Please wait {0} milliseconds. 0/{1} remaining."
    // )]
    RateLimitError(u128, u32),

    #[translation_id("stripe_error")]
    #[translate_fields(cause = 0)]
    // #[error("Error while interacting with payment processor: {0}")]
    Stripe(#[from] stripe::StripeError),
}

#[macro_export]
macro_rules! labrinth_error_type {
    ($error_enum:ty) => {
        impl $error_enum {
            pub fn as_api_error<'a>(
                &self,
            ) -> $crate::models::error::ApiError<'a> {
                self.as_localized_api_error("en")
            }

            pub fn as_localized_api_error<'a>(
                &self,
                language: &str,
            ) -> crate::models::error::ApiError<'a> {
                $crate::models::error::ApiError {
                    error: $crate::routes::error_id_for_error(self),
                    description: self.translated_message(language),
                }
            }

            pub fn localized_error_response(
                &self,
                req: &actix_web::HttpRequest,
            ) -> actix_web::HttpResponse {
                use actix_web::http::header::{ContentLanguage, QualityItem};

                let language = $crate::routes::parse_accept_language(req);
                let body = self.as_localized_api_error(language.as_str());

                actix_web::HttpResponse::build(self.status_code())
                    .append_header(ContentLanguage(vec![QualityItem::max(
                        language,
                    )]))
                    .json(body)
            }
        }
    };
}

labrinth_error_type!(ApiError);

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
            ApiError::Conflict(..) => StatusCode::CONFLICT,
            ApiError::TaxComplianceApi => StatusCode::INTERNAL_SERVER_ERROR,
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

pub fn parse_accept_language(req: &HttpRequest) -> LanguageTag {
    AcceptLanguage::parse(req)
        .ok()
        .and_then(|x| x.preference().into_item())
        .unwrap_or_else(|| LanguageTag::parse("en").unwrap())
}

pub fn error_id_for_error(error: &impl I18nEnum) -> &'static str {
    let translation_id = error.translation_id();
    translation_id
        .split_once('.')
        .map_or(translation_id, |(base, _)| base)
}

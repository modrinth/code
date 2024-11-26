use crate::file_hosting::FileHostingError;
use crate::routes::analytics::{page_view_ingest, playtime_ingest};
use crate::util::cors::default_cors;
use crate::util::env::parse_strings_from_var;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse};
use futures::FutureExt;

pub mod internal;
pub mod v2;
pub mod v3;

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
                            .body(r#"{"error":"api_deprecated","description":"您正在使用一个使用过时版本的 Modrinth API 的应用程序。请更新它或切换到另一个应用程序。对于开发人员：https://docs.modrinth.com/api/#versioning"}"#)
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
    #[error("运行环境错误")]
    Env(#[from] dotenvy::Error),
    #[error("上传文件时出错： {0}")]
    FileHosting(#[from] FileHostingError),
    #[error("数据库错误: {0}")]
    Database(#[from] crate::database::models::DatabaseError),
    #[error("数据库错误: {0}")]
    SqlxDatabase(#[from] sqlx::Error),
    #[error("Clickhouse数据库错误: {0}")]
    Clickhouse(#[from] clickhouse::error::Error),
    #[error("服务器内部错误: {0}")]
    Xml(String),
    #[error("反序列化错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("身份验证错误: {0}")]
    Authentication(#[from] crate::auth::AuthenticationError),
    #[error("自定义身份验证错误: {0}")]
    CustomAuthentication(String),
    #[error("无效输入: {0}")]
    InvalidInput(String),
    #[error("验证输入时出错: {0}")]
    Validation(String),
    #[error("搜索出错: {0}")]
    Search(#[from] meilisearch_sdk::errors::Error),
    #[error("索引错误: {0}")]
    Indexing(#[from] crate::search::indexing::IndexingError),
    #[error("付款错误: {0}")]
    Payments(String),
    #[error("Discord 错误: {0}")]
    Discord(String),
    #[error("验证码错误。请尝试重新提交。")]
    Turnstile,
    #[error("解码 Base62 时出错: {0}")]
    Decoding(#[from] crate::models::ids::DecodingError),
    #[error("图片解析错误: {0}")]
    ImageParse(#[from] image::ImageError),
    #[error("密码哈希错误: {0}")]
    PasswordHashing(#[from] argon2::password_hash::Error),
    #[error("密码强度检查错误: {0}")]
    PasswordStrengthCheck(#[from] zxcvbn::ZxcvbnError),
    #[error("{0}")]
    Mail(#[from] crate::auth::email::MailError),
    #[error("重新路由请求时出错: {0}")]
    Reroute(#[from] reqwest::Error),
    #[error("无法读取 Zip 存档: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("资源未找到")]
    NotFound,
    #[error("您已被限速。请等待 {0} 毫秒。0/{1} 剩余。")]
    RateLimitError(u128, u32),
    #[error("与支付处理器交互时出错: {0}")]
    Stripe(#[from] stripe::StripeError),
}

impl ApiError {
    pub fn as_api_error<'a>(&self) -> crate::models::error::ApiError<'a> {
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
                ApiError::Payments(..) => "payments_error",
                ApiError::Discord(..) => "discord_error",
                ApiError::Turnstile => "turnstile_error",
                ApiError::Decoding(..) => "decoding_error",
                ApiError::ImageParse(..) => "invalid_image",
                ApiError::PasswordHashing(..) => "password_hashing_error",
                ApiError::PasswordStrengthCheck(..) => "strength_check_error",
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
            ApiError::PasswordStrengthCheck(..) => StatusCode::BAD_REQUEST,
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
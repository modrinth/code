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
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("环境错误")]
    Env(#[from] dotenvy::Error),
    #[error("发生未知的数据库错误: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("数据库错误: {0}")]
    Database(#[from] crate::database::models::DatabaseError),
    #[error("解析 JSON 时出错: {0}")]
    SerDe(#[from] serde_json::Error),
    #[error("与外部提供商通信时出错: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("上传用户头像时出错")]
    FileHosting(#[from] FileHostingError),
    #[error("解码 PAT 时出错: {0}")]
    Decoding(#[from] crate::models::ids::DecodingError),
    #[error("{0}")]
    Mail(#[from] email::MailError),
    #[error("无效的认证凭据")]
    InvalidCredentials,
    #[error("认证方法无效")]
    InvalidAuthMethod,
    #[error("GitHub 令牌来自错误的客户端 ID")]
    InvalidClientId,
    #[error("用户邮箱/账户已在 BBSMC 注册")]
    DuplicateUser,
    #[error("发送的状态无效，可能需要获取新的 websocket")]
    SocketError,
    #[error("指定的回调 URL 无效")]
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

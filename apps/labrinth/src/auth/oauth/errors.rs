use super::ValidatedRedirectUri;
use crate::auth::AuthenticationError;
use crate::models::error::ApiError;
use crate::models::ids::DecodingError;
use actix_web::http::{header::LOCATION, StatusCode};
use actix_web::HttpResponse;

#[derive(thiserror::Error, Debug)]
#[error("{}", .error_type)]
pub struct OAuthError {
    #[source]
    pub error_type: OAuthErrorType,

    pub state: Option<String>,
    pub valid_redirect_uri: Option<ValidatedRedirectUri>,
}

impl<T> From<T> for OAuthError
where
    T: Into<OAuthErrorType>,
{
    fn from(value: T) -> Self {
        OAuthError::error(value.into())
    }
}

impl OAuthError {
    /// OAuth 请求失败，可能是因为无效的重定向 URI
    /// 或者在我们验证给定的 URI 之前失败，因此直接向调用者返回错误
    ///
    /// 参见：IETF RFC 6749 4.1.2.1 (https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1)
    pub fn error(error_type: impl Into<OAuthErrorType>) -> Self {
        Self {
            error_type: error_type.into(),
            valid_redirect_uri: None,
            state: None,
        }
    }

    /// OAuth 请求因无效的重定向 URI 以外的原因失败
    /// 因此将错误以 URL 编码的形式发送到重定向 URI
    ///
    /// 参见：IETF RFC 6749 4.1.2.1 (https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1)
    pub fn redirect(
        err: impl Into<OAuthErrorType>,
        state: &Option<String>,
        valid_redirect_uri: &ValidatedRedirectUri,
    ) -> Self {
        Self {
            error_type: err.into(),
            state: state.clone(),
            valid_redirect_uri: Some(valid_redirect_uri.clone()),
        }
    }
}

impl actix_web::ResponseError for OAuthError {
    fn status_code(&self) -> StatusCode {
        match self.error_type {
            OAuthErrorType::AuthenticationError(_)
            | OAuthErrorType::FailedScopeParse(_)
            | OAuthErrorType::ScopesTooBroad
            | OAuthErrorType::AccessDenied => {
                if self.valid_redirect_uri.is_some() {
                    StatusCode::OK
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
            OAuthErrorType::RedirectUriNotConfigured(_)
            | OAuthErrorType::ClientMissingRedirectURI { client_id: _ }
            | OAuthErrorType::InvalidAcceptFlowId
            | OAuthErrorType::MalformedId(_)
            | OAuthErrorType::InvalidClientId(_)
            | OAuthErrorType::InvalidAuthCode
            | OAuthErrorType::OnlySupportsAuthorizationCodeGrant(_)
            | OAuthErrorType::RedirectUriChanged(_)
            | OAuthErrorType::UnauthorizedClient => StatusCode::BAD_REQUEST,
            OAuthErrorType::ClientAuthenticationFailed => {
                StatusCode::UNAUTHORIZED
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        if let Some(ValidatedRedirectUri(mut redirect_uri)) =
            self.valid_redirect_uri.clone()
        {
            redirect_uri = format!(
                "{}?error={}&error_description={}",
                redirect_uri,
                self.error_type.error_name(),
                self.error_type,
            );

            if let Some(state) = self.state.as_ref() {
                redirect_uri = format!("{}&state={}", redirect_uri, state);
            }

            HttpResponse::Ok()
                .append_header((LOCATION, redirect_uri.clone()))
                .body(redirect_uri)
        } else {
            HttpResponse::build(self.status_code()).json(ApiError {
                error: &self.error_type.error_name(),
                description: self.error_type.to_string(),
            })
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum OAuthErrorType {
    #[error(transparent)]
    AuthenticationError(#[from] AuthenticationError),
    #[error("客户端 {} 没有指定重定向 URI", .client_id.0)]
    ClientMissingRedirectURI {
        client_id: crate::database::models::OAuthClientId,
    },
    #[error("提供的重定向 URI 与客户端中配置的不匹配")]
    RedirectUriNotConfigured(String),
    #[error("提供的 scope 格式错误或不对应已知的 scope ({0})")]
    FailedScopeParse(bitflags::parser::ParseError),
    #[error("提供的 scope 请求的范围比开发者应用程序配置的范围更广")]
    ScopesTooBroad,
    #[error("提供的 flow id 无效")]
    InvalidAcceptFlowId,
    #[error("提供的客户端 id 无效")]
    InvalidClientId(crate::database::models::OAuthClientId),
    #[error("提供的 ID 无法解码：{0}")]
    MalformedId(#[from] DecodingError),
    #[error("客户端身份验证失败")]
    ClientAuthenticationFailed,
    #[error("提供的授权码无效")]
    InvalidAuthCode,
    #[error("提供的客户端 id 与此授权码授予的 id 不匹配")]
    UnauthorizedClient,
    #[error("提供的重定向 URI 与此流程开始时提供的 URI 不完全匹配")]
    RedirectUriChanged(Option<String>),
    #[error("提供的授权类型 ({0}) 必须是 \"authorization_code\"")]
    OnlySupportsAuthorizationCodeGrant(String),
    #[error("资源所有者拒绝了请求")]
    AccessDenied,
}

impl From<crate::database::models::DatabaseError> for OAuthErrorType {
    fn from(value: crate::database::models::DatabaseError) -> Self {
        OAuthErrorType::AuthenticationError(value.into())
    }
}

impl From<sqlx::Error> for OAuthErrorType {
    fn from(value: sqlx::Error) -> Self {
        OAuthErrorType::AuthenticationError(value.into())
    }
}

impl OAuthErrorType {
    pub fn error_name(&self) -> String {
        // IETF RFC 6749 4.1.2.1 (https://datatracker.ietf.org/doc/html/rfc6749#autoid-38)
        // 以及 5.2 (https://datatracker.ietf.org/doc/html/rfc6749#section-5.2)
        match self {
            Self::RedirectUriNotConfigured(_)
            | Self::ClientMissingRedirectURI { client_id: _ } => "invalid_uri",
            Self::AuthenticationError(_) | Self::InvalidAcceptFlowId => {
                "server_error"
            }
            Self::RedirectUriChanged(_) | Self::MalformedId(_) => {
                "invalid_request"
            }
            Self::FailedScopeParse(_) | Self::ScopesTooBroad => "invalid_scope",
            Self::InvalidClientId(_) | Self::ClientAuthenticationFailed => {
                "invalid_client"
            }
            Self::InvalidAuthCode
            | Self::OnlySupportsAuthorizationCodeGrant(_) => "invalid_grant",
            Self::UnauthorizedClient => "unauthorized_client",
            Self::AccessDenied => "access_denied",
        }
        .to_string()
    }
}
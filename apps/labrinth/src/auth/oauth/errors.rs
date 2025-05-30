use super::ValidatedRedirectUri;
use crate::auth::AuthenticationError;
use crate::models::error::ApiError;
use actix_web::HttpResponse;
use actix_web::http::{StatusCode, header::LOCATION};
use ariadne::ids::DecodingError;

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
    /// The OAuth request failed either because of an invalid redirection URI
    /// or before we could validate the one we were given, so return an error
    /// directly to the caller
    ///
    /// See: IETF RFC 6749 4.1.2.1 (https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1)
    pub fn error(error_type: impl Into<OAuthErrorType>) -> Self {
        Self {
            error_type: error_type.into(),
            valid_redirect_uri: None,
            state: None,
        }
    }

    /// The OAuth request failed for a reason other than an invalid redirection URI
    /// So send the error in url-encoded form to the redirect URI
    ///
    /// See: IETF RFC 6749 4.1.2.1 (https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1)
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
                redirect_uri = format!("{redirect_uri}&state={state}");
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
    #[error("Client {} has no redirect URIs specified", .client_id.0)]
    ClientMissingRedirectURI {
        client_id: crate::database::models::DBOAuthClientId,
    },
    #[error(
        "The provided redirect URI did not match any configured in the client"
    )]
    RedirectUriNotConfigured(String),
    #[error(
        "The provided scope was malformed or did not correspond to known scopes ({0})"
    )]
    FailedScopeParse(bitflags::parser::ParseError),
    #[error(
        "The provided scope requested scopes broader than the developer app is configured with"
    )]
    ScopesTooBroad,
    #[error("The provided flow id was invalid")]
    InvalidAcceptFlowId,
    #[error("The provided client id was invalid")]
    InvalidClientId(crate::database::models::DBOAuthClientId),
    #[error("The provided ID could not be decoded: {0}")]
    MalformedId(#[from] DecodingError),
    #[error("Failed to authenticate client")]
    ClientAuthenticationFailed,
    #[error("The provided authorization grant code was invalid")]
    InvalidAuthCode,
    #[error(
        "The provided client id did not match the id this authorization code was granted to"
    )]
    UnauthorizedClient,
    #[error(
        "The provided redirect URI did not exactly match the uri originally provided when this flow began"
    )]
    RedirectUriChanged(Option<String>),
    #[error("The provided grant type ({0}) must be \"authorization_code\"")]
    OnlySupportsAuthorizationCodeGrant(String),
    #[error("The resource owner denied the request")]
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
        // And 5.2 (https://datatracker.ietf.org/doc/html/rfc6749#section-5.2)
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

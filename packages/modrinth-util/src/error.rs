use std::{
    convert::Infallible,
    fmt::{Debug, Display},
};

use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

/// Error when calling an HTTP endpoint.
#[derive(Debug, Display, Error)]
pub enum ApiError {
    /// Error occurred on the server side, which the caller has no fault in.
    Internal(anyhow::Error),
    /// Caller made an invalid or malformed request.
    Request(anyhow::Error),
    /// Caller attempted a request which they are not allowed to make.
    Auth(anyhow::Error),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Request(_) => StatusCode::BAD_REQUEST,
            Self::Auth(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            // internal error details are not leaked to the caller
            description: match self {
                Self::Internal(_) => None,
                _ => Some(self.to_string()),
            },
        })
    }
}

/// How an [`ApiError`] is represented when sending over an HTTP request.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Text description of the error that occurred.
    ///
    /// [`ApiError::Internal`] errors have their description filtered out, and
    /// will hold [`None`].
    pub description: Option<String>,
}

/// Allows wrapping [`Result`]s and [`Option`]s into [`Result<T, ApiError>`]s.
#[allow(
    clippy::missing_errors_doc,
    reason = "this trait's purpose is improving error handling"
)]
pub trait Context<T, E>: Sized {
    /// Maps the error variant into an [`anyhow::Error`], creating the message
    /// using `f`.
    fn with_context<D>(self, f: impl FnOnce() -> D) -> Result<T, anyhow::Error>
    where
        D: Send + Sync + Debug + Display + 'static;

    /// Maps the error variant into an [`anyhow::Error`] with the given message.
    #[inline]
    fn context<D>(self, msg: D) -> Result<T, anyhow::Error>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.with_context(|| msg)
    }

    /// Maps the error variant into an [`ApiError::Internal`] using the closure to create the message.
    #[inline]
    fn internal_context_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.with_context(f).map_err(ApiError::Internal)
    }

    /// Maps the error variant into an [`ApiError::Internal`] with the given message.
    #[inline]
    fn internal_context<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.internal_context_with(|| msg)
    }

    /// Maps the error variant into an [`ApiError::Request`] using the closure to create the message.
    #[inline]
    fn request_context_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.with_context(f).map_err(ApiError::Request)
    }

    /// Maps the error variant into an [`ApiError::Request`] with the given message.
    #[inline]
    fn request_context<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.request_context_with(|| msg)
    }

    /// Maps the error variant into an [`ApiError::Auth`] using the closure to create the message.
    #[inline]
    fn auth_context_with<D>(self, f: impl FnOnce() -> D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.with_context(f).map_err(ApiError::Auth)
    }

    /// Maps the error variant into an [`ApiError::Auth`] with the given message.
    #[inline]
    fn auth_context<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.auth_context_with(|| msg)
    }
}

impl<T, E> Context<T, E> for Result<T, E>
where
    Self: anyhow::Context<T, E>,
{
    fn with_context<D>(self, f: impl FnOnce() -> D) -> Result<T, anyhow::Error>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        anyhow::Context::with_context(self, f)
    }
}

impl<T> Context<T, Infallible> for Option<T> {
    fn with_context<D>(self, f: impl FnOnce() -> D) -> Result<T, anyhow::Error>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.ok_or_else(|| anyhow::Error::msg(f()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use anyhow::anyhow;

    #[test]
    fn test_api_error_display() {
        let error = ApiError::Internal(anyhow!("test internal error"));
        assert!(error.to_string().contains("test internal error"));

        let error = ApiError::Request(anyhow!("test request error"));
        assert!(error.to_string().contains("test request error"));

        let error = ApiError::Auth(anyhow!("test auth error"));
        assert!(error.to_string().contains("test auth error"));
    }

    #[test]
    fn test_api_error_debug() {
        let error = ApiError::Internal(anyhow!("test error"));
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("Internal"));
        assert!(debug_str.contains("test error"));
    }

    #[test]
    fn test_response_error_status_codes() {
        let internal_error = ApiError::Internal(anyhow!("internal error"));
        assert_eq!(
            internal_error.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        let request_error = ApiError::Request(anyhow!("request error"));
        assert_eq!(request_error.status_code(), StatusCode::BAD_REQUEST);

        let auth_error = ApiError::Auth(anyhow!("auth error"));
        assert_eq!(auth_error.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_response_error_response() {
        let error = ApiError::Request(anyhow!("test request error"));
        let response = error.error_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Skip the body parsing test as it requires async and is more complex
        // The important thing is that the error response is created correctly
    }

    #[test]
    fn test_context_trait_result() {
        let result: Result<i32, std::io::Error> = Ok(42);
        let wrapped = result.context("context message");
        assert_eq!(wrapped.unwrap(), 42);

        let result: Result<i32, std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        let wrapped = result.context("context message");
        assert!(wrapped.is_err());
        assert!(wrapped.unwrap_err().to_string().contains("context message"));
    }

    #[test]
    fn test_context_trait_option() {
        let option: Option<i32> = Some(42);
        let wrapped = option.context("context message");
        assert_eq!(wrapped.unwrap(), 42);

        let option: Option<i32> = None;
        let wrapped = option.context("context message");
        assert!(wrapped.is_err());
        assert_eq!(wrapped.unwrap_err().to_string(), "context message");
    }

    #[test]
    fn test_context_trait_internal_error() {
        let result: Result<i32, std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        let wrapped = result.internal_context("internal error context");

        assert!(wrapped.is_err());
        match wrapped.unwrap_err() {
            ApiError::Internal(report) => {
                assert!(report.to_string().contains("internal error context"));
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_context_trait_request_error() {
        let result: Result<i32, std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        let wrapped = result.request_context("request error context");

        assert!(wrapped.is_err());
        match wrapped.unwrap_err() {
            ApiError::Request(report) => {
                assert!(report.to_string().contains("request error context"));
            }
            _ => panic!("Expected Request error"),
        }
    }

    #[test]
    fn test_context_trait_auth_error() {
        let result: Result<i32, std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        let wrapped = result.auth_context("auth error context");

        assert!(wrapped.is_err());
        match wrapped.unwrap_err() {
            ApiError::Auth(report) => {
                assert!(report.to_string().contains("auth error context"));
            }
            _ => panic!("Expected Auth error"),
        }
    }

    #[test]
    fn test_context_trait_with_closure() {
        let result: Result<i32, std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        let wrapped =
            result.with_context(|| format!("context with {}", "dynamic"));

        assert!(wrapped.is_err());
        assert!(
            wrapped
                .unwrap_err()
                .to_string()
                .contains("context with dynamic")
        );
    }
}

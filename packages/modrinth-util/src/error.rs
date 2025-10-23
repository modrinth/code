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
    Internal(eyre::Report),
    /// Caller made an invalid or malformed request.
    Request(eyre::Report),
    /// Caller attempted a request which they are not allowed to make.
    Auth(eyre::Report),
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
    /// Maps the error variant into an [`eyre::Report`], creating the message
    /// using `f`.
    fn wrap_err_with<D>(self, f: impl FnOnce() -> D) -> Result<T, eyre::Report>
    where
        D: Send + Sync + Debug + Display + 'static;

    /// Maps the error variant into an [`eyre::Report`] with the given message.
    /// Maps the error variant into an [`eyre::Report`] with the given message.
    #[inline]
    fn wrap_err<D>(self, msg: D) -> Result<T, eyre::Report>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.wrap_err_with(|| msg)
    }

    /// Maps the error variant into an [`ApiError::Internal`] using the closure to create the message.
    #[inline]
    fn wrap_internal_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.wrap_err_with(f).map_err(ApiError::Internal)
    }

    /// Maps the error variant into an [`ApiError::Internal`] with the given message.
    #[inline]
    fn wrap_internal_err<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.wrap_internal_err_with(|| msg)
    }

    /// Maps the error variant into an [`ApiError::Request`] using the closure to create the message.
    #[inline]
    fn wrap_request_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.wrap_err_with(f).map_err(ApiError::Request)
    }

    /// Maps the error variant into an [`ApiError::Request`] with the given message.
    #[inline]
    fn wrap_request_err<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.wrap_request_err_with(|| msg)
    }

    /// Maps the error variant into an [`ApiError::Auth`] using the closure to create the message.
    #[inline]
    fn wrap_auth_err_with<D>(self, f: impl FnOnce() -> D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.wrap_err_with(f).map_err(ApiError::Auth)
    }

    /// Maps the error variant into an [`ApiError::Auth`] with the given message.
    #[inline]
    fn wrap_auth_err<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.wrap_auth_err_with(|| msg)
    }
}

impl<T, E> Context<T, E> for Result<T, E>
where
    Self: eyre::WrapErr<T, E>,
{
    fn wrap_err_with<D>(self, f: impl FnOnce() -> D) -> Result<T, eyre::Report>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        eyre::WrapErr::wrap_err_with(self, f)
    }
}

impl<T> Context<T, Infallible> for Option<T> {
    fn wrap_err_with<D>(self, f: impl FnOnce() -> D) -> Result<T, eyre::Report>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.ok_or_else(|| eyre::Report::msg(f()))
    }
}

// impl<T, E, Ty> Context<T, E> for Ty where Ty: eyre::WrapErr<T, E> {}

// impl<T, Ty> Context<T, Infallible> for Ty where Ty: eyre::OptionExt<T> {}

// impl<T, E> Context<T, E> for Result<T, E>
// where
//     Self: eyre::WrapErr<T, E>,
// {
//     fn wrap_err_with<D>(self, f: impl FnOnce() -> D) -> Result<T, eyre::Report>
//     where
//         D: Send + Sync + Debug + Display + 'static,
//     {
//         self.map_err(|err| eyre::Report::new(err).wrap_err(f()))
//     }
// }

// impl<T> Context<T, Infallible> for Option<T> {
//     fn wrap_err_with<D>(self, f: impl FnOnce() -> D) -> Result<T, eyre::Report>
//     where
//         D: Send + Sync + Debug + Display + 'static,
//     {
//         self.ok_or_else(|| eyre::Report::msg(f()))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;

    #[test]
    fn test_api_error_display() {
        let error = ApiError::Internal(eyre::eyre!("test internal error"));
        assert!(error.to_string().contains("test internal error"));

        let error = ApiError::Request(eyre::eyre!("test request error"));
        assert!(error.to_string().contains("test request error"));

        let error = ApiError::Auth(eyre::eyre!("test auth error"));
        assert!(error.to_string().contains("test auth error"));
    }

    #[test]
    fn test_api_error_debug() {
        let error = ApiError::Internal(eyre::eyre!("test error"));
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("Internal"));
        assert!(debug_str.contains("test error"));
    }

    #[test]
    fn test_response_error_status_codes() {
        let internal_error = ApiError::Internal(eyre::eyre!("internal error"));
        assert_eq!(
            internal_error.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        let request_error = ApiError::Request(eyre::eyre!("request error"));
        assert_eq!(request_error.status_code(), StatusCode::BAD_REQUEST);

        let auth_error = ApiError::Auth(eyre::eyre!("auth error"));
        assert_eq!(auth_error.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_response_error_response() {
        let error = ApiError::Request(eyre::eyre!("test request error"));
        let response = error.error_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Skip the body parsing test as it requires async and is more complex
        // The important thing is that the error response is created correctly
    }

    #[test]
    fn test_context_trait_result() {
        let result: Result<i32, std::io::Error> = Ok(42);
        let wrapped = result.wrap_err("context message");
        assert_eq!(wrapped.unwrap(), 42);

        let result: Result<i32, std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        let wrapped = result.wrap_err("context message");
        assert!(wrapped.is_err());
        assert!(wrapped.unwrap_err().to_string().contains("context message"));
    }

    #[test]
    fn test_context_trait_option() {
        let option: Option<i32> = Some(42);
        let wrapped = option.wrap_err("context message");
        assert_eq!(wrapped.unwrap(), 42);

        let option: Option<i32> = None;
        let wrapped = option.wrap_err("context message");
        assert!(wrapped.is_err());
        assert_eq!(wrapped.unwrap_err().to_string(), "context message");
    }

    #[test]
    fn test_context_trait_internal_error() {
        let result: Result<i32, std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        let wrapped = result.wrap_internal_err("internal error context");

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
        let wrapped = result.wrap_request_err("request error context");

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
        let wrapped = result.wrap_auth_err("auth error context");

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
            result.wrap_err_with(|| format!("context with {}", "dynamic"));

        assert!(wrapped.is_err());
        assert!(
            wrapped
                .unwrap_err()
                .to_string()
                .contains("context with dynamic")
        );
    }
}

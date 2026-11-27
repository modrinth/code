use std::{
    convert::Infallible,
    fmt::{Debug, Display},
};

use crate::routes::ApiError;

/// Adds context to an [`ApiError`] while preserving its HTTP status variant.
pub trait ApiContext<T>: Sized {
    /// Wraps the report held by the error variant with a lazily-created message.
    fn api_context_with<D>(self, f: impl FnOnce() -> D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static;

    /// Wraps the report held by the error variant with the given message.
    fn api_context<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.api_context_with(|| msg)
    }
}

impl<T> ApiContext<T> for Result<T, ApiError> {
    fn api_context_with<D>(self, f: impl FnOnce() -> D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.map_err(|error| error.context(f()))
    }
}

impl<T> ApiContext<T> for Result<T, crate::auth::AuthenticationError> {
    fn api_context_with<D>(self, f: impl FnOnce() -> D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.map_err(|error| {
            ApiError::Auth(anyhow::Error::new(error).context(f()))
        })
    }
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

    /// Maps the error variant into an [`ApiError::NotFound`] using the closure to create the message.
    #[inline]
    fn not_found_context_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.with_context(f).map_err(ApiError::NotFound)
    }

    /// Maps the error variant into an [`ApiError::NotFound`] with the given message.
    #[inline]
    fn not_found_context<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.not_found_context_with(|| msg)
    }

    /// Maps the error variant into an [`ApiError::Conflict`] using the closure to create the message.
    #[inline]
    fn conflict_context_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.with_context(f).map_err(ApiError::Conflict)
    }

    /// Maps the error variant into an [`ApiError::Conflict`] with the given message.
    #[inline]
    fn conflict_context<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.conflict_context_with(|| msg)
    }

    /// Maps the error variant into an [`ApiError::FailedDependency`] using the closure to create the message.
    #[inline]
    fn failed_dependency_context_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.with_context(f).map_err(ApiError::FailedDependency)
    }

    /// Maps the error variant into an [`ApiError::FailedDependency`] with the given message.
    #[inline]
    fn failed_dependency_context<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.failed_dependency_context_with(|| msg)
    }

    /// Maps the error variant into an [`ApiError::PreconditionRequired`] using the closure to create the message.
    #[inline]
    fn precondition_required_context_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.with_context(f).map_err(ApiError::PreconditionRequired)
    }

    /// Maps the error variant into an [`ApiError::PreconditionRequired`] with the given message.
    #[inline]
    fn precondition_required_context<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.precondition_required_context_with(|| msg)
    }

    /// Maps the error variant into an [`ApiError::PreconditionFailed`] using the closure to create the message.
    #[inline]
    fn precondition_failed_context_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.with_context(f).map_err(ApiError::PreconditionFailed)
    }

    /// Maps the error variant into an [`ApiError::PreconditionFailed`] with the given message.
    #[inline]
    fn precondition_failed_context<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.precondition_failed_context_with(|| msg)
    }

    /// Maps the error variant into an [`ApiError::RateLimit`] using the closure to create the message.
    #[inline]
    fn rate_limit_context_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.with_context(f).map_err(ApiError::RateLimit)
    }

    /// Maps the error variant into an [`ApiError::RateLimit`] with the given message.
    #[inline]
    fn rate_limit_context<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.rate_limit_context_with(|| msg)
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
    use actix_web::{ResponseError, http::StatusCode};
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

        let not_found_error = ApiError::NotFound(anyhow!("not found error"));
        assert_eq!(not_found_error.status_code(), StatusCode::NOT_FOUND);

        let conflict_error = ApiError::Conflict(anyhow!("conflict error"));
        assert_eq!(conflict_error.status_code(), StatusCode::CONFLICT);

        let dependency_error =
            ApiError::FailedDependency(anyhow!("dependency error"));
        assert_eq!(
            dependency_error.status_code(),
            StatusCode::FAILED_DEPENDENCY
        );

        let required_error = ApiError::PreconditionRequired(anyhow!(
            "precondition required error"
        ));
        assert_eq!(
            required_error.status_code(),
            StatusCode::PRECONDITION_REQUIRED
        );

        let failed_error =
            ApiError::PreconditionFailed(anyhow!("precondition failed error"));
        assert_eq!(failed_error.status_code(), StatusCode::PRECONDITION_FAILED);

        let rate_limit_error = ApiError::RateLimit(anyhow!("rate limit error"));
        assert_eq!(
            rate_limit_error.status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
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
    fn test_context_trait_status_errors() {
        let not_found: Option<i32> = None;
        assert!(matches!(
            not_found.not_found_context("missing value").unwrap_err(),
            ApiError::NotFound(_)
        ));

        let conflict: Option<i32> = None;
        assert!(matches!(
            conflict.conflict_context("conflicting value").unwrap_err(),
            ApiError::Conflict(_)
        ));

        let dependency: Option<i32> = None;
        assert!(matches!(
            dependency
                .failed_dependency_context("dependency failed")
                .unwrap_err(),
            ApiError::FailedDependency(_)
        ));

        let required: Option<i32> = None;
        assert!(matches!(
            required
                .precondition_required_context("precondition required")
                .unwrap_err(),
            ApiError::PreconditionRequired(_)
        ));

        let failed: Option<i32> = None;
        assert!(matches!(
            failed
                .precondition_failed_context("precondition failed")
                .unwrap_err(),
            ApiError::PreconditionFailed(_)
        ));

        let rate_limit: Option<i32> = None;
        assert!(matches!(
            rate_limit
                .rate_limit_context("rate limit exceeded")
                .unwrap_err(),
            ApiError::RateLimit(_)
        ));
    }

    #[test]
    fn test_api_context_preserves_authentication_error_kind() {
        use crate::auth::AuthenticationError;

        for source in [
            AuthenticationError::PrivilegedTokenExpired,
            AuthenticationError::InvalidCredentials,
            AuthenticationError::AccountLocked,
            AuthenticationError::DuplicateEmail,
        ] {
            let status = source.status_code();
            let name = source.error_name();
            let description = source.to_string();
            let result: Result<(), AuthenticationError> = Err(source);
            let error =
                result.api_context("checking authentication").unwrap_err();

            assert_eq!(error.status_code(), status);
            assert_eq!(error.as_api_error().error, name);
            assert_eq!(error.as_api_error().description, description);
        }
    }

    #[test]
    fn test_api_context_preserves_status_variant() {
        let result: Result<(), ApiError> =
            Err(ApiError::NotFound(anyhow!("missing value")));
        let error = result.api_context("fetching test value").unwrap_err();

        match error {
            ApiError::NotFound(report) => {
                assert_eq!(report.to_string(), "fetching test value");
                assert!(format!("{report:#}").contains("missing value"));
            }
            _ => panic!("expected NotFound error"),
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

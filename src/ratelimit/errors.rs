//! Errors that can occur during middleware processing stage
use actix_web::ResponseError;
use log::*;
use thiserror::Error;

/// Custom error type. Useful for logging and debugging different kinds of errors.
/// This type can be converted to Actix Error, which defaults to
/// InternalServerError
///
#[derive(Debug, Error)]
pub enum ARError {
    /// Read/Write error on store
    #[error("read/write operatiion failed: {0}")]
    ReadWriteError(String),

    /// Identifier error
    #[error("client identification failed")]
    IdentificationError,
    /// Limited Error
    #[error("You are being ratelimited. Please wait {reset} seconds. {remaining}/{max_requests} remaining.")]
    LimitedError {
        max_requests: usize,
        remaining: usize,
        reset: u64,
    },
}

impl ResponseError for ARError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            Self::LimitedError {
                max_requests,
                remaining,
                reset,
            } => {
                let mut response = actix_web::HttpResponse::TooManyRequests();
                response.insert_header(("x-ratelimit-limit", max_requests.to_string()));
                response.insert_header(("x-ratelimit-remaining", remaining.to_string()));
                response.insert_header(("x-ratelimit-reset", reset.to_string()));
                response.body(self.to_string())
            }
            _ => actix_web::HttpResponse::build(self.status_code()).body(self.to_string()),
        }
    }
}

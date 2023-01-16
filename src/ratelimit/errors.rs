//! Errors that can occur during middleware processing stage
use crate::models::error::ApiError;
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
    #[error("read/write operation failed: {0}")]
    ReadWrite(String),

    /// Identifier error
    #[error("client identification failed")]
    Identification,
    /// Limited Error
    #[error("You are being rate-limited. Please wait {reset} seconds. {remaining}/{max_requests} remaining.")]
    Limited {
        max_requests: usize,
        remaining: usize,
        reset: u64,
    },
}

impl ResponseError for ARError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            Self::Limited {
                max_requests,
                remaining,
                reset,
            } => {
                let mut response = actix_web::HttpResponse::TooManyRequests();
                response.insert_header((
                    "x-ratelimit-limit",
                    max_requests.to_string(),
                ));
                response.insert_header((
                    "x-ratelimit-remaining",
                    remaining.to_string(),
                ));
                response
                    .insert_header(("x-ratelimit-reset", reset.to_string()));
                response.json(ApiError {
                    error: "ratelimit_error",
                    description: &self.to_string(),
                })
            }
            _ => actix_web::HttpResponse::build(self.status_code()).json(
                ApiError {
                    error: "ratelimit_error",
                    description: &self.to_string(),
                },
            ),
        }
    }
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{
    application::{
        fs_service::FsError, instance_service::InstanceError,
        mod_service::ModError, server_source_service::SourceError,
    },
    ports::instance_store::StoreError,
};

/// Unified API error type that maps to HTTP responses.
#[derive(Debug)]
pub enum ApiError {
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    BadRequest(String),
    Conflict(String),
    Internal(String),
    UnprocessableEntity(String),
    TooManyRequests(String),
    ServiceUnavailable(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            Self::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m),
            Self::Forbidden(m) => (StatusCode::FORBIDDEN, m),
            Self::NotFound(m) => (StatusCode::NOT_FOUND, m),
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            Self::Conflict(m) => (StatusCode::CONFLICT, m),
            Self::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
            Self::UnprocessableEntity(m) => {
                (StatusCode::UNPROCESSABLE_ENTITY, m)
            }
            Self::TooManyRequests(m) => (StatusCode::TOO_MANY_REQUESTS, m),
            Self::ServiceUnavailable(m) => (StatusCode::SERVICE_UNAVAILABLE, m),
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

impl From<InstanceError> for ApiError {
    fn from(e: InstanceError) -> Self {
        match e {
            InstanceError::NotFound(id) => {
                Self::NotFound(format!("instance {id} not found"))
            }
            InstanceError::AlreadyRunning => {
                Self::Conflict("instance already running".into())
            }
            InstanceError::NotRunning => {
                Self::Conflict("instance not running".into())
            }
            InstanceError::ActorDead => Self::ServiceUnavailable(
                "instance actor is not responding".into(),
            ),
            InstanceError::Invalid(message) => Self::BadRequest(message),
            InstanceError::NotReady(message) => {
                Self::Conflict(format!("instance is not ready: {message}"))
            }
            e => Self::Internal(e.to_string()),
        }
    }
}

impl From<ModError> for ApiError {
    fn from(e: ModError) -> Self {
        match e {
            ModError::InstanceNotFound => {
                Self::NotFound("instance not found".into())
            }
            ModError::ModNotFound => Self::NotFound("mod not found".into()),
            ModError::ClientOnly => {
                Self::UnprocessableEntity("this mod is client-only".into())
            }
            ModError::NoModrinthId => {
                Self::BadRequest("mod has no modrinth project id".into())
            }
            ModError::InvalidFilename => {
                Self::BadRequest("invalid filename".into())
            }
            ModError::HashMismatch { .. } => Self::BadRequest(e.to_string()),
            e => Self::Internal(e.to_string()),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<StoreError> for ApiError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::NotFound(id) => {
                Self::NotFound(format!("instance {id} not found"))
            }
            StoreError::Database(e) => Self::Internal(e.to_string()),
            StoreError::Parse(e) => Self::Internal(format!("parse error: {e}")),
        }
    }
}

impl From<SourceError> for ApiError {
    fn from(e: SourceError) -> Self {
        match e {
            SourceError::NotFound => {
                Self::NotFound("instance not found".into())
            }
            SourceError::NotLinked => {
                Self::NotFound("server is not linked to a source".into())
            }
            SourceError::AlreadyLinked => Self::Conflict(e.to_string()),
            SourceError::Invalid(message) => Self::BadRequest(message),
            SourceError::Instance(e) => e.into(),
            e => Self::Internal(e.to_string()),
        }
    }
}

impl From<FsError> for ApiError {
    fn from(e: FsError) -> Self {
        match e {
            FsError::NotFound => {
                ApiError::NotFound("instance not found".into())
            }
            FsError::PathTraversal => {
                ApiError::Unauthorized("path traversal rejected".into())
            }
            FsError::NotAFile => {
                ApiError::BadRequest("path is a directory, not a file".into())
            }
            e => ApiError::Internal(e.to_string()),
        }
    }
}

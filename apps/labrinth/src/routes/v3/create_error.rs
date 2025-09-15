use crate::auth::AuthenticationError;
use crate::database::models;
use crate::file_hosting::FileHostingError;
use crate::models::error::AsApiError;
use crate::routes::error::ApiError;
use crate::search::indexing::IndexingError;
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use ariadne::i18n_enum;
use derive_more::Display;
use image::ImageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("Environment Error")]
    EnvError(#[from] dotenvy::Error),
    #[error("An unknown database error occurred")]
    SqlxDatabaseError(#[from] sqlx::Error),
    #[error("Database Error: {0}")]
    DatabaseError(#[from] models::DatabaseError),
    #[error("Indexing Error: {0}")]
    IndexingError(#[from] IndexingError),
    #[error("Error while parsing multipart payload: {0}")]
    MultipartError(#[from] actix_multipart::MultipartError),
    #[error("Error while parsing JSON: {0}")]
    SerDeError(#[from] serde_json::Error),
    #[error("Error while validating input: {0}")]
    ValidationError(String),
    #[error("Error while uploading file: {0}")]
    FileHostingError(#[from] FileHostingError),
    #[error("Error while validating uploaded file: {0}")]
    FileValidationError(#[from] crate::validate::ValidationError),
    #[error("{0}")]
    MissingValueError(MissingValuePart),
    #[error("Invalid format for image: {0}")]
    InvalidIconFormat(ApiError),
    #[error("Error with multipart data: {0}")]
    InvalidInput(String), // TODO: Use an I18nEnum instead of a String
    #[error("Invalid loader: {0}")]
    InvalidLoader(String),
    #[error("Invalid category: {0}")]
    InvalidCategory(String),
    #[error("Invalid file type for version file: {0}")]
    InvalidFileType(String),
    #[error("Slug is already taken!")]
    SlugCollision,
    #[error("Authentication Error: {0}")]
    Unauthorized(#[from] AuthenticationError),
    #[error("Authentication Error: {0}")]
    CustomAuthenticationError(String), // TODO: Use an I18nEnum instead of a String
    #[error("Image Parsing Error: {0}")]
    ImageError(#[from] ImageError),
}

i18n_enum!(
    CreateError,
    root_key: "labrinth.error.creation",
    EnvError(..) => "environment_error",
    SqlxDatabaseError(..) => "database_error.unknown",
    DatabaseError(cause) => "database_error",
    IndexingError(cause) => "indexing_error",
    MultipartError(cause) => "invalid_input.multipart",
    SerDeError(cause) => "invalid_input.parsing",
    ValidationError(cause) => "invalid_input.validation",
    FileHostingError(cause) => "file_hosting_error",
    FileValidationError(cause) => "invalid_input.file",
    MissingValueError(transparent cause) => "invalid_input.missing_value",
    InvalidIconFormat(cause) => "invalid_input.icon",
    InvalidInput(cause) => "invalid_input",
    InvalidLoader(loader) => "invalid_input.loader",
    InvalidCategory(category) => "invalid_input.category",
    InvalidFileType(extension) => "invalid_input.file_type",
    SlugCollision! => "invalid_input.slug_collision",
    Unauthorized(cause) => "unauthorized",
    CustomAuthenticationError(reason) => "unauthorized.custom",
    ImageError(cause) => "invalid_image",
);

#[derive(Copy, Clone, Debug, Display)]
pub enum MissingValuePart {
    #[display("No `data` field in multipart upload")]
    DataField,
    #[display("Missing content name")]
    ContentName,
    #[display("Missing content file name")]
    ContentFileName,
    #[display("Missing content file extension")]
    ContentFileExtension,
    #[display("Missing project id")]
    ProjectId,
}

i18n_enum!(
    MissingValuePart,
    root_key: "labrinth.error.creation.missing_value",
    DataField! => "data_field",
    ContentName! => "content_name",
    ContentFileName! => "content_file_name",
    ContentFileExtension! => "content_file_extension",
    ProjectId! => "project_id",
);

impl actix_web::ResponseError for CreateError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateError::EnvError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateError::SqlxDatabaseError(..) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            CreateError::DatabaseError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateError::IndexingError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateError::FileHostingError(..) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            CreateError::SerDeError(..) => StatusCode::BAD_REQUEST,
            CreateError::MultipartError(..) => StatusCode::BAD_REQUEST,
            CreateError::MissingValueError(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidIconFormat(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidInput(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidLoader(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidCategory(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidFileType(..) => StatusCode::BAD_REQUEST,
            CreateError::Unauthorized(..) => StatusCode::UNAUTHORIZED,
            CreateError::CustomAuthenticationError(..) => {
                StatusCode::UNAUTHORIZED
            }
            CreateError::SlugCollision => StatusCode::BAD_REQUEST,
            CreateError::ValidationError(..) => StatusCode::BAD_REQUEST,
            CreateError::FileValidationError(..) => StatusCode::BAD_REQUEST,
            CreateError::ImageError(..) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.as_api_error())
    }
}

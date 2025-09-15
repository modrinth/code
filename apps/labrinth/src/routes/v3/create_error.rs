use crate::auth::AuthenticationError;
use crate::database::models;
use crate::database::models::loader_fields::VersionFieldParseError;
use crate::file_hosting::FileHostingError;
use crate::models::error::AsApiError;
use crate::models::ids::ImageId;
use crate::models::projects::VersionStatus;
use crate::routes::error::ApiError;
use crate::search::indexing::IndexingError;
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use ariadne::i18n_enum;
use derive_more::{Display, From};
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
    InvalidInput(CreationInvalidInput),
    #[error("Error with multipart data: {0}")]
    InvalidLoaderField(#[from] VersionFieldParseError),
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
    InvalidLoaderField(cause) => "invalid_input",
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

#[derive(Debug, Display, From)]
pub enum CreationInvalidInput {
    #[display("Failed to get created team.")]
    FailedGettingNewTeam,
    #[display("`data` field must come before file fields")]
    DataFieldOutOfOrder,
    #[display("Duplicate multipart field name")]
    DuplicateMultipartField,
    #[display("Projects can only have one icon")]
    MultipleIcons,
    #[display("Only one gallery image can be featured.")]
    MultipleFeaturedGallery,
    #[display("Gallery image exceeds the maximum of {_0}.")]
    GalleryImageTooLarge(&'static str),
    #[display("File `{_0}` (field {_1}) isn't specified in the versions data")]
    FileNotSpecified(String, String),
    #[display("Some files were specified in initial_versions but not uploaded")]
    InitialVersionsFilesMissing,
    #[display("Invalid organization ID specified!")]
    InvalidOrganizationId,
    #[display("Project submitted for review with no initial versions")]
    NoInitialVersions,
    #[from]
    #[display("Invalid SPDX license identifier: {_0}")]
    InvalidLicenseId(spdx::ParseError),
    #[display("Link platform {_0} does not exist.")]
    NonexistentLinkPlatform(String),
    #[display("Image {_0} is not unused or in the '{_1}' context")]
    ImproperContextImage(ImageId, &'static str),
    #[display("Image {_0} does not exist")]
    NonexistentImage(ImageId),
    #[display("Found project id in initial version for new project")]
    ProjectIdInInitialVersion,
    #[display("Icons must be smaller than {_0}")]
    IconTooLarge(&'static str),
    #[display("Status '{_0}' cannot be requested")]
    CannotRequestStatus(VersionStatus),
    #[display("An invalid project id was supplied")]
    InvalidProjectId,
    #[display("An invalid version id was supplied")]
    InvalidVersionId,
    #[display("`data` field is required")]
    MissingDataField,
    #[display("Versions must have at least one file uploaded to them")]
    MissingAnyFiles,
    #[display("At least one file must be specified")]
    NoFilesSpecified,
    #[display("Duplicate files are not allowed to be uploaded to Modrinth!")]
    DuplicateFiles,
    #[display("File names must not contain slashes!")]
    FileNameHasSlashes,
    #[display(
        "Project file exceeds the maximum of {_0}. Contact a moderator or admin to request permission to upload larger files."
    )]
    ProjectFileTooLarge(&'static str),
    #[display("Loader field '{_0}' does not exist for any loaders supplied")]
    NonexistentLoaderField(String),
    #[display("Missing mandatory loader fields: {_0}")]
    MissingLoaderFields(String),
    #[display("No json segment found in multipart.")]
    NoJsonInMultipart,
    Validation(String),
}

i18n_enum!(
    CreationInvalidInput,
    root_key: "labrinth.error.creation.invalid_input",
    FailedGettingNewTeam! => "failed_getting_new_team",
    DataFieldOutOfOrder! => "data_field_out_of_order",
    DuplicateMultipartField! => "duplicate_multipart_field",
    MultipleIcons! => "multiple_icons",
    MultipleFeaturedGallery! => "multiple_featured_gallery",
    GalleryImageTooLarge(limit) => "gallery_image_too_large",
    FileNotSpecified(file_name, in_field) => "file_not_specified",
    InitialVersionsFilesMissing! => "initial_versions_files_missing",
    InvalidOrganizationId! => "invalid_organization_id",
    NoInitialVersions! => "no_initial_versions",
    InvalidLicenseId(cause) => "invalid_license_id",
    NonexistentLinkPlatform(platform) => "nonexistent_link_platform",
    ImproperContextImage(image_id, proper_context) => "improper_context_image",
    NonexistentImage(image_id) => "nonexistent_image",
    ProjectIdInInitialVersion! => "project_id_in_initial_version",
    IconTooLarge(limit) => "icon_too_large",
    CannotRequestStatus(status) => "cannot_request_status",
    InvalidProjectId! => "invalid_project_id",
    InvalidVersionId! => "invalid_version_id",
    MissingDataField! => "missing_data_field",
    MissingAnyFiles! => "missing_any_files",
    NoFilesSpecified! => "no_files_specified",
    DuplicateFiles! => "duplicate_files",
    FileNameHasSlashes! => "file_name_has_slashes",
    ProjectFileTooLarge(limit) => "project_file_too_large",
    NonexistentLoaderField(field) => "nonexistent_loader_field",
    MissingLoaderFields(fields) => "missing_loader_fields",
    NoJsonInMultipart! => "no_json_in_multipart",
    Validation(transparent reason) => "validation",
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
            CreateError::InvalidLoaderField(..) => StatusCode::BAD_REQUEST,
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

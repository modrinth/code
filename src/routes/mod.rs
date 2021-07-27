use actix_web::web;

mod v1;
pub use v1::v1_config;

mod auth;
mod index;
mod maven;
mod moderation;
mod not_found;
mod notifications;
mod project_creation;
mod projects;
mod reports;
mod tags;
mod teams;
mod users;
mod version_creation;
mod version_file;
mod versions;

pub use auth::config as auth_config;
pub use tags::config as tags_config;

pub use self::index::index_get;
pub use self::not_found::not_found;
use crate::file_hosting::FileHostingError;

pub fn v2_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v2/")
            .configure(auth_config)
            .configure(tags_config)
            .configure(projects_config)
            .configure(versions_config)
            .configure(teams_config)
            .configure(users_config)
            .configure(moderation_config)
            .configure(reports_config)
            .configure(notifications_config),
    );
}

pub fn projects_config(cfg: &mut web::ServiceConfig) {
    cfg.service(projects::project_search);
    cfg.service(projects::projects_get);
    cfg.service(project_creation::project_create);

    cfg.service(
        web::scope("project")
            .service(projects::project_get)
            .service(projects::project_delete)
            .service(projects::project_edit)
            .service(projects::project_icon_edit)
            .service(projects::delete_project_icon)
            .service(projects::add_gallery_item)
            .service(projects::edit_gallery_item)
            .service(projects::delete_gallery_item)
            .service(projects::project_follow)
            .service(projects::project_unfollow)
            .service(teams::team_members_get_project)
            .service(
                web::scope("{project_id}")
                    .service(versions::version_list)
                    .service(projects::dependency_list),
            ),
    );
}

pub fn maven_config(cfg: &mut web::ServiceConfig) {
    cfg.service(maven::maven_metadata);
    cfg.service(maven::version_file);
}

pub fn versions_config(cfg: &mut web::ServiceConfig) {
    cfg.service(versions::versions_get);
    cfg.service(version_creation::version_create);
    cfg.service(
        web::scope("version")
            .service(versions::version_get)
            .service(versions::version_delete)
            .service(version_creation::upload_file_to_version)
            .service(versions::version_edit),
    );
    cfg.service(
        web::scope("version_file")
            .service(version_file::delete_file)
            .service(version_file::get_version_from_hash)
            .service(version_file::download_version)
            .service(version_file::get_update_from_hash),
    );

    cfg.service(
        web::scope("version_files")
            .service(version_file::get_versions_from_hashes)
            .service(version_file::download_files)
            .service(version_file::update_files),
    );
}

pub fn users_config(cfg: &mut web::ServiceConfig) {
    cfg.service(users::user_auth_get);

    cfg.service(users::users_get);
    cfg.service(
        web::scope("user")
            .service(users::user_get)
            .service(users::projects_list)
            .service(users::user_delete)
            .service(users::user_edit)
            .service(users::user_icon_edit)
            .service(users::user_notifications)
            .service(users::user_follows),
    );
}

pub fn teams_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("team")
            .service(teams::team_members_get)
            .service(teams::edit_team_member)
            .service(teams::transfer_ownership)
            .service(teams::add_team_member)
            .service(teams::join_team)
            .service(teams::remove_team_member),
    );
}

pub fn notifications_config(cfg: &mut web::ServiceConfig) {
    cfg.service(notifications::notifications_get);
    cfg.service(notifications::notification_delete);

    cfg.service(
        web::scope("notification")
            .service(notifications::notification_get)
            .service(notifications::notification_delete),
    );
}

pub fn moderation_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("moderation").service(moderation::get_projects));
}

pub fn reports_config(cfg: &mut web::ServiceConfig) {
    cfg.service(reports::reports);
    cfg.service(reports::report_create);
    cfg.service(reports::delete_report);
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Environment Error")]
    EnvError(#[from] dotenv::Error),
    #[error("Error while uploading file")]
    FileHostingError(#[from] FileHostingError),
    #[error("Database Error: {0}")]
    DatabaseError(#[from] crate::database::models::DatabaseError),
    #[error("Database Error: {0}")]
    SqlxDatabaseError(#[from] sqlx::Error),
    #[error("Internal server error: {0}")]
    XmlError(String),
    #[error("Deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Authentication Error: {0}")]
    AuthenticationError(#[from] crate::util::auth::AuthenticationError),
    #[error("Authentication Error: {0}")]
    CustomAuthenticationError(String),
    #[error("Invalid Input: {0}")]
    InvalidInputError(String),
    #[error("Error while validating input: {0}")]
    ValidationError(String),
    #[error("Search Error: {0}")]
    SearchError(#[from] meilisearch_sdk::errors::Error),
    #[error("Indexing Error: {0}")]
    IndexingError(#[from] crate::search::indexing::IndexingError),
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ApiError::EnvError(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::DatabaseError(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::SqlxDatabaseError(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::AuthenticationError(..) => actix_web::http::StatusCode::UNAUTHORIZED,
            ApiError::CustomAuthenticationError(..) => actix_web::http::StatusCode::UNAUTHORIZED,
            ApiError::XmlError(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::JsonError(..) => actix_web::http::StatusCode::BAD_REQUEST,
            ApiError::SearchError(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::IndexingError(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::FileHostingError(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InvalidInputError(..) => actix_web::http::StatusCode::BAD_REQUEST,
            ApiError::ValidationError(..) => actix_web::http::StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> actix_web::web::HttpResponse {
        actix_web::web::HttpResponse::build(self.status_code()).json(
            crate::models::error::ApiError {
                error: match self {
                    ApiError::EnvError(..) => "environment_error",
                    ApiError::SqlxDatabaseError(..) => "database_error",
                    ApiError::DatabaseError(..) => "database_error",
                    ApiError::AuthenticationError(..) => "unauthorized",
                    ApiError::CustomAuthenticationError(..) => "unauthorized",
                    ApiError::XmlError(..) => "xml_error",
                    ApiError::JsonError(..) => "json_error",
                    ApiError::SearchError(..) => "search_error",
                    ApiError::IndexingError(..) => "indexing_error",
                    ApiError::FileHostingError(..) => "file_hosting_error",
                    ApiError::InvalidInputError(..) => "invalid_input",
                    ApiError::ValidationError(..) => "invalid_input",
                },
                description: &self.to_string(),
            },
        )
    }
}

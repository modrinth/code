use actix_web::web;

mod auth;
mod index;
mod mod_creation;
mod moderation;
mod mods;
mod not_found;
mod tags;
mod teams;
mod users;
mod version_creation;
mod versions;

pub use auth::config as auth_config;
pub use tags::config as tags_config;

pub use self::index::index_get;
pub use self::not_found::not_found;
use crate::file_hosting::FileHostingError;

pub fn mods_config(cfg: &mut web::ServiceConfig) {
    cfg.service(mods::mod_search);
    cfg.service(mods::mods_get);
    cfg.service(mod_creation::mod_create);

    cfg.service(
        web::scope("mod")
            .service(mods::mod_slug_get)
            .service(mods::mod_get)
            .service(mods::mod_delete)
            .service(mods::mod_edit)
            .service(mods::mod_icon_edit)
            .service(web::scope("{mod_id}").service(versions::version_list)),
    );
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
            .service(versions::delete_file)
            .service(versions::get_version_from_hash)
            .service(versions::download_version),
    );
}

pub fn users_config(cfg: &mut web::ServiceConfig) {
    cfg.service(users::user_auth_get);

    cfg.service(users::users_get);
    cfg.service(
        web::scope("user")
            .service(users::user_username_get)
            .service(users::user_get)
            .service(users::mods_list)
            .service(users::user_delete)
            .service(users::user_edit)
            .service(users::user_icon_edit)
            .service(users::teams),
    );
}

pub fn teams_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("team")
            .service(teams::team_members_get)
            .service(teams::edit_team_member)
            .service(teams::add_team_member)
            .service(teams::join_team)
            .service(teams::remove_team_member),
    );
}

pub fn moderation_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("moderation")
            .service(moderation::mods)
            .service(moderation::versions),
    );
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Environment Error")]
    EnvError(#[from] dotenv::Error),
    #[error("Error while uploading file")]
    FileHostingError(#[from] FileHostingError),
    #[error("Internal server error")]
    DatabaseError(#[from] crate::database::models::DatabaseError),
    #[error("Deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Authentication Error: {0}")]
    AuthenticationError(#[from] crate::auth::AuthenticationError),
    #[error("Authentication Error: {0}")]
    CustomAuthenticationError(String),
    #[error("Invalid Input: {0}")]
    InvalidInputError(String),
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
            ApiError::AuthenticationError(..) => actix_web::http::StatusCode::UNAUTHORIZED,
            ApiError::CustomAuthenticationError(..) => actix_web::http::StatusCode::UNAUTHORIZED,
            ApiError::JsonError(..) => actix_web::http::StatusCode::BAD_REQUEST,
            ApiError::SearchError(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::IndexingError(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::FileHostingError(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InvalidInputError(..) => actix_web::http::StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> actix_web::web::HttpResponse {
        actix_web::web::HttpResponse::build(self.status_code()).json(
            crate::models::error::ApiError {
                error: match self {
                    ApiError::EnvError(..) => "environment_error",
                    ApiError::DatabaseError(..) => "database_error",
                    ApiError::AuthenticationError(..) => "unauthorized",
                    ApiError::CustomAuthenticationError(..) => "unauthorized",
                    ApiError::JsonError(..) => "json_error",
                    ApiError::SearchError(..) => "search_error",
                    ApiError::IndexingError(..) => "indexing_error",
                    ApiError::FileHostingError(..) => "file_hosting_error",
                    ApiError::InvalidInputError(..) => "invalid_input",
                },
                description: &self.to_string(),
            },
        )
    }
}

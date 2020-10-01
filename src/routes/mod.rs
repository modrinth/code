use actix_web::web;

mod auth;
mod index;
mod mod_creation;
mod mods;
mod not_found;
mod tags;
mod users;
mod version_creation;
mod versions;

pub use auth::config as auth_config;
pub use tags::config as tags_config;

pub use self::index::index_get;
pub use self::not_found::not_found;

pub fn mods_config(cfg: &mut web::ServiceConfig) {
    cfg.service(mods::mod_search);
    cfg.service(mod_creation::mod_create);

    cfg.service(
        web::scope("mod")
            .service(mods::mod_get)
            .service(mods::mod_delete)
            .service(web::scope("{mod_id}").configure(versions_config)),
    );
}

pub fn versions_config(cfg: &mut web::ServiceConfig) {
    cfg.service(versions::version_list)
        .service(version_creation::version_create)
        .service(
            web::scope("version")
                .service(versions::version_get)
                .service(versions::version_delete)
                .service(version_creation::upload_file_to_version),
        );
}

pub fn users_config(cfg: &mut web::ServiceConfig) {
    cfg.service(users::user_auth_get);

    cfg.service(
        web::scope("user")
            .service(users::user_get)
            .service(users::user_delete),
    );
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Internal server error")]
    DatabaseError(#[from] crate::database::models::DatabaseError),
    #[error("Authentication Error")]
    AuthenticationError,
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ApiError::DatabaseError(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::AuthenticationError => actix_web::http::StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> actix_web::web::HttpResponse {
        actix_web::web::HttpResponse::build(self.status_code()).json(
            crate::models::error::ApiError {
                error: match self {
                    ApiError::DatabaseError(..) => "database_error",
                    ApiError::AuthenticationError => "unauthorized",
                },
                description: &self.to_string(),
            },
        )
    }
}

mod notifications;
mod openapi;
pub(crate) mod project_creation;
mod projects;
mod reports;
mod statistics;
pub mod tags;
mod teams;
mod threads;
mod users;
mod version_creation;
pub mod version_file;
mod versions;

pub use super::ApiError;
use crate::util::cors::default_cors;
use actix_web::web;
pub use openapi::ApiDoc;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v2")
            .wrap(default_cors())
            .configure(super::internal::session::config)
            .configure(super::internal::flows::config)
            .configure(super::internal::pats::config)
            .configure(super::internal::admin::config)
            .configure(notifications::config)
            .configure(project_creation::config)
            .configure(projects::config)
            .configure(reports::config)
            .configure(statistics::config)
            .configure(tags::config)
            .configure(teams::config)
            .configure(threads::config)
            .configure(users::config)
            .configure(version_file::config)
            .configure(versions::config),
    );
}

pub(crate) mod moderation;
mod notifications;
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

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v2")
            .wrap(default_cors())
            .configure(super::internal::session::config)
            .configure(super::internal::flows::config)
            .configure(super::internal::pats::config)
            .configure(super::internal::admin::config)
            .configure(moderation::config)
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

#[derive(utoipa::OpenApi)]
#[openapi(
    nest(
        (path = "/v2", api = super::internal::session::ApiDoc),
        (path = "/v2", api = super::internal::flows::ApiDoc),
        (path = "/v2", api = super::internal::pats::RouteDoc),
        (path = "/v2", api = super::internal::admin::ApiDoc),
        (path = "/v2", api = moderation::ApiDoc),
        (path = "/v2", api = notifications::ApiDoc),
        (path = "/v2", api = project_creation::RouteDoc),
        (path = "/v2", api = projects::ApiDoc),
        (path = "/v2", api = reports::RouteDoc),
        (path = "/v2", api = statistics::RouteDoc),
        (path = "/v2", api = tags::ApiDoc),
        (path = "/v2", api = teams::ApiDoc),
        (path = "/v2", api = threads::ApiDoc),
        (path = "/v2", api = users::ApiDoc),
        (path = "/v2", api = version_file::ApiDoc),
        (path = "/v2", api = versions::ApiDoc),
    )
)]
pub struct ApiDoc;

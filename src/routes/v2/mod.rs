mod admin;
mod analytics_get;
mod collections;
mod images;
mod moderation;
mod notifications;
mod organizations;
pub(crate) mod project_creation;
mod projects;
mod reports;
mod statistics;
mod tags;
mod teams;
mod threads;
mod users;
mod version_creation;
mod version_file;
mod versions;

pub use super::ApiError;
use crate::util::cors::default_cors;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("v2")
            .wrap(default_cors())
            .configure(admin::config)
            .configure(analytics_get::config)
            .configure(crate::auth::session::config)
            .configure(crate::auth::flows::config)
            .configure(crate::auth::pats::config)
            .configure(moderation::config)
            .configure(notifications::config)
            .configure(organizations::config)
            .configure(project_creation::config)
            .configure(collections::config)
            .configure(images::config)
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

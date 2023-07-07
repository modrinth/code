mod admin;
mod moderation;
mod notifications;
mod pats;
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

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("v2")
            .configure(admin::config)
            .configure(crate::auth::config)
            .configure(moderation::config)
            .configure(notifications::config)
            .configure(pats::config)
            .configure(project_creation::config)
            // SHOULD CACHE
            .configure(projects::config)
            .configure(reports::config)
            // should cache in future
            .configure(statistics::config)
            // should cache in future
            .configure(tags::config)
            // should cache
            .configure(teams::config)
            .configure(threads::config)
            // should cache
            .configure(users::config)
            // should cache in future
            .configure(version_file::config)
            // SHOULD CACHE
            .configure(versions::config),
    );
}

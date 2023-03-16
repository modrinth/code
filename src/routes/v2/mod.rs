mod admin;
mod auth;
mod midas;
mod moderation;
mod notifications;
pub(crate) mod project_creation;
mod projects;
mod reports;
mod statistics;
mod tags;
mod teams;
mod users;
mod version_creation;
mod version_file;
mod versions;

pub use super::ApiError;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("v2")
            .configure(admin::config)
            .configure(auth::config)
            .configure(midas::config)
            .configure(moderation::config)
            .configure(notifications::config)
            .configure(project_creation::config)
            .configure(projects::config)
            .configure(reports::config)
            .configure(statistics::config)
            .configure(tags::config)
            .configure(teams::config)
            .configure(users::config)
            .configure(version_creation::config)
            .configure(version_file::config)
            .configure(versions::config),
    );
}

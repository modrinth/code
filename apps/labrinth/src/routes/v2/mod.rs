mod moderation;
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

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/v2")
            .wrap(default_cors())
            .configure(super::internal::admin::config)
            // Todo: separate these- they need to also follow v2-v3 conversion
            .configure(super::internal::session::config)
            .configure(super::internal::flows::config)
            .configure(super::internal::pats::config)
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

pub fn utoipa_config(
    cfg: &mut utoipa_actix_web::service_config::ServiceConfig,
) {
    cfg.service(
        utoipa_actix_web::scope("/v2")
            .guard(actix_web::guard::fn_guard(|_| false))
            .wrap(default_cors())
            .configure(moderation::utoipa_config)
            .configure(notifications::utoipa_config)
            .configure(project_creation::utoipa_config)
            .configure(projects::utoipa_config)
            .configure(reports::utoipa_config)
            .configure(statistics::utoipa_config)
            .configure(tags::utoipa_config)
            .configure(teams::utoipa_config)
            .configure(threads::utoipa_config)
            .configure(users::utoipa_config)
            .configure(version_file::utoipa_config)
            .configure(versions::utoipa_config),
    );
}

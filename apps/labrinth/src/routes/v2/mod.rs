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
        actix_web::web::scope("v2")
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
            .wrap(default_cors())
            .configure(projects::utoipa_config_root)
            .configure(versions::utoipa_config_root)
            .configure(users::utoipa_config_root)
            .configure(notifications::utoipa_config_root)
            .configure(teams::utoipa_config_root)
            .configure(threads::utoipa_config_root)
            .configure(reports::utoipa_config_root)
            .configure(statistics::utoipa_config_root)
            .configure(project_creation::utoipa_config_root)
            .configure(version_creation::utoipa_config_root),
    );
    cfg.service(
        utoipa_actix_web::scope("/v2/project")
            .wrap(default_cors())
            .configure(projects::utoipa_config),
    );
    cfg.service(
        utoipa_actix_web::scope("/v2/version")
            .wrap(default_cors())
            .configure(versions::utoipa_config),
    );
    cfg.service(
        utoipa_actix_web::scope("/v2/version_file")
            .wrap(default_cors())
            .configure(version_file::utoipa_config),
    );
    cfg.service(
        utoipa_actix_web::scope("/v2/version_files")
            .wrap(default_cors())
            .configure(version_file::utoipa_config_files),
    );
    cfg.service(
        utoipa_actix_web::scope("/v2/user")
            .wrap(default_cors())
            .configure(users::utoipa_config),
    );
    cfg.service(
        utoipa_actix_web::scope("/v2/team")
            .wrap(default_cors())
            .configure(teams::utoipa_config),
    );
    cfg.service(
        utoipa_actix_web::scope("/v2/tag")
            .wrap(default_cors())
            .configure(tags::utoipa_config),
    );
    cfg.service(
        utoipa_actix_web::scope("/v2/notification")
            .wrap(default_cors())
            .configure(notifications::utoipa_config),
    );
    cfg.service(
        utoipa_actix_web::scope("/v2/thread")
            .wrap(default_cors())
            .configure(threads::utoipa_config),
    );
    cfg.service(
        utoipa_actix_web::scope("/v2/message")
            .wrap(default_cors())
            .configure(threads::utoipa_config_message),
    );
    cfg.service(
        utoipa_actix_web::scope("/v2/moderation")
            .wrap(default_cors())
            .configure(moderation::utoipa_config),
    );
}

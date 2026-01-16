pub(crate) mod admin;
pub mod affiliate;
pub mod billing;
pub mod delphi;
pub mod external_notifications;
pub mod flows;
pub mod gdpr;
pub mod gotenberg;
pub mod medal;
pub mod moderation;
pub mod mural;
pub mod pats;
pub mod session;
pub mod statuses;

pub use super::ApiError;
use super::v3::oauth_clients;
use crate::util::cors::default_cors;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("_internal")
            .wrap(default_cors())
            .configure(admin::config)
            .configure(oauth_clients::config)
            .configure(session::config)
            .configure(flows::config)
            .configure(pats::config)
            .configure(billing::config)
            .configure(gdpr::config)
            .configure(gotenberg::config)
            .configure(statuses::config)
            .configure(medal::config)
            .configure(external_notifications::config)
            .configure(mural::config)
            .configure(delphi::config),
    );
}

pub fn utoipa_config(
    cfg: &mut utoipa_actix_web::service_config::ServiceConfig,
) {
    cfg.service(
        utoipa_actix_web::scope("/_internal/moderation")
            .wrap(default_cors())
            .configure(moderation::config),
    )
    .service(
        utoipa_actix_web::scope("/_internal/affiliate")
            .wrap(default_cors())
            .configure(affiliate::config),
    );
}

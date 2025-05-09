pub(crate) mod admin;
pub mod billing;
pub mod flows;
pub mod gdpr;
pub mod moderation;
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
            .configure(moderation::config)
            .configure(billing::config)
            .configure(gdpr::config)
            .configure(statuses::config),
    );
}

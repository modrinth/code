pub mod admin;
pub mod affiliate;
pub mod attribution;
pub mod billing;
pub mod campaign;
pub mod delphi;
pub mod external_notifications;
pub mod flows;
pub mod gdpr;
pub mod globals;
pub mod gotenberg;
pub mod medal;
pub mod moderation;
pub mod mural;
pub mod pats;
pub mod search;
pub mod server_ping;
pub mod session;
pub mod statuses;

pub use super::ApiError;
use super::v3::oauth_clients;
use crate::util::cors::default_cors;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/_internal")
            .wrap(default_cors())
            .configure(admin::config)
            .configure(session::config)
            .configure(flows::config)
            .configure(pats::config)
            .configure(oauth_clients::config)
            .service(web::scope("/moderation").configure(moderation::config))
            .service(web::scope("/affiliate").configure(affiliate::config))
            .service(web::scope("/campaign").configure(campaign::config))
            .service(web::scope("/search-management").configure(search::config))
            .service(web::scope("/globals").configure(globals::config))
            .service(web::scope("/server-ping").configure(server_ping::config))
            .service(web::scope("/attribution").configure(attribution::config))
            .configure(billing::config)
            .configure(delphi::config)
            .configure(external_notifications::config)
            .configure(gdpr::config)
            .configure(gotenberg::config)
            .configure(medal::config)
            .configure(mural::config)
            .configure(statuses::config),
    )
    .service(
        web::scope("/v3/analytics-event")
            .wrap(default_cors())
            .configure(super::v3::analytics_event::config),
    );
}

#[derive(utoipa::OpenApi)]
#[openapi(
    nest(
        (path = "/_internal", api = admin::ApiDoc),
        (path = "/_internal", api = session::ApiDoc),
        (path = "/_internal", api = flows::ApiDoc),
        (path = "/_internal", api = pats::RouteDoc),
        (path = "/_internal", api = oauth_clients::ApiDoc),
        (path = "/_internal/moderation", api = moderation::ApiDoc),
        (path = "/_internal/affiliate", api = affiliate::RouteDoc),
        (path = "/_internal/campaign", api = campaign::RouteDoc),
        (path = "/_internal/search-management", api = search::RouteDoc),
        (path = "/_internal/globals", api = globals::RouteDoc),
        (path = "/_internal/server-ping", api = server_ping::RouteDoc),
        (path = "/_internal/attribution", api = attribution::RouteDoc),
        (path = "/_internal/billing", api = billing::RouteDoc),
        (path = "/_internal/delphi", api = delphi::RouteDoc),
        (path = "/_internal", api = external_notifications::RouteDoc),
        (path = "/_internal/gdpr", api = gdpr::RouteDoc),
        (path = "/_internal", api = gotenberg::RouteDoc),
        (path = "/_internal/medal", api = medal::RouteDoc),
        (path = "/_internal", api = mural::RouteDoc),
        (path = "/_internal", api = statuses::RouteDoc),
        (path = "/v3/analytics-event", api = super::v3::analytics_event::RouteDoc),
    )
)]
pub struct ApiDoc;

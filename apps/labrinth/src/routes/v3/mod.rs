pub use super::ApiError;
use crate::util::cors::default_cors;
use actix_web::{HttpResponse, web};
use serde_json::json;

pub mod analytics_event;
pub mod analytics_get;
pub mod collections;
pub mod content;
pub mod friends;
pub mod images;
pub mod limits;
pub mod notifications;
pub mod organizations;
pub mod payouts;
pub mod project_creation;
pub mod projects;
pub mod reports;
pub mod shared_instance_version_creation;
pub mod shared_instances;
pub mod statistics;
pub mod tags;
pub mod teams;
pub mod threads;
pub mod users;
pub mod version_creation;
pub mod version_file;
pub mod versions;

pub mod oauth_clients;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v3/analytics")
            .wrap(default_cors())
            .configure(analytics_get::config),
    );
    cfg.service(
        web::scope("/v3/payout")
            .wrap(default_cors())
            .configure(payouts::config),
    );
    cfg.service(
        web::scope("/v3/project")
            .wrap(default_cors())
            .configure(projects::project_config)
            .configure(project_creation::config),
    );
    cfg.service(
        web::scope("/v3")
            .wrap(default_cors())
            .configure(limits::config)
            .configure(collections::config)
            .configure(images::config)
            .configure(notifications::config)
            .configure(oauth_clients::config)
            .configure(organizations::config)
            .service(payouts::paypal_webhook)
            .service(payouts::tremendous_webhook)
            .configure(projects::config)
            .configure(reports::config)
            .configure(shared_instance_version_creation::config)
            .configure(shared_instances::config)
            .configure(statistics::config)
            .configure(tags::config)
            .configure(teams::config)
            .configure(threads::config)
            .configure(users::config)
            .configure(version_file::config)
            .configure(versions::config)
            .configure(friends::config),
    );
    cfg.configure(content::config);
}

#[derive(utoipa::OpenApi)]
#[openapi(
    nest(
        (path = "/v3/analytics", api = analytics_get::ApiDoc),
        (path = "/v3/payout", api = payouts::PayoutRoutesDoc),
        (path = "/v3/project", api = projects::ProjectRoutesDoc),
        (path = "/v3/project", api = project_creation::ApiDoc),
        (path = "/v3/project", api = teams::ProjectRoutesDoc),
        (path = "/v3/project", api = versions::ProjectRoutesDoc),
        (path = "/v3", api = limits::RouteDoc),
        (path = "/v3", api = collections::RouteDoc),
        (path = "/v3", api = images::RouteDoc),
        (path = "/v3", api = notifications::RouteDoc),
        (path = "/v3", api = oauth_clients::ApiDoc),
        (path = "/v3", api = organizations::RouteDoc),
        (path = "/v3", api = payouts::WebhookRoutesDoc),
        (path = "/v3", api = projects::RootRoutesDoc),
        (path = "/v3", api = reports::RouteDoc),
        (path = "/v3", api = shared_instance_version_creation::RouteDoc),
        (path = "/v3", api = shared_instances::RouteDoc),
        (path = "/v3", api = statistics::RouteDoc),
        (path = "/v3", api = tags::RouteDoc),
        (path = "/v3", api = teams::RootRoutesDoc),
        (path = "/v3", api = threads::RouteDoc),
        (path = "/v3", api = users::RouteDoc),
        (path = "/v3", api = version_creation::RouteDoc),
        (path = "/v3", api = version_file::RouteDoc),
        (path = "/v3", api = versions::RootRoutesDoc),
        (path = "/v3", api = friends::RouteDoc),
        (path = "/v3", api = content::RouteDoc),
    )
)]
pub struct ApiDoc;

pub async fn hello_world() -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(json!({
        "hello": "world",
    })))
}

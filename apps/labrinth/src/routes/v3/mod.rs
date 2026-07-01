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
        web::scope("v3")
            .wrap(default_cors())
            .configure(limits::config)
            .configure(collections::config)
            .configure(content::config)
            .configure(images::config)
            .configure(notifications::config)
            .configure(organizations::config)
            .configure(payouts::webhook_config)
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
}

pub fn utoipa_config(
    cfg: &mut utoipa_actix_web::service_config::ServiceConfig,
) {
    cfg.service(
        utoipa_actix_web::scope("/v3/analytics")
            .wrap(default_cors())
            .configure(analytics_get::config),
    );
    cfg.service(
        utoipa_actix_web::scope("/v3/payout")
            .wrap(default_cors())
            .configure(payouts::config),
    );
    cfg.service(
        utoipa_actix_web::scope("/v3/project")
            .wrap(default_cors())
            .configure(projects::utoipa_config)
            .configure(project_creation::config),
    );
    cfg.service(
        utoipa_actix_web::scope("/v3")
            .wrap(default_cors())
            .service(friends::add_friend)
            .service(friends::remove_friend)
            .service(friends::friends)
            .service(projects::project_search)
            .service(projects::project_search_post)
            .service(oauth_clients::get_client)
            .service(oauth_clients::get_clients)
            .service(oauth_clients::oauth_client_create)
            .service(oauth_clients::oauth_client_delete)
            .service(oauth_clients::oauth_client_edit)
            .service(oauth_clients::oauth_client_icon_edit)
            .service(oauth_clients::oauth_client_icon_delete)
            .service(oauth_clients::get_user_oauth_authorizations)
            .service(oauth_clients::revoke_oauth_authorization),
    );
    cfg.configure(content::utoipa_config);
}

pub async fn hello_world() -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(json!({
        "hello": "world",
    })))
}

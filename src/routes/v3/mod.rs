pub use super::ApiError;
use crate::util::cors::default_cors;
use actix_web::{web, HttpResponse};
use serde_json::json;

pub mod admin;
pub mod analytics_get;
pub mod collections;
pub mod images;
pub mod moderation;
pub mod notifications;
pub mod organizations;
pub mod payouts;
pub mod project_creation;
pub mod projects;
pub mod reports;
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
            .configure(admin::config)
            .configure(analytics_get::config)
            // TODO: write tests that catch these
            .configure(oauth_clients::config)
            .configure(crate::auth::session::config)
            .configure(crate::auth::flows::config)
            .configure(crate::auth::pats::config)
            .configure(collections::config)
            .configure(images::config)
            .configure(moderation::config)
            .configure(notifications::config)
            .configure(organizations::config)
            .configure(project_creation::config)
            .configure(projects::config)
            .configure(reports::config)
            .configure(statistics::config)
            .configure(tags::config)
            .configure(teams::config)
            .configure(threads::config)
            .configure(users::config)
            .configure(version_file::config)
            .configure(payouts::config)
            .configure(versions::config),
    );
}

pub async fn hello_world() -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(json!({
        "hello": "world",
    })))
}

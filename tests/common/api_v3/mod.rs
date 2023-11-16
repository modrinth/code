#![allow(dead_code)]

use super::environment::LocalService;
use actix_web::dev::ServiceResponse;
use std::rc::Rc;

pub mod oauth;
pub mod oauth_clients;
pub mod organization;
pub mod project;
pub mod request_data;
pub mod tags;
pub mod team;
pub mod version;

#[derive(Clone)]
pub struct ApiV3 {
    pub test_app: Rc<dyn LocalService>,
}

impl ApiV3 {
    pub async fn call(&self, req: actix_http::Request) -> ServiceResponse {
        self.test_app.call(req).await.unwrap()
    }

    pub async fn reset_search_index(&self) -> ServiceResponse {
        let req = actix_web::test::TestRequest::post()
            .uri("/v3/admin/_force_reindex")
            .append_header((
                "Modrinth-Admin",
                dotenvy::var("LABRINTH_ADMIN_KEY").unwrap(),
            ))
            .to_request();
        self.call(req).await
    }
}

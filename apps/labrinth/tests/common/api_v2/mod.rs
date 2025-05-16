#![allow(dead_code)]

use super::{
    api_common::{Api, ApiBuildable},
    environment::LocalService,
};
use actix_web::{App, dev::ServiceResponse, test};
use async_trait::async_trait;
use labrinth::LabrinthConfig;
use std::rc::Rc;

pub mod project;
pub mod request_data;
pub mod tags;
pub mod team;
pub mod user;
pub mod version;

#[derive(Clone)]
pub struct ApiV2 {
    pub test_app: Rc<dyn LocalService>,
}

#[async_trait(?Send)]
impl ApiBuildable for ApiV2 {
    async fn build(labrinth_config: LabrinthConfig) -> Self {
        let app = App::new().configure(|cfg| {
            labrinth::app_config(cfg, labrinth_config.clone())
        });
        let test_app: Rc<dyn LocalService> =
            Rc::new(test::init_service(app).await);

        Self { test_app }
    }
}

#[async_trait(?Send)]
impl Api for ApiV2 {
    async fn call(&self, req: actix_http::Request) -> ServiceResponse {
        self.test_app.call(req).await.unwrap()
    }

    async fn reset_search_index(&self) -> ServiceResponse {
        let req = actix_web::test::TestRequest::post()
            .uri("/v2/admin/_force_reindex")
            .append_header((
                "Modrinth-Admin",
                dotenvy::var("LABRINTH_ADMIN_KEY").unwrap(),
            ))
            .to_request();
        self.call(req).await
    }
}

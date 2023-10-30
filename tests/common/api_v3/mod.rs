#![allow(dead_code)]

use super::environment::LocalService;
use actix_web::dev::ServiceResponse;
use std::rc::Rc;

pub mod oauth;
pub mod oauth_clients;

#[derive(Clone)]
pub struct ApiV3 {
    pub test_app: Rc<dyn LocalService>,
}

impl ApiV3 {
    pub async fn call(&self, req: actix_http::Request) -> ServiceResponse {
        self.test_app.call(req).await.unwrap()
    }
}

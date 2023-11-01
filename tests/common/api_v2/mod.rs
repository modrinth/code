#![allow(dead_code)]

use super::environment::LocalService;
use actix_web::dev::ServiceResponse;
use std::rc::Rc;

pub mod organization;
pub mod project;
pub mod team;
pub mod version;

#[derive(Clone)]
pub struct ApiV2 {
    pub test_app: Rc<dyn LocalService>,
}

impl ApiV2 {
    pub async fn call(&self, req: actix_http::Request) -> ServiceResponse {
        self.test_app.call(req).await.unwrap()
    }
}

#![allow(dead_code)]

use std::rc::Rc;

use super::{
    api_v2::ApiV2,
    asserts::assert_status,
    database::{TemporaryDatabase, FRIEND_USER_ID, USER_USER_PAT},
    dummy_data,
};
use crate::common::setup;
use actix_http::StatusCode;
use actix_web::{dev::ServiceResponse, test, App};
use futures::Future;

pub async fn with_test_environment<Fut>(f: impl FnOnce(TestEnvironment) -> Fut)
where
    Fut: Future<Output = ()>,
{
    let test_env = TestEnvironment::build_with_dummy().await;
    let db = test_env.db.clone();

    f(test_env).await;

    db.cleanup().await;
}

// A complete test environment, with a test actix app and a database.
// Must be called in an #[actix_rt::test] context. It also simulates a
// temporary sqlx db like #[sqlx::test] would.
// Use .call(req) on it directly to make a test call as if test::call_service(req) were being used.
pub struct TestEnvironment {
    test_app: Rc<Box<dyn LocalService>>,
    pub db: TemporaryDatabase,
    pub v2: ApiV2,

    pub dummy: Option<dummy_data::DummyData>,
}

impl TestEnvironment {
    pub async fn build_with_dummy() -> Self {
        let mut test_env = Self::build().await;
        let dummy = dummy_data::add_dummy_data(&test_env).await;
        test_env.dummy = Some(dummy);
        test_env
    }

    pub async fn build() -> Self {
        let db = TemporaryDatabase::create().await;
        let labrinth_config = setup(&db).await;
        let app = App::new().configure(|cfg| labrinth::app_config(cfg, labrinth_config.clone()));
        let test_app: Rc<Box<dyn LocalService>> = Rc::new(Box::new(test::init_service(app).await));
        Self {
            v2: ApiV2 {
                test_app: test_app.clone(),
            },
            test_app,
            db,
            dummy: None,
        }
    }
    pub async fn cleanup(self) {
        self.db.cleanup().await;
    }

    pub async fn call(&self, req: actix_http::Request) -> ServiceResponse {
        self.test_app.call(req).await.unwrap()
    }

    pub async fn generate_friend_user_notification(&self) {
        let resp = self
            .v2
            .add_user_to_team(
                &self.dummy.as_ref().unwrap().alpha_team_id,
                FRIEND_USER_ID,
                USER_USER_PAT,
            )
            .await;
        assert_status(resp, StatusCode::NO_CONTENT);
    }
}

pub trait LocalService {
    fn call(
        &self,
        req: actix_http::Request,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ServiceResponse, actix_web::Error>>>,
    >;
}
impl<S> LocalService for S
where
    S: actix_web::dev::Service<
        actix_http::Request,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
{
    fn call(
        &self,
        req: actix_http::Request,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ServiceResponse, actix_web::Error>>>,
    > {
        Box::pin(self.call(req))
    }
}

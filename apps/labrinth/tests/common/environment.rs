#![allow(dead_code)]

use super::{
    api_common::{Api, ApiBuildable, generic::GenericApi},
    api_v2::ApiV2,
    api_v3::ApiV3,
    database::{FRIEND_USER_ID, TemporaryDatabase, USER_USER_PAT},
    dummy_data,
};
use crate::{assert_status, common::setup};
use actix_http::StatusCode;
use actix_web::dev::ServiceResponse;
use futures::Future;

pub async fn with_test_environment<Fut, A>(
    max_connections: Option<u32>,
    f: impl FnOnce(TestEnvironment<A>) -> Fut,
) where
    Fut: Future<Output = ()>,
    A: ApiBuildable + 'static,
{
    let test_env: TestEnvironment<A> =
        TestEnvironment::build(max_connections).await;
    let db = test_env.db.clone();
    f(test_env).await;
    db.cleanup().await;
}

pub async fn with_test_environment_all<Fut, F>(
    max_connections: Option<u32>,
    f: F,
) where
    Fut: Future<Output = ()>,
    F: Fn(TestEnvironment<GenericApi>) -> Fut,
{
    println!("Test environment: API v3");
    let test_env_api_v3 =
        TestEnvironment::<ApiV3>::build(max_connections).await;
    let test_env_api_v3 = TestEnvironment {
        db: test_env_api_v3.db.clone(),
        api: GenericApi::V3(test_env_api_v3.api),
        setup_api: test_env_api_v3.setup_api,
        dummy: test_env_api_v3.dummy,
    };
    let db = test_env_api_v3.db.clone();
    f(test_env_api_v3).await;
    db.cleanup().await;

    println!("Test environment: API v2");
    let test_env_api_v2 =
        TestEnvironment::<ApiV2>::build(max_connections).await;
    let test_env_api_v2 = TestEnvironment {
        db: test_env_api_v2.db.clone(),
        api: GenericApi::V2(test_env_api_v2.api),
        setup_api: test_env_api_v2.setup_api,
        dummy: test_env_api_v2.dummy,
    };
    let db = test_env_api_v2.db.clone();
    f(test_env_api_v2).await;
    db.cleanup().await;
}

// A complete test environment, with a test actix app and a database.
// Must be called in an #[actix_rt::test] context. It also simulates a
// temporary sqlx db like #[sqlx::test] would.
// Use .call(req) on it directly to make a test call as if test::call_service(req) were being used.
#[derive(Clone)]
pub struct TestEnvironment<A> {
    pub db: TemporaryDatabase,
    pub api: A,
    pub setup_api: ApiV3, // Used for setting up tests only (ie: in ScopesTest)
    pub dummy: dummy_data::DummyData,
}

impl<A: ApiBuildable> TestEnvironment<A> {
    async fn build(max_connections: Option<u32>) -> Self {
        let db = TemporaryDatabase::create(max_connections).await;
        let labrinth_config = setup(&db).await;
        let api = A::build(labrinth_config.clone()).await;
        let setup_api = ApiV3::build(labrinth_config).await;
        let dummy = dummy_data::get_dummy_data(&setup_api).await;
        Self {
            db,
            api,
            setup_api,
            dummy,
        }
    }
    pub async fn build_setup_api(db: &TemporaryDatabase) -> ApiV3 {
        let labrinth_config = setup(db).await;
        ApiV3::build(labrinth_config).await
    }
}

impl<A: Api> TestEnvironment<A> {
    pub async fn cleanup(self) {
        self.db.cleanup().await;
    }

    pub async fn call(&self, req: actix_http::Request) -> ServiceResponse {
        self.api.call(req).await
    }

    // Setup data, create a friend user notification
    pub async fn generate_friend_user_notification(&self) {
        let resp = self
            .api
            .add_user_to_team(
                &self.dummy.project_alpha.team_id,
                FRIEND_USER_ID,
                None,
                None,
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);
    }

    // Setup data, assert that a user can read notifications
    pub async fn assert_read_notifications_status(
        &self,
        user_id: &str,
        pat: Option<&str>,
        status_code: StatusCode,
    ) {
        let resp = self.api.get_user_notifications(user_id, pat).await;
        assert_status!(&resp, status_code);
    }

    // Setup data, assert that a user can read projects notifications
    pub async fn assert_read_user_projects_status(
        &self,
        user_id: &str,
        pat: Option<&str>,
        status_code: StatusCode,
    ) {
        let resp = self.api.get_user_projects(user_id, pat).await;
        assert_status!(&resp, status_code);
    }
}

pub trait LocalService {
    fn call(
        &self,
        req: actix_http::Request,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<ServiceResponse, actix_web::Error>,
                >,
        >,
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
        Box<
            dyn std::future::Future<
                    Output = Result<ServiceResponse, actix_web::Error>,
                >,
        >,
    > {
        Box::pin(self.call(req))
    }
}

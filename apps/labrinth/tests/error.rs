use actix_http::StatusCode;
use bytes::Bytes;
use common::api_common::ApiProject;
use ntex::web::test;

use common::api_v3::ApiV3;
use common::database::USER_USER_PAT;
use common::environment::{with_test_environment, TestEnvironment};

mod common;

#[ntex::test]
pub async fn error_404_body() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            // 3 errors should have 404 as non-blank body, for missing resources
            let api = &test_env.api;
            let resp = api.get_project("does-not-exist", USER_USER_PAT).await;
            assert_status!(&resp, StatusCode::NOT_FOUND);
            let body = test::read_body(resp).await;
            let empty_bytes = Bytes::from_static(b"");
            assert_ne!(body, empty_bytes);
        },
    )
    .await;
}

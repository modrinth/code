use crate::assert_status;
use crate::common::api_common::ApiProject;

use actix_http::StatusCode;
use actix_web::test;
use bytes::Bytes;

use crate::common::database::USER_USER_PAT;
use crate::common::{
    api_v2::ApiV2,
    environment::{TestEnvironment, with_test_environment},
};
#[actix_rt::test]
pub async fn error_404_empty() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
            // V2 errors should have 404 as blank body, for missing resources
            let api = &test_env.api;
            let resp = api.get_project("does-not-exist", USER_USER_PAT).await;
            assert_status!(&resp, StatusCode::NOT_FOUND);
            let body = test::read_body(resp).await;
            let empty_bytes = Bytes::from_static(b"");
            assert_eq!(body, empty_bytes);
        },
    )
    .await;
}

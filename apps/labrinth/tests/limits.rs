use common::api_v3::ApiV3;
use common::database::USER_USER_PAT;
use common::environment::{TestEnvironment, with_test_environment};

pub mod common;

#[actix_rt::test]
pub async fn limits() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let limits = api.get_limits(USER_USER_PAT).await;
            assert!(limits.max.projects < u64::MAX);
            assert!(limits.max.organizations < u64::MAX);
            assert!(limits.max.collections < u64::MAX);
        },
    )
    .await;
}

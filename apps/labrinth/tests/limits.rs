use common::api_v3::ApiV3;
use common::database::USER_USER_PAT;
use common::environment::{TestEnvironment, with_test_environment};

use crate::common::api_common::ApiProject;

pub mod common;

#[actix_rt::test]
pub async fn limits() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;

            let project_limits = api.get_project_limits(USER_USER_PAT).await;
            assert_eq!(project_limits.current, 2);
            assert!(project_limits.max < u64::MAX);

            api.add_public_project(
                "limit-test-project",
                None,
                None,
                USER_USER_PAT,
            )
            .await;
            let project_limits = api.get_project_limits(USER_USER_PAT).await;
            assert_eq!(project_limits.current, 3);
        },
    )
    .await;
}

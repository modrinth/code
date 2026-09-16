use actix_http::StatusCode;
use actix_web::test;
use common::api_v3::ApiV3;
use common::database::{USER_USER_ID_PARSED, USER_USER_PAT};
use common::dummy_data::TestFile;
use common::environment::{TestEnvironment, with_test_environment};
use labrinth::database::PgPool;
use labrinth::database::models::{
    DBProjectId, DBUserId, user_limits::DBUserLimits,
};
use serde_json::Value;

use crate::common::api_common::{ApiProject, ApiVersion};

pub mod common;

async fn set_version_limits(
    pool: &PgPool,
    versions_per_project: u64,
    versions_per_day: u64,
) {
    let defaults = DBUserLimits::get_defaults(pool).await.unwrap();
    DBUserLimits {
        user_id: Some(DBUserId(USER_USER_ID_PARSED)),
        versions_per_project,
        versions_per_day,
        ..defaults
    }
    .upsert(pool)
    .await
    .unwrap();
}

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

#[actix_rt::test]
pub async fn max_versions_per_project() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let project_id = test_env.dummy.project_alpha.project_id_parsed;
            let db_project_id = DBProjectId::from(project_id);
            let current_project_versions = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM versions WHERE mod_id = $1",
                db_project_id as DBProjectId,
            )
            .fetch_one(&test_env.db.pool)
            .await
            .unwrap()
            .unwrap_or(0) as u64;

            set_version_limits(
                &test_env.db.pool,
                current_project_versions + 1,
                10_000,
            )
            .await;

            let allowed = api
                .add_public_version(
                    project_id,
                    "project-limit-allowed",
                    TestFile::build_random_jar(),
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&allowed, StatusCode::OK);

            let blocked = api
                .add_public_version(
                    project_id,
                    "project-limit-blocked",
                    TestFile::build_random_jar(),
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&blocked, StatusCode::BAD_REQUEST);
            let error: Value = test::read_body_json(blocked).await;
            assert_eq!(error["error"], "limit_reached");
            assert_eq!(error["description"], "project version limit reached");
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn max_versions_uploaded_per_day() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let project_id = test_env.dummy.project_alpha.project_id_parsed;
            let current_daily_versions = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM versions
                WHERE author_id = $1
                    AND date_published >= (
                        (NOW() AT TIME ZONE 'UTC')::date AT TIME ZONE 'UTC'
                    )",
                DBUserId(USER_USER_ID_PARSED) as DBUserId,
            )
            .fetch_one(&test_env.db.pool)
            .await
            .unwrap()
            .unwrap_or(0) as u64;

            set_version_limits(
                &test_env.db.pool,
                10_000,
                current_daily_versions + 1,
            )
            .await;

            let allowed = api
                .add_public_version(
                    project_id,
                    "daily-limit-allowed",
                    TestFile::build_random_jar(),
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&allowed, StatusCode::OK);

            let blocked = api
                .add_public_version(
                    project_id,
                    "daily-limit-blocked",
                    TestFile::build_random_jar(),
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&blocked, StatusCode::BAD_REQUEST);
            let error: Value = test::read_body_json(blocked).await;
            assert_eq!(error["error"], "limit_reached");
            assert_eq!(
                error["description"],
                "daily version upload limit reached"
            );
        },
    )
    .await;
}

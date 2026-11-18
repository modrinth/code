use actix_http::StatusCode;
use actix_web::test;
use common::api_common::{Api, ApiProject, AppendsOptionalPat};
use common::api_v2::ApiV2;
use common::database::*;
use common::environment::{TestEnvironment, with_test_environment};

use labrinth::models::projects::{
    ProjectDelta, ProjectReviewStatus, ProjectVisibility,
    ReviewStatusProvenance,
};
use labrinth::test::db::initialize_project_state;
use serde_json::{Value, json};

pub mod common;

type Environment = TestEnvironment<ApiV2>;

const SUMMARY: &str = "Configurable exploration tools for adventurous players.";
const BODY: &str = "Discover custom structures and explore new dimensions with a collection of configurable tools. Install the mod with Fabric, then configure each feature independently to suit your world. Both single-player adventures and multiplayer servers are supported.";

async fn initialize(env: &Environment, status: ProjectReviewStatus) {
    let id = &env.dummy.project_alpha.project_id;
    let response = env.api.get_project(id, ADMIN_USER_PAT).await;
    assert_status!(&response, StatusCode::OK);
    initialize_project_state(
        &response,
        id,
        status,
        Some(ProjectVisibility::Listed),
    )
    .await
    .unwrap();
}

async fn project(env: &Environment) -> Value {
    let response = env
        .api
        .get_project(&env.dummy.project_alpha.project_id, ADMIN_USER_PAT)
        .await;
    assert_status!(&response, StatusCode::OK);
    test::read_body_json(response).await
}

async fn state(env: &Environment) -> (String, String) {
    let row = sqlx::query!(
        r#"SELECT review_status, visibility FROM mods WHERE id = $1"#,
        env.dummy.project_alpha.project_id_parsed.0 as i64,
    )
    .fetch_one(&**env.db.ro_pool)
    .await
    .unwrap();
    (row.review_status, row.visibility)
}

async fn review_history(env: &Environment) -> Vec<(Option<i64>, Value)> {
    sqlx::query!(
        r#"SELECT done_by, delta FROM project_history WHERE project_id = $1 ORDER BY occurred_at, id"#,
        env.dummy.project_alpha.project_id_parsed.0 as i64,
    )
    .fetch_all(&**env.db.ro_pool)
    .await
    .unwrap()
    .into_iter()
    .filter(|row| matches!(serde_json::from_value::<ProjectDelta>(row.delta.clone()).unwrap(), ProjectDelta::ReviewStatus { .. }))
    .map(|row| (row.done_by, row.delta))
    .collect()
}

#[actix_rt::test]
async fn combined_edits_are_validated_before_submission() {
    with_test_environment(None, |env: Environment| async move {
        initialize(&env, ProjectReviewStatus::Draft).await;
        let id = &env.dummy.project_alpha.project_id;
        let response = env
            .api
            .edit_project(id, json!({"body": "Too short"}), USER_USER_PAT)
            .await;
        assert_status!(&response, StatusCode::NO_CONTENT);
        let before = review_history(&env).await;
        let response = env.api.edit_project(id, json!({
            "body": BODY, "description": SUMMARY, "status": "processing",
            "client_side": "unsupported", "server_side": "required",
        }), USER_USER_PAT).await;
        assert_status!(&response, StatusCode::NO_CONTENT);
        let saved = project(&env).await;
        assert_eq!(saved["body"], BODY);
        assert_eq!(saved["description"], SUMMARY);
        assert_eq!(saved["client_side"], "unsupported");
        assert_eq!(saved["server_side"], "required");
        assert_eq!(state(&env).await, ("in_review".into(), "listed".into()));
        assert_eq!(review_history(&env).await.len(), before.len() + 1);
    })
    .await;
}

#[actix_rt::test]
async fn invalid_ordinary_edits_prevent_review_transition() {
    with_test_environment(None, |env: Environment| async move {
        initialize(&env, ProjectReviewStatus::Draft).await;
        let original = project(&env).await;
        let before = review_history(&env).await;
        let response = env.api.edit_project(&env.dummy.project_alpha.project_id, json!({
            "description": "x".repeat(257), "body": BODY, "status": "processing",
        }), USER_USER_PAT).await;
        assert_status!(&response, StatusCode::BAD_REQUEST);
        assert_eq!(project(&env).await, original);
        assert_eq!(state(&env).await, ("draft".into(), "listed".into()));
        assert_eq!(review_history(&env).await, before);
    }).await;
}

#[actix_rt::test]
async fn slug_edit_submits_using_canonical_id() {
    with_test_environment(None, |env: Environment| async move {
        initialize(&env, ProjectReviewStatus::Draft).await;
        let before = review_history(&env).await;
        let response = env
            .api
            .edit_project(
                &env.dummy.project_alpha.project_slug,
                json!({
                    "slug": "renamed-v2-review-project", "body": BODY,
                    "description": SUMMARY, "status": "processing",
                }),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&response, StatusCode::NO_CONTENT);
        assert_eq!(project(&env).await["slug"], "renamed-v2-review-project");
        assert_eq!(state(&env).await, ("in_review".into(), "listed".into()));
        assert_eq!(review_history(&env).await.len(), before.len() + 1);
    })
    .await;
}

#[actix_rt::test]
async fn moderator_legacy_status_changes_have_user_provenance() {
    with_test_environment(None, |env: Environment| async move {
        for (legacy, target) in [
            ("rejected", ProjectReviewStatus::Rejected),
            ("approved", ProjectReviewStatus::Approved),
        ] {
            initialize(&env, ProjectReviewStatus::InReview).await;
            let before = review_history(&env).await;
            let response = env
                .api
                .edit_project(
                    &env.dummy.project_alpha.project_id,
                    json!({"status": legacy}),
                    MOD_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);
            assert_eq!(
                state(&env).await,
                (target.as_str().into(), "listed".into())
            );
            let after = review_history(&env).await;
            assert_eq!(after.len(), before.len() + 1);
            let (actor, delta) = after.last().unwrap();
            assert_eq!(*actor, Some(MOD_USER_ID_PARSED));
            match serde_json::from_value::<ProjectDelta>(delta.clone()).unwrap()
            {
                ProjectDelta::ReviewStatus {
                    from,
                    to,
                    because: ReviewStatusProvenance::ByUser { reason },
                } => {
                    assert_eq!(from, ProjectReviewStatus::InReview);
                    assert_eq!(to, target);
                    assert!(delta["because"].get("user_id").is_none());
                    assert_eq!(reason, None);
                }
                other => panic!("unexpected review delta: {other:?}"),
            }
        }
    })
    .await;
}

#[actix_rt::test]
async fn approved_owner_legacy_status_only_changes_visibility() {
    with_test_environment(None, |env: Environment| async move {
        initialize(&env, ProjectReviewStatus::Approved).await;
        let before = review_history(&env).await;
        for visibility in ["unlisted", "private"] {
            let response = env
                .api
                .edit_project(
                    &env.dummy.project_alpha.project_id,
                    json!({"status": visibility}),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);
            assert_eq!(
                state(&env).await,
                ("approved".into(), visibility.into())
            );
            assert_eq!(review_history(&env).await, before);
        }
    })
    .await;
}

#[actix_rt::test]
async fn draft_requested_status_is_preserved_when_submitted() {
    with_test_environment(None, |env: Environment| async move {
        initialize(&env, ProjectReviewStatus::Draft).await;
        let id = &env.dummy.project_alpha.project_id;
        let before = review_history(&env).await;
        let response = env
            .api
            .edit_project(
                id,
                json!({"requested_status": "unlisted"}),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&response, StatusCode::NO_CONTENT);
        assert_eq!(state(&env).await, ("draft".into(), "unlisted".into()));
        assert_eq!(review_history(&env).await, before);
        let response = env
            .api
            .edit_project(id, json!({"status": "processing"}), USER_USER_PAT)
            .await;
        assert_status!(&response, StatusCode::NO_CONTENT);
        assert_eq!(state(&env).await, ("in_review".into(), "unlisted".into()));
        assert_eq!(project(&env).await["requested_status"], "unlisted");
        assert_eq!(review_history(&env).await.len(), before.len() + 1);
    })
    .await;
}

#[actix_rt::test]
async fn unsupported_legacy_statuses_do_not_apply_edits() {
    with_test_environment(None, |env: Environment| async move {
        initialize(&env, ProjectReviewStatus::Draft).await;
        let original = project(&env).await;
        let before = review_history(&env).await;
        for field in ["status", "requested_status"] {
            for unsupported in ["unknown", "scheduled"] {
                let mut body = json!({"body": BODY, "description": SUMMARY});
                body[field] = json!(unsupported);
                let response = env
                    .api
                    .edit_project(
                        &env.dummy.project_alpha.project_id,
                        body,
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&response, StatusCode::BAD_REQUEST);
                assert_eq!(project(&env).await, original);
                assert_eq!(
                    state(&env).await,
                    ("draft".into(), "listed".into())
                );
                assert_eq!(review_history(&env).await, before);
            }
        }
    })
    .await;
}

/// Legacy PATCH commits ordinary edits before the separate review transition;
/// a later permission or review-validation failure does not roll those edits back.
#[actix_rt::test]
async fn ordinary_edits_survive_later_review_failure() {
    with_test_environment(None, |env: Environment| async move {
        for (status, body, expected) in [
            ("approved", BODY, StatusCode::FORBIDDEN),
            ("processing", "Too short", StatusCode::BAD_REQUEST),
            ("processing", SUMMARY, StatusCode::BAD_REQUEST),
        ] {
            initialize(&env, ProjectReviewStatus::Draft).await;
            let before = review_history(&env).await;
            let response = env
                .api
                .edit_project(
                    &env.dummy.project_alpha.project_id,
                    json!({
                        "body": body, "description": SUMMARY, "status": status,
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, expected);
            let saved = project(&env).await;
            assert_eq!(saved["body"], body);
            assert_eq!(saved["description"], SUMMARY);
            assert_eq!(state(&env).await, ("draft".into(), "listed".into()));
            assert_eq!(review_history(&env).await, before);
        }
    })
    .await;
}

#[actix_rt::test]
async fn explicit_v2_review_state_route_remains_available() {
    with_test_environment(None, |env: Environment| async move {
        initialize(&env, ProjectReviewStatus::Draft).await;
        let before = review_history(&env).await;
        let response = env
            .api
            .call(
                test::TestRequest::post()
                    .uri(&format!(
                        "/v2/project/{}/review-state",
                        env.dummy.project_alpha.project_id
                    ))
                    .append_pat(USER_USER_PAT)
                    .set_json(json!({"review_status": "in_review"}))
                    .to_request(),
            )
            .await;
        assert_status!(&response, StatusCode::NO_CONTENT);
        assert_eq!(state(&env).await, ("in_review".into(), "listed".into()));
        assert_eq!(review_history(&env).await.len(), before.len() + 1);
    })
    .await;
}

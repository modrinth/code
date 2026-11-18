use actix_http::StatusCode;
use actix_web::{dev::ServiceResponse, test};
use common::api_common::{
    Api, ApiProject, ApiTeams, AppendsOptionalPat, generic::GenericApi,
};
use common::database::*;
use common::environment::{TestEnvironment, with_test_environment_all};
use labrinth::database::models::moderation_lock_item::{
    DBModerationLock, LOCK_EXPIRY_MINUTES,
};
use labrinth::database::models::{DBProjectId, DBUserId};
use labrinth::models::projects::{
    ProjectDelta, ProjectReviewStatus, ProjectVisibility,
    ReviewStatusProvenance,
};
use labrinth::test::db::initialize_project_state;
use serde_json::{Value, json};

pub mod common;

type Environment = TestEnvironment<GenericApi>;

#[actix_rt::test]
async fn open_moderation_issues_block_submission_and_approval() {
    with_test_environment_all(None, |env| async move {
        initialize(
            &env,
            ProjectReviewStatus::Draft,
            Some(ProjectVisibility::Private),
        )
        .await;
        let response = env
            .api
            .call(
                test::TestRequest::put()
                    .uri(&format!(
                        "/v3/thread/{}/issue",
                        env.dummy.project_alpha.thread_id
                    ))
                    .append_pat(MOD_USER_PAT)
                    .set_json(json!({"issues": [{
                        "why": {"message": "Resolve before review"},
                        "facets": [{"what": {"type": "mark_addressed"}}]
                    }]}))
                    .to_request(),
            )
            .await;
        assert_status!(&response, StatusCode::NO_CONTENT);
        for (source, target, pat) in [
            (ProjectReviewStatus::Draft, "in_review", USER_USER_PAT),
            (ProjectReviewStatus::Rejected, "in_review", USER_USER_PAT),
            (ProjectReviewStatus::InReview, "approved", MOD_USER_PAT),
        ] {
            initialize(&env, source, None).await;
            sqlx::query!(
                r#"UPDATE mods SET moderation_message = 'Changes required',
                    moderation_message_body = 'Resolve the outstanding issue',
                    queued = NOW() - INTERVAL '2 days',
                    initial_queued = NOW() - INTERVAL '3 days'
                    WHERE id = $1"#,
                env.dummy.project_alpha.project_id_parsed.0 as i64,
            )
            .execute(&env.db.pool)
            .await
            .unwrap();
            let before = history(&env).await;
            let metadata_before = review_metadata(&env).await;
            assert!(!metadata_before["queued"].is_null());
            assert!(!metadata_before["initial_queued"].is_null());
            assert_eq!(
                metadata_before["moderation_message"],
                "Changes required"
            );
            assert_eq!(
                metadata_before["moderation_message_body"],
                "Resolve the outstanding issue"
            );
            let thread_before = thread(&env).await;
            let notifications_before = notifications(&env).await;
            let response =
                update(&env, json!({"review_status": target}), pat).await;
            assert_status!(&response, StatusCode::BAD_REQUEST);
            assert_state(&env, source, ProjectVisibility::Private).await;
            assert_eq!(history(&env).await, before);
            assert_eq!(review_metadata(&env).await, metadata_before);
            assert_eq!(thread(&env).await, thread_before);
            assert_eq!(notifications(&env).await, notifications_before);
        }
    })
    .await;
}

async fn review_metadata(env: &Environment) -> Value {
    sqlx::query_scalar!(
        r#"SELECT to_jsonb(snapshot) AS "metadata!" FROM (
            SELECT queued, initial_queued, moderation_message,
                moderation_message_body, approved, approved_because
            FROM mods WHERE id = $1
        ) snapshot"#,
        env.dummy.project_alpha.project_id_parsed.0 as i64,
    )
    .fetch_one(&**env.db.ro_pool)
    .await
    .unwrap()
}

async fn thread(env: &Environment) -> Value {
    let response = env
        .api
        .get_thread(&env.dummy.project_alpha.thread_id, MOD_USER_PAT)
        .await;
    assert_status!(&response, StatusCode::OK);
    test::read_body_json(response).await
}

async fn notifications(env: &Environment) -> Value {
    let response = env
        .api
        .get_user_notifications(USER_USER_ID, USER_USER_PAT)
        .await;
    assert_status!(&response, StatusCode::OK);
    test::read_body_json(response).await
}

#[actix_rt::test]
async fn owner_cannot_return_unapproved_project_to_draft() {
    with_test_environment_all(None, |env| async move {
        for source in [
            ProjectReviewStatus::Rejected,
            ProjectReviewStatus::Withheld,
            ProjectReviewStatus::InReview,
        ] {
            initialize(&env, source, Some(ProjectVisibility::Private)).await;
            let before = history(&env).await;
            let response =
                update(&env, json!({"review_status": "draft"}), USER_USER_PAT)
                    .await;
            assert_status!(&response, StatusCode::FORBIDDEN);
            assert_state(&env, source, ProjectVisibility::Private).await;
            assert_eq!(history(&env).await, before);
        }
    })
    .await;
}

#[actix_rt::test]
async fn resolved_rejected_resubmission_auto_approves() {
    with_test_environment_all(None, |env| async move {
        for visibility in [ProjectVisibility::Unlisted, ProjectVisibility::Private] {
            initialize(&env, ProjectReviewStatus::Rejected, Some(visibility)).await;
            let response = env.api.call(test::TestRequest::put()
                .uri(&format!("/v3/thread/{}/issue", env.dummy.project_alpha.thread_id))
                .append_pat(MOD_USER_PAT)
                .set_json(json!({"issues": [{
                    "why": {"message": "Remove unrelated tags"},
                    "facets": [{"what": {"type": "remove_tags", "value": {"tags": ["magic"]}}}]
                }]})).to_request()).await;
            assert_status!(&response, StatusCode::NO_CONTENT);
            sqlx::query!(
                r#"UPDATE mods SET initial_queued = NOW() - INTERVAL '3 days' WHERE id = $1"#,
                env.dummy.project_alpha.project_id_parsed.0 as i64,
            ).execute(&env.db.pool).await.unwrap();
            assert!(!review_metadata(&env).await["initial_queued"].is_null());
            let before_thread = thread(&env).await;
            let issues = before_thread["issues"].as_array().unwrap();
            assert!(!issues.is_empty());
            assert!(issues.iter().all(|issue| issue["verdict"] == "resolved"));
            let mut expected_ids = issues.iter().map(|issue| issue["id"].as_str().unwrap().to_owned()).collect::<Vec<_>>();
            expected_ids.sort();
            let before = history(&env).await;
            let response = update(&env, json!({"review_status": "in_review"}), USER_USER_PAT).await;
            assert_status!(&response, StatusCode::NO_CONTENT);
            assert_state(&env, ProjectReviewStatus::Approved, visibility).await;
            assert!(review_metadata(&env).await["initial_queued"].is_null());
            let after = history(&env).await;
            assert_eq!(after.len(), before.len() + 1);
            let (done_by, delta) = after.last().unwrap();
            assert_eq!(*done_by, None);
            assert_eq!(delta["kind"], "review_status");
            assert_eq!(delta["from"], "rejected");
            assert_eq!(delta["to"], "approved");
            assert_eq!(delta["because"]["kind"], "auto_approval");
            assert_eq!(delta["because"]["trigger"], json!({"type": "resubmission", "user_id": USER_USER_ID}));
            let mut actual_ids = delta["because"]["resolved_issue_ids"].as_array().unwrap().iter().map(|id| id.as_str().unwrap().to_owned()).collect::<Vec<_>>();
            actual_ids.sort();
            assert_eq!(actual_ids, expected_ids);
        }
    }).await;
}

#[actix_rt::test]
async fn moderator_can_set_native_review_targets() {
    with_test_environment_all(None, |env| async move {
        for target in [
            ProjectReviewStatus::Draft,
            ProjectReviewStatus::InReview,
            ProjectReviewStatus::Approved,
            ProjectReviewStatus::Rejected,
            ProjectReviewStatus::Withheld,
        ] {
            let source = if target == ProjectReviewStatus::Draft {
                ProjectReviewStatus::InReview
            } else {
                ProjectReviewStatus::Draft
            };
            initialize(&env, source, Some(ProjectVisibility::Unlisted)).await;
            let before = history(&env).await;
            let response = update(
                &env,
                json!({"review_status": target.as_str()}),
                MOD_USER_PAT,
            )
            .await;
            assert_status!(&response, StatusCode::NO_CONTENT);
            assert_state(&env, target, ProjectVisibility::Unlisted).await;
            assert_eq!(history(&env).await.len(), before.len() + 1);
        }
    })
    .await;
}

async fn initialize(
    env: &Environment,
    status: ProjectReviewStatus,
    visibility: Option<ProjectVisibility>,
) {
    let id = &env.dummy.project_alpha.project_id;
    let response = env.api.get_project(id, ADMIN_USER_PAT).await;
    assert_status!(&response, StatusCode::OK);
    initialize_project_state(&response, id, status, visibility)
        .await
        .unwrap();
}

async fn update(
    env: &Environment,
    body: Value,
    pat: Option<&str>,
) -> ServiceResponse {
    let prefix = match &env.api {
        GenericApi::V2(_) => "v2",
        GenericApi::V3(_) => "v3",
    };
    env.api
        .call(
            test::TestRequest::post()
                .uri(&format!(
                    "/{prefix}/project/{}/review-state",
                    env.dummy.project_alpha.project_id
                ))
                .append_pat(pat)
                .set_json(body)
                .to_request(),
        )
        .await
}

async fn history(env: &Environment) -> Vec<(Option<i64>, Value)> {
    sqlx::query!(
        r#"SELECT done_by, delta FROM project_history WHERE project_id = $1 ORDER BY occurred_at, id"#,
        env.dummy.project_alpha.project_id_parsed.0 as i64,
    )
    .fetch_all(&**env.db.ro_pool)
    .await.unwrap()
    .into_iter().map(|row| (row.done_by, row.delta)).collect()
}

async fn assert_state(
    env: &Environment,
    status: ProjectReviewStatus,
    visibility: ProjectVisibility,
) {
    let row = sqlx::query!(
        r#"SELECT review_status, visibility FROM mods WHERE id = $1"#,
        env.dummy.project_alpha.project_id_parsed.0 as i64,
    )
    .fetch_one(&**env.db.ro_pool)
    .await
    .unwrap();
    assert_eq!(row.review_status, status.as_str());
    assert_eq!(row.visibility, visibility.as_str());
}

#[actix_rt::test]
async fn owner_submission_preserves_visibility() {
    with_test_environment_all(None, |env| async move {
        for visibility in
            [ProjectVisibility::Unlisted, ProjectVisibility::Private]
        {
            initialize(&env, ProjectReviewStatus::Draft, Some(visibility))
                .await;
            let before = history(&env).await;
            let response = update(
                &env,
                json!({"review_status": "in_review"}),
                USER_USER_PAT,
            )
            .await;
            assert_status!(&response, StatusCode::NO_CONTENT);
            assert_state(&env, ProjectReviewStatus::InReview, visibility).await;
            assert_eq!(history(&env).await.len(), before.len() + 1);
        }
    })
    .await;
}

#[actix_rt::test]
async fn owner_submission_requires_valid_project() {
    with_test_environment_all(None, |env| async move {
        initialize(
            &env,
            ProjectReviewStatus::Draft,
            Some(ProjectVisibility::Private),
        )
        .await;
        let response = env
            .api
            .edit_project(
                &env.dummy.project_alpha.project_id,
                json!({"description": "Too short"}),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&response, StatusCode::NO_CONTENT);
        let before = history(&env).await;
        let response =
            update(&env, json!({"review_status": "in_review"}), USER_USER_PAT)
                .await;
        assert_status!(&response, StatusCode::BAD_REQUEST);
        assert_state(
            &env,
            ProjectReviewStatus::Draft,
            ProjectVisibility::Private,
        )
        .await;
        assert_eq!(history(&env).await, before);
    })
    .await;
}

#[actix_rt::test]
async fn owner_cannot_self_approve_or_change_approved_review() {
    with_test_environment_all(None, |env| async move {
        for source in
            [ProjectReviewStatus::Draft, ProjectReviewStatus::Approved]
        {
            initialize(&env, source, Some(ProjectVisibility::Unlisted)).await;
            let before = history(&env).await;
            let targets = if source == ProjectReviewStatus::Approved {
                vec!["in_review", "draft", "rejected", "withheld"]
            } else {
                vec!["approved", "rejected", "withheld"]
            };
            for target in targets {
                let response = update(
                    &env,
                    json!({"review_status": target}),
                    USER_USER_PAT,
                )
                .await;
                assert_status!(&response, StatusCode::FORBIDDEN);
                assert_state(&env, source, ProjectVisibility::Unlisted).await;
                assert_eq!(history(&env).await, before);
            }
        }
    })
    .await;
}

#[actix_rt::test]
async fn moderator_approval_records_user_and_reason_without_changing_visibility()
 {
    with_test_environment_all(None, |env| async move {
        for visibility in [ProjectVisibility::Unlisted, ProjectVisibility::Private] {
            initialize(&env, ProjectReviewStatus::InReview, Some(visibility)).await;
            let before = history(&env).await;
            let response = update(&env, json!({"review_status": "approved", "reason": "Reviewed project and files"}), MOD_USER_PAT).await;
            assert_status!(&response, StatusCode::NO_CONTENT);
            assert_state(&env, ProjectReviewStatus::Approved, visibility).await;
            let after = history(&env).await;
            assert_eq!(after.len(), before.len() + 1);
            let (done_by, delta) = after.last().unwrap();
            assert_eq!(*done_by, Some(MOD_USER_ID_PARSED));
            match serde_json::from_value::<ProjectDelta>(delta.clone()).unwrap() {
                ProjectDelta::ReviewStatus { from, to, because: ReviewStatusProvenance::ByUser { user_id, reason } } => {
                    assert_eq!(from, ProjectReviewStatus::InReview);
                    assert_eq!(to, ProjectReviewStatus::Approved);
                    assert_eq!(DBUserId::from(user_id), DBUserId(MOD_USER_ID_PARSED));
                    assert_eq!(reason.as_deref(), Some("Reviewed project and files"));
                }
                other => panic!("unexpected audit delta: {other:?}"),
            }
        }
    }).await;
}

#[actix_rt::test]
async fn same_state_is_no_content_without_audit() {
    with_test_environment_all(None, |env| async move {
        for status in [ProjectReviewStatus::Draft, ProjectReviewStatus::InReview, ProjectReviewStatus::Approved, ProjectReviewStatus::Rejected, ProjectReviewStatus::Withheld] {
            initialize(&env, status, Some(ProjectVisibility::Private)).await;
            let before = history(&env).await;
            let response = update(&env, json!({"review_status": status.as_str(), "reason": "No change"}), MOD_USER_PAT).await;
            assert_status!(&response, StatusCode::NO_CONTENT);
            assert_state(&env, status, ProjectVisibility::Private).await;
            assert_eq!(history(&env).await, before);
        }
    }).await;
}

#[actix_rt::test]
async fn invalid_schema_and_unauthorized_requests_do_not_mutate() {
    with_test_environment_all(None, |env| async move {
        initialize(
            &env,
            ProjectReviewStatus::Draft,
            Some(ProjectVisibility::Private),
        )
        .await;
        let before = history(&env).await;
        for body in [
            json!({}),
            json!({"review_status": "processing"}),
            json!({"review_status": "unlisted"}),
            json!({"review_status": "unknown"}),
            json!({"review_status": null}),
            json!({"review_status": 1}),
            json!({"review_status": "in_review", "reason": 1}),
        ] {
            let response = update(&env, body, MOD_USER_PAT).await;
            assert_status!(&response, StatusCode::BAD_REQUEST);
        }
        for pat in [None, FRIEND_USER_PAT] {
            let response =
                update(&env, json!({"review_status": "in_review"}), pat).await;
            assert_status!(&response, StatusCode::FORBIDDEN);
        }
        assert_state(
            &env,
            ProjectReviewStatus::Draft,
            ProjectVisibility::Private,
        )
        .await;
        assert_eq!(history(&env).await, before);
    })
    .await;
}

#[actix_rt::test]
async fn moderation_locks_gate_review_transitions() {
    with_test_environment_all(None, |env| async move {
        let id = DBProjectId::from(env.dummy.project_alpha.project_id_parsed);
        for scenario in ["absent", "own", "other", "expired", "admin"] {
            initialize(&env, ProjectReviewStatus::InReview, Some(ProjectVisibility::Unlisted)).await;
            DBModerationLock::release(id, DBUserId(MOD_USER_ID_PARSED), &env.db.pool).await.unwrap();
            DBModerationLock::release(id, DBUserId(ADMIN_USER_ID_PARSED), &env.db.pool).await.unwrap();
            if scenario != "absent" {
                let holder = if scenario == "own" || scenario == "admin" { MOD_USER_ID_PARSED } else { ADMIN_USER_ID_PARSED };
                DBModerationLock::acquire(id, DBUserId(holder), &env.db.pool).await.unwrap().unwrap();
            }
            if scenario == "expired" {
                sqlx::query!(r#"UPDATE moderation_locks SET locked_at = NOW() - ($1::bigint * INTERVAL '1 minute') - INTERVAL '1 second' WHERE project_id = $2"#, LOCK_EXPIRY_MINUTES, id.0)
                    .execute(&env.db.pool).await.unwrap();
            }
            let before = history(&env).await;
            let pat = if scenario == "admin" { ADMIN_USER_PAT } else { MOD_USER_PAT };
            let response = update(&env, json!({"review_status": "rejected"}), pat).await;
            if scenario == "other" {
                assert_status!(&response, StatusCode::FORBIDDEN);
                assert_state(&env, ProjectReviewStatus::InReview, ProjectVisibility::Unlisted).await;
                assert_eq!(history(&env).await, before);
            } else {
                assert_status!(&response, StatusCode::NO_CONTENT);
                assert_state(&env, ProjectReviewStatus::Rejected, ProjectVisibility::Unlisted).await;
                assert_eq!(history(&env).await.len(), before.len() + 1);
            }
        }
    }).await;
}

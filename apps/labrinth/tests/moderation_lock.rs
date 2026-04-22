use actix_http::StatusCode;
use actix_web::http::header;
use actix_web::test;
use bytes::Bytes;
use common::api_common::Api;
use common::database::*;
use common::environment::{TestEnvironment, with_test_environment};
use labrinth::database::models::moderation_lock_item::{
    DBModerationLock, LOCK_EXPIRY_MINUTES,
};
use labrinth::database::models::project_item::DBProject;
use labrinth::database::models::{DBProjectId, DBUserId};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::Barrier;

pub mod common;

/// Force a project into `processing` status directly in the database.
/// Clears the Redis project row so `DBProject::get` used by PATCH sees the new status.
async fn set_project_processing(
    project_id: i64,
    project_slug: &str,
    pool: &labrinth::database::PgPool,
    redis: &labrinth::database::redis::RedisPool,
) {
    sqlx::query("UPDATE mods SET status = 'processing' WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .expect("failed to set project to processing status");

    DBProject::clear_cache(
        DBProjectId(project_id),
        Some(project_slug.to_string()),
        None,
        redis,
    )
    .await
    .expect("failed to clear project cache after status update");
}

/// Back-date a lock's `locked_at` so it appears expired.
async fn expire_lock(project_id: i64, pool: &labrinth::database::PgPool) {
    sqlx::query(
        "UPDATE moderation_locks SET locked_at = NOW() - ($1::bigint * INTERVAL '1 minute') - INTERVAL '1 second' WHERE project_id = $2",
    )
    .bind(LOCK_EXPIRY_MINUTES)
    .bind(project_id)
    .execute(pool)
    .await
    .expect("failed to expire lock");
}

#[actix_rt::test]
async fn test_get_with_user_returns_none_when_unlocked() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let pool = &test_env.db.pool;

            let row = DBModerationLock::get_with_user(project_id, pool)
                .await
                .expect("get_with_user should not error");

            assert!(
                row.is_none(),
                "no row in moderation_locks means no active lock record"
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_release_deletes_lock_for_holder() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod_id = DBUserId(MOD_USER_ID_PARSED);
            let pool = &test_env.db.pool;

            DBModerationLock::acquire(project_id, mod_id, pool)
                .await
                .unwrap()
                .unwrap();

            let released = DBModerationLock::release(project_id, mod_id, pool)
                .await
                .expect("release should not error");
            assert!(released, "holder should delete their lock");

            let after = DBModerationLock::get_with_user(project_id, pool)
                .await
                .unwrap();
            assert!(after.is_none(), "lock row should be gone after release");
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_release_noop_for_wrong_moderator() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod_id = DBUserId(MOD_USER_ID_PARSED);
            let other = DBUserId(ADMIN_USER_ID_PARSED);
            let pool = &test_env.db.pool;

            DBModerationLock::acquire(project_id, mod_id, pool)
                .await
                .unwrap()
                .unwrap();

            let released = DBModerationLock::release(project_id, other, pool)
                .await
                .expect("release should not error");
            assert!(!released, "non-holder must not delete the lock");

            let still = DBModerationLock::get_with_user(project_id, pool)
                .await
                .unwrap();
            assert!(
                still.is_some(),
                "lock should remain when wrong user releases"
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_release_noop_when_no_lock() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod_id = DBUserId(MOD_USER_ID_PARSED);
            let pool = &test_env.db.pool;

            let released = DBModerationLock::release(project_id, mod_id, pool)
                .await
                .expect("release should not error");
            assert!(!released, "releasing a non-existent lock is a no-op");
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_cleanup_expired_removes_stale_locks() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod_id = DBUserId(MOD_USER_ID_PARSED);
            let pool = &test_env.db.pool;

            DBModerationLock::acquire(project_id, mod_id, pool)
                .await
                .unwrap()
                .unwrap();
            expire_lock(project_id.0, pool).await;

            let n = DBModerationLock::cleanup_expired(pool)
                .await
                .expect("cleanup should not error");
            assert!(n >= 1, "at least one expired lock should be removed");

            let after = DBModerationLock::get_with_user(project_id, pool)
                .await
                .unwrap();
            assert!(
                after.is_none(),
                "expired lock should be gone after cleanup"
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_lock_acquire_fresh() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod_id = DBUserId(MOD_USER_ID_PARSED);
            let pool = &test_env.db.pool;

            let result = DBModerationLock::acquire(project_id, mod_id, pool)
                .await
                .expect("acquire should not return a DB error");

            assert!(
                result.is_ok(),
                "moderator should acquire fresh lock without contention"
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_lock_acquire_refresh_own() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod_id = DBUserId(MOD_USER_ID_PARSED);
            let pool = &test_env.db.pool;

            DBModerationLock::acquire(project_id, mod_id, pool)
                .await
                .unwrap()
                .unwrap();

            let result = DBModerationLock::acquire(project_id, mod_id, pool)
                .await
                .expect("second acquire should not return a DB error");

            assert!(
                result.is_ok(),
                "same moderator should be able to refresh own lock"
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_lock_acquire_blocked_by_other() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod1 = DBUserId(MOD_USER_ID_PARSED);
            let mod2 = DBUserId(ADMIN_USER_ID_PARSED);
            let pool = &test_env.db.pool;

            DBModerationLock::acquire(project_id, mod1, pool)
                .await
                .unwrap()
                .unwrap();

            let result = DBModerationLock::acquire(project_id, mod2, pool)
                .await
                .expect("acquire should not return a DB error");

            assert!(result.is_err(), "second moderator should be blocked");
            let blocking_lock = result.unwrap_err();
            assert_eq!(
                blocking_lock.moderator_id, mod1,
                "blocking lock should belong to the first moderator"
            );
            assert!(
                !blocking_lock.expired,
                "blocking lock should not be expired"
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_lock_acquire_takeover_expired() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod1 = DBUserId(MOD_USER_ID_PARSED);
            let mod2 = DBUserId(ADMIN_USER_ID_PARSED);
            let pool = &test_env.db.pool;

            DBModerationLock::acquire(project_id, mod1, pool)
                .await
                .unwrap()
                .unwrap();

            expire_lock(project_id.0, pool).await;

            let result = DBModerationLock::acquire(project_id, mod2, pool)
                .await
                .expect("acquire should not return a DB error");

            assert!(result.is_ok(), "mod2 should acquire over expired lock");

            let lock = DBModerationLock::get_with_user(project_id, pool)
                .await
                .unwrap()
                .expect("lock should exist after takeover");
            assert_eq!(
                lock.moderator_id, mod2,
                "lock should belong to mod2 after takeover"
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_lock_acquire_previous_holder_blocked_after_takeover() {
    with_test_environment(None, |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
        let project_id = DBProjectId(test_env.dummy.project_beta.project_id_parsed.0 as i64);
        let mod1 = DBUserId(MOD_USER_ID_PARSED);
        let mod2 = DBUserId(ADMIN_USER_ID_PARSED);
        let pool = &test_env.db.pool;

        DBModerationLock::acquire(project_id, mod1, pool)
            .await
            .unwrap()
            .unwrap();
        expire_lock(project_id.0, pool).await;
        DBModerationLock::acquire(project_id, mod2, pool)
            .await
            .unwrap()
            .unwrap();

        let result = DBModerationLock::acquire(project_id, mod1, pool)
            .await
            .expect("acquire should not return a DB error");

        assert!(result.is_err(), "previous holder must not regain lock while new holder is active");
        assert_eq!(result.unwrap_err().moderator_id, mod2);
    })
    .await;
}

#[actix_rt::test]
async fn test_lock_acquire_concurrent() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod1 = DBUserId(MOD_USER_ID_PARSED);
            let mod2 = DBUserId(ADMIN_USER_ID_PARSED);
            let pool1 = test_env.db.pool.clone();
            let pool2 = test_env.db.pool.clone();
            let barrier = Arc::new(Barrier::new(2));
            let barrier_a = barrier.clone();
            let barrier_b = barrier.clone();

            let h1 = tokio::spawn(async move {
                barrier_a.wait().await;
                DBModerationLock::acquire(project_id, mod1, &pool1).await
            });
            let h2 = tokio::spawn(async move {
                barrier_b.wait().await;
                DBModerationLock::acquire(project_id, mod2, &pool2).await
            });
            let (o1, o2) = tokio::join!(h1, h2);
            let r1 = o1
                .expect("task 1 panicked")
                .expect("acquire 1 should not error");
            let r2 = o2
                .expect("task 2 panicked")
                .expect("acquire 2 should not error");

            let successes =
                [r1.is_ok(), r2.is_ok()].iter().filter(|&&ok| ok).count();
            assert_eq!(
                successes, 1,
                "exactly one of two concurrent acquires must win"
            );

            let winner_id = if r1.is_ok() { mod1 } else { mod2 };
            let loser = if r1.is_err() { r1 } else { r2 };
            let blocking = loser.unwrap_err();
            assert_eq!(
                blocking.moderator_id, winner_id,
                "loser's blocking lock should point to the winner"
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_http_get_lock_status_unlocked() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let pid = test_env.dummy.project_alpha.project_id.as_str();
            let req = test::TestRequest::get()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();

            let resp = api.call(req).await;
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Value = test::read_body_json(resp).await;
            assert_eq!(body["locked"], false);
            assert_eq!(body["is_own_lock"], false);
            assert!(body["locked_by"].is_null());
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_http_get_lock_status_own_lock() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let pid = test_env.dummy.project_alpha.project_id.as_str();

            let post = test::TestRequest::post()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();
            let acquire_resp = api.call(post).await;
            assert_eq!(acquire_resp.status(), StatusCode::OK);
            let acquire: Value = test::read_body_json(acquire_resp).await;
            assert_eq!(acquire["success"], true);
            assert_eq!(acquire["is_own_lock"], true);

            let get = test::TestRequest::get()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();
            let resp = api.call(get).await;
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Value = test::read_body_json(resp).await;
            assert_eq!(body["locked"], true);
            assert_eq!(body["is_own_lock"], true);
            assert_eq!(body["expired"], false);
            assert!(body["expires_at"].is_string());
            assert_eq!(body["locked_by"]["username"], "Moderator");
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_http_get_lock_status_other_users_lock() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let pid = test_env.dummy.project_alpha.project_id.as_str();

            let post = test::TestRequest::post()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", ADMIN_USER_PAT.unwrap()))
                .to_request();
            assert_eq!(api.call(post).await.status(), StatusCode::OK);

            let get = test::TestRequest::get()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();
            let resp = api.call(get).await;
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Value = test::read_body_json(resp).await;
            assert_eq!(body["locked"], true);
            assert_eq!(body["is_own_lock"], false);
            assert_eq!(body["locked_by"]["username"], "Admin");
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_http_post_acquire_blocked_returns_200_with_success_false() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let pid = test_env.dummy.project_alpha.project_id.as_str();

            let first = test::TestRequest::post()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();
            assert_eq!(api.call(first).await.status(), StatusCode::OK);

            let second = test::TestRequest::post()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", ADMIN_USER_PAT.unwrap()))
                .to_request();
            let resp = api.call(second).await;
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Value = test::read_body_json(resp).await;
            assert_eq!(body["success"], false);
            assert_eq!(body["is_own_lock"], false);
            assert_eq!(body["locked_by"]["username"], "Moderator");
            assert_eq!(body["expired"], false);
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_lock_force_acquire_steals_active_lock() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod1 = DBUserId(MOD_USER_ID_PARSED);
            let mod2 = DBUserId(ADMIN_USER_ID_PARSED);
            let pool = &test_env.db.pool;

            DBModerationLock::acquire(project_id, mod1, pool)
                .await
                .unwrap()
                .unwrap();

            DBModerationLock::force_acquire(project_id, mod2, pool)
                .await
                .expect("force_acquire should not error");

            let lock = DBModerationLock::get_with_user(project_id, pool)
                .await
                .unwrap()
                .expect("lock should exist after override");
            assert_eq!(
                lock.moderator_id, mod2,
                "lock holder should be the overriding moderator"
            );
            assert!(!lock.expired, "fresh override lock should not be expired");
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_http_post_override_active_lock_succeeds() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let pid = test_env.dummy.project_alpha.project_id.as_str();

            let first = test::TestRequest::post()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();
            assert_eq!(api.call(first).await.status(), StatusCode::OK);

            let override_req = test::TestRequest::post()
                .uri(&format!("/_internal/moderation/lock/{pid}/override"))
                .append_header(("Authorization", ADMIN_USER_PAT.unwrap()))
                .to_request();
            let resp = api.call(override_req).await;
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Value = test::read_body_json(resp).await;
            assert_eq!(body["success"], true);
            assert_eq!(body["is_own_lock"], true);

            let get = test::TestRequest::get()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", ADMIN_USER_PAT.unwrap()))
                .to_request();
            let get_resp = api.call(get).await;
            assert_eq!(get_resp.status(), StatusCode::OK);
            let status: Value = test::read_body_json(get_resp).await;
            assert_eq!(status["locked"], true);
            assert_eq!(status["is_own_lock"], true);
            assert_eq!(status["locked_by"]["username"], "Admin");
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_http_delete_release_success_then_noop() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let pid = test_env.dummy.project_alpha.project_id.as_str();

            let post = test::TestRequest::post()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();
            assert_eq!(api.call(post).await.status(), StatusCode::OK);

            let del = test::TestRequest::delete()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();
            let resp = api.call(del).await;
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Value = test::read_body_json(resp).await;
            assert_eq!(body["success"], true);

            let del2 = test::TestRequest::delete()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();
            let resp2 = api.call(del2).await;
            assert_eq!(resp2.status(), StatusCode::OK);
            let body2: Value = test::read_body_json(resp2).await;
            assert_eq!(body2["success"], false);
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_http_post_release_beacon_body_releases_without_auth_header() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let pid = test_env.dummy.project_alpha.project_id.as_str();

            let post = test::TestRequest::post()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();
            assert_eq!(api.call(post).await.status(), StatusCode::OK);

            let pat = MOD_USER_PAT.unwrap();
            let beacon = test::TestRequest::post()
                .uri(&format!("/_internal/moderation/lock/{pid}/release"))
                .append_header((header::CONTENT_TYPE, "text/plain"))
                .set_payload(Bytes::copy_from_slice(pat.as_bytes()))
                .to_request();
            let resp = api.call(beacon).await;
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Value = test::read_body_json(resp).await;
            assert_eq!(body["success"], true);

            let get = test::TestRequest::get()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();
            let get_resp = api.call(get).await;
            assert_eq!(get_resp.status(), StatusCode::OK);
            let status: Value = test::read_body_json(get_resp).await;
            assert_eq!(status["locked"], false);
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_http_delete_release_not_holder() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let pid = test_env.dummy.project_alpha.project_id.as_str();

            let post = test::TestRequest::post()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", ADMIN_USER_PAT.unwrap()))
                .to_request();
            assert_eq!(api.call(post).await.status(), StatusCode::OK);

            let del = test::TestRequest::delete()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();
            let resp = api.call(del).await;
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Value = test::read_body_json(resp).await;
            assert_eq!(body["success"], false);
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_http_lock_rejects_non_moderator() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let pid = test_env.dummy.project_alpha.project_id.as_str();
            let req = test::TestRequest::get()
                .uri(&format!("/_internal/moderation/lock/{pid}"))
                .append_header(("Authorization", USER_USER_PAT.unwrap()))
                .to_request();

            let resp = api.call(req).await;
            assert_eq!(
                resp.status(),
                StatusCode::UNAUTHORIZED,
                "internal lock endpoints require moderator (or admin) role"
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_http_lock_unknown_project_not_found() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let req = test::TestRequest::get()
                .uri("/_internal/moderation/lock/zzzzzzzz")
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();

            let resp = api.call(req).await;
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_http_delete_all_locks_admin() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let pid_a = test_env.dummy.project_alpha.project_id.as_str();
            let pid_b = test_env.dummy.project_beta.project_id.as_str();

            for pid in [pid_a, pid_b] {
                let post = test::TestRequest::post()
                    .uri(&format!("/_internal/moderation/lock/{pid}"))
                    .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                    .to_request();
                assert_eq!(api.call(post).await.status(), StatusCode::OK);
            }

            let del = test::TestRequest::delete()
                .uri("/_internal/moderation/locks")
                .append_header(("Authorization", ADMIN_USER_PAT.unwrap()))
                .to_request();
            let resp = api.call(del).await;
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Value = test::read_body_json(resp).await;
            assert!(body["deleted_count"].as_u64().unwrap_or(0) >= 2);

            let get = test::TestRequest::get()
                .uri(&format!("/_internal/moderation/lock/{pid_a}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .to_request();
            let st: Value = test::read_body_json(api.call(get).await).await;
            assert_eq!(st["locked"], false);
        },
    )
    .await;
}

/// Moderator without a lock cannot move a project out of processing (401 + lock message).
#[actix_rt::test]
async fn test_complete_review_requires_lock() {
    with_test_environment(None, |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
        let api = &test_env.api;
        let alpha_id = &test_env.dummy.project_alpha.project_id;

        set_project_processing(
            test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            test_env.dummy.project_alpha.project_slug.as_str(),
            &test_env.db.pool,
            &test_env.db.redis_pool,
        )
        .await;

        let req = actix_web::test::TestRequest::patch()
            .uri(&format!("/v3/project/{alpha_id}"))
            .append_header(("Authorization", MOD_USER_PAT.unwrap()))
            .set_json(json!({ "status": "approved" }))
            .to_request();

        let resp = api.call(req).await;
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "moderator without lock should not be able to complete review: {}",
            String::from_utf8_lossy(test::read_body(resp).await.as_ref())
        );
    })
    .await;
}

#[actix_rt::test]
async fn test_complete_review_blocked_when_other_holds_lock() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let alpha_id = &test_env.dummy.project_alpha.project_id;
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let pool = &test_env.db.pool;

            set_project_processing(
                project_id.0,
                test_env.dummy.project_alpha.project_slug.as_str(),
                pool,
                &test_env.db.redis_pool,
            )
            .await;
            DBModerationLock::acquire(
                project_id,
                DBUserId(ADMIN_USER_ID_PARSED),
                pool,
            )
            .await
            .unwrap()
            .unwrap();

            let req = actix_web::test::TestRequest::patch()
                .uri(&format!("/v3/project/{alpha_id}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .set_json(json!({ "status": "approved" }))
                .to_request();

            let resp = api.call(req).await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            let body = test::read_body(resp).await;
            let msg = String::from_utf8_lossy(&body);
            assert!(
                msg.contains("Admin") || msg.contains("admin"),
                "error should name the holding moderator: {msg}"
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_complete_review_fails_when_lock_expired() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let alpha_id = &test_env.dummy.project_alpha.project_id;
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod_id = DBUserId(MOD_USER_ID_PARSED);
            let pool = &test_env.db.pool;

            set_project_processing(
                project_id.0,
                test_env.dummy.project_alpha.project_slug.as_str(),
                pool,
                &test_env.db.redis_pool,
            )
            .await;
            DBModerationLock::acquire(project_id, mod_id, pool)
                .await
                .unwrap()
                .unwrap();
            expire_lock(project_id.0, pool).await;

            let req = actix_web::test::TestRequest::patch()
                .uri(&format!("/v3/project/{alpha_id}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .set_json(json!({ "status": "approved" }))
                .to_request();

            let resp = api.call(req).await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_complete_review_with_lock_succeeds() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let alpha_id = &test_env.dummy.project_alpha.project_id;
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod_id = DBUserId(MOD_USER_ID_PARSED);
            let pool = &test_env.db.pool;

            set_project_processing(
                project_id.0,
                test_env.dummy.project_alpha.project_slug.as_str(),
                pool,
                &test_env.db.redis_pool,
            )
            .await;

            DBModerationLock::acquire(project_id, mod_id, pool)
                .await
                .unwrap()
                .expect("moderator should acquire lock");

            let req = actix_web::test::TestRequest::patch()
                .uri(&format!("/v3/project/{alpha_id}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .set_json(json!({ "status": "approved" }))
                .to_request();

            let resp = api.call(req).await;
            assert_eq!(
                resp.status(),
                StatusCode::NO_CONTENT,
                "moderator with valid lock should be able to approve: {}",
                String::from_utf8_lossy(test::read_body(resp).await.as_ref())
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_non_processing_status_change_does_not_require_lock() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let alpha_id = &test_env.dummy.project_alpha.project_id;

            let req = actix_web::test::TestRequest::patch()
                .uri(&format!("/v3/project/{alpha_id}"))
                .append_header(("Authorization", MOD_USER_PAT.unwrap()))
                .set_json(json!({ "status": "unlisted" }))
                .to_request();

            let resp = api.call(req).await;
            assert_eq!(
                resp.status(),
                StatusCode::NO_CONTENT,
                "approved → unlisted should not require a moderation lock: {}",
                String::from_utf8_lossy(test::read_body(resp).await.as_ref())
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_admin_can_complete_without_lock() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let alpha_id = &test_env.dummy.project_alpha.project_id;
            let project_id_raw =
                test_env.dummy.project_alpha.project_id_parsed.0 as i64;

            set_project_processing(
                project_id_raw,
                test_env.dummy.project_alpha.project_slug.as_str(),
                &test_env.db.pool,
                &test_env.db.redis_pool,
            )
            .await;

            let req = actix_web::test::TestRequest::patch()
                .uri(&format!("/v3/project/{alpha_id}"))
                .append_header(("Authorization", ADMIN_USER_PAT.unwrap()))
                .set_json(json!({ "status": "approved" }))
                .to_request();

            let resp = api.call(req).await;
            assert_eq!(
                resp.status(),
                StatusCode::NO_CONTENT,
                "admin should be able to approve without a lock: {}",
                String::from_utf8_lossy(test::read_body(resp).await.as_ref())
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_lock_expires_at_populated() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod_id = DBUserId(MOD_USER_ID_PARSED);
            let pool = &test_env.db.pool;

            DBModerationLock::acquire(project_id, mod_id, pool)
                .await
                .unwrap()
                .unwrap();

            let lock = DBModerationLock::get_with_user(project_id, pool)
                .await
                .unwrap()
                .expect("lock should exist");

            let expected_expiry =
                lock.locked_at + chrono::Duration::minutes(LOCK_EXPIRY_MINUTES);
            assert_eq!(
                lock.expires_at, expected_expiry,
                "expires_at should be locked_at + LOCK_EXPIRY_MINUTES"
            );
            assert!(
                !lock.expired,
                "freshly acquired lock should not be expired"
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_lock_expired_flag() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let project_id = DBProjectId(
                test_env.dummy.project_alpha.project_id_parsed.0 as i64,
            );
            let mod_id = DBUserId(MOD_USER_ID_PARSED);
            let pool = &test_env.db.pool;

            DBModerationLock::acquire(project_id, mod_id, pool)
                .await
                .unwrap()
                .unwrap();

            expire_lock(project_id.0, pool).await;

            let lock = DBModerationLock::get_with_user(project_id, pool)
                .await
                .unwrap()
                .expect("lock should exist");

            assert!(
                lock.expired,
                "lock backdated past TTL should be reported as expired"
            );
        },
    )
    .await;
}

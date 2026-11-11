use actix_http::StatusCode;
use actix_web::test;
use chrono::{Duration, Utc};
use common::api_v3::ApiV3;
use common::{
    database::*,
    environment::{
        TestEnvironment, with_test_environment, with_test_environment_all,
    },
};

use labrinth::models::pats::Scopes;
use labrinth::test::pats::create_test_pat;
use serde_json::json;

use crate::common::api_common::{ApiUser, AppendsOptionalPat};

pub mod common;

// Staff can grant the staff scopes their role allows, and no others
#[actix_rt::test]
pub async fn staff_scope_grants_follow_role() {
    with_test_environment_all(None, |test_env| async move {
        for (pat, scopes, expected) in [
            (
                MOD_USER_PAT,
                Scopes::MODERATION_READ | Scopes::MODERATION_WRITE,
                StatusCode::OK,
            ),
            (MOD_USER_PAT, Scopes::ADMIN_READ, StatusCode::BAD_REQUEST),
            (ADMIN_USER_PAT, Scopes::staff(), StatusCode::OK),
        ] {
            let req = test::TestRequest::post()
                .uri("/_internal/pat")
                .append_pat(pat)
                .set_json(json!({
                    "scopes": scopes,
                    "name": "staff_scope_grants_follow_role Test",
                    "expires": Utc::now() + Duration::days(1),
                }))
                .to_request();
            let resp = test_env.call(req).await;
            assert_status!(&resp, expected);
        }
    })
    .await;
}

// A staff PAT loses its staff access once its user is no longer staff
#[actix_rt::test]
pub async fn staff_scopes_revoked_on_demotion() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pat = create_test_pat(
                Scopes::MODERATION_READ,
                MOD_USER_ID_PARSED,
                &test_env.db,
            )
            .await;
            let pid = test_env.dummy.project_alpha.project_id.clone();
            let lock_status = || {
                test::TestRequest::get()
                    .uri(&format!("/_internal/moderation/lock/{pid}"))
                    .append_pat(Some(&pat))
                    .to_request()
            };

            let resp = test_env.call(lock_status()).await;
            assert_status!(&resp, StatusCode::OK);

            let resp = test_env
                .api
                .edit_user(
                    MOD_USER_ID,
                    json!({ "role": "developer" }),
                    ADMIN_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let resp = test_env.call(lock_status()).await;
            assert_status!(&resp, StatusCode::UNAUTHORIZED);
        },
    )
    .await;
}

// Full pat test:
// - create a PAT and ensure it can be used for the scope
// - ensure access token is not returned for any PAT in GET
// - ensure PAT can be patched to change scopes
// - ensure PAT can be patched to change expiry
// - ensure expired PATs cannot be used
// - ensure PATs can be deleted
#[actix_rt::test]
pub async fn pat_full_test() {
    with_test_environment_all(None, |test_env| async move {
        // Create a PAT for a full test
        let req = test::TestRequest::post()
            .uri("/_internal/pat")
            .append_pat(USER_USER_PAT)
            .set_json(json!({
                "scopes": Scopes::COLLECTION_CREATE, // Collection create as an easily tested example
                "name": "test_pat_scopes Test",
                "expires": Utc::now() + Duration::days(1),
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_status!(&resp, StatusCode::OK);
        let success: serde_json::Value = test::read_body_json(resp).await;
        let id = success["id"].as_str().unwrap();

        // Has access token and correct scopes
        assert!(success["access_token"].as_str().is_some());
        assert_eq!(
            success["scopes"].as_u64().unwrap(),
            Scopes::COLLECTION_CREATE.bits()
        );
        let access_token = success["access_token"].as_str().unwrap();

        // Get PAT again
        let req = test::TestRequest::get()
            .append_pat(USER_USER_PAT)
            .uri("/_internal/pat")
            .to_request();
        let resp = test_env.call(req).await;
        assert_status!(&resp, StatusCode::OK);
        let success: serde_json::Value = test::read_body_json(resp).await;

        // Ensure access token is NOT returned for any PATs
        for pat in success.as_array().unwrap() {
            assert!(pat["access_token"].as_str().is_none());
        }

        // Create mock test for using PAT
        let mock_pat_test = |token: &str| {
            let token = token.to_string();
            async {
                // This uses a route directly instead of an api call because it doesn't really matter and we
                // want it to succeed no matter what.
                // This is an arbitrary request.
                let req = test::TestRequest::post()
                    .uri("/v3/collection")
                    .append_header(("Authorization", token))
                    .set_json(json!({
                        "name": "Test Collection 1",
                        "description": "Test Collection Description"
                    }))
                    .to_request();
                let resp = test_env.call(req).await;
                resp.status().as_u16()
            }
        };

        assert_eq!(mock_pat_test(access_token).await, 200);

        // Change scopes and test again
        let req = test::TestRequest::patch()
            .uri(&format!("/_internal/pat/{id}"))
            .append_pat(USER_USER_PAT)
            .set_json(json!({
                "scopes": 0,
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);
        assert_eq!(mock_pat_test(access_token).await, 401); // No longer works

        // Change scopes back, and set expiry to the past, and test again
        let req = test::TestRequest::patch()
            .uri(&format!("/_internal/pat/{id}"))
            .append_pat(USER_USER_PAT)
            .set_json(json!({
                "scopes": Scopes::COLLECTION_CREATE,
                "expires": Utc::now() + Duration::seconds(1), // expires in 1 second
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // Wait 1 second before testing again for expiry
        tokio::time::sleep(Duration::seconds(1).to_std().unwrap()).await;
        assert_eq!(mock_pat_test(access_token).await, 401); // No longer works

        // Change everything back to normal and test again
        let req = test::TestRequest::patch()
            .uri(&format!("/_internal/pat/{id}"))
            .append_pat(USER_USER_PAT)
            .set_json(json!({
                "expires": Utc::now() + Duration::days(1), // no longer expired!
            }))
            .to_request();

        println!("PAT ID FOR TEST: {id}");
        let resp = test_env.call(req).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);
        assert_eq!(mock_pat_test(access_token).await, 200); // Works again

        // Patching to a bad expiry should fail
        let req = test::TestRequest::patch()
            .uri(&format!("/_internal/pat/{id}"))
            .append_pat(USER_USER_PAT)
            .set_json(json!({
                "expires": Utc::now() - Duration::days(1), // Past
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // Similar to above with PAT creation, patching to a bad scope should fail
        for i in 0..64 {
            let scope = Scopes::from_bits_truncate(1 << i);
            if !Scopes::all().contains(scope) {
                continue;
            }

            let req = test::TestRequest::patch()
                .uri(&format!("/_internal/pat/{id}"))
                .append_pat(USER_USER_PAT)
                .set_json(json!({
                    "scopes": scope.bits(),
                }))
                .to_request();
            let resp = test_env.call(req).await;
            assert_eq!(
                resp.status().as_u16(),
                if scope.is_restricted() || scope.intersects(Scopes::staff()) {
                    400
                } else {
                    204
                }
            );
        }

        // Delete PAT
        let req = test::TestRequest::delete()
            .append_pat(USER_USER_PAT)
            .uri(&format!("/_internal/pat/{id}"))
            .to_request();
        let resp = test_env.call(req).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);
    })
    .await;
}

// Test illegal PAT setting, both in POST and PATCH
#[actix_rt::test]
pub async fn bad_pats() {
    with_test_environment_all(None, |test_env| async move {
        // Creating a PAT with no name should fail
        let req = test::TestRequest::post()
            .uri("/_internal/pat")
            .append_pat(USER_USER_PAT)
            .set_json(json!({
                "scopes": Scopes::COLLECTION_CREATE, // Collection create as an easily tested example
                "expires": Utc::now() + Duration::days(1),
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // Name too short or too long should fail
        for name in ["n", "this_name_is_too_long".repeat(16).as_str()] {
            let req = test::TestRequest::post()
                .uri("/_internal/pat")
                .append_pat(USER_USER_PAT)
                .set_json(json!({
                    "name": name,
                    "scopes": Scopes::COLLECTION_CREATE, // Collection create as an easily tested example
                    "expires": Utc::now() + Duration::days(1),
                }))
                .to_request();
            let resp = test_env.call(req).await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);
        }

        // Creating a PAT with an expiry in the past should fail
        let req = test::TestRequest::post()
            .uri("/_internal/pat")
            .append_pat(USER_USER_PAT)
            .set_json(json!({
                "scopes": Scopes::COLLECTION_CREATE, // Collection create as an easily tested example
                "name": "test_pat_scopes Test",
                "expires": Utc::now() - Duration::days(1),
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // Make a PAT with each scope, with the result varying by whether that scope is restricted
        for i in 0..64 {
            let scope = Scopes::from_bits_truncate(1 << i);
            if !Scopes::all().contains(scope) {
                continue;
            }
            let req = test::TestRequest::post()
                .uri("/_internal/pat")
                .append_pat(USER_USER_PAT)
                .set_json(json!({
                    "scopes": scope.bits(),
                    "name": format!("test_pat_scopes Name {}", i),
                    "expires": Utc::now() + Duration::days(1),
                }))
                .to_request();
            let resp = test_env.call(req).await;
            assert_eq!(
                resp.status().as_u16(),
                if scope.is_restricted() || scope.intersects(Scopes::staff()) {
                    400
                } else {
                    200
                }
            );
        }

        // Create a 'good' PAT for patching
        let req = test::TestRequest::post()
            .uri("/_internal/pat")
            .append_pat(USER_USER_PAT)
            .set_json(json!({
                "scopes": Scopes::COLLECTION_CREATE,
                "name": "test_pat_scopes Test",
                "expires": Utc::now() + Duration::days(1),
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_status!(&resp, StatusCode::OK);
        let success: serde_json::Value = test::read_body_json(resp).await;
        let id = success["id"].as_str().unwrap();

        // Patching to a bad name should fail
        for name in ["n", "this_name_is_too_long".repeat(16).as_str()] {
            let req = test::TestRequest::post()
                .uri("/_internal/pat")
                .append_pat(USER_USER_PAT)
                .set_json(json!({
                    "name": name,
                }))
                .to_request();
            let resp = test_env.call(req).await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);
        }

        // Patching to a bad expiry should fail
        let req = test::TestRequest::patch()
            .uri(&format!("/_internal/pat/{id}"))
            .append_pat(USER_USER_PAT)
            .set_json(json!({
                "expires": Utc::now() - Duration::days(1), // Past
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // Similar to above with PAT creation, patching to a bad scope should fail
        for i in 0..64 {
            let scope = Scopes::from_bits_truncate(1 << i);
            if !Scopes::all().contains(scope) {
                continue;
            }

            let req = test::TestRequest::patch()
                .uri(&format!("/_internal/pat/{id}"))
                .append_pat(USER_USER_PAT)
                .set_json(json!({
                    "scopes": scope.bits(),
                }))
                .to_request();
            let resp = test_env.call(req).await;
            assert_eq!(
                resp.status().as_u16(),
                if scope.is_restricted() || scope.intersects(Scopes::staff()) {
                    400
                } else {
                    204
                }
            );
        }
    })
    .await;
}

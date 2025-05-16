use actix_http::StatusCode;
use actix_web::test;
use chrono::{Duration, Utc};
use common::{database::*, environment::with_test_environment_all};

use labrinth::models::pats::Scopes;
use serde_json::json;

use crate::common::api_common::AppendsOptionalPat;

mod common;

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
                // This uses a route directly instead of an api call because it doesn't relaly matter and we
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
                if scope.is_restricted() { 400 } else { 204 }
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
                if scope.is_restricted() { 400 } else { 200 }
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
                if scope.is_restricted() { 400 } else { 204 }
            );
        }
    })
    .await;
}

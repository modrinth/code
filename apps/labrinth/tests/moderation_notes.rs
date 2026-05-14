use actix_http::StatusCode;
use actix_web::test;
use common::{
    api_common::{Api, AppendsOptionalPat},
    database::{MOD_USER_PAT, USER_USER_ID, USER_USER_PAT},
    environment::with_test_environment_all,
};
use serde_json::{Value, json};

pub mod common;

#[actix_rt::test]
pub async fn moderation_notes_users() {
    with_test_environment_all(None, |test_env| async move {
        let api = test_env.api;

        let resp = api
            .call(
                test::TestRequest::get()
                    .uri(&format!("/v3/user/{USER_USER_ID}"))
                    .append_pat(MOD_USER_PAT)
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        let body: Value = test::read_body_json(resp).await;
        assert!(body.get("moderation_notes").unwrap().is_null());

        let resp = api
            .call(
                test::TestRequest::get()
                    .uri(&format!("/v3/user/{USER_USER_ID}"))
                    .append_pat(USER_USER_PAT)
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        let body: Value = test::read_body_json(resp).await;
        assert!(body.get("moderation_notes").is_none());

        let resp = api
            .call(
                test::TestRequest::patch()
                    .uri(&format!("/v3/user/{USER_USER_ID}/notes"))
                    .append_pat(MOD_USER_PAT)
                    .set_json(
                        json!({ "notes": "first note", "user_rating": 1 }),
                    )
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT); // OK without If-Match for the first patch

        let resp = api
            .call(
                test::TestRequest::patch()
                    .uri(&format!("/v3/user/{USER_USER_ID}/notes"))
                    .append_pat(MOD_USER_PAT)
                    .append_header(("If-Match", "0"))
                    .set_json(json!({}))
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        let resp = api
            .call(
                test::TestRequest::patch()
                    .uri(&format!("/v3/user/{USER_USER_ID}/notes"))
                    .append_pat(USER_USER_PAT)
                    .append_header(("If-Match", "0"))
                    .set_json(json!({ "notes": "first note" }))
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::UNAUTHORIZED);

        let resp = api
            .call(
                test::TestRequest::patch()
                    .uri(&format!("/v3/user/{USER_USER_ID}/notes"))
                    .append_pat(MOD_USER_PAT)
                    .set_json(json!({
                        "notes": "first note",
                        "user_rating": 2,
                    }))
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::PRECONDITION_REQUIRED); // Needs If-Match moving forward

        let resp = api
            .call(
                test::TestRequest::get()
                    .uri(&format!("/v3/user/{USER_USER_ID}"))
                    .append_pat(MOD_USER_PAT)
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["moderation_notes"]["notes"], "first note");
        assert_eq!(body["moderation_notes"]["user_rating"], 1);
        assert_eq!(body["moderation_notes"]["version"], 1);
        assert_eq!(body["moderation_notes"]["last_author"], "2");

        let resp = api
            .call(
                test::TestRequest::patch()
                    .uri(&format!("/v3/user/{USER_USER_ID}/notes"))
                    .append_pat(MOD_USER_PAT)
                    .append_header(("If-Match", "0"))
                    .set_json(json!({ "notes": "stale note" }))
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::PRECONDITION_FAILED);

        let resp = api
            .call(
                test::TestRequest::patch()
                    .uri(&format!("/v3/user/{USER_USER_ID}/notes"))
                    .append_pat(MOD_USER_PAT)
                    .append_header(("If-Match", "1"))
                    .set_json(json!({ "user_rating": 4 }))
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        let resp = api
            .call(
                test::TestRequest::get()
                    .uri(&format!("/v3/user/{USER_USER_ID}"))
                    .append_pat(MOD_USER_PAT)
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["moderation_notes"]["notes"], "first note");
        assert_eq!(body["moderation_notes"]["user_rating"], 4);
        assert_eq!(body["moderation_notes"]["version"], 2);

        let user_ids = serde_json::to_string(&vec![USER_USER_ID]).unwrap();
        let resp = api
            .call(
                test::TestRequest::get()
                    .uri(&format!(
                        "/v3/users?ids={}",
                        urlencoding::encode(&user_ids)
                    ))
                    .append_pat(MOD_USER_PAT)
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body[0]["moderation_notes"]["version"], 2);

        let resp = api
            .call(
                test::TestRequest::get()
                    .uri(&format!(
                        "/v3/users?ids={}",
                        urlencoding::encode(&user_ids)
                    ))
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        let body: Value = test::read_body_json(resp).await;
        assert!(body[0].get("moderation_notes").is_none());
    })
    .await;
}

#[actix_rt::test]
pub async fn moderation_notes_organizations() {
    with_test_environment_all(None, |test_env| async move {
        let api = test_env.api;
        let organization_id =
            test_env.dummy.organization_zeta.organization_id.clone();

        let resp = api
            .call(
                test::TestRequest::get()
                    .uri(&format!("/v3/organization/{organization_id}"))
                    .append_pat(MOD_USER_PAT)
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        let body: Value = test::read_body_json(resp).await;
        assert!(body.get("moderation_notes").unwrap().is_null());

        let resp = api
            .call(
                test::TestRequest::patch()
                    .uri(&format!("/v3/organization/{organization_id}/notes"))
                    .append_pat(MOD_USER_PAT)
                    .append_header(("If-Match", "0"))
                    .set_json(json!({
                        "notes": "org note",
                        "user_rating": -1,
                    }))
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::PRECONDITION_FAILED); // Shouldn't have If-Match for the first patch

        let resp = api
            .call(
                test::TestRequest::patch()
                    .uri(&format!("/v3/organization/{organization_id}/notes"))
                    .append_pat(MOD_USER_PAT)
                    .set_json(json!({
                        "notes": "org note",
                        "user_rating": -1,
                    }))
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        let resp = api
            .call(
                test::TestRequest::get()
                    .uri(&format!("/v3/organization/{organization_id}"))
                    .append_pat(USER_USER_PAT)
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        let body: Value = test::read_body_json(resp).await;
        assert!(body.get("moderation_notes").is_none());

        let resp = api
            .call(
                test::TestRequest::patch()
                    .uri(&format!("/v3/organization/{organization_id}/notes"))
                    .append_pat(MOD_USER_PAT)
                    .append_header(("If-Match", "1"))
                    .set_json(json!({ "notes": "updated org note" }))
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        let resp = api
            .call(
                test::TestRequest::patch()
                    .uri(&format!("/v3/organization/{organization_id}/notes"))
                    .append_pat(MOD_USER_PAT)
                    .set_json(json!({
                        "notes": "new note",
                    }))
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::PRECONDITION_REQUIRED); // Needs If-Match moving forward

        let ids =
            serde_json::to_string(&vec![organization_id.as_str()]).unwrap();
        let resp = api
            .call(
                test::TestRequest::get()
                    .uri(&format!(
                        "/v3/organizations?ids={}",
                        urlencoding::encode(&ids)
                    ))
                    .append_pat(MOD_USER_PAT)
                    .to_request(),
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body[0]["moderation_notes"]["notes"], "updated org note");
        assert_eq!(body[0]["moderation_notes"]["user_rating"], -1);
        assert_eq!(body[0]["moderation_notes"]["version"], 2);
    })
    .await;
}

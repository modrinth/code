use actix_http::StatusCode;
use actix_web::http::header::{CACHE_CONTROL, PRAGMA};
use actix_web::test;
use common::{
    api_v3::oauth::get_redirect_location_query_params,
    api_v3::{
        ApiV3,
        oauth::{
            get_auth_code_from_redirect_params, get_authorize_accept_flow_id,
        },
    },
    database::FRIEND_USER_ID,
    database::{FRIEND_USER_PAT, USER_USER_ID, USER_USER_PAT},
    dummy_data::DummyOAuthClientAlpha,
    environment::{TestEnvironment, with_test_environment},
};
use labrinth::auth::oauth::TokenResponse;

mod common;

#[actix_rt::test]
async fn oauth_flow_happy_path() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let DummyOAuthClientAlpha {
            valid_redirect_uri: base_redirect_uri,
            client_id,
            client_secret,
        } = &env.dummy.oauth_client_alpha;

        // Initiate authorization
        let redirect_uri = format!("{base_redirect_uri}?foo=bar");
        let original_state = "1234";
        let resp = env
            .api
            .oauth_authorize(
                client_id,
                Some("USER_READ NOTIFICATION_READ"),
                Some(&redirect_uri),
                Some(original_state),
                FRIEND_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        let flow_id = get_authorize_accept_flow_id(resp).await;

        // Accept the authorization request
        let resp = env.api.oauth_accept(&flow_id, FRIEND_USER_PAT).await;
        assert_status!(&resp, StatusCode::OK);
        let query = get_redirect_location_query_params(&resp);

        let auth_code = query.get("code").unwrap();
        let state = query.get("state").unwrap();
        let foo_val = query.get("foo").unwrap();
        assert_eq!(state, original_state);
        assert_eq!(foo_val, "bar");

        // Get the token
        let resp = env
            .api
            .oauth_token(
                auth_code.to_string(),
                Some(redirect_uri.clone()),
                client_id.to_string(),
                client_secret,
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        assert_eq!(resp.headers().get(CACHE_CONTROL).unwrap(), "no-store");
        assert_eq!(resp.headers().get(PRAGMA).unwrap(), "no-cache");
        let token_resp: TokenResponse = test::read_body_json(resp).await;

        // Validate the token works
        env.assert_read_notifications_status(
            FRIEND_USER_ID,
            Some(&token_resp.access_token),
            StatusCode::OK,
        )
        .await;
    })
    .await;
}

#[actix_rt::test]
async fn oauth_authorize_for_already_authorized_scopes_returns_auth_code() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let DummyOAuthClientAlpha { client_id, .. } =
            env.dummy.oauth_client_alpha;

        let resp = env
            .api
            .oauth_authorize(
                &client_id,
                Some("USER_READ NOTIFICATION_READ"),
                None,
                Some("1234"),
                USER_USER_PAT,
            )
            .await;
        let flow_id = get_authorize_accept_flow_id(resp).await;
        env.api.oauth_accept(&flow_id, USER_USER_PAT).await;

        let resp = env
            .api
            .oauth_authorize(
                &client_id,
                Some("USER_READ"),
                None,
                Some("5678"),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
    })
    .await;
}

#[actix_rt::test]
async fn get_oauth_token_with_already_used_auth_code_fails() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let DummyOAuthClientAlpha {
            client_id,
            client_secret,
            ..
        } = env.dummy.oauth_client_alpha;

        let resp = env
            .api
            .oauth_authorize(&client_id, None, None, None, USER_USER_PAT)
            .await;
        let flow_id = get_authorize_accept_flow_id(resp).await;

        let resp = env.api.oauth_accept(&flow_id, USER_USER_PAT).await;
        let auth_code = get_auth_code_from_redirect_params(&resp).await;

        let resp = env
            .api
            .oauth_token(
                auth_code.clone(),
                None,
                client_id.clone(),
                &client_secret,
            )
            .await;
        assert_status!(&resp, StatusCode::OK);

        let resp = env
            .api
            .oauth_token(auth_code, None, client_id, &client_secret)
            .await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);
    })
    .await;
}

#[actix_rt::test]
async fn authorize_with_broader_scopes_can_complete_flow() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let DummyOAuthClientAlpha {
            client_id,
            client_secret,
            ..
        } = env.dummy.oauth_client_alpha.clone();

        let first_access_token = env
            .api
            .complete_full_authorize_flow(
                &client_id,
                &client_secret,
                Some("PROJECT_READ"),
                None,
                None,
                USER_USER_PAT,
            )
            .await;
        let second_access_token = env
            .api
            .complete_full_authorize_flow(
                &client_id,
                &client_secret,
                Some("PROJECT_READ NOTIFICATION_READ"),
                None,
                None,
                USER_USER_PAT,
            )
            .await;

        env.assert_read_notifications_status(
            USER_USER_ID,
            Some(&first_access_token),
            StatusCode::UNAUTHORIZED,
        )
        .await;
        env.assert_read_user_projects_status(
            USER_USER_ID,
            Some(&first_access_token),
            StatusCode::OK,
        )
        .await;

        env.assert_read_notifications_status(
            USER_USER_ID,
            Some(&second_access_token),
            StatusCode::OK,
        )
        .await;
        env.assert_read_user_projects_status(
            USER_USER_ID,
            Some(&second_access_token),
            StatusCode::OK,
        )
        .await;
    })
    .await;
}

#[actix_rt::test]
async fn oauth_authorize_with_broader_scopes_requires_user_accept() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let client_id = env.dummy.oauth_client_alpha.client_id;
        let resp = env
            .api
            .oauth_authorize(
                &client_id,
                Some("USER_READ"),
                None,
                None,
                USER_USER_PAT,
            )
            .await;
        let flow_id = get_authorize_accept_flow_id(resp).await;
        env.api.oauth_accept(&flow_id, USER_USER_PAT).await;

        let resp = env
            .api
            .oauth_authorize(
                &client_id,
                Some("USER_READ NOTIFICATION_READ"),
                None,
                None,
                USER_USER_PAT,
            )
            .await;

        assert_status!(&resp, StatusCode::OK);
        get_authorize_accept_flow_id(resp).await; // ensure we can deser this without error to really confirm
    })
    .await;
}

#[actix_rt::test]
async fn reject_authorize_ends_authorize_flow() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let client_id = env.dummy.oauth_client_alpha.client_id;
        let resp = env
            .api
            .oauth_authorize(&client_id, None, None, None, USER_USER_PAT)
            .await;
        let flow_id = get_authorize_accept_flow_id(resp).await;

        let resp = env.api.oauth_reject(&flow_id, USER_USER_PAT).await;
        assert_status!(&resp, StatusCode::OK);

        let resp = env.api.oauth_accept(&flow_id, USER_USER_PAT).await;
        assert_any_status_except!(&resp, StatusCode::OK);
    })
    .await;
}

#[actix_rt::test]
async fn accept_authorize_after_already_accepting_fails() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let client_id = env.dummy.oauth_client_alpha.client_id;
        let resp = env
            .api
            .oauth_authorize(&client_id, None, None, None, USER_USER_PAT)
            .await;
        let flow_id = get_authorize_accept_flow_id(resp).await;
        let resp = env.api.oauth_accept(&flow_id, USER_USER_PAT).await;
        assert_status!(&resp, StatusCode::OK);

        let resp = env.api.oauth_accept(&flow_id, USER_USER_PAT).await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);
    })
    .await;
}

#[actix_rt::test]
async fn revoke_authorization_after_issuing_token_revokes_token() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let DummyOAuthClientAlpha {
            client_id,
            client_secret,
            ..
        } = &env.dummy.oauth_client_alpha;
        let access_token = env
            .api
            .complete_full_authorize_flow(
                client_id,
                client_secret,
                Some("NOTIFICATION_READ"),
                None,
                None,
                USER_USER_PAT,
            )
            .await;
        env.assert_read_notifications_status(
            USER_USER_ID,
            Some(&access_token),
            StatusCode::OK,
        )
        .await;

        let resp = env
            .api
            .revoke_oauth_authorization(client_id, USER_USER_PAT)
            .await;
        assert_status!(&resp, StatusCode::OK);

        env.assert_read_notifications_status(
            USER_USER_ID,
            Some(&access_token),
            StatusCode::UNAUTHORIZED,
        )
        .await;
    })
    .await;
}

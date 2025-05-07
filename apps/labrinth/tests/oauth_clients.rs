use actix_http::StatusCode;
use actix_web::test;
use common::{
    api_v3::ApiV3,
    database::{FRIEND_USER_ID, FRIEND_USER_PAT, USER_USER_ID, USER_USER_PAT},
    dummy_data::DummyOAuthClientAlpha,
    environment::{TestEnvironment, with_test_environment},
    get_json_val_str,
};
use labrinth::{
    models::{
        oauth_clients::{OAuthClient, OAuthClientCreationResult},
        pats::Scopes,
    },
    routes::v3::oauth_clients::OAuthClientEdit,
};

use common::database::USER_USER_ID_PARSED;

mod common;

#[actix_rt::test]
async fn can_create_edit_get_oauth_client() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let client_name = "test_client".to_string();
        let redirect_uris = vec![
            "https://modrinth.com".to_string(),
            "https://modrinth.com/a".to_string(),
        ];
        let resp = env
            .api
            .add_oauth_client(
                client_name.clone(),
                Scopes::all() - Scopes::restricted(),
                redirect_uris.clone(),
                FRIEND_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        let creation_result: OAuthClientCreationResult =
            test::read_body_json(resp).await;
        let client_id = get_json_val_str(creation_result.client.id);

        let url = Some("https://modrinth.com".to_string());
        let description = Some("test description".to_string());
        let edited_redirect_uris = vec![
            redirect_uris[0].clone(),
            "https://modrinth.com/b".to_string(),
        ];
        let edit = OAuthClientEdit {
            name: None,
            max_scopes: None,
            redirect_uris: Some(edited_redirect_uris.clone()),
            url: Some(url.clone()),
            description: Some(description.clone()),
        };
        let resp = env
            .api
            .edit_oauth_client(&client_id, edit, FRIEND_USER_PAT)
            .await;
        assert_status!(&resp, StatusCode::OK);

        let clients = env
            .api
            .get_user_oauth_clients(FRIEND_USER_ID, FRIEND_USER_PAT)
            .await;
        assert_eq!(1, clients.len());
        assert_eq!(url, clients[0].url);
        assert_eq!(description, clients[0].description);
        assert_eq!(client_name, clients[0].name);
        assert_eq!(2, clients[0].redirect_uris.len());
        assert_eq!(edited_redirect_uris[0], clients[0].redirect_uris[0].uri);
        assert_eq!(edited_redirect_uris[1], clients[0].redirect_uris[1].uri);
    })
    .await;
}

#[actix_rt::test]
async fn create_oauth_client_with_restricted_scopes_fails() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let resp = env
            .api
            .add_oauth_client(
                "test_client".to_string(),
                Scopes::restricted(),
                vec!["https://modrinth.com".to_string()],
                FRIEND_USER_PAT,
            )
            .await;

        assert_status!(&resp, StatusCode::BAD_REQUEST);
    })
    .await;
}

#[actix_rt::test]
async fn get_oauth_client_for_client_creator_succeeds() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let DummyOAuthClientAlpha { client_id, .. } =
            env.dummy.oauth_client_alpha.clone();

        let resp = env
            .api
            .get_oauth_client(client_id.clone(), USER_USER_PAT)
            .await;

        assert_status!(&resp, StatusCode::OK);
        let client: OAuthClient = test::read_body_json(resp).await;
        assert_eq!(get_json_val_str(client.id), client_id);
    })
    .await;
}

#[actix_rt::test]
async fn can_delete_oauth_client() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let client_id = env.dummy.oauth_client_alpha.client_id.clone();
        let resp = env.api.delete_oauth_client(&client_id, USER_USER_PAT).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        let clients = env
            .api
            .get_user_oauth_clients(USER_USER_ID, USER_USER_PAT)
            .await;
        assert_eq!(0, clients.len());
    })
    .await;
}

#[actix_rt::test]
async fn delete_oauth_client_after_issuing_access_tokens_revokes_tokens() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let DummyOAuthClientAlpha {
            client_id,
            client_secret,
            ..
        } = env.dummy.oauth_client_alpha.clone();
        let access_token = env
            .api
            .complete_full_authorize_flow(
                &client_id,
                &client_secret,
                Some("NOTIFICATION_READ"),
                None,
                None,
                USER_USER_PAT,
            )
            .await;

        env.api.delete_oauth_client(&client_id, USER_USER_PAT).await;

        env.assert_read_notifications_status(
            USER_USER_ID,
            Some(&access_token),
            StatusCode::UNAUTHORIZED,
        )
        .await;
    })
    .await;
}

#[actix_rt::test]
async fn can_list_user_oauth_authorizations() {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let DummyOAuthClientAlpha {
            client_id,
            client_secret,
            ..
        } = env.dummy.oauth_client_alpha.clone();
        env.api
            .complete_full_authorize_flow(
                &client_id,
                &client_secret,
                None,
                None,
                None,
                USER_USER_PAT,
            )
            .await;

        let authorizations =
            env.api.get_user_oauth_authorizations(USER_USER_PAT).await;
        assert_eq!(1, authorizations.len());
        assert_eq!(USER_USER_ID_PARSED, authorizations[0].user_id.0 as i64);
    })
    .await;
}

#![recursion_limit = "256"]

use actix_http::StatusCode;
use actix_web::{http::Method, test};
use chrono::{Duration, Utc};
use common::{
    api_common::AppendsOptionalPat,
    api_v3::{
        ApiV3,
        oauth::{
            get_auth_code_from_redirect_params, get_authorize_accept_flow_id,
        },
    },
    database::{ADMIN_USER_PAT, FRIEND_USER_PAT, MOD_USER_PAT, USER_USER_PAT},
    environment::{TestEnvironment, with_test_environment},
};
use labrinth::{
    auth::{
        AuthenticationError, get_user_from_headers,
        validate::{
            get_full_user_from_headers_allow_locked,
            get_maybe_user_from_headers, get_user_record_from_bearer_token,
        },
    },
    database::models::{
        DBUserId, flow_item::DBFlow, session_item::SessionBuilder,
    },
    env::ENV,
    models::pats::Scopes,
    queue::session::AuthQueue,
};
use serde_json::{Value, json};

pub mod common;

async fn set_account_locked(
    env: &TestEnvironment<ApiV3>,
    user: &str,
    account_locked: bool,
) {
    let response = env
        .call(
            test::TestRequest::patch()
                .uri(&format!("/v3/user/{user}"))
                .append_pat(ADMIN_USER_PAT)
                .set_json(json!({"account_locked": account_locked}))
                .to_request(),
        )
        .await;
    assert_status!(&response, StatusCode::NO_CONTENT);
}

#[actix_rt::test]
async fn account_locked_is_visible_only_to_self_and_admin_in_both_api_versions()
{
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
		let mut restricted_tokens = Vec::new();
		for pat in [USER_USER_PAT, ADMIN_USER_PAT] {
			let response = env
				.call(
					test::TestRequest::post()
						.uri("/_internal/pat")
						.append_pat(pat)
						.set_json(json!({
							"name": "account lock visibility",
							"scopes": Scopes::empty(),
							"expires": Utc::now() + Duration::days(1),
						}))
						.to_request(),
				)
				.await;
			assert_status!(&response, StatusCode::OK);
			let body: Value = test::read_body_json(response).await;
			restricted_tokens
				.push(body["access_token"].as_str().unwrap().to_owned());
		}
		for account_locked in [false, true] {
			set_account_locked(&env, "3", account_locked).await;
			for version in ["v2", "v3"] {
				for (pat, visible) in [
					(None, false),
					(FRIEND_USER_PAT, false),
					(MOD_USER_PAT, false),
					(USER_USER_PAT, true),
					(ADMIN_USER_PAT, true),
					(Some(restricted_tokens[0].as_str()), true),
					(Some(restricted_tokens[1].as_str()), true),
				] {
					let locked_viewer = account_locked
						&& (pat == USER_USER_PAT
							|| pat == Some(restricted_tokens[0].as_str()));
					for target in ["3", "User"] {
						let response = env
							.call(
								test::TestRequest::get()
									.uri(&format!("/{version}/user/{target}"))
									.append_pat(pat)
									.to_request(),
							)
							.await;
						if locked_viewer {
							assert_status!(&response, StatusCode::FORBIDDEN);
							continue;
						}
						assert_status!(&response, StatusCode::OK);
						let body: Value = test::read_body_json(response).await;
						assert_eq!(
							body.get("account_locked"),
							visible.then(|| json!(account_locked)).as_ref()
						);
						assert_eq!(
							body["email"].is_string(),
							pat == ADMIN_USER_PAT
						);
						if restricted_tokens
							.iter()
							.any(|token| pat == Some(token.as_str()))
						{
							for field in [
								"github_id",
								"discord_id",
								"steam_id",
								"moderation_notes",
								"payout_data",
							] {
								assert!(
									body[field].is_null(),
									"{field} requires session access"
								);
							}
						}
					}

					let response = env
						.call(
							test::TestRequest::get()
								.uri(&format!(
									"/{version}/users?ids=%5B%223%22,%225%22%5D"
								))
								.append_pat(pat)
								.to_request(),
						)
						.await;
					if locked_viewer {
						assert_status!(&response, StatusCode::FORBIDDEN);
						continue;
					}
					assert_status!(&response, StatusCode::OK);
					let body: Vec<Value> = test::read_body_json(response).await;
					assert_eq!(body.len(), 2);
					for user in body {
						let expected = if user["id"] == "3" {
							visible.then(|| json!(account_locked))
						} else {
							(pat == ADMIN_USER_PAT
								|| pat == Some(restricted_tokens[1].as_str()))
							.then(|| json!(false))
						};
						assert_eq!(
							user.get("account_locked"),
							expected.as_ref()
						);
					}
				}
				let response = env
					.call(
						test::TestRequest::get()
							.uri(&format!("/{version}/user"))
							.append_pat(USER_USER_PAT)
							.to_request(),
					)
					.await;
				assert_status!(&response, StatusCode::OK);
				let body: Value = test::read_body_json(response).await;
				assert_eq!(body["account_locked"], json!(account_locked));
				match account_locked {
					false => assert!(body["email"].is_string()),
					true => {
						for field in [
							"email",
							"email_verified",
							"auth_providers",
							"has_password",
							"has_totp",
							"payout_data",
							"stripe_customer_id",
							"allow_friend_requests",
							"eligibility_verified_at",
							"github_id",
							"discord_id",
							"steam_id",
							"moderation_notes",
						] {
							assert!(
								body[field].is_null(),
								"{field} must not be exposed to a locked account"
							);
						}
					}
				}
			}
		}
	})
	.await;
}

#[actix_rt::test]
async fn existing_pat_session_and_oauth_credentials_obey_account_locks_and_cache_invalidation()
 {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
        let client = &env.dummy.oauth_client_alpha;
        let oauth_token = env
            .api
            .complete_full_authorize_flow(
                &client.client_id,
                &client.client_secret,
                Some("USER_READ USER_WRITE"),
                None,
                None,
                USER_USER_PAT,
            )
            .await;
        let mut transaction = env.db.pool.begin().await.unwrap();
        SessionBuilder {
            session: "mra_account_locked_test".into(),
            user_id: DBUserId(3),
            os: None,
            platform: None,
            city: None,
            country: None,
            ip: "127.0.0.1".into(),
            user_agent: "account lock test".into(),
            expires: None,
            session_expires: None,
        }
        .insert(&mut transaction)
        .await
        .unwrap();
        transaction.commit().await.unwrap();
        let tokens = [
            USER_USER_PAT.unwrap(),
            "mra_account_locked_test",
            oauth_token.as_str(),
        ];
        let queue = AuthQueue::new();
        for account_locked in [false, true, false] {
            set_account_locked(&env, "3", account_locked).await;
            for token in tokens {
                let req = test::TestRequest::get()
                    .append_pat(Some(token))
                    .append_header((
                        "x-ratelimit-key",
                        ENV.RATE_LIMIT_IGNORE_KEY.as_str(),
                    ))
                    .to_http_request();
                let result = get_user_from_headers(
                    &req,
                    &*env.db.pool,
                    &env.db.redis_pool,
                    &queue,
                    Scopes::USER_READ,
                )
                .await;
                match account_locked {
                    true => assert!(matches!(
                        result,
                        Err(AuthenticationError::AccountLocked)
                    )),
                    false => assert_eq!(
                        result.unwrap().1.account_locked,
                        Some(false)
                    ),
                }
                let (_, user) = get_full_user_from_headers_allow_locked(
                    &req,
                    &*env.db.pool,
                    &env.db.redis_pool,
                    &queue,
                    Scopes::USER_READ,
                )
                .await
                .unwrap();
                assert_eq!(user.account_locked, account_locked);
                let response = env
                    .call(
                        test::TestRequest::get()
                            .uri("/v3/user")
                            .append_pat(Some(token))
                            .append_header((
                                "x-ratelimit-key",
                                ENV.RATE_LIMIT_IGNORE_KEY.as_str(),
                            ))
                            .to_request(),
                    )
                    .await;
                assert_status!(&response, StatusCode::OK);
                let body: Value = test::read_body_json(response).await;
                assert_eq!(body["account_locked"], json!(account_locked));
                if account_locked {
                    assert!(body["auth_providers"].is_null());
                    assert!(body["email"].is_null());
                    assert!(matches!(
                        get_maybe_user_from_headers(
                            &req,
                            &*env.db.pool,
                            &env.db.redis_pool,
                            &queue,
                            Scopes::SESSION_ACCESS,
                        )
                        .await,
                        Err(AuthenticationError::AccountLocked)
                    ));
                    assert!(matches!(
                        get_user_record_from_bearer_token(
                            &req,
                            Some(token),
                            &*env.db.pool,
                            &env.db.redis_pool,
                            &queue,
                            true,
                        )
                        .await,
                        Err(AuthenticationError::AccountLocked)
                    ));
                }
            }
        }
        let req = test::TestRequest::get()
            .append_pat(Some(&oauth_token))
            .to_http_request();
        let result = get_full_user_from_headers_allow_locked(
            &req,
            &*env.db.pool,
            &env.db.redis_pool,
            &queue,
            Scopes::USER_AUTH_WRITE,
        )
        .await;
        assert!(matches!(
            result,
            Err(AuthenticationError::InvalidCredentials)
        ));
    })
    .await;
}

#[actix_rt::test]
async fn locked_accounts_cannot_mutate_resources_read_sensitive_data_or_unlock_themselves()
 {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
		for pat in [USER_USER_PAT, MOD_USER_PAT] {
			let response = env.call(test::TestRequest::patch().uri("/v3/user/3")
				.append_pat(pat).set_json(json!({"account_locked": true})).to_request()).await;
			assert_status!(&response, StatusCode::UNAUTHORIZED);
		}
		set_account_locked(&env, "3", true).await;
		let project = format!("/v3/project/{}", env.dummy.project_alpha.project_id);
		let validation = format!("{project}/validate");
		for (method, uri, body) in [
			(Method::GET, project.as_str(), json!(null)),
			(Method::GET, validation.as_str(), json!(null)),
			(Method::GET, "/v3/user/3/projects", json!(null)),
			(Method::GET, "/v3/organizations?ids=%5B%5D", json!(null)),
			(Method::GET, "/v3/collections?ids=%5B%5D", json!(null)),
			(Method::GET, "/v3/versions?ids=%5B%5D", json!(null)),
			(Method::POST, "/analytics/view", json!({"url":"https://modrinth.com/"})),
			(Method::PATCH, "/v3/user/3", json!({"account_locked":false})),
			(Method::PATCH, project.as_str(), json!({"title":"changed"})),
			(Method::PATCH, "/v3/organization/missing", json!({"name":"changed"})),
			(Method::PATCH, "/v3/collection/missing", json!({"name":"changed"})),
			(Method::GET, "/_internal/pat", json!(null)),
			(Method::POST, "/_internal/pat", json!({"name":"locked token", "scopes":0, "expires":"2099-01-01T00:00:00Z"})),
			(Method::GET, "/_internal/billing/subscriptions", json!(null)),
			(Method::GET, "/_internal/billing/payment_methods", json!(null)),
		] {
			let response = env.call(test::TestRequest::default().method(method).uri(uri)
				.append_pat(USER_USER_PAT).set_json(body).to_request()).await;
			assert_status!(&response, StatusCode::FORBIDDEN);
			let body: Value = test::read_body_json(response).await;
			assert_eq!(body["error"], "auth_error", "{uri}");
		}
		for pat in [None, Some("mrp_invalid")] {
			let response = env.call(test::TestRequest::get().uri(&project)
				.append_pat(pat).to_request()).await;
			assert_status!(&response, StatusCode::OK);
		}
		set_account_locked(&env, "1", true).await;
		let response = env.call(test::TestRequest::patch().uri("/v3/user/3")
			.append_pat(ADMIN_USER_PAT).set_json(json!({"account_locked":false})).to_request()).await;
		assert_status!(&response, StatusCode::FORBIDDEN);
		for version in ["v2", "v3"] {
			let response = env.call(test::TestRequest::get()
				.uri(&format!("/{version}/user"))
				.append_pat(ADMIN_USER_PAT).to_request()).await;
			assert_status!(&response, StatusCode::OK);
			let body: Value = test::read_body_json(response).await;
			assert_eq!(body["account_locked"], true);
			for field in ["email", "auth_providers", "github_id", "discord_id", "steam_id", "payout_data", "moderation_notes"] {
				assert!(body[field].is_null(), "{field} must not be exposed to a locked admin");
			}
		}
	}).await;
}

#[actix_rt::test]
async fn flows_created_before_lock_cannot_reset_password_verify_email_or_issue_tokens()
 {
    with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
		let password_flow = DBFlow::ForgotPassword { user_id: DBUserId(3) }.insert(Duration::hours(1), &env.db.redis_pool).await.unwrap();
		let email_flow = DBFlow::ConfirmEmail { user_id: DBUserId(3), confirm_email: "user@modrinth.com".into() }.insert(Duration::hours(1), &env.db.redis_pool).await.unwrap();
		let login_flow = DBFlow::Login2FA { user_id: DBUserId(3) }.insert(Duration::hours(1), &env.db.redis_pool).await.unwrap();
		let client = &env.dummy.oauth_client_alpha;
		let response = env.api.oauth_authorize(&client.client_id, Some("USER_READ"), None, None, USER_USER_PAT).await;
		let flow = get_authorize_accept_flow_id(response).await;
		let response = env.api.oauth_accept(&flow, USER_USER_PAT).await;
		let auth_code = get_auth_code_from_redirect_params(&response).await;
		set_account_locked(&env, "3", true).await;

		for (method, uri, body) in [
			(Method::PATCH, "/_internal/auth/password", json!({"flow":password_flow,"new_password":"a long secure test password 489313"})),
			(Method::POST, "/_internal/auth/email/verify", json!({"flow":email_flow})),
			(Method::POST, "/_internal/auth/login/2fa", json!({"flow":login_flow,"code":"123456"})),
		] {
			let response = env.call(test::TestRequest::default().method(method).uri(uri).set_json(body).to_request()).await;
			assert_status!(&response, StatusCode::FORBIDDEN);
			let body: Value = test::read_body_json(response).await;
			assert_eq!(body["error"], "auth_error");
		}
		let response = env.api.oauth_token(auth_code, None, client.client_id.clone(), &client.client_secret).await;
		assert_status!(&response, StatusCode::BAD_REQUEST);
		let body: Value = test::read_body_json(response).await;
		assert_eq!(body["error"], "invalid_grant");
		assert!(body.get("access_token").is_none());
	}).await;
}

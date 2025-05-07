use std::collections::HashMap;

use actix_http::StatusCode;
use actix_web::http::header::{AUTHORIZATION, LOCATION};
use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use labrinth::auth::oauth::{
    OAuthClientAccessRequest, RespondToOAuthClientScopes, TokenRequest,
    TokenResponse,
};

use crate::{
    assert_status,
    common::api_common::{Api, AppendsOptionalPat},
};

use super::ApiV3;

impl ApiV3 {
    pub async fn complete_full_authorize_flow(
        &self,
        client_id: &str,
        client_secret: &str,
        scope: Option<&str>,
        redirect_uri: Option<&str>,
        state: Option<&str>,
        user_pat: Option<&str>,
    ) -> String {
        let auth_resp = self
            .oauth_authorize(client_id, scope, redirect_uri, state, user_pat)
            .await;
        let flow_id = get_authorize_accept_flow_id(auth_resp).await;
        let redirect_resp = self.oauth_accept(&flow_id, user_pat).await;
        let auth_code =
            get_auth_code_from_redirect_params(&redirect_resp).await;
        let token_resp = self
            .oauth_token(auth_code, None, client_id.to_string(), client_secret)
            .await;
        get_access_token(token_resp).await
    }

    pub async fn oauth_authorize(
        &self,
        client_id: &str,
        scope: Option<&str>,
        redirect_uri: Option<&str>,
        state: Option<&str>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let uri = generate_authorize_uri(client_id, scope, redirect_uri, state);
        let req = TestRequest::get().uri(&uri).append_pat(pat).to_request();
        self.call(req).await
    }

    pub async fn oauth_accept(
        &self,
        flow: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        self.call(
            TestRequest::post()
                .uri("/_internal/oauth/accept")
                .append_pat(pat)
                .set_json(RespondToOAuthClientScopes {
                    flow: flow.to_string(),
                })
                .to_request(),
        )
        .await
    }

    pub async fn oauth_reject(
        &self,
        flow: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        self.call(
            TestRequest::post()
                .uri("/_internal/oauth/reject")
                .append_pat(pat)
                .set_json(RespondToOAuthClientScopes {
                    flow: flow.to_string(),
                })
                .to_request(),
        )
        .await
    }

    pub async fn oauth_token(
        &self,
        auth_code: String,
        original_redirect_uri: Option<String>,
        client_id: String,
        client_secret: &str,
    ) -> ServiceResponse {
        self.call(
            TestRequest::post()
                .uri("/_internal/oauth/token")
                .append_header((AUTHORIZATION, client_secret))
                .set_form(TokenRequest {
                    grant_type: "authorization_code".to_string(),
                    code: auth_code,
                    redirect_uri: original_redirect_uri,
                    client_id: serde_json::from_str(&format!(
                        "\"{client_id}\""
                    ))
                    .unwrap(),
                })
                .to_request(),
        )
        .await
    }
}

pub fn generate_authorize_uri(
    client_id: &str,
    scope: Option<&str>,
    redirect_uri: Option<&str>,
    state: Option<&str>,
) -> String {
    format!(
        "/_internal/oauth/authorize?client_id={}{}{}{}",
        urlencoding::encode(client_id),
        optional_query_param("redirect_uri", redirect_uri),
        optional_query_param("scope", scope),
        optional_query_param("state", state),
    )
}

pub async fn get_authorize_accept_flow_id(response: ServiceResponse) -> String {
    assert_status!(&response, StatusCode::OK);
    test::read_body_json::<OAuthClientAccessRequest, _>(response)
        .await
        .flow_id
}

pub async fn get_auth_code_from_redirect_params(
    response: &ServiceResponse,
) -> String {
    assert_status!(response, StatusCode::OK);
    let query_params = get_redirect_location_query_params(response);
    query_params.get("code").unwrap().to_string()
}

pub async fn get_access_token(response: ServiceResponse) -> String {
    assert_status!(&response, StatusCode::OK);
    test::read_body_json::<TokenResponse, _>(response)
        .await
        .access_token
}

pub fn get_redirect_location_query_params(
    response: &ServiceResponse,
) -> actix_web::web::Query<HashMap<String, String>> {
    let redirect_location = response
        .headers()
        .get(LOCATION)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    actix_web::web::Query::<HashMap<String, String>>::from_query(
        redirect_location.split_once('?').unwrap().1,
    )
    .unwrap()
}

fn optional_query_param(key: &str, value: Option<&str>) -> String {
    if let Some(val) = value {
        format!("&{key}={}", urlencoding::encode(val))
    } else {
        "".to_string()
    }
}

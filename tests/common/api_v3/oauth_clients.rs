use actix_http::StatusCode;
use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use labrinth::{
    models::{
        oauth_clients::{OAuthClient, OAuthClientAuthorization},
        pats::Scopes,
    },
    routes::v3::oauth_clients::OAuthClientEdit,
};
use reqwest::header::AUTHORIZATION;
use serde_json::json;

use crate::common::{api_common::Api, asserts::assert_status};

use super::ApiV3;

impl ApiV3 {
    pub async fn add_oauth_client(
        &self,
        name: String,
        max_scopes: Scopes,
        redirect_uris: Vec<String>,
        pat: &str,
    ) -> ServiceResponse {
        let max_scopes = max_scopes.bits();
        let req = TestRequest::post()
            .uri("/v3/oauth/app")
            .append_header((AUTHORIZATION, pat))
            .set_json(json!({
                "name": name,
                "max_scopes": max_scopes,
                "redirect_uris": redirect_uris
            }))
            .to_request();

        self.call(req).await
    }

    pub async fn get_user_oauth_clients(&self, user_id: &str, pat: &str) -> Vec<OAuthClient> {
        let req = TestRequest::get()
            .uri(&format!("/v3/user/{}/oauth_apps", user_id))
            .append_header((AUTHORIZATION, pat))
            .to_request();
        let resp = self.call(req).await;
        assert_status(&resp, StatusCode::OK);

        test::read_body_json(resp).await
    }

    pub async fn get_oauth_client(&self, client_id: String, pat: &str) -> ServiceResponse {
        let req = TestRequest::get()
            .uri(&format!("/v3/oauth/app/{}", client_id))
            .append_header((AUTHORIZATION, pat))
            .to_request();

        self.call(req).await
    }

    pub async fn edit_oauth_client(
        &self,
        client_id: &str,
        edit: OAuthClientEdit,
        pat: &str,
    ) -> ServiceResponse {
        let req = TestRequest::patch()
            .uri(&format!("/v3/oauth/app/{}", urlencoding::encode(client_id)))
            .set_json(edit)
            .append_header((AUTHORIZATION, pat))
            .to_request();

        self.call(req).await
    }

    pub async fn delete_oauth_client(&self, client_id: &str, pat: &str) -> ServiceResponse {
        let req = TestRequest::delete()
            .uri(&format!("/v3/oauth/app/{}", client_id))
            .append_header((AUTHORIZATION, pat))
            .to_request();

        self.call(req).await
    }

    pub async fn revoke_oauth_authorization(&self, client_id: &str, pat: &str) -> ServiceResponse {
        let req = TestRequest::delete()
            .uri(&format!(
                "/v3/oauth/authorizations?client_id={}",
                urlencoding::encode(client_id)
            ))
            .append_header((AUTHORIZATION, pat))
            .to_request();
        self.call(req).await
    }

    pub async fn get_user_oauth_authorizations(&self, pat: &str) -> Vec<OAuthClientAuthorization> {
        let req = TestRequest::get()
            .uri("/v3/oauth/authorizations")
            .append_header((AUTHORIZATION, pat))
            .to_request();
        let resp = self.call(req).await;
        assert_status(&resp, StatusCode::OK);

        test::read_body_json(resp).await
    }
}

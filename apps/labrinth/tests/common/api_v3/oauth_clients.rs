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
use serde_json::json;

use crate::{
    assert_status,
    common::api_common::{Api, AppendsOptionalPat},
};

use super::ApiV3;

impl ApiV3 {
    pub async fn add_oauth_client(
        &self,
        name: String,
        max_scopes: Scopes,
        redirect_uris: Vec<String>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let max_scopes = max_scopes.bits();
        let req = TestRequest::post()
            .uri("/_internal/oauth/app")
            .append_pat(pat)
            .set_json(json!({
                "name": name,
                "max_scopes": max_scopes,
                "redirect_uris": redirect_uris
            }))
            .to_request();

        self.call(req).await
    }

    pub async fn get_user_oauth_clients(
        &self,
        user_id: &str,
        pat: Option<&str>,
    ) -> Vec<OAuthClient> {
        let req = TestRequest::get()
            .uri(&format!("/v3/user/{user_id}/oauth_apps"))
            .append_pat(pat)
            .to_request();
        let resp = self.call(req).await;
        assert_status!(&resp, StatusCode::OK);

        test::read_body_json(resp).await
    }

    pub async fn get_oauth_client(
        &self,
        client_id: String,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = TestRequest::get()
            .uri(&format!("/_internal/oauth/app/{client_id}"))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    pub async fn edit_oauth_client(
        &self,
        client_id: &str,
        edit: OAuthClientEdit,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = TestRequest::patch()
            .uri(&format!(
                "/_internal/oauth/app/{}",
                urlencoding::encode(client_id)
            ))
            .set_json(edit)
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    pub async fn delete_oauth_client(
        &self,
        client_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = TestRequest::delete()
            .uri(&format!("/_internal/oauth/app/{client_id}"))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    pub async fn revoke_oauth_authorization(
        &self,
        client_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = TestRequest::delete()
            .uri(&format!(
                "/_internal/oauth/authorizations?client_id={}",
                urlencoding::encode(client_id)
            ))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    pub async fn get_user_oauth_authorizations(
        &self,
        pat: Option<&str>,
    ) -> Vec<OAuthClientAuthorization> {
        let req = TestRequest::get()
            .uri("/_internal/oauth/authorizations")
            .append_pat(pat)
            .to_request();
        let resp = self.call(req).await;
        assert_status!(&resp, StatusCode::OK);

        test::read_body_json(resp).await
    }
}

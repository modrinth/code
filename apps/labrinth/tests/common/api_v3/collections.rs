use actix_http::StatusCode;
use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use bytes::Bytes;
use labrinth::models::{collections::Collection, v3::projects::Project};
use serde_json::json;

use crate::{
    assert_status,
    common::api_common::{Api, AppendsOptionalPat, request_data::ImageData},
};

use super::ApiV3;

impl ApiV3 {
    pub async fn create_collection(
        &self,
        collection_title: &str,
        description: &str,
        projects: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri("/v3/collection")
            .append_pat(pat)
            .set_json(json!({
                "name": collection_title,
                "description": description,
                "projects": projects,
            }))
            .to_request();
        self.call(req).await
    }

    pub async fn get_collection(
        &self,
        id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = TestRequest::get()
            .uri(&format!("/v3/collection/{id}"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    pub async fn get_collection_deserialized(
        &self,
        id: &str,
        pat: Option<&str>,
    ) -> Collection {
        let resp = self.get_collection(id, pat).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_collections(
        &self,
        ids: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse {
        let ids = serde_json::to_string(ids).unwrap();
        let req = test::TestRequest::get()
            .uri(&format!(
                "/v3/collections?ids={}",
                urlencoding::encode(&ids)
            ))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    pub async fn get_collection_projects(
        &self,
        id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v3/collection/{id}/projects"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    pub async fn get_collection_projects_deserialized(
        &self,
        id: &str,
        pat: Option<&str>,
    ) -> Vec<Project> {
        let resp = self.get_collection_projects(id, pat).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn edit_collection(
        &self,
        id: &str,
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v3/collection/{id}"))
            .append_pat(pat)
            .set_json(patch)
            .to_request();

        self.call(req).await
    }

    pub async fn edit_collection_icon(
        &self,
        id: &str,
        icon: Option<ImageData>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        if let Some(icon) = icon {
            // If an icon is provided, upload it
            let req = test::TestRequest::patch()
                .uri(&format!(
                    "/v3/collection/{id}/icon?ext={ext}",
                    ext = icon.extension
                ))
                .append_pat(pat)
                .set_payload(Bytes::from(icon.icon))
                .to_request();

            self.call(req).await
        } else {
            // If no icon is provided, delete the icon
            let req = test::TestRequest::delete()
                .uri(&format!("/v3/collection/{id}/icon"))
                .append_pat(pat)
                .to_request();

            self.call(req).await
        }
    }

    pub async fn delete_collection(
        &self,
        id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v3/collection/{id}"))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    pub async fn get_user_collections(
        &self,
        user_id_or_username: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v3/user/{user_id_or_username}/collections"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    pub async fn get_user_collections_deserialized_common(
        &self,
        user_id_or_username: &str,
        pat: Option<&str>,
    ) -> Vec<Collection> {
        let resp = self.get_user_collections(user_id_or_username, pat).await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let projects: Vec<Project> = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(projects).unwrap();
        serde_json::from_value(value).unwrap()
    }
}

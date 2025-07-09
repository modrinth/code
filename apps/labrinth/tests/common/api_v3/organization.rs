use crate::{
    assert_status,
    common::api_common::{Api, AppendsOptionalPat, request_data::ImageData},
};
use actix_http::StatusCode;
use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use ariadne::ids::UserId;
use bytes::Bytes;
use labrinth::models::{organizations::Organization, v3::projects::Project};
use serde_json::json;

use super::ApiV3;

impl ApiV3 {
    pub async fn create_organization(
        &self,
        organization_title: &str,
        organization_slug: &str,
        description: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri("/v3/organization")
            .append_pat(pat)
            .set_json(json!({
                "name": organization_title,
                "slug": organization_slug,
                "description": description,
            }))
            .to_request();
        self.call(req).await
    }

    pub async fn get_organization(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = TestRequest::get()
            .uri(&format!("/v3/organization/{id_or_title}"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    pub async fn get_organization_deserialized(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> Organization {
        let resp = self.get_organization(id_or_title, pat).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_organizations(
        &self,
        ids_or_titles: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse {
        let ids_or_titles = serde_json::to_string(ids_or_titles).unwrap();
        let req = test::TestRequest::get()
            .uri(&format!(
                "/v3/organizations?ids={}",
                urlencoding::encode(&ids_or_titles)
            ))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    pub async fn get_organization_projects(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v3/organization/{id_or_title}/projects"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    pub async fn get_organization_projects_deserialized(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> Vec<Project> {
        let resp = self.get_organization_projects(id_or_title, pat).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn edit_organization(
        &self,
        id_or_title: &str,
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v3/organization/{id_or_title}"))
            .append_pat(pat)
            .set_json(patch)
            .to_request();

        self.call(req).await
    }

    pub async fn edit_organization_icon(
        &self,
        id_or_title: &str,
        icon: Option<ImageData>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        if let Some(icon) = icon {
            // If an icon is provided, upload it
            let req = test::TestRequest::patch()
                .uri(&format!(
                    "/v3/organization/{id_or_title}/icon?ext={ext}",
                    ext = icon.extension
                ))
                .append_pat(pat)
                .set_payload(Bytes::from(icon.icon))
                .to_request();

            self.call(req).await
        } else {
            // If no icon is provided, delete the icon
            let req = test::TestRequest::delete()
                .uri(&format!("/v3/organization/{id_or_title}/icon"))
                .append_pat(pat)
                .to_request();

            self.call(req).await
        }
    }

    pub async fn delete_organization(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v3/organization/{id_or_title}"))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    pub async fn organization_add_project(
        &self,
        id_or_title: &str,
        project_id_or_slug: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri(&format!("/v3/organization/{id_or_title}/projects"))
            .append_pat(pat)
            .set_json(json!({
                "project_id": project_id_or_slug,
            }))
            .to_request();

        self.call(req).await
    }

    pub async fn organization_remove_project(
        &self,
        id_or_title: &str,
        project_id_or_slug: &str,
        new_owner_user_id: UserId,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!(
                "/v3/organization/{id_or_title}/projects/{project_id_or_slug}"
            ))
            .set_json(json!({
                "new_owner": new_owner_user_id,
            }))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }
}

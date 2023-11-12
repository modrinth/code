use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use bytes::Bytes;
use labrinth::models::{organizations::Organization, v2::projects::LegacyProject};
use serde_json::json;

use crate::common::request_data::ImageData;

use super::ApiV2;

impl ApiV2 {
    pub async fn create_organization(
        &self,
        organization_title: &str,
        description: &str,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri("/v2/organization")
            .append_header(("Authorization", pat))
            .set_json(json!({
                "title": organization_title,
                "description": description,
            }))
            .to_request();
        self.call(req).await
    }

    pub async fn get_organization(&self, id_or_title: &str, pat: &str) -> ServiceResponse {
        let req = TestRequest::get()
            .uri(&format!("/v2/organization/{id_or_title}"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    pub async fn get_organization_deserialized(
        &self,
        id_or_title: &str,
        pat: &str,
    ) -> Organization {
        let resp = self.get_organization(id_or_title, pat).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn get_organization_projects(&self, id_or_title: &str, pat: &str) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/organization/{id_or_title}/projects"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    pub async fn get_organization_projects_deserialized(
        &self,
        id_or_title: &str,
        pat: &str,
    ) -> Vec<LegacyProject> {
        let resp = self.get_organization_projects(id_or_title, pat).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn edit_organization(
        &self,
        id_or_title: &str,
        patch: serde_json::Value,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v2/organization/{id_or_title}"))
            .append_header(("Authorization", pat))
            .set_json(patch)
            .to_request();

        self.call(req).await
    }

    pub async fn edit_organization_icon(
        &self,
        id_or_title: &str,
        icon: Option<ImageData>,
        pat: &str,
    ) -> ServiceResponse {
        if let Some(icon) = icon {
            // If an icon is provided, upload it
            let req = test::TestRequest::patch()
                .uri(&format!(
                    "/v2/organization/{id_or_title}/icon?ext={ext}",
                    ext = icon.extension
                ))
                .append_header(("Authorization", pat))
                .set_payload(Bytes::from(icon.icon))
                .to_request();

            self.call(req).await
        } else {
            // If no icon is provided, delete the icon
            let req = test::TestRequest::delete()
                .uri(&format!("/v2/organization/{id_or_title}/icon"))
                .append_header(("Authorization", pat))
                .to_request();

            self.call(req).await
        }
    }

    pub async fn delete_organization(&self, id_or_title: &str, pat: &str) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v2/organization/{id_or_title}"))
            .append_header(("Authorization", pat))
            .to_request();

        self.call(req).await
    }

    pub async fn organization_add_project(
        &self,
        id_or_title: &str,
        project_id_or_slug: &str,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri(&format!("/v2/organization/{id_or_title}/projects"))
            .append_header(("Authorization", pat))
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
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!(
                "/v2/organization/{id_or_title}/projects/{project_id_or_slug}"
            ))
            .append_header(("Authorization", pat))
            .to_request();

        self.call(req).await
    }
}

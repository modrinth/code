use actix_http::StatusCode;
use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use bytes::Bytes;
use labrinth::models::projects::{Project, Version};
use serde_json::json;

use crate::common::{
    actix::AppendsMultipart,
    asserts::assert_status,
    database::MOD_USER_PAT,
    request_data::{ImageData, ProjectCreationRequestData},
};

use super::ApiV2;

impl ApiV2 {
    pub async fn add_public_project(
        &self,
        creation_data: ProjectCreationRequestData,
        pat: &str,
    ) -> (Project, Vec<Version>) {
        // Add a project.
        let req = TestRequest::post()
            .uri("/v2/project")
            .append_header(("Authorization", pat))
            .set_multipart(creation_data.segment_data)
            .to_request();
        let resp = self.call(req).await;
        assert_status(&resp, StatusCode::OK);

        // Approve as a moderator.
        let req = TestRequest::patch()
            .uri(&format!("/v2/project/{}", creation_data.slug))
            .append_header(("Authorization", MOD_USER_PAT))
            .set_json(json!(
                {
                    "status": "approved"
                }
            ))
            .to_request();
        let resp = self.call(req).await;
        assert_status(&resp, StatusCode::NO_CONTENT);

        let project = self
            .get_project_deserialized(&creation_data.slug, pat)
            .await;

        // Get project's versions
        let req = TestRequest::get()
            .uri(&format!("/v2/project/{}/version", creation_data.slug))
            .append_header(("Authorization", pat))
            .to_request();
        let resp = self.call(req).await;
        let versions: Vec<Version> = test::read_body_json(resp).await;

        (project, versions)
    }

    pub async fn remove_project(&self, project_slug_or_id: &str, pat: &str) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v2/project/{project_slug_or_id}"))
            .append_header(("Authorization", pat))
            .to_request();
        let resp = self.call(req).await;
        assert_eq!(resp.status(), 204);
        resp
    }

    pub async fn get_project(&self, id_or_slug: &str, pat: &str) -> ServiceResponse {
        let req = TestRequest::get()
            .uri(&format!("/v2/project/{id_or_slug}"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }
    pub async fn get_project_deserialized(&self, id_or_slug: &str, pat: &str) -> Project {
        let resp = self.get_project(id_or_slug, pat).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn get_user_projects(&self, user_id_or_username: &str, pat: &str) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/user/{}/projects", user_id_or_username))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    pub async fn get_user_projects_deserialized(
        &self,
        user_id_or_username: &str,
        pat: &str,
    ) -> Vec<Project> {
        let resp = self.get_user_projects(user_id_or_username, pat).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn get_version_from_hash(
        &self,
        hash: &str,
        algorithm: &str,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/version_file/{hash}?algorithm={algorithm}"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    pub async fn get_version_from_hash_deserialized(
        &self,
        hash: &str,
        algorithm: &str,
        pat: &str,
    ) -> Version {
        let resp = self.get_version_from_hash(hash, algorithm, pat).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn edit_project(
        &self,
        id_or_slug: &str,
        patch: serde_json::Value,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v2/project/{id_or_slug}"))
            .append_header(("Authorization", pat))
            .set_json(patch)
            .to_request();

        self.call(req).await
    }

    pub async fn edit_project_bulk(
        &self,
        ids_or_slugs: impl IntoIterator<Item = &str>,
        patch: serde_json::Value,
        pat: &str,
    ) -> ServiceResponse {
        let projects_str = ids_or_slugs
            .into_iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(",");
        let req = test::TestRequest::patch()
            .uri(&format!(
                "/v2/projects?ids={encoded}",
                encoded = urlencoding::encode(&format!("[{projects_str}]"))
            ))
            .append_header(("Authorization", pat))
            .set_json(patch)
            .to_request();

        self.call(req).await
    }

    pub async fn edit_project_icon(
        &self,
        id_or_slug: &str,
        icon: Option<ImageData>,
        pat: &str,
    ) -> ServiceResponse {
        if let Some(icon) = icon {
            // If an icon is provided, upload it
            let req = test::TestRequest::patch()
                .uri(&format!(
                    "/v2/project/{id_or_slug}/icon?ext={ext}",
                    ext = icon.extension
                ))
                .append_header(("Authorization", pat))
                .set_payload(Bytes::from(icon.icon))
                .to_request();

            self.call(req).await
        } else {
            // If no icon is provided, delete the icon
            let req = test::TestRequest::delete()
                .uri(&format!("/v2/project/{id_or_slug}/icon"))
                .append_header(("Authorization", pat))
                .to_request();

            self.call(req).await
        }
    }
}

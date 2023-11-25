use std::collections::HashMap;

use actix_http::StatusCode;
use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use labrinth::{search::SearchResults, util::actix::AppendsMultipart};
use rust_decimal::Decimal;
use serde_json::json;

use crate::common::{
    api_common::{
        models::{CommonImageData, CommonProject, CommonVersion},
        Api, ApiProject,
    },
    asserts::assert_status,
    database::MOD_USER_PAT,
    dummy_data::TestFile,
};

use super::{request_data::get_public_project_creation_data, ApiV3};

#[async_trait(?Send)]
impl ApiProject for ApiV3 {
    async fn add_public_project(
        &self,
        slug: &str,
        version_jar: Option<TestFile>,
        modify_json: Option<json_patch::Patch>,
        pat: &str,
    ) -> (CommonProject, Vec<CommonVersion>) {
        let creation_data = get_public_project_creation_data(slug, version_jar, modify_json);

        // Add a project.
        let req = TestRequest::post()
            .uri("/v3/project")
            .append_header(("Authorization", pat))
            .set_multipart(creation_data.segment_data)
            .to_request();
        let resp = self.call(req).await;
        assert_status(&resp, StatusCode::OK);

        // Approve as a moderator.
        let req = TestRequest::patch()
            .uri(&format!("/v3/project/{}", creation_data.slug))
            .append_header(("Authorization", MOD_USER_PAT))
            .set_json(json!(
                {
                    "status": "approved"
                }
            ))
            .to_request();
        let resp = self.call(req).await;
        assert_status(&resp, StatusCode::NO_CONTENT);

        let project = self.get_project(&creation_data.slug, pat).await;
        let project = test::read_body_json(project).await;

        // Get project's versions
        let req = TestRequest::get()
            .uri(&format!("/v3/project/{}/version", creation_data.slug))
            .append_header(("Authorization", pat))
            .to_request();
        let resp = self.call(req).await;
        let versions: Vec<CommonVersion> = test::read_body_json(resp).await;

        (project, versions)
    }

    async fn remove_project(&self, project_slug_or_id: &str, pat: &str) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v3/project/{project_slug_or_id}"))
            .append_header(("Authorization", pat))
            .to_request();
        let resp = self.call(req).await;
        assert_eq!(resp.status(), 204);
        resp
    }

    async fn get_project(&self, id_or_slug: &str, pat: &str) -> ServiceResponse {
        let req = TestRequest::get()
            .uri(&format!("/v3/project/{id_or_slug}"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    async fn get_project_deserialized_common(&self, id_or_slug: &str, pat: &str) -> CommonProject {
        let resp = self.get_project(id_or_slug, pat).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    async fn get_user_projects(&self, user_id_or_username: &str, pat: &str) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v3/user/{}/projects", user_id_or_username))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    async fn get_user_projects_deserialized_common(
        &self,
        user_id_or_username: &str,
        pat: &str,
    ) -> Vec<CommonProject> {
        let resp = self.get_user_projects(user_id_or_username, pat).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    async fn edit_project(
        &self,
        id_or_slug: &str,
        patch: serde_json::Value,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v3/project/{id_or_slug}"))
            .append_header(("Authorization", pat))
            .set_json(patch)
            .to_request();

        self.call(req).await
    }

    async fn edit_project_bulk(
        &self,
        ids_or_slugs: &[&str],
        patch: serde_json::Value,
        pat: &str,
    ) -> ServiceResponse {
        let projects_str = ids_or_slugs
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(",");
        let req = test::TestRequest::patch()
            .uri(&format!(
                "/v3/projects?ids={encoded}",
                encoded = urlencoding::encode(&format!("[{projects_str}]"))
            ))
            .append_header(("Authorization", pat))
            .set_json(patch)
            .to_request();

        self.call(req).await
    }

    async fn edit_project_icon(
        &self,
        id_or_slug: &str,
        icon: Option<CommonImageData>,
        pat: &str,
    ) -> ServiceResponse {
        if let Some(icon) = icon {
            // If an icon is provided, upload it
            let req = test::TestRequest::patch()
                .uri(&format!(
                    "/v3/project/{id_or_slug}/icon?ext={ext}",
                    ext = icon.extension
                ))
                .append_header(("Authorization", pat))
                .set_payload(Bytes::from(icon.icon))
                .to_request();

            self.call(req).await
        } else {
            // If no icon is provided, delete the icon
            let req = test::TestRequest::delete()
                .uri(&format!("/v3/project/{id_or_slug}/icon"))
                .append_header(("Authorization", pat))
                .to_request();

            self.call(req).await
        }
    }

    async fn search_deserialized_common(
        &self,
        query: Option<&str>,
        facets: Option<serde_json::Value>,
        pat: &str,
    ) -> SearchResults {
        let query_field = if let Some(query) = query {
            format!("&query={}", urlencoding::encode(query))
        } else {
            "".to_string()
        };

        let facets_field = if let Some(facets) = facets {
            format!("&facets={}", urlencoding::encode(&facets.to_string()))
        } else {
            "".to_string()
        };

        let req = test::TestRequest::get()
            .uri(&format!("/v3/search?{}{}", query_field, facets_field))
            .append_header(("Authorization", pat))
            .to_request();
        let resp = self.call(req).await;
        let status = resp.status();
        assert_eq!(status, 200);
        test::read_body_json(resp).await
    }
}

impl ApiV3 {
    pub async fn get_analytics_revenue(
        &self,
        id_or_slugs: Vec<&str>,
        ids_are_version_ids: bool,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        resolution_minutes: Option<u32>,
        pat: &str,
    ) -> ServiceResponse {
        let pv_string = if ids_are_version_ids {
            let version_string: String = serde_json::to_string(&id_or_slugs).unwrap();
            let version_string = urlencoding::encode(&version_string);
            format!("version_ids={}", version_string)
        } else {
            let projects_string: String = serde_json::to_string(&id_or_slugs).unwrap();
            let projects_string = urlencoding::encode(&projects_string);
            format!("project_ids={}", projects_string)
        };

        let mut extra_args = String::new();
        if let Some(start_date) = start_date {
            let start_date = start_date.to_rfc3339();
            // let start_date = serde_json::to_string(&start_date).unwrap();
            let start_date = urlencoding::encode(&start_date);
            extra_args.push_str(&format!("&start_date={start_date}"));
        }
        if let Some(end_date) = end_date {
            let end_date = end_date.to_rfc3339();
            // let end_date = serde_json::to_string(&end_date).unwrap();
            let end_date = urlencoding::encode(&end_date);
            extra_args.push_str(&format!("&end_date={end_date}"));
        }
        if let Some(resolution_minutes) = resolution_minutes {
            extra_args.push_str(&format!("&resolution_minutes={}", resolution_minutes));
        }

        let req = test::TestRequest::get()
            .uri(&format!("/v3/analytics/revenue?{pv_string}{extra_args}",))
            .append_header(("Authorization", pat))
            .to_request();

        self.call(req).await
    }

    pub async fn get_analytics_revenue_deserialized(
        &self,
        id_or_slugs: Vec<&str>,
        ids_are_version_ids: bool,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        resolution_minutes: Option<u32>,
        pat: &str,
    ) -> HashMap<String, HashMap<i64, Decimal>> {
        let resp = self
            .get_analytics_revenue(
                id_or_slugs,
                ids_are_version_ids,
                start_date,
                end_date,
                resolution_minutes,
                pat,
            )
            .await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }
}

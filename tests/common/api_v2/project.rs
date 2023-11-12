use std::collections::HashMap;

use actix_http::StatusCode;
use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use labrinth::{
    models::v2::projects::{LegacyProject, LegacyVersion},
    search::SearchResults,
    util::actix::AppendsMultipart,
};
use rust_decimal::Decimal;
use serde_json::json;

use crate::common::{
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
    ) -> (LegacyProject, Vec<LegacyVersion>) {
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
        let versions: Vec<LegacyVersion> = test::read_body_json(resp).await;

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
    pub async fn get_project_deserialized(&self, id_or_slug: &str, pat: &str) -> LegacyProject {
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
    ) -> Vec<LegacyProject> {
        let resp = self.get_user_projects(user_id_or_username, pat).await;
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

    pub async fn search_deserialized(
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
            .uri(&format!("/v2/search?{}{}", query_field, facets_field))
            .append_header(("Authorization", pat))
            .to_request();
        let resp = self.call(req).await;
        let status = resp.status();
        assert_eq!(status, 200);
        test::read_body_json(resp).await
    }

    pub async fn get_analytics_revenue(
        &self,
        id_or_slugs: Vec<&str>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        resolution_minutes: Option<u32>,
        pat: &str,
    ) -> ServiceResponse {
        let projects_string = serde_json::to_string(&id_or_slugs).unwrap();
        let projects_string = urlencoding::encode(&projects_string);

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
            .uri(&format!(
                "/v2/analytics/revenue?{projects_string}{extra_args}",
            ))
            .append_header(("Authorization", pat))
            .to_request();

        self.call(req).await
    }

    pub async fn get_analytics_revenue_deserialized(
        &self,
        id_or_slugs: Vec<&str>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        resolution_minutes: Option<u32>,
        pat: &str,
    ) -> HashMap<String, HashMap<i64, Decimal>> {
        let resp = self
            .get_analytics_revenue(id_or_slugs, start_date, end_date, resolution_minutes, pat)
            .await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }
}

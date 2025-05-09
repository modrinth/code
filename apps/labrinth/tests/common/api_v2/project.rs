use std::collections::HashMap;

use crate::{
    assert_status,
    common::{
        api_common::{
            Api, ApiProject, AppendsOptionalPat,
            models::{CommonItemType, CommonProject, CommonVersion},
            request_data::{ImageData, ProjectCreationRequestData},
        },
        dummy_data::TestFile,
    },
};
use actix_http::StatusCode;
use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use async_trait::async_trait;
use bytes::Bytes;
use labrinth::{
    models::v2::{projects::LegacyProject, search::LegacySearchResults},
    util::actix::AppendsMultipart,
};
use serde_json::json;

use crate::common::database::MOD_USER_PAT;

use super::{
    ApiV2,
    request_data::{self, get_public_project_creation_data},
};

impl ApiV2 {
    pub async fn get_project_deserialized(
        &self,
        id_or_slug: &str,
        pat: Option<&str>,
    ) -> LegacyProject {
        let resp = self.get_project(id_or_slug, pat).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_user_projects_deserialized(
        &self,
        user_id_or_username: &str,
        pat: Option<&str>,
    ) -> Vec<LegacyProject> {
        let resp = self.get_user_projects(user_id_or_username, pat).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn search_deserialized(
        &self,
        query: Option<&str>,
        facets: Option<serde_json::Value>,
        pat: Option<&str>,
    ) -> LegacySearchResults {
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
            .uri(&format!("/v2/search?{query_field}{facets_field}"))
            .append_pat(pat)
            .to_request();
        let resp = self.call(req).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }
}

#[async_trait(?Send)]
impl ApiProject for ApiV2 {
    async fn add_public_project(
        &self,
        slug: &str,
        version_jar: Option<TestFile>,
        modify_json: Option<json_patch::Patch>,
        pat: Option<&str>,
    ) -> (CommonProject, Vec<CommonVersion>) {
        let creation_data =
            get_public_project_creation_data(slug, version_jar, modify_json);

        // Add a project.
        let slug = creation_data.slug.clone();
        let resp = self.create_project(creation_data, pat).await;
        assert_status!(&resp, StatusCode::OK);

        // Approve as a moderator.
        let req = TestRequest::patch()
            .uri(&format!("/v2/project/{slug}"))
            .append_pat(MOD_USER_PAT)
            .set_json(json!(
                {
                    "status": "approved"
                }
            ))
            .to_request();
        let resp = self.call(req).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        let project = self.get_project_deserialized_common(&slug, pat).await;

        // Get project's versions
        let req = TestRequest::get()
            .uri(&format!("/v2/project/{slug}/version"))
            .append_pat(pat)
            .to_request();
        let resp = self.call(req).await;
        let versions: Vec<CommonVersion> = test::read_body_json(resp).await;

        (project, versions)
    }

    async fn get_public_project_creation_data_json(
        &self,
        slug: &str,
        version_jar: Option<&TestFile>,
    ) -> serde_json::Value {
        request_data::get_public_project_creation_data_json(slug, version_jar)
    }

    async fn create_project(
        &self,
        creation_data: ProjectCreationRequestData,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = TestRequest::post()
            .uri("/v2/project")
            .append_pat(pat)
            .set_multipart(creation_data.segment_data)
            .to_request();
        self.call(req).await
    }

    async fn remove_project(
        &self,
        project_slug_or_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v2/project/{project_slug_or_id}"))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    async fn get_project(
        &self,
        id_or_slug: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = TestRequest::get()
            .uri(&format!("/v2/project/{id_or_slug}"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn get_project_deserialized_common(
        &self,
        id_or_slug: &str,
        pat: Option<&str>,
    ) -> CommonProject {
        let resp = self.get_project(id_or_slug, pat).await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let project: LegacyProject = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(project).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn get_projects(
        &self,
        ids_or_slugs: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse {
        let ids_or_slugs = serde_json::to_string(ids_or_slugs).unwrap();
        let req = test::TestRequest::get()
            .uri(&format!(
                "/v2/projects?ids={encoded}",
                encoded = urlencoding::encode(&ids_or_slugs)
            ))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn get_project_dependencies(
        &self,
        id_or_slug: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = TestRequest::get()
            .uri(&format!("/v2/project/{id_or_slug}/dependencies"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn get_user_projects(
        &self,
        user_id_or_username: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/user/{user_id_or_username}/projects"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn get_user_projects_deserialized_common(
        &self,
        user_id_or_username: &str,
        pat: Option<&str>,
    ) -> Vec<CommonProject> {
        let resp = self.get_user_projects(user_id_or_username, pat).await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let projects: Vec<LegacyProject> = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(projects).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn edit_project(
        &self,
        id_or_slug: &str,
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v2/project/{id_or_slug}"))
            .append_pat(pat)
            .set_json(patch)
            .to_request();

        self.call(req).await
    }

    async fn edit_project_bulk(
        &self,
        ids_or_slugs: &[&str],
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let projects_str = ids_or_slugs
            .iter()
            .map(|s| format!("\"{s}\""))
            .collect::<Vec<_>>()
            .join(",");
        let req = test::TestRequest::patch()
            .uri(&format!(
                "/v2/projects?ids={encoded}",
                encoded = urlencoding::encode(&format!("[{projects_str}]"))
            ))
            .append_pat(pat)
            .set_json(patch)
            .to_request();

        self.call(req).await
    }

    async fn edit_project_icon(
        &self,
        id_or_slug: &str,
        icon: Option<ImageData>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        if let Some(icon) = icon {
            // If an icon is provided, upload it
            let req = test::TestRequest::patch()
                .uri(&format!(
                    "/v2/project/{id_or_slug}/icon?ext={ext}",
                    ext = icon.extension
                ))
                .append_pat(pat)
                .set_payload(Bytes::from(icon.icon))
                .to_request();

            self.call(req).await
        } else {
            // If no icon is provided, delete the icon
            let req = test::TestRequest::delete()
                .uri(&format!("/v2/project/{id_or_slug}/icon"))
                .append_pat(pat)
                .to_request();

            self.call(req).await
        }
    }

    async fn create_report(
        &self,
        report_type: &str,
        id: &str,
        item_type: CommonItemType,
        body: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri("/v2/report")
            .append_pat(pat)
            .set_json(json!(
                {
                    "report_type": report_type,
                    "item_id": id,
                    "item_type": item_type.as_str(),
                    "body": body,
                }
            ))
            .to_request();

        self.call(req).await
    }

    async fn get_report(&self, id: &str, pat: Option<&str>) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/report/{id}"))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    async fn get_reports(
        &self,
        ids: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse {
        let ids_str = serde_json::to_string(ids).unwrap();
        let req = test::TestRequest::get()
            .uri(&format!(
                "/v2/reports?ids={encoded}",
                encoded = urlencoding::encode(&ids_str)
            ))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    async fn get_user_reports(&self, pat: Option<&str>) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri("/v2/report")
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    async fn delete_report(
        &self,
        id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v2/report/{id}"))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    async fn edit_report(
        &self,
        id: &str,
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v2/report/{id}"))
            .append_pat(pat)
            .set_json(patch)
            .to_request();

        self.call(req).await
    }

    async fn get_thread(&self, id: &str, pat: Option<&str>) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v3/thread/{id}"))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    async fn get_threads(
        &self,
        ids: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse {
        let ids_str = serde_json::to_string(ids).unwrap();
        let req = test::TestRequest::get()
            .uri(&format!(
                "/v3/threads?ids={encoded}",
                encoded = urlencoding::encode(&ids_str)
            ))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    async fn write_to_thread(
        &self,
        id: &str,
        r#type: &str,
        content: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri(&format!("/v2/thread/{id}"))
            .append_pat(pat)
            .set_json(json!({
                "body" : {
                    "type": r#type,
                    "body": content,
                }
            }))
            .to_request();

        self.call(req).await
    }

    async fn get_moderation_inbox(&self, pat: Option<&str>) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri("/v2/thread/inbox")
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    async fn read_thread(
        &self,
        id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri(&format!("/v2/thread/{id}/read"))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    async fn delete_thread_message(
        &self,
        id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v2/message/{id}"))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn add_gallery_item(
        &self,
        id_or_slug: &str,
        image: ImageData,
        featured: bool,
        title: Option<String>,
        description: Option<String>,
        ordering: Option<i32>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let mut url = format!(
            "/v2/project/{id_or_slug}/gallery?ext={ext}&featured={featured}",
            ext = image.extension,
            featured = featured
        );
        if let Some(title) = title {
            url.push_str(&format!("&title={title}"));
        }
        if let Some(description) = description {
            url.push_str(&format!("&description={description}"));
        }
        if let Some(ordering) = ordering {
            url.push_str(&format!("&ordering={ordering}"));
        }

        let req = test::TestRequest::post()
            .uri(&url)
            .append_pat(pat)
            .set_payload(Bytes::from(image.icon))
            .to_request();

        self.call(req).await
    }

    async fn edit_gallery_item(
        &self,
        id_or_slug: &str,
        image_url: &str,
        patch: HashMap<String, String>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let mut url = format!(
            "/v2/project/{id_or_slug}/gallery?url={image_url}",
            image_url = urlencoding::encode(image_url)
        );

        for (key, value) in patch {
            url.push_str(&format!(
                "&{key}={value}",
                key = key,
                value = urlencoding::encode(&value)
            ));
        }

        let req = test::TestRequest::patch()
            .uri(&url)
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn remove_gallery_item(
        &self,
        id_or_slug: &str,
        url: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v2/project/{id_or_slug}/gallery?url={url}"))
            .append_pat(pat)
            .to_request();

        self.call(req).await
    }
}

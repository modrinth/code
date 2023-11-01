use actix_http::{header::AUTHORIZATION, StatusCode};
use actix_web::{dev::ServiceResponse, test};
use labrinth::models::projects::Version;
use serde_json::json;

use crate::common::{self, actix::AppendsMultipart, asserts::assert_status};

use super::ApiV2;

pub fn url_encode_json_serialized_vec(elements: &[String]) -> String {
    let serialized = serde_json::to_string(&elements).unwrap();
    urlencoding::encode(&serialized).to_string()
}

impl ApiV2 {
    pub async fn create_default_version(
        &self,
        project_id: &str,
        ordering: Option<i32>,
        pat: &str,
    ) -> Version {
        let json_data = json!(
                {
                    "project_id": project_id,
                    "file_parts": ["basic-mod-different.jar"],
                    "version_number": "1.2.3.4",
                    "version_title": "start",
                    "dependencies": [],
                    "game_versions": ["1.20.1"] ,
                    "release_channel": "release",
                    "loaders": ["fabric"],
                    "featured": true,
                    "ordering": ordering,
                }
        );
        let json_segment = common::actix::MultipartSegment {
            name: "data".to_string(),
            filename: None,
            content_type: Some("application/json".to_string()),
            data: common::actix::MultipartSegmentData::Text(
                serde_json::to_string(&json_data).unwrap(),
            ),
        };
        let file_segment = common::actix::MultipartSegment {
            name: "basic-mod-different.jar".to_string(),
            filename: Some("basic-mod.jar".to_string()),
            content_type: Some("application/java-archive".to_string()),
            data: common::actix::MultipartSegmentData::Binary(
                include_bytes!("../../../tests/files/basic-mod-different.jar").to_vec(),
            ),
        };

        let request = test::TestRequest::post()
            .uri("/v2/version")
            .set_multipart(vec![json_segment.clone(), file_segment.clone()])
            .append_header((AUTHORIZATION, pat))
            .to_request();
        let resp = self.call(request).await;
        assert_status(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_versions(&self, version_ids: Vec<String>, pat: &str) -> Vec<Version> {
        let ids = url_encode_json_serialized_vec(&version_ids);
        let request = test::TestRequest::get()
            .uri(&format!("/v2/versions?ids={}", ids))
            .append_header((AUTHORIZATION, pat))
            .to_request();
        let resp = self.call(request).await;
        assert_status(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn edit_version_ordering(
        &self,
        version_id: &str,
        ordering: Option<i32>,
        pat: &str,
    ) -> ServiceResponse {
        let request = test::TestRequest::patch()
            .uri(&format!("/v2/version/{version_id}"))
            .set_json(json!(
                {
                    "ordering": ordering
                }
            ))
            .append_header((AUTHORIZATION, pat))
            .to_request();
        self.call(request).await
    }
}

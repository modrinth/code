use std::collections::HashMap;

use actix_http::{header::AUTHORIZATION, StatusCode};
use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use labrinth::{
    models::{projects::VersionType, v3::projects::Version},
    routes::v3::version_file::FileUpdateData,
    util::actix::AppendsMultipart,
};
use serde_json::json;

use crate::common::asserts::assert_status;

use super::{request_data::VersionCreationRequestData, ApiV3};

pub fn url_encode_json_serialized_vec(elements: &[String]) -> String {
    let serialized = serde_json::to_string(&elements).unwrap();
    urlencoding::encode(&serialized).to_string()
}

impl ApiV3 {
    pub async fn add_public_version(
        &self,
        creation_data: VersionCreationRequestData,
        pat: &str,
    ) -> ServiceResponse {
        // Add a project.
        let req = TestRequest::post()
            .uri("/v3/version")
            .append_header(("Authorization", pat))
            .set_multipart(creation_data.segment_data)
            .to_request();
        self.call(req).await
    }

    pub async fn add_public_version_deserialized(
        &self,
        creation_data: VersionCreationRequestData,
        pat: &str,
    ) -> Version {
        let resp = self.add_public_version(creation_data, pat).await;
        assert_status(&resp, StatusCode::OK);
        let value: serde_json::Value = test::read_body_json(resp).await;
        let version_id = value["id"].as_str().unwrap();
        self.get_version_deserialized(version_id, pat).await
    }

    pub async fn get_version(&self, id: &str, pat: &str) -> ServiceResponse {
        let req = TestRequest::get()
            .uri(&format!("/v3/version/{id}"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    pub async fn get_version_deserialized(&self, id: &str, pat: &str) -> Version {
        let resp = self.get_version(id, pat).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn edit_version(
        &self,
        version_id: &str,
        patch: serde_json::Value,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v3/version/{version_id}"))
            .append_header(("Authorization", pat))
            .set_json(patch)
            .to_request();

        self.call(req).await
    }

    pub async fn get_version_from_hash(
        &self,
        hash: &str,
        algorithm: &str,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v3/version_file/{hash}?algorithm={algorithm}"))
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

    pub async fn get_versions_from_hashes(
        &self,
        hashes: &[&str],
        algorithm: &str,
        pat: &str,
    ) -> ServiceResponse {
        let req = TestRequest::post()
            .uri("/v3/version_files")
            .append_header(("Authorization", pat))
            .set_json(json!({
                "hashes": hashes,
                "algorithm": algorithm,
            }))
            .to_request();
        self.call(req).await
    }

    pub async fn get_versions_from_hashes_deserialized(
        &self,
        hashes: &[&str],
        algorithm: &str,
        pat: &str,
    ) -> HashMap<String, Version> {
        let resp = self.get_versions_from_hashes(hashes, algorithm, pat).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn get_update_from_hash(
        &self,
        hash: &str,
        algorithm: &str,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: &str,
    ) -> ServiceResponse {
        let mut json = json!({});
        if let Some(loaders) = loaders {
            json["loaders"] = serde_json::to_value(loaders).unwrap();
        }
        if let Some(game_versions) = game_versions {
            json["loader_fields"] = json!({
                "game_versions": game_versions,
            });
        }
        if let Some(version_types) = version_types {
            json["version_types"] = serde_json::to_value(version_types).unwrap();
        }

        let req = test::TestRequest::post()
            .uri(&format!(
                "/v3/version_file/{hash}/update?algorithm={algorithm}"
            ))
            .append_header(("Authorization", pat))
            .set_json(json)
            .to_request();
        self.call(req).await
    }

    pub async fn get_update_from_hash_deserialized(
        &self,
        hash: &str,
        algorithm: &str,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: &str,
    ) -> Version {
        let resp = self
            .get_update_from_hash(hash, algorithm, loaders, game_versions, version_types, pat)
            .await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn update_files(
        &self,
        algorithm: &str,
        hashes: Vec<String>,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: &str,
    ) -> ServiceResponse {
        let mut json = json!({
            "algorithm": algorithm,
            "hashes": hashes,
        });
        if let Some(loaders) = loaders {
            json["loaders"] = serde_json::to_value(loaders).unwrap();
        }
        if let Some(game_versions) = game_versions {
            json["loader_fields"] = json!({
                "game_versions": game_versions,
            });
        }
        if let Some(version_types) = version_types {
            json["version_types"] = serde_json::to_value(version_types).unwrap();
        }

        let req = test::TestRequest::post()
            .uri("/v3/version_files/update")
            .append_header(("Authorization", pat))
            .set_json(json)
            .to_request();
        self.call(req).await
    }

    pub async fn update_files_deserialized(
        &self,
        algorithm: &str,
        hashes: Vec<String>,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: &str,
    ) -> HashMap<String, Version> {
        let resp = self
            .update_files(
                algorithm,
                hashes,
                loaders,
                game_versions,
                version_types,
                pat,
            )
            .await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn update_individual_files(
        &self,
        algorithm: &str,
        hashes: Vec<FileUpdateData>,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri("/v3/version_files/update_individual")
            .append_header(("Authorization", pat))
            .set_json(json!({
                "algorithm": algorithm,
                "hashes": hashes
            }))
            .to_request();
        self.call(req).await
    }

    pub async fn update_individual_files_deserialized(
        &self,
        algorithm: &str,
        hashes: Vec<FileUpdateData>,
        pat: &str,
    ) -> HashMap<String, Version> {
        let resp = self.update_individual_files(algorithm, hashes, pat).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    // TODO: Not all fields are tested currently in the v3 tests, only the v2-v3 relevant ones are
    #[allow(clippy::too_many_arguments)]
    pub async fn get_project_versions(
        &self,
        project_id_slug: &str,
        game_versions: Option<Vec<String>>,
        loaders: Option<Vec<String>>,
        featured: Option<bool>,
        version_type: Option<VersionType>,
        limit: Option<usize>,
        offset: Option<usize>,
        pat: &str,
    ) -> ServiceResponse {
        let mut query_string = String::new();
        if let Some(game_versions) = game_versions {
            query_string.push_str(&format!(
                "&game_versions={}",
                urlencoding::encode(&serde_json::to_string(&game_versions).unwrap())
            ));
        }
        if let Some(loaders) = loaders {
            query_string.push_str(&format!(
                "&loaders={}",
                urlencoding::encode(&serde_json::to_string(&loaders).unwrap())
            ));
        }
        if let Some(featured) = featured {
            query_string.push_str(&format!("&featured={}", featured));
        }
        if let Some(version_type) = version_type {
            query_string.push_str(&format!("&version_type={}", version_type));
        }
        if let Some(limit) = limit {
            let limit = limit.to_string();
            query_string.push_str(&format!("&limit={}", limit));
        }
        if let Some(offset) = offset {
            let offset = offset.to_string();
            query_string.push_str(&format!("&offset={}", offset));
        }

        let req = test::TestRequest::get()
            .uri(&format!(
                "/v3/project/{project_id_slug}/version?{}",
                query_string.trim_start_matches('&')
            ))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_project_versions_deserialized(
        &self,
        slug: &str,
        game_versions: Option<Vec<String>>,
        loaders: Option<Vec<String>>,
        featured: Option<bool>,
        version_type: Option<VersionType>,
        limit: Option<usize>,
        offset: Option<usize>,
        pat: &str,
    ) -> Vec<Version> {
        let resp = self
            .get_project_versions(
                slug,
                game_versions,
                loaders,
                featured,
                version_type,
                limit,
                offset,
                pat,
            )
            .await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    // TODO: remove redundancy in these functions

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
                    "client_side": "required",
                    "server_side": "optional",
                    "release_channel": "release",
                    "loaders": ["fabric"],
                    "featured": true,
                    "ordering": ordering,
                }
        );
        let json_segment = labrinth::util::actix::MultipartSegment {
            name: "data".to_string(),
            filename: None,
            content_type: Some("application/json".to_string()),
            data: labrinth::util::actix::MultipartSegmentData::Text(
                serde_json::to_string(&json_data).unwrap(),
            ),
        };
        let file_segment = labrinth::util::actix::MultipartSegment {
            name: "basic-mod-different.jar".to_string(),
            filename: Some("basic-mod.jar".to_string()),
            content_type: Some("application/java-archive".to_string()),
            data: labrinth::util::actix::MultipartSegmentData::Binary(
                include_bytes!("../../../tests/files/basic-mod-different.jar").to_vec(),
            ),
        };

        let request = test::TestRequest::post()
            .uri("/v3/version")
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
            .uri(&format!("/v3/versions?ids={}", ids))
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
            .uri(&format!("/v3/version/{version_id}"))
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

use std::collections::HashMap;

use super::{
    ApiV2,
    request_data::{self, get_public_version_creation_data},
};
use crate::{
    assert_status,
    common::{
        api_common::{
            Api, ApiVersion, AppendsOptionalPat, models::CommonVersion,
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
use labrinth::models::ids::ProjectId;
use labrinth::{
    models::{projects::VersionType, v2::projects::LegacyVersion},
    routes::v2::version_file::FileUpdateData,
    util::actix::AppendsMultipart,
};
use serde_json::json;

pub fn url_encode_json_serialized_vec(elements: &[String]) -> String {
    let serialized = serde_json::to_string(&elements).unwrap();
    urlencoding::encode(&serialized).to_string()
}

impl ApiV2 {
    pub async fn get_version_deserialized(
        &self,
        id: &str,
        pat: Option<&str>,
    ) -> LegacyVersion {
        let resp = self.get_version(id, pat).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_version_from_hash_deserialized(
        &self,
        hash: &str,
        algorithm: &str,
        pat: Option<&str>,
    ) -> LegacyVersion {
        let resp = self.get_version_from_hash(hash, algorithm, pat).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_versions_from_hashes_deserialized(
        &self,
        hashes: &[&str],
        algorithm: &str,
        pat: Option<&str>,
    ) -> HashMap<String, LegacyVersion> {
        let resp = self.get_versions_from_hashes(hashes, algorithm, pat).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn update_individual_files(
        &self,
        algorithm: &str,
        hashes: Vec<FileUpdateData>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri("/v2/version_files/update_individual")
            .append_pat(pat)
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
        pat: Option<&str>,
    ) -> HashMap<String, LegacyVersion> {
        let resp = self.update_individual_files(algorithm, hashes, pat).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }
}

#[async_trait(?Send)]
impl ApiVersion for ApiV2 {
    async fn add_public_version(
        &self,
        project_id: ProjectId,
        version_number: &str,
        version_jar: TestFile,
        ordering: Option<i32>,
        modify_json: Option<json_patch::Patch>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let creation_data = get_public_version_creation_data(
            project_id,
            version_number,
            version_jar,
            ordering,
            modify_json,
        );

        // Add a project.
        let req = TestRequest::post()
            .uri("/v2/version")
            .append_pat(pat)
            .set_multipart(creation_data.segment_data)
            .to_request();
        self.call(req).await
    }

    async fn add_public_version_deserialized_common(
        &self,
        project_id: ProjectId,
        version_number: &str,
        version_jar: TestFile,
        ordering: Option<i32>,
        modify_json: Option<json_patch::Patch>,
        pat: Option<&str>,
    ) -> CommonVersion {
        let resp = self
            .add_public_version(
                project_id,
                version_number,
                version_jar,
                ordering,
                modify_json,
                pat,
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: LegacyVersion = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn get_version(
        &self,
        id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = TestRequest::get()
            .uri(&format!("/v2/version/{id}"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn get_version_deserialized_common(
        &self,
        id: &str,
        pat: Option<&str>,
    ) -> CommonVersion {
        let resp = self.get_version(id, pat).await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: LegacyVersion = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn download_version_redirect(
        &self,
        hash: &str,
        algorithm: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/version_file/{hash}/download",))
            .set_json(json!({
                "algorithm": algorithm,
            }))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn edit_version(
        &self,
        version_id: &str,
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v2/version/{version_id}"))
            .append_pat(pat)
            .set_json(patch)
            .to_request();

        self.call(req).await
    }

    async fn get_version_from_hash(
        &self,
        hash: &str,
        algorithm: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/version_file/{hash}?algorithm={algorithm}"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn get_version_from_hash_deserialized_common(
        &self,
        hash: &str,
        algorithm: &str,
        pat: Option<&str>,
    ) -> CommonVersion {
        let resp = self.get_version_from_hash(hash, algorithm, pat).await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: LegacyVersion = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn get_versions_from_hashes(
        &self,
        hashes: &[&str],
        algorithm: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = TestRequest::post()
            .uri("/v2/version_files")
            .append_pat(pat)
            .set_json(json!({
                "hashes": hashes,
                "algorithm": algorithm,
            }))
            .to_request();
        self.call(req).await
    }

    async fn get_versions_from_hashes_deserialized_common(
        &self,
        hashes: &[&str],
        algorithm: &str,
        pat: Option<&str>,
    ) -> HashMap<String, CommonVersion> {
        let resp = self.get_versions_from_hashes(hashes, algorithm, pat).await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: HashMap<String, LegacyVersion> =
            test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn get_update_from_hash(
        &self,
        hash: &str,
        algorithm: &str,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri(&format!(
                "/v2/version_file/{hash}/update?algorithm={algorithm}"
            ))
            .append_pat(pat)
            .set_json(json!({
                "loaders": loaders,
                "game_versions": game_versions,
                "version_types": version_types,
            }))
            .to_request();
        self.call(req).await
    }

    async fn get_update_from_hash_deserialized_common(
        &self,
        hash: &str,
        algorithm: &str,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: Option<&str>,
    ) -> CommonVersion {
        let resp = self
            .get_update_from_hash(
                hash,
                algorithm,
                loaders,
                game_versions,
                version_types,
                pat,
            )
            .await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: LegacyVersion = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn update_files(
        &self,
        algorithm: &str,
        hashes: Vec<String>,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri("/v2/version_files/update")
            .append_pat(pat)
            .set_json(json!({
                "algorithm": algorithm,
                "hashes": hashes,
                "loaders": loaders,
                "game_versions": game_versions,
                "version_types": version_types,
            }))
            .to_request();
        self.call(req).await
    }

    async fn update_files_deserialized_common(
        &self,
        algorithm: &str,
        hashes: Vec<String>,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: Option<&str>,
    ) -> HashMap<String, CommonVersion> {
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
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: HashMap<String, LegacyVersion> =
            test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }

    // TODO: Not all fields are tested currently in the V2 tests, only the v2-v3 relevant ones are
    #[allow(clippy::too_many_arguments)]
    async fn get_project_versions(
        &self,
        project_id_slug: &str,
        game_versions: Option<Vec<String>>,
        loaders: Option<Vec<String>>,
        featured: Option<bool>,
        version_type: Option<VersionType>,
        limit: Option<usize>,
        offset: Option<usize>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let mut query_string = String::new();
        if let Some(game_versions) = game_versions {
            query_string.push_str(&format!(
                "&game_versions={}",
                urlencoding::encode(
                    &serde_json::to_string(&game_versions).unwrap()
                )
            ));
        }
        if let Some(loaders) = loaders {
            query_string.push_str(&format!(
                "&loaders={}",
                urlencoding::encode(&serde_json::to_string(&loaders).unwrap())
            ));
        }
        if let Some(featured) = featured {
            query_string.push_str(&format!("&featured={featured}"));
        }
        if let Some(version_type) = version_type {
            query_string.push_str(&format!("&version_type={version_type}"));
        }
        if let Some(limit) = limit {
            let limit = limit.to_string();
            query_string.push_str(&format!("&limit={limit}"));
        }
        if let Some(offset) = offset {
            let offset = offset.to_string();
            query_string.push_str(&format!("&offset={offset}"));
        }

        let req = test::TestRequest::get()
            .uri(&format!(
                "/v2/project/{project_id_slug}/version?{}",
                query_string.trim_start_matches('&')
            ))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn get_project_versions_deserialized_common(
        &self,
        slug: &str,
        game_versions: Option<Vec<String>>,
        loaders: Option<Vec<String>>,
        featured: Option<bool>,
        version_type: Option<VersionType>,
        limit: Option<usize>,
        offset: Option<usize>,
        pat: Option<&str>,
    ) -> Vec<CommonVersion> {
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
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: Vec<LegacyVersion> = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn edit_version_ordering(
        &self,
        version_id: &str,
        ordering: Option<i32>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let request = test::TestRequest::patch()
            .uri(&format!("/v2/version/{version_id}"))
            .set_json(json!(
                {
                    "ordering": ordering
                }
            ))
            .append_pat(pat)
            .to_request();
        self.call(request).await
    }

    async fn get_versions(
        &self,
        version_ids: Vec<String>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let ids = url_encode_json_serialized_vec(&version_ids);
        let request = test::TestRequest::get()
            .uri(&format!("/v2/versions?ids={ids}"))
            .append_pat(pat)
            .to_request();
        self.call(request).await
    }

    async fn get_versions_deserialized_common(
        &self,
        version_ids: Vec<String>,
        pat: Option<&str>,
    ) -> Vec<CommonVersion> {
        let resp = self.get_versions(version_ids, pat).await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: Vec<LegacyVersion> = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn upload_file_to_version(
        &self,
        version_id: &str,
        file: &TestFile,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let m = request_data::get_public_creation_data_multipart(
            &json!({
                "file_parts": [file.filename()]
            }),
            Some(file),
        );
        let request = test::TestRequest::post()
            .uri(&format!("/v2/version/{version_id}/file"))
            .append_pat(pat)
            .set_multipart(m)
            .to_request();
        self.call(request).await
    }

    async fn remove_version(
        &self,
        version_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let request = test::TestRequest::delete()
            .uri(&format!("/v2/version/{version_id}"))
            .append_pat(pat)
            .to_request();
        self.call(request).await
    }

    async fn remove_version_file(
        &self,
        hash: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let request = test::TestRequest::delete()
            .uri(&format!("/v2/version_file/{hash}"))
            .append_pat(pat)
            .to_request();
        self.call(request).await
    }
}

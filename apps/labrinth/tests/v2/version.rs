use crate::assert_status;
use crate::common::api_common::{ApiProject, ApiVersion};
use crate::common::api_v2::ApiV2;
use actix_http::StatusCode;
use actix_web::test;
use futures::StreamExt;
use labrinth::models::ids::VersionId;
use labrinth::{
    models::projects::{Loader, VersionStatus, VersionType},
    routes::v2::version_file::FileUpdateData,
};
use serde_json::json;

use crate::common::api_v2::request_data::get_public_project_creation_data;
use crate::common::dummy_data::{DummyProjectAlpha, DummyProjectBeta};
use crate::common::environment::{TestEnvironment, with_test_environment};
use crate::common::{
    database::{ENEMY_USER_PAT, USER_USER_PAT},
    dummy_data::TestFile,
};

#[actix_rt::test]
pub async fn test_patch_version() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
            let api = &test_env.api;

            let alpha_version_id = &test_env.dummy.project_alpha.version_id;

            // // First, we do some patch requests that should fail.
            // // Failure because the user is not authorized.
            let resp = api
                .edit_version(
                    alpha_version_id,
                    json!({
                        "name": "test 1",
                    }),
                    ENEMY_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::UNAUTHORIZED);

            // Failure because these are illegal requested statuses for a normal user.
            for req in ["unknown", "scheduled"] {
                let resp = api
                    .edit_version(
                        alpha_version_id,
                        json!({
                            "status": req,
                            // requested status it not set here, but in /schedule
                        }),
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::BAD_REQUEST);
            }

            // Sucessful request to patch many fields.
            let resp = api
                .edit_version(
                    alpha_version_id,
                    json!({
                        "name": "new version name",
                        "version_number": "1.3.0",
                        "changelog": "new changelog",
                        "version_type": "beta",
                        // // "dependencies": [], TODO: test this
                        "game_versions": ["1.20.5"],
                        "loaders": ["forge"],
                        "featured": false,
                        // "primary_file": [], TODO: test this
                        // // "downloads": 0, TODO: moderator exclusive
                        "status": "draft",
                        // // "filetypes": ["jar"], TODO: test this
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let version = api
                .get_version_deserialized(alpha_version_id, USER_USER_PAT)
                .await;
            assert_eq!(version.name, "new version name");
            assert_eq!(version.version_number, "1.3.0");
            assert_eq!(version.changelog, "new changelog");
            assert_eq!(
                version.version_type,
                serde_json::from_str::<VersionType>("\"beta\"").unwrap()
            );
            assert_eq!(version.game_versions, vec!["1.20.5"]);
            assert_eq!(version.loaders, vec![Loader("forge".to_string())]);
            assert!(!version.featured);
            assert_eq!(version.status, VersionStatus::from_string("draft"));

            // These ones are checking the v2-v3 rerouting, we eneusre that only 'game_versions'
            // works as expected, as well as only 'loaders'
            let resp = api
                .edit_version(
                    alpha_version_id,
                    json!({
                        "game_versions": ["1.20.1", "1.20.2", "1.20.4"],
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let version = api
                .get_version_deserialized(alpha_version_id, USER_USER_PAT)
                .await;
            assert_eq!(
                version.game_versions,
                vec!["1.20.1", "1.20.2", "1.20.4"]
            );
            assert_eq!(version.loaders, vec![Loader("forge".to_string())]); // From last patch

            let resp = api
                .edit_version(
                    alpha_version_id,
                    json!({
                        "loaders": ["fabric"],
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let version = api
                .get_version_deserialized(alpha_version_id, USER_USER_PAT)
                .await;
            assert_eq!(
                version.game_versions,
                vec!["1.20.1", "1.20.2", "1.20.4"]
            ); // From last patch
            assert_eq!(version.loaders, vec![Loader("fabric".to_string())]);
        },
    )
    .await;
}

#[actix_rt::test]
async fn version_updates() {
    // Test setup and dummy data
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
            let api = &test_env.api;
            let DummyProjectAlpha {
                project_id: alpha_project_id,
                project_id_parsed: alpha_project_id_parsed,
                version_id: alpha_version_id,
                file_hash: alpha_version_hash,
                ..
            } = &test_env.dummy.project_alpha;
            let DummyProjectBeta {
                version_id: beta_version_id,
                file_hash: beta_version_hash,
                ..
            } = &test_env.dummy.project_beta;

            // Quick test, using get version from hash
            let version = api
                .get_version_from_hash_deserialized(
                    alpha_version_hash,
                    "sha1",
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(&version.id.to_string(), alpha_version_id);

            // Get versions from hash
            let versions = api
                .get_versions_from_hashes_deserialized(
                    &[alpha_version_hash.as_str(), beta_version_hash.as_str()],
                    "sha1",
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(versions.len(), 2);
            assert_eq!(
                &versions[alpha_version_hash].id.to_string(),
                alpha_version_id
            );
            assert_eq!(
                &versions[beta_version_hash].id.to_string(),
                beta_version_id
            );

            // When there is only the one version, there should be no updates
            let version = api
                .get_update_from_hash_deserialized_common(
                    alpha_version_hash,
                    "sha1",
                    None,
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(&version.id.to_string(), alpha_version_id);

            let versions = api
                .update_files_deserialized_common(
                    "sha1",
                    vec![alpha_version_hash.to_string()],
                    None,
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(versions.len(), 1);
            assert_eq!(
                &versions[alpha_version_hash].id.to_string(),
                alpha_version_id
            );

            // Add 3 new versions, 1 before, and 2 after, with differing game_version/version_types/loaders
            let mut update_ids = vec![];
            for (version_number, patch_value) in [
                (
                    "0.9.9",
                    json!({
                        "game_versions": ["1.20.1"],
                    }),
                ),
                (
                    "1.5.0",
                    json!({
                        "game_versions": ["1.20.3"],
                        "loaders": ["fabric"],
                    }),
                ),
                (
                    "1.5.1",
                    json!({
                        "game_versions": ["1.20.4"],
                        "loaders": ["forge"],
                        "version_type": "beta"
                    }),
                ),
            ]
            .iter()
            {
                let version = api
                    .add_public_version_deserialized_common(
                        *alpha_project_id_parsed,
                        version_number,
                        TestFile::build_random_jar(),
                        None,
                        None,
                        USER_USER_PAT,
                    )
                    .await;
                update_ids.push(version.id);

                // Patch using json
                api.edit_version(
                    &version.id.to_string(),
                    patch_value.clone(),
                    USER_USER_PAT,
                )
                .await;
            }

            let check_expected =
                |game_versions: Option<Vec<String>>,
                 loaders: Option<Vec<String>>,
                 version_types: Option<Vec<String>>,
                 result_id: Option<VersionId>| async move {
                    let (success, result_id) = match result_id {
                        Some(id) => (true, id),
                        None => (false, VersionId(0)),
                    };
                    // get_update_from_hash
                    let resp = api
                        .get_update_from_hash(
                            alpha_version_hash,
                            "sha1",
                            loaders.clone(),
                            game_versions.clone(),
                            version_types.clone(),
                            USER_USER_PAT,
                        )
                        .await;
                    if success {
                        assert_status!(&resp, StatusCode::OK);
                        let body: serde_json::Value =
                            test::read_body_json(resp).await;
                        let id = body["id"].as_str().unwrap();
                        assert_eq!(id, &result_id.to_string());
                    } else {
                        assert_status!(&resp, StatusCode::NOT_FOUND);
                    }

                    // update_files
                    let versions = api
                        .update_files_deserialized_common(
                            "sha1",
                            vec![alpha_version_hash.to_string()],
                            loaders.clone(),
                            game_versions.clone(),
                            version_types.clone(),
                            USER_USER_PAT,
                        )
                        .await;
                    if success {
                        assert_eq!(versions.len(), 1);
                        let first = versions.iter().next().unwrap();
                        assert_eq!(first.1.id, result_id);
                    } else {
                        assert_eq!(versions.len(), 0);
                    }

                    // update_individual_files
                    let hashes = vec![FileUpdateData {
                        hash: alpha_version_hash.to_string(),
                        loaders,
                        game_versions,
                        version_types: version_types.map(|v| {
                            v.into_iter()
                                .map(|v| {
                                    serde_json::from_str(&format!("\"{v}\""))
                                        .unwrap()
                                })
                                .collect()
                        }),
                    }];
                    let versions = api
                        .update_individual_files_deserialized(
                            "sha1",
                            hashes,
                            USER_USER_PAT,
                        )
                        .await;
                    if success {
                        assert_eq!(versions.len(), 1);
                        let first = versions.iter().next().unwrap();
                        assert_eq!(first.1.id, result_id);
                    } else {
                        assert_eq!(versions.len(), 0);
                    }
                };

            let tests = vec![
                check_expected(
                    Some(vec!["1.20.1".to_string()]),
                    None,
                    None,
                    Some(update_ids[0]),
                ),
                check_expected(
                    Some(vec!["1.20.3".to_string()]),
                    None,
                    None,
                    Some(update_ids[1]),
                ),
                check_expected(
                    Some(vec!["1.20.4".to_string()]),
                    None,
                    None,
                    Some(update_ids[2]),
                ),
                // Loader restrictions
                check_expected(
                    None,
                    Some(vec!["fabric".to_string()]),
                    None,
                    Some(update_ids[1]),
                ),
                check_expected(
                    None,
                    Some(vec!["forge".to_string()]),
                    None,
                    Some(update_ids[2]),
                ),
                // Version type restrictions
                check_expected(
                    None,
                    None,
                    Some(vec!["release".to_string()]),
                    Some(update_ids[1]),
                ),
                check_expected(
                    None,
                    None,
                    Some(vec!["beta".to_string()]),
                    Some(update_ids[2]),
                ),
                // Specific combination
                check_expected(
                    None,
                    Some(vec!["fabric".to_string()]),
                    Some(vec!["release".to_string()]),
                    Some(update_ids[1]),
                ),
                // Impossible combination
                check_expected(
                    None,
                    Some(vec!["fabric".to_string()]),
                    Some(vec!["beta".to_string()]),
                    None,
                ),
                // No restrictions, should do the last one
                check_expected(None, None, None, Some(update_ids[2])),
            ];

            // Wait on all tests, 4 at a time
            futures::stream::iter(tests)
                .buffer_unordered(4)
                .collect::<Vec<_>>()
                .await;

            // We do a couple small tests for get_project_versions_deserialized as well
            // TODO: expand this more.
            let versions = api
                .get_project_versions_deserialized_common(
                    alpha_project_id,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(versions.len(), 4);
            let versions = api
                .get_project_versions_deserialized_common(
                    alpha_project_id,
                    None,
                    Some(vec!["forge".to_string()]),
                    None,
                    None,
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(versions.len(), 1);
        },
    )
    .await;
}

#[actix_rt::test]
async fn add_version_project_types_v2() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
            // Since v2 no longer keeps project_type at the project level but the version level,
            // we have to test that the project_type is set correctly when adding a version, if its done in separate requests.
            let api = &test_env.api;

            // Create a project in v2 with project_type = modpack, and no initial version set.
            let (test_project, test_versions) = api
                .add_public_project("test-modpack", None, None, USER_USER_PAT)
                .await;
            assert_eq!(test_versions.len(), 0); // No initial version set

            // Get as v2 project
            let test_project = api
                .get_project_deserialized(
                    &test_project.slug.unwrap(),
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(test_project.project_type, "project"); // No project_type set, as no versions are set
            // Default to 'project' if none are found
            // This is a known difference between older v2 ,but is acceptable.
            // This would be the appropriate test on older v2:
            // assert_eq!(test_project.project_type, "modpack");

            // Create a version with a modpack file attached
            let test_version = api
                .add_public_version_deserialized_common(
                    test_project.id,
                    "1.0.0",
                    TestFile::build_random_mrpack(),
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;

            // When we get the version as v2, it should display 'fabric' as the loader (and no project_type)
            let test_version = api
                .get_version_deserialized(
                    &test_version.id.to_string(),
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(
                test_version.loaders,
                vec![Loader("fabric".to_string())]
            );

            // When we get the project as v2, it should display 'modpack' as the project_type, and 'fabric' as the loader
            let test_project = api
                .get_project_deserialized(
                    &test_project.slug.unwrap(),
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(test_project.project_type, "modpack");
            assert_eq!(test_project.loaders, vec!["fabric"]);

            // When we get the version as v3, it should display 'mrpack' as the loader, and 'modpack' as the project_type
            // When we get the project as v3, it should display 'modpack' as the project_type, and 'mrpack' as the loader

            // The project should be a modpack project
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_incorrect_file_parts() {
    // Ensures that a version get that 'should' have mrpack_loaders does still display them
    //   if the file is 'mrpack' but the file_parts are incorrect
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
            let api = &test_env.api;

            // Patch to set the file_parts to something incorrect
            let patch = json!([{
                "op": "add",
                "path": "/file_parts",
                "value": ["invalid.zip"] // one file, wrong non-mrpack extension
            }]);

            // Create an empty project
            let slug = "test-project";
            let creation_data =
                get_public_project_creation_data(slug, None, None);
            let resp = api.create_project(creation_data, USER_USER_PAT).await;
            assert_status!(&resp, StatusCode::OK);

            // Get the project
            let project =
                api.get_project_deserialized(slug, USER_USER_PAT).await;
            assert_eq!(project.project_type, "project");

            // Create a version with a mrpack file, but incorrect file_parts
            let resp = api
                .add_public_version(
                    project.id,
                    "1.0.0",
                    TestFile::build_random_mrpack(),
                    None,
                    Some(serde_json::from_value(patch).unwrap()),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::OK);

            // Get the project now, which should be now correctly identified as a modpack
            let project =
                api.get_project_deserialized(slug, USER_USER_PAT).await;
            assert_eq!(project.project_type, "modpack");
            assert_eq!(project.loaders, vec!["fabric"]);
        },
    )
    .await;
}

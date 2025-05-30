use std::collections::HashSet;

use actix_http::StatusCode;
use actix_web::test;
use common::api_v3::ApiV3;
use common::environment::{TestEnvironment, with_test_environment};
use itertools::Itertools;
use labrinth::database::models::legacy_loader_fields::MinecraftGameVersion;
use labrinth::models::v3;
use serde_json::json;

use crate::common::api_common::{ApiProject, ApiVersion};
use crate::common::api_v3::request_data::get_public_project_creation_data;
use crate::common::database::*;

use crate::common::dummy_data::{
    DummyProjectAlpha, DummyProjectBeta, TestFile,
};

// importing common module.
mod common;

#[actix_rt::test]

async fn creating_loader_fields() {
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        let api = &test_env.api;
        let DummyProjectAlpha {
            project_id: alpha_project_id,
            project_id_parsed: alpha_project_id_parsed,
            version_id: alpha_version_id,
            ..
        } = &test_env.dummy.project_alpha;
        let DummyProjectBeta {
            project_id_parsed: beta_project_id_parsed,
            ..
        } = &test_env.dummy.project_beta;

        // ALL THE FOLLOWING FOR CREATE AND PATCH
        // Cannot create a version with an extra argument that cannot be tied to a loader field ("invalid loader field")
        // TODO: - Create project
        // - Create version
        let resp = api
            .add_public_version(
                *alpha_project_id_parsed,
                "1.0.0",
                TestFile::build_random_jar(),
                None,
                Some(
                    serde_json::from_value(json!([{
                            "op": "add",
                            "path": "/invalid",
                            "value": "invalid"
                    }]))
                    .unwrap(),
                ),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);
        // - Patch
        let resp = api
            .edit_version(
                alpha_version_id,
                json!({
                    "invalid": "invalid"
                }),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // Cannot create a version with a loader field that isnt used by the loader
        // TODO: - Create project
        // - Create version
        let resp = api
            .add_public_version(
                *alpha_project_id_parsed,
                "1.0.0",
                TestFile::build_random_jar(),
                None,
                Some(
                    serde_json::from_value(json!([{
                        "op": "add",
                        "path": "/mrpack_loaders",
                        "value": ["fabric"]
                    }]))
                    .unwrap(),
                ),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);
        // - Patch
        let resp = api
            .edit_version(
                alpha_version_id,
                json!({
                    "mrpack_loaders": ["fabric"]
                }),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // Cannot create a version without an applicable loader field that is not optional
        // TODO: - Create project
        // - Create version
        let resp = api
            .add_public_version(
                *alpha_project_id_parsed,
                "1.0.0",
                TestFile::build_random_jar(),
                None,
                Some(
                    serde_json::from_value(json!([{
                        "op": "remove",
                        "path": "/singleplayer"
                    }]))
                    .unwrap(),
                ),
                USER_USER_PAT,
            )
            .await;

        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // Cannot create a version without a loader field array that has a minimum of 1
        // TODO: - Create project
        // - Create version
        let resp = api
            .add_public_version(
                *alpha_project_id_parsed,
                "1.0.0",
                TestFile::build_random_jar(),
                None,
                Some(
                    serde_json::from_value(json!([{
                        "op": "remove",
                        "path": "/game_versions"
                    }]))
                    .unwrap(),
                ),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // TODO: Create a test for too many elements in the array when we have a LF that has a max (past max)
        // Cannot create a version with a loader field array that has fewer than the minimum elements
        // TODO: - Create project
        // - Create version
        let resp: actix_web::dev::ServiceResponse = api
            .add_public_version(
                *alpha_project_id_parsed,
                "1.0.0",
                TestFile::build_random_jar(),
                None,
                Some(
                    serde_json::from_value(json!([{
                        "op": "add",
                        "path": "/game_versions",
                        "value": []
                    }]))
                    .unwrap(),
                ),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // - Patch
        let resp = api
            .edit_version(
                alpha_version_id,
                json!({
                    "game_versions": []
                }),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // Cannot create an invalid data type for the loader field type (including bad variant for the type)
        for bad_type_game_versions in [
            json!(1),
            json!([1]),
            json!("1.20.1"),
            json!(["singleplayer"]),
        ] {
            // TODO: - Create project
            // - Create version
            let resp = api
                .add_public_version(
                    *alpha_project_id_parsed,
                    "1.0.0",
                    TestFile::build_random_jar(),
                    None,
                    Some(
                        serde_json::from_value(json!([{
                            "op": "add",
                            "path": "/game_versions",
                            "value": bad_type_game_versions
                        }]))
                        .unwrap(),
                    ),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);

            // - Patch
            let resp = api
                .edit_version(
                    alpha_version_id,
                    json!({
                        "game_versions": bad_type_game_versions
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);
        }

        // Can create with optional loader fields (other tests have checked if we can create without them)
        // TODO: - Create project
        // - Create version
        let v = api
            .add_public_version_deserialized(
                *alpha_project_id_parsed,
                "1.0.0",
                TestFile::build_random_jar(),
                None,
                Some(
                    serde_json::from_value(json!([{
                        "op": "add",
                        "path": "/test_fabric_optional",
                        "value": 555
                    }]))
                    .unwrap(),
                ),
                USER_USER_PAT,
            )
            .await;
        assert_eq!(v.fields.get("test_fabric_optional").unwrap(), &json!(555));
        // - Patch
        let resp = api
            .edit_version(
                alpha_version_id,
                json!({
                    "test_fabric_optional": 555
                }),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);
        let v = api
            .get_version_deserialized(alpha_version_id, USER_USER_PAT)
            .await;
        assert_eq!(v.fields.get("test_fabric_optional").unwrap(), &json!(555));

        // Simply setting them as expected works
        // - Create
        let v = api
            .add_public_version_deserialized(
                *alpha_project_id_parsed,
                "1.0.0",
                TestFile::build_random_jar(),
                None,
                Some(
                    serde_json::from_value(json!([{
                        "op": "add",
                        "path": "/game_versions",
                        "value": ["1.20.1", "1.20.2"]
                    }, {
                        "op": "add",
                        "path": "/singleplayer",
                        "value": false
                    }, {
                        "op": "add",
                        "path": "/server_only",
                        "value": true
                    }]))
                    .unwrap(),
                ),
                USER_USER_PAT,
            )
            .await;
        assert_eq!(
            v.fields.get("game_versions").unwrap(),
            &json!(["1.20.1", "1.20.2"])
        );
        assert_eq!(v.fields.get("singleplayer").unwrap(), &json!(false));
        assert_eq!(v.fields.get("server_only").unwrap(), &json!(true));
        // - Patch
        let resp = api
            .edit_version(
                alpha_version_id,
                json!({
                    "game_versions": ["1.20.1", "1.20.2"],
                    "singleplayer": false,
                    "server_only": true
                }),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);
        let v = api
            .get_version_deserialized(alpha_version_id, USER_USER_PAT)
            .await;
        assert_eq!(
            v.fields.get("game_versions").unwrap(),
            &json!(["1.20.1", "1.20.2"])
        );

        // Now that we've created a version, we need to make sure that the Project's loader fields are updated (aggregate)
        // First, add a new version
        api.add_public_version_deserialized(
            *alpha_project_id_parsed,
            "1.0.1",
            TestFile::build_random_jar(),
            None,
            Some(
                serde_json::from_value(json!([{
                    "op": "add",
                    "path": "/game_versions",
                    "value": ["1.20.5"]
                }, {
                    "op": "add",
                    "path": "/singleplayer",
                    "value": false
                }]))
                .unwrap(),
            ),
            USER_USER_PAT,
        )
        .await;

        // Also, add one to the beta project
        api.add_public_version_deserialized(
            *beta_project_id_parsed,
            "1.0.1",
            TestFile::build_random_jar(),
            None,
            Some(
                serde_json::from_value(json!([{
                    "op": "add",
                    "path": "/game_versions",
                    "value": ["1.20.4"]
                }]))
                .unwrap(),
            ),
            USER_USER_PAT,
        )
        .await;

        let project = api
            .get_project_deserialized(
                &alpha_project_id.to_string(),
                USER_USER_PAT,
            )
            .await;
        assert_eq!(
            project.fields.get("game_versions").unwrap(),
            &[json!("1.20.1"), json!("1.20.2"), json!("1.20.5")]
        );
        assert!(
            project
                .fields
                .get("singleplayer")
                .unwrap()
                .contains(&json!(false))
        );
        assert!(
            project
                .fields
                .get("singleplayer")
                .unwrap()
                .contains(&json!(true))
        );
    })
    .await
}

#[actix_rt::test]
async fn get_loader_fields_variants() {
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        let api = &test_env.api;

        let game_versions = api
            .get_loader_field_variants_deserialized("game_versions")
            .await;

        // These tests match dummy data and will need to be updated if the dummy data changes
        // Versions should be ordered by:
        // - ordering
        // - ordering ties settled by date added to database
        // - We also expect presentation of NEWEST to OLDEST
        // - All null orderings are treated as older than any non-null ordering
        // (for this test, the 1.20.1, etc, versions are all null ordering)
        let game_version_versions = game_versions
            .into_iter()
            .map(|x| x.value)
            .collect::<Vec<_>>();
        assert_eq!(
            game_version_versions,
            [
                "Ordering_Negative1",
                "Ordering_Positive100",
                "1.20.5",
                "1.20.4",
                "1.20.3",
                "1.20.2",
                "1.20.1"
            ]
        );
    })
    .await
}

#[actix_rt::test]
async fn get_available_loader_fields() {
    // Get available loader fields for a given loader
    // (ie: which fields are relevant for 'fabric', etc)
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let loaders = api.get_loaders_deserialized().await;

            let fabric_loader_fields = loaders
                .iter()
                .find(|x| x.name == "fabric")
                .unwrap()
                .supported_fields
                .clone()
                .into_iter()
                .collect::<HashSet<_>>();
            assert_eq!(
                fabric_loader_fields,
                [
                    "game_versions",
                    "singleplayer",
                    "client_and_server",
                    "client_only",
                    "server_only",
                    "test_fabric_optional" // exists for testing
                ]
                .iter()
                .map(|s| s.to_string())
                .collect()
            );

            let mrpack_loader_fields = loaders
                .iter()
                .find(|x| x.name == "mrpack")
                .unwrap()
                .supported_fields
                .clone()
                .into_iter()
                .collect::<HashSet<_>>();
            assert_eq!(
                mrpack_loader_fields,
                [
                    "game_versions",
                    "singleplayer",
                    "client_and_server",
                    "client_only",
                    "server_only",
                    // mrpack has all the general fields as well as this
                    "mrpack_loaders"
                ]
                .iter()
                .map(|s| s.to_string())
                .collect()
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_multi_get_redis_cache() {
    // Ensures a multi-project get including both modpacks and mods ddoes not
    // incorrectly cache loader fields
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;

            // Create 5 modpacks
            let mut modpacks = Vec::new();
            for i in 0..5 {
                let slug = format!("test-modpack-{i}");

                let creation_data = get_public_project_creation_data(
                    &slug,
                    Some(TestFile::build_random_mrpack()),
                    None,
                );
                let resp =
                    api.create_project(creation_data, USER_USER_PAT).await;
                assert_status!(&resp, StatusCode::OK);
                modpacks.push(slug);
            }

            // Create 5 mods
            let mut mods = Vec::new();
            for i in 0..5 {
                let slug = format!("test-mod-{i}");

                let creation_data = get_public_project_creation_data(
                    &slug,
                    Some(TestFile::build_random_jar()),
                    None,
                );
                let resp =
                    api.create_project(creation_data, USER_USER_PAT).await;
                assert_status!(&resp, StatusCode::OK);
                mods.push(slug);
            }

            // Get all 10 projects
            let project_slugs = modpacks
                .iter()
                .map(|x| x.as_str())
                .chain(mods.iter().map(|x| x.as_str()))
                .collect_vec();
            let resp = api.get_projects(&project_slugs, USER_USER_PAT).await;
            assert_status!(&resp, StatusCode::OK);
            let projects: Vec<v3::projects::Project> =
                test::read_body_json(resp).await;
            assert_eq!(projects.len(), 10);

            // Ensure all 5 modpacks have 'mrpack_loaders', and all 5 mods do not
            for project in projects.iter() {
                if modpacks.contains(project.slug.as_ref().unwrap()) {
                    assert!(project.fields.contains_key("mrpack_loaders"));
                } else if mods.contains(project.slug.as_ref().unwrap()) {
                    assert!(!project.fields.contains_key("mrpack_loaders"));
                } else {
                    panic!("Unexpected project slug: {:?}", project.slug);
                }
            }

            // Get a version from each project
            let version_ids_modpacks = projects
                .iter()
                .filter(|x| modpacks.contains(x.slug.as_ref().unwrap()))
                .map(|x| x.versions[0])
                .collect_vec();
            let version_ids_mods = projects
                .iter()
                .filter(|x| mods.contains(x.slug.as_ref().unwrap()))
                .map(|x| x.versions[0])
                .collect_vec();
            let version_ids = version_ids_modpacks
                .iter()
                .chain(version_ids_mods.iter())
                .map(|x| x.to_string())
                .collect_vec();
            let resp = api.get_versions(version_ids, USER_USER_PAT).await;
            assert_status!(&resp, StatusCode::OK);
            let versions: Vec<v3::projects::Version> =
                test::read_body_json(resp).await;
            assert_eq!(versions.len(), 10);

            // Ensure all 5 versions from modpacks have 'mrpack_loaders', and all 5 versions from mods do not
            for version in versions.iter() {
                if version_ids_modpacks.contains(&version.id) {
                    assert!(version.fields.contains_key("mrpack_loaders"));
                } else if version_ids_mods.contains(&version.id) {
                    assert!(!version.fields.contains_key("mrpack_loaders"));
                } else {
                    panic!("Unexpected version id: {:?}", version.id);
                }
            }
        },
    )
    .await;
}

#[actix_rt::test]
async fn minecraft_game_version_update() {
    // We simulate adding a Minecraft game version, to ensure other data doesn't get overwritten
    // This is basically a test for the insertion/concatenation query
    // This doesn't use a route (as this behaviour isn't exposed via a route, but a scheduled URL call)
    // We just interact with the labrinth functions directly
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        let api = &test_env.api;

        // First, get a list of all gameversions
        let game_versions = api
            .get_loader_field_variants_deserialized("game_versions")
            .await;

        // A couple specific checks- in the dummy data, all game versions are marked as major=false except 1.20.5
        let name_to_major = game_versions
            .iter()
            .map(|x| {
                (
                    x.value.clone(),
                    x.metadata.get("major").unwrap().as_bool().unwrap(),
                )
            })
            .collect::<std::collections::HashMap<_, _>>();
        for (name, major) in name_to_major {
            if name == "1.20.5" {
                assert!(major);
            } else {
                assert!(!major);
            }
        }

        // Now, we add a new game version, directly to the db
        let pool = test_env.db.pool.clone();
        let redis = test_env.db.redis_pool.clone();
        MinecraftGameVersion::builder()
            .version("1.20.6")
            .unwrap()
            .version_type("release")
            .unwrap()
            .created(
                // now
                &chrono::Utc::now(),
            )
            .insert(&pool, &redis)
            .await
            .unwrap();

        // Check again
        let game_versions = api
            .get_loader_field_variants_deserialized("game_versions")
            .await;

        let name_to_major = game_versions
            .iter()
            .map(|x| {
                (
                    x.value.clone(),
                    x.metadata.get("major").unwrap().as_bool().unwrap(),
                )
            })
            .collect::<std::collections::HashMap<_, _>>();
        // Confirm that the new version is there
        assert!(name_to_major.contains_key("1.20.6"));
        // Confirm metadata is unaltered
        for (name, major) in name_to_major {
            if name == "1.20.5" {
                assert!(major);
            } else {
                assert!(!major);
            }
        }
    })
    .await
}

use std::collections::HashSet;

use actix_http::StatusCode;
use actix_web::test;
use ariadne::ids::base62_impl::parse_base62;
use chrono::Utc;
use common::api_v3::ApiV3;
use common::environment::{TestEnvironment, with_test_environment};
use itertools::Itertools;
use labrinth::database::models::{
    DBUserId, DBVersionId,
    user_limits::DBUserLimits,
    version_item::{VERSIONS_NAMESPACE, VersionQueryResult},
};
use labrinth::models::v3;
use serde_json::json;
use xredis::RedisValue;

use crate::common::api_common::{ApiProject, ApiVersion};
use crate::common::api_v3::request_data::get_public_project_creation_data;
use crate::common::database::*;

use crate::common::dummy_data::{
    DummyProjectAlpha, DummyProjectBeta, TestFile,
};

pub mod common;

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

        // Environment is optional for Minecraft versions
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
                        "path": "/environment"
                    }]))
                    .unwrap(),
                ),
                USER_USER_PAT,
            )
            .await;

        assert_status!(&resp, StatusCode::OK);

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

        // Removed arbitrary loader fields are rejected on create and patch
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
                        "path": "/test_fabric_optional",
                        "value": 555
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
                    "test_fabric_optional": 555
                }),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);
        let v = api
            .get_version_deserialized(alpha_version_id, USER_USER_PAT)
            .await;
        assert!(!v.fields.contains_key("test_fabric_optional"));

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
                        "path": "/environment",
                        "value": "client_or_server_prefers_both"
                    }]))
                    .unwrap(),
                ),
                USER_USER_PAT,
            )
            .await;
        assert_eq!(&v.fields["game_versions"], &json!(["1.20.1", "1.20.2"]));
        assert_eq!(
            &v.fields["environment"],
            &json!("client_or_server_prefers_both")
        );
        // - Patch
        let resp = api
            .edit_version(
                alpha_version_id,
                json!({
                    "game_versions": ["1.20.1", "1.20.2"],
                    "environment": "client_or_server_prefers_both"
                }),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);
        let v = api
            .get_version_deserialized(alpha_version_id, USER_USER_PAT)
            .await;
        assert_eq!(&v.fields["game_versions"], &json!(["1.20.1", "1.20.2"]));

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
                    "path": "/environment",
                    "value": "client_or_server"
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
            project.fields["game_versions"]
                .iter()
                .collect::<HashSet<_>>(),
            [json!("1.20.1"), json!("1.20.2"), json!("1.20.5")]
                .iter()
                .collect::<HashSet<_>>()
        );
        assert!(
            project.fields["environment"].contains(&json!("client_or_server"))
        );
        assert!(
            project.fields["environment"]
                .contains(&json!("client_or_server_prefers_both"))
        );
    })
    .await
}

#[actix_rt::test]
async fn version_edit_uses_submitted_loaders_for_field_validation() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let version_id = &test_env.dummy.project_alpha.version_id;

            let response = api
                .edit_version(
                    version_id,
                    json!({
                        "loaders": ["bukkit"]
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);

            let response = api
                .edit_version(
                    version_id,
                    json!({
                        "loaders": ["mrpack"],
                        "game_versions": ["1.20.1"],
                        "environment": "client_only",
                        "mrpack_loaders": ["fabric"]
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);

            let version = api
                .get_version_deserialized(version_id, USER_USER_PAT)
                .await;
            assert_eq!(version.loaders.len(), 1);
            assert_eq!(version.loaders[0].0, "mrpack");
            assert_eq!(version.fields["game_versions"], json!(["1.20.1"]));
            assert_eq!(version.fields["environment"], json!("client_only"));
            assert_eq!(version.fields["mrpack_loaders"], json!(["fabric"]));
        },
    )
    .await;
}

#[actix_rt::test]
async fn version_edit_uses_authoritative_loaders_for_field_validation() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let version_id = &test_env.dummy.project_alpha.version_id;
            api.get_version_deserialized(version_id, USER_USER_PAT)
                .await;

            let parsed_version_id = parse_base62(version_id).unwrap();
            let mut redis = test_env.db.redis_pool.connect().await.unwrap();
            let version_key =
                redis.key().entity(VERSIONS_NAMESPACE, parsed_version_id);
            let cached_version: RedisValue<
                VersionQueryResult,
                DBVersionId,
                String,
            > = redis.get_deserialized(&version_key).await.unwrap().unwrap();
            let mut stale_version = cached_version.value().clone();
            stale_version.loaders = vec!["mrpack".to_string()];
            redis
                .set_serialized(
                    &version_key,
                    &RedisValue::new(
                        DBVersionId(parsed_version_id as i64),
                        None::<String>,
                        Utc::now().timestamp(),
                        stale_version,
                    ),
                    None,
                )
                .await
                .unwrap();
            drop(redis);

            let response = api
                .edit_version(
                    version_id,
                    json!({
                        "mrpack_loaders": ["fabric"]
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::BAD_REQUEST);

            let body: serde_json::Value = test::read_body_json(response).await;
            assert_eq!(
                body["description"],
                "`mrpack_loaders` requires the `mrpack` loader"
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn version_edit_rejects_incompatible_loader_fields() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let version_id = &test_env.dummy.project_alpha.version_id;

            let response = api
                .edit_version(
                    version_id,
                    json!({
                        "loaders": ["bukkit"]
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);

            let response = api
                .edit_version(
                    version_id,
                    json!({
                        "mrpack_loaders": ["fabric"],
                        "environment": "client_only"
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::BAD_REQUEST);
            let body: serde_json::Value = test::read_body_json(response).await;
            assert_eq!(body["error"], "request_error");
            assert_eq!(
                body["description"],
                "`mrpack_loaders` requires the `mrpack` loader"
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn get_loader_fields_variants() {
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        let api = &test_env.api;

        let game_versions = api
            .get_loader_field_variants_deserialized("game_versions")
            .await;

        // Minecraft versions are presented newest to oldest by release date.
        let game_version_versions = game_versions
            .into_iter()
            .map(|x| x.value)
            .collect::<Vec<_>>();
        assert_eq!(
            game_version_versions,
            [
                "1.20.5", "1.20.4", "1.20.3", "1.20.2", "1.20.1", "18w43b",
                "1.13.2", "1.3"
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
                ["game_versions", "environment"]
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
                    "environment",
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
            let defaults =
                DBUserLimits::get_defaults(&test_env.db.pool).await.unwrap();
            DBUserLimits {
                user_id: Some(DBUserId(USER_USER_ID_PARSED)),
                projects_per_day: 32,
                ..defaults
            }
            .upsert(&test_env.db.pool)
            .await
            .unwrap();

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
            for project in &projects {
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
            for version in &versions {
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

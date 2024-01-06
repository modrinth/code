use std::collections::HashSet;

use actix_http::StatusCode;
use common::api_v3::ApiV3;
use common::environment::{with_test_environment, TestEnvironment};
use serde_json::json;

use crate::common::api_common::ApiVersion;
use crate::common::database::*;

use crate::common::dummy_data::{DummyProjectAlpha, DummyProjectBeta, TestFile};

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
            .get_project_deserialized(&alpha_project_id.to_string(), USER_USER_PAT)
            .await;
        assert_eq!(
            project.fields.get("game_versions").unwrap(),
            &[json!("1.20.1"), json!("1.20.2"), json!("1.20.5")]
        );
        assert!(project
            .fields
            .get("singleplayer")
            .unwrap()
            .contains(&json!(false)));
        assert!(project
            .fields
            .get("singleplayer")
            .unwrap()
            .contains(&json!(true)));
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
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
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
    })
    .await;
}

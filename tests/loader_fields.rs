use std::collections::HashSet;

use common::api_v3::ApiV3;
use common::environment::{with_test_environment, TestEnvironment};
use serde_json::json;

use crate::common::api_common::ApiVersion;
use crate::common::database::*;

use crate::common::dummy_data::TestFile;

// importing common module.
mod common;

#[actix_rt::test]
async fn creating_loader_fields() {
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        let api = &test_env.api;

        let alpha_project_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_alpha
            .project_id
            .clone();
        let alpha_project_id = serde_json::from_str(&format!("\"{}\"", alpha_project_id)).unwrap();
        let alpha_version_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_alpha
            .version_id
            .clone();

        // ALL THE FOLLOWING FOR CREATE AND PATCH
        // Cannot create a version with an extra argument that cannot be tied to a loader field ("invalid loader field")
        // TODO: - Create project
        // - Create version
        let resp = api
            .add_public_version(
                alpha_project_id,
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
        assert_eq!(resp.status(), 400);
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
        assert_eq!(resp.status(), 400);

        // Cannot create a version with a loader field that isnt used by the loader
        // TODO: - Create project
        // - Create version
        let resp = api
            .add_public_version(
                alpha_project_id,
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
        assert_eq!(resp.status(), 400);
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
        assert_eq!(resp.status(), 400);

        // Cannot create a version without an applicable loader field that is not optional
        // TODO: - Create project
        // - Create version
        let resp = api
            .add_public_version(
                alpha_project_id,
                "1.0.0",
                TestFile::build_random_jar(),
                None,
                Some(
                    serde_json::from_value(json!([{
                        "op": "remove",
                        "path": "/client_side"
                    }]))
                    .unwrap(),
                ),
                USER_USER_PAT,
            )
            .await;

        assert_eq!(resp.status(), 400);

        // Cannot create a version without a loader field array that has a minimum of 1
        // TODO: - Create project
        // - Create version
        let resp = api
            .add_public_version(
                alpha_project_id,
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
        assert_eq!(resp.status(), 400);

        // TODO: Create a test for too many elements in the array when we have a LF that has a max (past max)
        // Cannot create a version with a loader field array that has fewer than the minimum elements
        // TODO: - Create project
        // - Create version
        let resp: actix_web::dev::ServiceResponse = api
            .add_public_version(
                alpha_project_id,
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
        assert_eq!(resp.status(), 400);

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
        assert_eq!(resp.status(), 400);

        // Cannot create an invalid data type for the loader field type (including bad variant for the type)
        for bad_type_game_versions in [
            json!(1),
            json!([1]),
            json!("1.20.1"),
            json!(["client_side"]),
        ] {
            // TODO: - Create project
            // - Create version
            let resp = api
                .add_public_version(
                    alpha_project_id,
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
            assert_eq!(resp.status(), 400);

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
            assert_eq!(resp.status(), 400);
        }

        // Can create with optional loader fields (other tests have checked if we can create without them)
        // TODO: - Create project
        // - Create version
        let v = api
            .add_public_version_deserialized(
                alpha_project_id,
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
        assert_eq!(resp.status(), 204);
        let v = api
            .get_version_deserialized(alpha_version_id, USER_USER_PAT)
            .await;
        assert_eq!(v.fields.get("test_fabric_optional").unwrap(), &json!(555));

        // Simply setting them as expected works
        // - Create
        let v = api
            .add_public_version_deserialized(
                alpha_project_id,
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
                        "path": "/client_side",
                        "value": "optional"
                    }, {
                        "op": "add",
                        "path": "/server_side",
                        "value": "required"
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
        assert_eq!(v.fields.get("client_side").unwrap(), &json!("optional"));
        assert_eq!(v.fields.get("server_side").unwrap(), &json!("required"));
        // - Patch
        let resp = api
            .edit_version(
                alpha_version_id,
                json!({
                    "game_versions": ["1.20.1", "1.20.2"],
                    "client_side": "optional",
                    "server_side": "required"
                }),
                USER_USER_PAT,
            )
            .await;
        assert_eq!(resp.status(), 204);
        let v = api
            .get_version_deserialized(alpha_version_id, USER_USER_PAT)
            .await;
        assert_eq!(
            v.fields.get("game_versions").unwrap(),
            &json!(["1.20.1", "1.20.2"])
        );
    })
    .await
}

#[actix_rt::test]
async fn get_loader_fields() {
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        let api = &test_env.api;

        let game_versions = api
            .get_loader_field_variants_deserialized("game_versions")
            .await;
        let side_types = api
            .get_loader_field_variants_deserialized("client_side")
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

        let side_type_names = side_types
            .into_iter()
            .map(|x| x.value)
            .collect::<HashSet<_>>();
        assert_eq!(
            side_type_names,
            ["unknown", "required", "optional", "unsupported"]
                .iter()
                .map(|s| s.to_string())
                .collect()
        );
    })
    .await
}

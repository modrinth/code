use std::collections::HashSet;

use common::environment::TestEnvironment;
use serde_json::json;

use crate::common::api_v3::request_data::get_public_version_creation_data;
use crate::common::database::*;

use crate::common::dummy_data::TestFile;

// importing common module.
mod common;

#[actix_rt::test]
async fn creating_loader_fields() {
    let test_env = TestEnvironment::build(None).await;
    let api = &test_env.v3;

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
    let version_data = get_public_version_creation_data(
        alpha_project_id,
        "1.0.0",
        TestFile::build_random_jar(),
        Some(|j: &mut serde_json::Value| {
            j["invalid"] = json!("invalid");
        }),
    );
    let resp = api.add_public_version(version_data, USER_USER_PAT).await;
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
    let version_data = get_public_version_creation_data(
        alpha_project_id,
        "1.0.0",
        TestFile::build_random_jar(),
        Some(|j: &mut serde_json::Value| {
            // This is only for mrpacks, not mods/jars
            j["mrpack_loaders"] = json!(["fabric"]);
        }),
    );
    let resp = api.add_public_version(version_data, USER_USER_PAT).await;
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
    let version_data = get_public_version_creation_data(
        alpha_project_id,
        "1.0.0",
        TestFile::build_random_jar(),
        Some(|j: &mut serde_json::Value| {
            let j = j.as_object_mut().unwrap();
            j.remove("client_side");
        }),
    );
    let resp = api.add_public_version(version_data, USER_USER_PAT).await;
    assert_eq!(resp.status(), 400);

    // Cannot create a version without a loader field array that has a minimum of 1
    // TODO: - Create project
    // - Create version
    let version_data = get_public_version_creation_data(
        alpha_project_id,
        "1.0.0",
        TestFile::build_random_jar(),
        Some(|j: &mut serde_json::Value| {
            let j = j.as_object_mut().unwrap();
            j.remove("game_versions");
        }),
    );
    let resp = api.add_public_version(version_data, USER_USER_PAT).await;
    assert_eq!(resp.status(), 400);

    // TODO: Create a test for too many elements in the array when we have a LF that has a max (past max)
    // Cannot create a version with a loader field array that has fewer than the minimum elements
    // TODO: - Create project
    // - Create version
    let version_data = get_public_version_creation_data(
        alpha_project_id,
        "1.0.0",
        TestFile::build_random_jar(),
        Some(|j: &mut serde_json::Value| {
            let j: &mut serde_json::Map<String, serde_json::Value> = j.as_object_mut().unwrap();
            j["game_versions"] = json!([]);
        }),
    );
    let resp: actix_web::dev::ServiceResponse =
        api.add_public_version(version_data, USER_USER_PAT).await;
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
        let version_data = get_public_version_creation_data(
            alpha_project_id,
            "1.0.0",
            TestFile::build_random_jar(),
            Some(|j: &mut serde_json::Value| {
                let j: &mut serde_json::Map<String, serde_json::Value> = j.as_object_mut().unwrap();
                j["game_versions"] = bad_type_game_versions.clone();
            }),
        );
        let resp = api.add_public_version(version_data, USER_USER_PAT).await;
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
    let version_data = get_public_version_creation_data(
        alpha_project_id,
        "1.0.0",
        TestFile::build_random_jar(),
        Some(|j: &mut serde_json::Value| {
            j["test_fabric_optional"] = json!(555);
        }),
    );
    let v = api
        .add_public_version_deserialized(version_data, USER_USER_PAT)
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
    let version_data = get_public_version_creation_data(
        alpha_project_id,
        "1.0.0",
        TestFile::build_random_jar(),
        Some(|j: &mut serde_json::Value| {
            let j: &mut serde_json::Map<String, serde_json::Value> = j.as_object_mut().unwrap();
            j["game_versions"] = json!(["1.20.1", "1.20.2"]);
            j["client_side"] = json!("optional");
            j["server_side"] = json!("required");
        }),
    );
    let v = api
        .add_public_version_deserialized(version_data, USER_USER_PAT)
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

    test_env.cleanup().await;
}

#[actix_rt::test]
async fn get_loader_fields() {
    let test_env = TestEnvironment::build(None).await;
    let api = &test_env.v3;

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

    test_env.cleanup().await;
}

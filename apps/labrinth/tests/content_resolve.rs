use actix_http::StatusCode;
use actix_web::dev::ServiceResponse;
use actix_web::test;
use common::api_common::{Api, ApiProject, ApiVersion};
use common::api_v3::ApiV3;
use common::database::USER_USER_PAT;
use common::dummy_data::TestFile;
use common::environment::{TestEnvironment, with_test_environment};
use labrinth::models::ids::ProjectId;
use modrinth_content_management::{ResolveContentPlan, SkippedReason};
use serde_json::{Value, json};

pub mod common;

struct TestContent {
    project_id: ProjectId,
    version_id: String,
}

fn required_project(content: &TestContent) -> Value {
    json!({
        "project_id": content.project_id,
        "dependency_type": "required",
    })
}

async fn add_content(
    api: &ApiV3,
    slug: &str,
    file: TestFile,
    loaders: &[&str],
    game_versions: &[&str],
    dependencies: Value,
) -> TestContent {
    let modify_json = serde_json::from_value(json!([
        { "op": "replace", "path": "/initial_versions/0/loaders", "value": loaders },
        { "op": "replace", "path": "/initial_versions/0/game_versions", "value": game_versions },
        { "op": "replace", "path": "/initial_versions/0/dependencies", "value": dependencies },
    ]))
    .unwrap();
    let (project, versions) = api
        .add_public_project(slug, Some(file), Some(modify_json), USER_USER_PAT)
        .await;

    TestContent {
        project_id: project.id,
        version_id: versions[0].id.to_string(),
    }
}

async fn add_resource_pack(api: &ApiV3, slug: &str) -> TestContent {
    add_content(
        api,
        slug,
        TestFile::build_random_resource_pack(),
        &["minecraft"],
        &["1.20.1"],
        json!([]),
    )
    .await
}

async fn add_mod(api: &ApiV3, slug: &str, dependencies: Value) -> TestContent {
    add_content(
        api,
        slug,
        TestFile::build_random_jar(),
        &["fabric"],
        &["1.20.1"],
        dependencies,
    )
    .await
}

async fn add_datapack(
    api: &ApiV3,
    slug: &str,
    game_versions: &[&str],
    dependencies: Value,
) -> TestContent {
    add_content(
        api,
        slug,
        TestFile::build_random_datapack(),
        &["datapack"],
        game_versions,
        dependencies,
    )
    .await
}

async fn resolve(
    api: &ApiV3,
    project_id: ProjectId,
    version_id: Option<&str>,
    content_type: &str,
    loader: &str,
    game_version: &str,
) -> ServiceResponse {
    let preferences = json!({
        "game_versions": [game_version],
        "loaders": [loader],
    });
    let req = test::TestRequest::post()
        .uri("/v3/content/resolve")
        .set_json(json!({
            "project_id": project_id,
            "version_id": version_id,
            "content_type": content_type,
            "selected": preferences,
            "target": preferences,
        }))
        .to_request();

    api.call(req).await
}

async fn resolve_plan(
    api: &ApiV3,
    content: &TestContent,
    content_type: &str,
    loader: &str,
    game_version: &str,
) -> ResolveContentPlan {
    let resp = resolve(
        api,
        content.project_id,
        Some(&content.version_id),
        content_type,
        loader,
        game_version,
    )
    .await;
    assert_status!(&resp, StatusCode::OK);

    test::read_body_json(resp).await
}

#[actix_rt::test]
async fn datapack_resolves_required_resource_pack() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let resource_pack = add_resource_pack(api, "required-pack").await;
            let datapack = add_datapack(
                api,
                "datapack",
                &["1.20.1"],
                json!([required_project(&resource_pack)]),
            )
            .await;

            let plan =
                resolve_plan(api, &datapack, "datapack", "datapack", "1.20.1")
                    .await;

            assert_eq!(plan.primary.version_id, datapack.version_id);
            assert_eq!(plan.dependencies.len(), 1);
            assert_eq!(
                plan.dependencies[0].version_id,
                resource_pack.version_id
            );
            assert_eq!(
                plan.dependencies[0].project_id,
                resource_pack.project_id.to_string()
            );
            assert_eq!(
                plan.dependencies[0].dependent_on_version_id.as_deref(),
                Some(datapack.version_id.as_str())
            );
            assert!(plan.skipped.is_empty());
        },
    )
    .await;
}

#[actix_rt::test]
async fn mod_resolves_required_resource_pack() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let resource_pack = add_resource_pack(api, "required-pack").await;
            let content = add_mod(
                api,
                "fabric-mod",
                json!([required_project(&resource_pack)]),
            )
            .await;

            let plan =
                resolve_plan(api, &content, "mod", "fabric", "1.20.1").await;

            assert_eq!(
                plan.dependencies[0].version_id,
                resource_pack.version_id
            );
            assert!(plan.skipped.is_empty());
        },
    )
    .await;
}

#[actix_rt::test]
async fn resource_pack_resolves_required_datapack() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let datapack =
                add_datapack(api, "required-datapack", &["1.20.1"], json!([]))
                    .await;
            let resource_pack = add_content(
                api,
                "resource-pack",
                TestFile::build_random_resource_pack(),
                &["minecraft"],
                &["1.20.1"],
                json!([required_project(&datapack)]),
            )
            .await;

            let plan = resolve_plan(
                api,
                &resource_pack,
                "resourcepack",
                "minecraft",
                "1.20.1",
            )
            .await;

            assert_eq!(plan.dependencies[0].version_id, datapack.version_id);
            assert!(plan.skipped.is_empty());
        },
    )
    .await;
}

#[actix_rt::test]
async fn dependency_prefers_matching_loader_over_newer_universal_loader() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let dependency = add_mod(api, "hybrid-dependency", json!([])).await;
            let modify_json = serde_json::from_value(json!([
                { "op": "replace", "path": "/loaders", "value": ["minecraft"] },
            ]))
            .unwrap();
            api.add_public_version_deserialized(
                dependency.project_id,
                "2.0.0",
                TestFile::build_random_resource_pack(),
                None,
                Some(modify_json),
                USER_USER_PAT,
            )
            .await;
            let content = add_mod(
                api,
                "fabric-mod",
                json!([required_project(&dependency)]),
            )
            .await;

            let plan =
                resolve_plan(api, &content, "mod", "fabric", "1.20.1").await;

            assert_eq!(plan.dependencies[0].version_id, dependency.version_id);
        },
    )
    .await;
}

#[actix_rt::test]
async fn universal_dependency_must_match_game_version() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let resource_pack = add_resource_pack(api, "required-pack").await;
            let datapack = add_datapack(
                api,
                "datapack",
                &["1.20.1", "1.20.2"],
                json!([required_project(&resource_pack)]),
            )
            .await;

            let plan =
                resolve_plan(api, &datapack, "datapack", "datapack", "1.20.2")
                    .await;

            assert!(plan.dependencies.is_empty());
            assert_eq!(
                plan.skipped[0].reason,
                SkippedReason::NoCompatibleVersion
            );
            assert_eq!(
                plan.skipped[0].dependent_on_version_id.as_deref(),
                Some(datapack.version_id.as_str())
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn mod_dependency_of_datapack_is_skipped_with_parent_version() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let dependency = add_mod(api, "required-mod", json!([])).await;
            let datapack = add_datapack(
                api,
                "datapack",
                &["1.20.1"],
                json!([required_project(&dependency)]),
            )
            .await;

            let plan =
                resolve_plan(api, &datapack, "datapack", "datapack", "1.20.1")
                    .await;

            assert!(plan.dependencies.is_empty());
            assert_eq!(
                plan.skipped[0].reason,
                SkippedReason::NoCompatibleVersion
            );
            assert_eq!(
                plan.skipped[0].project_id,
                dependency.project_id.to_string()
            );
            assert_eq!(
                plan.skipped[0].dependent_on_version_id.as_deref(),
                Some(datapack.version_id.as_str())
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn hidden_version_dependency_is_skipped_with_parent_version() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let dependency = add_resource_pack(api, "hidden-pack").await;
            let datapack = add_datapack(
                api,
                "datapack",
                &["1.20.1"],
                json!([{
                    "version_id": dependency.version_id,
                    "dependency_type": "required",
                }]),
            )
            .await;
            let resp = api
                .edit_version(
                    &dependency.version_id,
                    json!({ "status": "draft" }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let plan =
                resolve_plan(api, &datapack, "datapack", "datapack", "1.20.1")
                    .await;

            assert!(plan.dependencies.is_empty());
            assert_eq!(plan.skipped[0].reason, SkippedReason::MissingVersion);
            assert_eq!(
                plan.skipped[0].dependent_on_version_id.as_deref(),
                Some(datapack.version_id.as_str())
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn mixed_loader_dependency_is_not_universal() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let dependency = add_content(
                api,
                "mixed-dependency",
                TestFile::build_random_jar(),
                &["fabric", "minecraft"],
                &["1.20.1"],
                json!([]),
            )
            .await;
            let content = add_mod(
                api,
                "fabric-mod",
                json!([required_project(&dependency)]),
            )
            .await;

            let plan =
                resolve_plan(api, &content, "mod", "forge", "1.20.1").await;

            assert!(plan.dependencies.is_empty());
            assert_eq!(
                plan.skipped[0].reason,
                SkippedReason::NoCompatibleVersion
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn primary_version_does_not_use_universal_loaders() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let resource_pack = add_resource_pack(api, "resource-pack").await;

            let resp = resolve(
                api,
                resource_pack.project_id,
                None,
                "mod",
                "fabric",
                "1.20.1",
            )
            .await;

            assert_status!(&resp, StatusCode::BAD_REQUEST);
        },
    )
    .await;
}

#[actix_rt::test]
async fn repeated_resolution_returns_same_plan() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let resource_pack = add_resource_pack(api, "required-pack").await;
            let datapack = add_datapack(
                api,
                "datapack",
                &["1.20.1"],
                json!([required_project(&resource_pack)]),
            )
            .await;

            let first =
                resolve_plan(api, &datapack, "datapack", "datapack", "1.20.1")
                    .await;
            let second =
                resolve_plan(api, &datapack, "datapack", "datapack", "1.20.1")
                    .await;

            assert_eq!(first, second);
            assert_eq!(
                second.dependencies[0].version_id,
                resource_pack.version_id
            );
        },
    )
    .await;
}

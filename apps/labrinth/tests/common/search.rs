#![allow(dead_code)]

use std::{collections::HashMap, sync::Arc};

use actix_http::StatusCode;
use serde_json::json;

use crate::{
    assert_status,
    common::{
        api_common::{Api, ApiProject, ApiVersion},
        database::{FRIEND_USER_PAT, MOD_USER_PAT, USER_USER_PAT},
        dummy_data::{DUMMY_CATEGORIES, TestFile},
    },
};

use super::{api_v3::ApiV3, environment::TestEnvironment};

pub async fn setup_search_projects(
    test_env: &TestEnvironment<ApiV3>,
) -> Arc<HashMap<u64, u64>> {
    // Test setup and dummy data
    let api = &test_env.api;
    let test_name = test_env.db.database_name.clone();
    let zeta_organization_id =
        &test_env.dummy.organization_zeta.organization_id;

    // Add dummy projects of various categories for searchability
    let mut project_creation_futures = vec![];

    let create_async_future =
        |id: u64,
         pat: Option<&'static str>,
         is_modpack: bool,
         modify_json: Option<json_patch::Patch>| {
            let slug = format!("{test_name}-searchable-project-{id}");

            let jar = if is_modpack {
                TestFile::build_random_mrpack()
            } else {
                TestFile::build_random_jar()
            };
            async move {
                // Add a project- simple, should work.
                let req =
                    api.add_public_project(&slug, Some(jar), modify_json, pat);
                let (project, _) = req.await;

                // Approve, so that the project is searchable
                let resp = api
                    .edit_project(
                        &project.id.to_string(),
                        json!({
                            "status": "approved"
                        }),
                        MOD_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::NO_CONTENT);
                (project.id.0, id)
            }
        };

    // Test project 0
    let id = 0;
    let modify_json = serde_json::from_value(json!([
        { "op": "add", "path": "/categories", "value": DUMMY_CATEGORIES[4..6] },
        { "op": "add", "path": "/initial_versions/0/server_only", "value": true },
        { "op": "add", "path": "/license_id", "value": "LGPL-3.0-or-later" },
    ]))
    .unwrap();
    project_creation_futures.push(create_async_future(
        id,
        USER_USER_PAT,
        false,
        Some(modify_json),
    ));

    // Test project 1
    let id = 1;
    let modify_json = serde_json::from_value(json!([
        { "op": "add", "path": "/categories", "value": DUMMY_CATEGORIES[0..2] },
        { "op": "add", "path": "/initial_versions/0/client_only", "value": false },
    ]))
    .unwrap();
    project_creation_futures.push(create_async_future(
        id,
        USER_USER_PAT,
        false,
        Some(modify_json),
    ));

    // Test project 2
    let id = 2;
    let modify_json = serde_json::from_value(json!([
        { "op": "add", "path": "/categories", "value": DUMMY_CATEGORIES[0..2] },
        { "op": "add", "path": "/initial_versions/0/server_only", "value": true },
        { "op": "add", "path": "/name", "value": "Mysterious Project" },
    ]))
    .unwrap();
    project_creation_futures.push(create_async_future(
        id,
        USER_USER_PAT,
        false,
        Some(modify_json),
    ));

    // Test project 3
    let id = 3;
    let modify_json = serde_json::from_value(json!([
        { "op": "add", "path": "/categories", "value": DUMMY_CATEGORIES[0..3] },
        { "op": "add", "path": "/initial_versions/0/server_only", "value": true },
        { "op": "add", "path": "/initial_versions/0/game_versions", "value": ["1.20.4"] },
        { "op": "add", "path": "/name", "value": "Mysterious Project" },
        { "op": "add", "path": "/license_id", "value": "LicenseRef-All-Rights-Reserved" },
    ]))
    .unwrap();
    project_creation_futures.push(create_async_future(
        id,
        FRIEND_USER_PAT,
        false,
        Some(modify_json),
    ));

    // Test project 4
    let id = 4;
    let modify_json = serde_json::from_value(json!([
        { "op": "add", "path": "/categories", "value": DUMMY_CATEGORIES[0..3] },
        { "op": "add", "path": "/initial_versions/0/client_only", "value": false },
        { "op": "add", "path": "/initial_versions/0/game_versions", "value": ["1.20.5"] },
    ]))
    .unwrap();
    project_creation_futures.push(create_async_future(
        id,
        USER_USER_PAT,
        true,
        Some(modify_json),
    ));

    // Test project 5
    let id = 5;
    let modify_json = serde_json::from_value(json!([
        { "op": "add", "path": "/categories", "value": DUMMY_CATEGORIES[5..6] },
        { "op": "add", "path": "/initial_versions/0/client_only", "value": false },
        { "op": "add", "path": "/initial_versions/0/game_versions", "value": ["1.20.5"] },
        { "op": "add", "path": "/license_id", "value": "LGPL-3.0-or-later" },
    ]))
    .unwrap();
    project_creation_futures.push(create_async_future(
        id,
        USER_USER_PAT,
        false,
        Some(modify_json),
    ));

    // Test project 6
    let id = 6;
    let modify_json = serde_json::from_value(json!([
        { "op": "add", "path": "/categories", "value": DUMMY_CATEGORIES[5..6] },
        { "op": "add", "path": "/initial_versions/0/client_only", "value": false },
        { "op": "add", "path": "/initial_versions/0/server_only", "value": true },
        { "op": "add", "path": "/license_id", "value": "LGPL-3.0-or-later" },
    ]))
    .unwrap();
    project_creation_futures.push(create_async_future(
        id,
        FRIEND_USER_PAT,
        false,
        Some(modify_json),
    ));

    // Test project 7 (testing the search bug)
    // This project has an initial private forge version that is 1.20.3, and a fabric 1.20.5 version.
    // This means that a search for fabric + 1.20.3 or forge + 1.20.5 should not return this project.
    let id = 7;
    let modify_json = serde_json::from_value(json!([
        { "op": "add", "path": "/categories", "value": DUMMY_CATEGORIES[5..6] },
        { "op": "add", "path": "/initial_versions/0/client_only", "value": false },
        { "op": "add", "path": "/initial_versions/0/server_only", "value": true },
        { "op": "add", "path": "/license_id", "value": "LGPL-3.0-or-later" },
        { "op": "add", "path": "/initial_versions/0/loaders", "value": ["forge"] },
        { "op": "add", "path": "/initial_versions/0/game_versions", "value": ["1.20.2"] },
    ]))
    .unwrap();
    project_creation_futures.push(create_async_future(
        id,
        USER_USER_PAT,
        false,
        Some(modify_json),
    ));

    // Test project 9 (organization)
    // This project gets added to the Zeta organization automatically
    let id = 9;
    let modify_json = serde_json::from_value(json!([
        { "op": "add", "path": "/organization_id", "value": zeta_organization_id },
    ]))
    .unwrap();
    project_creation_futures.push(create_async_future(
        id,
        USER_USER_PAT,
        false,
        Some(modify_json),
    ));

    // Await all project creation
    // Returns a mapping of:
    // project id -> test id
    let id_conversion: Arc<HashMap<u64, u64>> = Arc::new(
        futures::future::join_all(project_creation_futures)
            .await
            .into_iter()
            .collect(),
    );

    // Create a second version for project 7
    let project_7 = api
        .get_project_deserialized_common(
            &format!("{test_name}-searchable-project-7"),
            USER_USER_PAT,
        )
        .await;
    api.add_public_version(
        project_7.id,
        "1.0.0",
        TestFile::build_random_jar(),
        None,
        None,
        USER_USER_PAT,
    )
    .await;

    // Forcibly reset the search index
    let resp = api.reset_search_index().await;
    assert_status!(&resp, StatusCode::NO_CONTENT);

    id_conversion
}

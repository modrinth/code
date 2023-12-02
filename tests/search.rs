use common::api_v3::ApiV3;
use common::database::*;
use common::dummy_data::TestFile;
use common::dummy_data::DUMMY_CATEGORIES;

use common::environment::with_test_environment;
use common::environment::TestEnvironment;
use futures::stream::StreamExt;
use labrinth::models::ids::base62_impl::parse_base62;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

use crate::common::api_common::Api;
use crate::common::api_common::ApiProject;
use crate::common::api_common::ApiVersion;

mod common;

// TODO: Revisit this with the new modify_json in the version maker
// That change here should be able to simplify it vastly

#[actix_rt::test]
async fn search_projects() {
    // Test setup and dummy data
    with_test_environment(Some(10), |test_env: TestEnvironment<ApiV3>| async move {
        let api = &test_env.api;
        let test_name = test_env.db.database_name.clone();

        // Add dummy projects of various categories for searchability
        let mut project_creation_futures = vec![];

        let create_async_future =
            |id: u64,
             pat: &'static str,
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
                    let req = api.add_public_project(&slug, Some(jar), modify_json, pat);
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
                    assert_eq!(resp.status(), 204);
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

        // Pairs of:
        // 1. vec of search facets
        // 2. expected project ids to be returned by this search
        let pairs = vec![
            (json!([["categories:fabric"]]), vec![0, 1, 2, 3, 4, 5, 6, 7]),
            (json!([["categories:forge"]]), vec![7]),
            (
                json!([["categories:fabric", "categories:forge"]]),
                vec![0, 1, 2, 3, 4, 5, 6, 7],
            ),
            (json!([["categories:fabric"], ["categories:forge"]]), vec![]),
            (
                json!([
                    ["categories:fabric"],
                    [&format!("categories:{}", DUMMY_CATEGORIES[0])],
                ]),
                vec![1, 2, 3, 4],
            ),
            (json!([["project_types:modpack"]]), vec![4]),
            (json!([["client_only:true"]]), vec![0, 2, 3, 7]),
            (json!([["server_only:true"]]), vec![0, 2, 3, 6, 7]),
            (json!([["open_source:true"]]), vec![0, 1, 2, 4, 5, 6, 7]),
            (json!([["license:MIT"]]), vec![1, 2, 4]),
            (json!([[r#"name:'Mysterious Project'"#]]), vec![2, 3]),
            (json!([["author:user"]]), vec![0, 1, 2, 4, 5, 7]),
            (json!([["game_versions:1.20.5"]]), vec![4, 5]),
            // bug fix
            (
                json!([
                    // Only the forge one has 1.20.2, so its true that this project 'has'
                    // 1.20.2 and a fabric version, but not true that it has a 1.20.2 fabric version.
                    ["categories:fabric"],
                    ["game_versions:1.20.2"]
                ]),
                vec![],
            ),
            // Project type change
            // Modpack should still be able to search based on former loader, even though technically the loader is 'mrpack'
            (json!([["categories:mrpack"]]), vec![4]),
            (
                json!([["categories:mrpack"], ["categories:fabric"]]),
                vec![4],
            ),
            (
                json!([
                    ["categories:mrpack"],
                    ["categories:fabric"],
                    ["project_types:modpack"]
                ]),
                vec![4],
            ),
        ];
        // TODO: versions, game versions
        // Untested:
        // - downloads                      (not varied)
        // - color                          (not varied)
        // - created_timestamp              (not varied)
        // - modified_timestamp             (not varied)
        // TODO: multiple different project types test

        // Forcibly reset the search index
        let resp = api.reset_search_index().await;
        assert_eq!(resp.status(), 204);

        // Test searches
        let stream = futures::stream::iter(pairs);
        stream
            .for_each_concurrent(1, |(facets, mut expected_project_ids)| {
                let id_conversion = id_conversion.clone();
                let test_name = test_name.clone();
                async move {
                    let projects = api
                        .search_deserialized(Some(&test_name), Some(facets.clone()), USER_USER_PAT)
                        .await;
                    let mut found_project_ids: Vec<u64> = projects
                        .hits
                        .into_iter()
                        .map(|p| id_conversion[&parse_base62(&p.project_id).unwrap()])
                        .collect();
                    expected_project_ids.sort();
                    found_project_ids.sort();
                    assert_eq!(found_project_ids, expected_project_ids);
                }
            })
            .await;
    })
    .await;
}

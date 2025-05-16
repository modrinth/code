use crate::assert_status;
use crate::common::api_common::Api;
use crate::common::api_common::ApiProject;
use crate::common::api_common::ApiVersion;
use crate::common::api_v2::ApiV2;

use crate::common::database::*;
use crate::common::dummy_data::DUMMY_CATEGORIES;
use crate::common::dummy_data::TestFile;
use crate::common::environment::TestEnvironment;
use crate::common::environment::with_test_environment;
use actix_http::StatusCode;
use ariadne::ids::base62_impl::parse_base62;
use futures::stream::StreamExt;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

#[actix_rt::test]
async fn search_projects() {
    // Test setup and dummy data
    with_test_environment(Some(10), |test_env: TestEnvironment<ApiV2>| async move {
        let api = &test_env.api;
        let test_name = test_env.db.database_name.clone();

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
                    assert_status!(&resp, StatusCode::NO_CONTENT);
                    (project.id.0, id)
                }
            };

        // Test project 0
        let id = 0;
        let modify_json = serde_json::from_value(json!([
            { "op": "add", "path": "/categories", "value": DUMMY_CATEGORIES[4..6] },
            { "op": "add", "path": "/server_side", "value": "required" },
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
            { "op": "add", "path": "/client_side", "value": "optional" },
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
            { "op": "add", "path": "/server_side", "value": "required" },
            { "op": "add", "path": "/title", "value": "Mysterious Project" },
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
            { "op": "add", "path": "/server_side", "value": "required" },
            { "op": "add", "path": "/initial_versions/0/game_versions", "value": ["1.20.4"] },
            { "op": "add", "path": "/title", "value": "Mysterious Project" },
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
            { "op": "add", "path": "/client_side", "value": "optional" },
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
            { "op": "add", "path": "/client_side", "value": "optional" },
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
            { "op": "add", "path": "/client_side", "value": "optional" },
            { "op": "add", "path": "/server_side", "value": "required" },
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
        // This project has an initial private forge version that is 1.20.2, and a fabric 1.20.1 version.
        // This means that a search for fabric + 1.20.1 or forge + 1.20.1 should not return this project,
        // but a search for fabric + 1.20.1 should, and it should include both versions in the data.
        let id = 7;
        let modify_json = serde_json::from_value(json!([
            { "op": "add", "path": "/categories", "value": DUMMY_CATEGORIES[5..6] },
            { "op": "add", "path": "/client_side", "value": "optional" },
            { "op": "add", "path": "/server_side", "value": "required" },
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

        // Test project 8
        // Server side unsupported
        let id = 8;
        let modify_json = serde_json::from_value(json!([
            { "op": "add", "path": "/server_side", "value": "unsupported" },
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
            .get_project_deserialized(&format!("{test_name}-searchable-project-7"), USER_USER_PAT)
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
            (
                json!([["categories:fabric"]]),
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            ),
            (json!([["categories:forge"]]), vec![7]),
            (
                json!([["categories:fabric", "categories:forge"]]),
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
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
            // Formerly included 7, but with v2 changes, this is no longer the case.
            // This is because we assume client_side/server_side with subsequent versions.
            (json!([["client_side:required"]]), vec![0, 2, 3, 8]),
            (json!([["server_side:required"]]), vec![0, 2, 3, 6, 7]),
            (json!([["open_source:true"]]), vec![0, 1, 2, 4, 5, 6, 7, 8]),
            (json!([["license:MIT"]]), vec![1, 2, 4, 8]),
            (json!([[r#"title:'Mysterious Project'"#]]), vec![2, 3]),
            (json!([["author:user"]]), vec![0, 1, 2, 4, 5, 7, 8]),
            (json!([["versions:1.20.5"]]), vec![4, 5]),
            // bug fix
            (
                json!([
                    // Only the forge one has 1.20.2, so its true that this project 'has'
                    // 1.20.2 and a fabric version, but not true that it has a 1.20.2 fabric version.
                    ["categories:fabric"],
                    ["versions:1.20.2"]
                ]),
                vec![],
            ),
            (
                json!([
                    // But it does have a 1.20.2 forge version, so this should return it.
                    ["categories:forge"],
                    ["versions:1.20.2"]
                ]),
                vec![7],
            ),
            // Project type change
            // Modpack should still be able to search based on former loader, even though technically the loader is 'mrpack'
            // (json!([["categories:mrpack"]]), vec![4]),
            // (
            //     json!([["categories:mrpack"], ["categories:fabric"]]),
            //     vec![4],
            // ),
            (
                json!([
                    // ["categories:mrpack"],
                    ["categories:fabric"],
                    ["project_type:modpack"]
                ]),
                vec![4],
            ),
            (
                json!([["client_side:optional"], ["server_side:optional"]]),
                vec![1, 4, 5],
            ),
            (json!([["server_side:optional"]]), vec![1, 4, 5]),
            (json!([["server_side:unsupported"]]), vec![8]),
        ];

        // TODO: Untested:
        // - downloads                      (not varied)
        // - color                          (not varied)
        // - created_timestamp              (not varied)
        // - modified_timestamp             (not varied)

        // Forcibly reset the search index
        let resp = api.reset_search_index().await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // Test searches
        let stream = futures::stream::iter(pairs);
        stream
            .for_each_concurrent(1, |(facets, mut expected_project_ids)| {
                let id_conversion = id_conversion.clone();
                let test_name = test_name.clone();
                async move {
                    let projects = api
                        .search_deserialized(
                            Some(&format!("\"&{test_name}\"")),
                            Some(facets.clone()),
                            USER_USER_PAT,
                        )
                        .await;
                    let mut found_project_ids: Vec<u64> = projects
                        .hits
                        .into_iter()
                        .map(|p| id_conversion[&parse_base62(&p.project_id).unwrap()])
                        .collect();
                    expected_project_ids.sort();
                    found_project_ids.sort();
                    println!("Facets: {facets:?}");
                    assert_eq!(found_project_ids, expected_project_ids);
                }
            })
            .await;

        // A couple additional tests for the search type returned, making sure it is properly translated back
        let client_side_required = api
            .search_deserialized(
                Some(&format!("\"&{test_name}\"")),
                Some(json!([["client_side:required"]])),
                USER_USER_PAT,
            )
            .await;
        for hit in client_side_required.hits {
            assert_eq!(hit.client_side, "required".to_string());
        }

        let server_side_required = api
            .search_deserialized(
                Some(&format!("\"&{test_name}\"")),
                Some(json!([["server_side:required"]])),
                USER_USER_PAT,
            )
            .await;
        for hit in server_side_required.hits {
            assert_eq!(hit.server_side, "required".to_string());
        }

        let client_side_unsupported = api
            .search_deserialized(
                Some(&format!("\"&{test_name}\"")),
                Some(json!([["client_side:unsupported"]])),
                USER_USER_PAT,
            )
            .await;
        for hit in client_side_unsupported.hits {
            assert_eq!(hit.client_side, "unsupported".to_string());
        }

        let client_side_optional_server_side_optional = api
            .search_deserialized(
                Some(&format!("\"&{test_name}\"")),
                Some(json!([["client_side:optional"], ["server_side:optional"]])),
                USER_USER_PAT,
            )
            .await;
        for hit in client_side_optional_server_side_optional.hits {
            assert_eq!(hit.client_side, "optional".to_string());
            assert_eq!(hit.server_side, "optional".to_string());
        }

        // Ensure game_versions return correctly, but also correctly aggregated
        // over all versions of a project
        let game_versions = api
            .search_deserialized(
                Some(&format!("\"&{test_name}\"")),
                Some(json!([["categories:forge"], ["versions:1.20.2"]])),
                USER_USER_PAT,
            )
            .await;
        assert_eq!(game_versions.hits.len(), 1);
        for hit in game_versions.hits {
            assert_eq!(
                hit.versions,
                vec!["1.20.1".to_string(), "1.20.2".to_string()]
            );
            assert!(hit.categories.contains(&"forge".to_string()));
            assert!(hit.categories.contains(&"fabric".to_string()));
            assert!(hit.display_categories.contains(&"forge".to_string()));
            assert!(hit.display_categories.contains(&"fabric".to_string()));

            // Also, ensure author is correctly capitalized
            assert_eq!(hit.author, "User".to_string());
        }
    })
    .await;
}

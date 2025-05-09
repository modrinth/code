use actix_http::StatusCode;
use common::api_v3::ApiV3;
use common::database::*;

use common::dummy_data::DUMMY_CATEGORIES;

use ariadne::ids::base62_impl::parse_base62;
use common::environment::TestEnvironment;
use common::environment::with_test_environment;
use common::search::setup_search_projects;
use futures::stream::StreamExt;
use serde_json::json;

use crate::common::api_common::Api;
use crate::common::api_common::ApiProject;

mod common;

// TODO: Revisit this wit   h the new modify_json in the version maker
// That change here should be able to simplify it vastly

#[actix_rt::test]
async fn search_projects() {
    // Test setup and dummy data
    with_test_environment(
        Some(10),
        |test_env: TestEnvironment<ApiV3>| async move {
            let id_conversion = setup_search_projects(&test_env).await;

            let api = &test_env.api;
            let test_name = test_env.db.database_name.clone();

            // Pairs of:
            // 1. vec of search facets
            // 2. expected project ids to be returned by this search
            let pairs = vec![
                (
                    json!([["categories:fabric"]]),
                    vec![0, 1, 2, 3, 4, 5, 6, 7, 9],
                ),
                (json!([["categories:forge"]]), vec![7]),
                (
                    json!([["categories:fabric", "categories:forge"]]),
                    vec![0, 1, 2, 3, 4, 5, 6, 7, 9],
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
                (json!([["client_only:true"]]), vec![0, 2, 3, 7, 9]),
                (json!([["server_only:true"]]), vec![0, 2, 3, 6, 7]),
                (json!([["open_source:true"]]), vec![0, 1, 2, 4, 5, 6, 7, 9]),
                (json!([["license:MIT"]]), vec![1, 2, 4, 9]),
                (json!([[r#"name:'Mysterious Project'"#]]), vec![2, 3]),
                (json!([["author:user"]]), vec![0, 1, 2, 4, 5, 7, 9]), // Organization test '9' is included here as user is owner of org
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
                // (json!([["categories:mrpack"]]), vec![4]),
                // (
                //     json!([["categories:fabric"]]),
                //     vec![4],
                // ),
                (
                    json!([["categories:fabric"], ["project_types:modpack"]]),
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
                            .map(|p| {
                                id_conversion
                                    [&parse_base62(&p.project_id).unwrap()]
                            })
                            .collect();
                        let num_hits = projects.total_hits;
                        expected_project_ids.sort();
                        found_project_ids.sort();
                        println!("Facets: {facets:?}");
                        assert_eq!(found_project_ids, expected_project_ids);
                        assert_eq!(num_hits, { expected_project_ids.len() });
                    }
                })
                .await;
        },
    )
    .await;
}

#[actix_rt::test]
async fn index_swaps() {
    with_test_environment(
        Some(10),
        |test_env: TestEnvironment<ApiV3>| async move {
            // Reindex
            let resp = test_env.api.reset_search_index().await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Now we should get results
            let projects = test_env
                .api
                .search_deserialized(
                    None,
                    Some(json!([["categories:fabric"]])),
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(projects.total_hits, 1);
            assert!(projects.hits[0].slug.as_ref().unwrap().contains("alpha"));

            // Delete the project
            let resp =
                test_env.api.remove_project("alpha", USER_USER_PAT).await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // We should not get any results, because the project has been deleted
            let projects = test_env
                .api
                .search_deserialized(
                    None,
                    Some(json!([["categories:fabric"]])),
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(projects.total_hits, 0);

            // But when we reindex, it should be gone
            let resp = test_env.api.reset_search_index().await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let projects = test_env
                .api
                .search_deserialized(
                    None,
                    Some(json!([["categories:fabric"]])),
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(projects.total_hits, 0);

            // Reindex again, should still be gone
            let resp = test_env.api.reset_search_index().await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let projects = test_env
                .api
                .search_deserialized(
                    None,
                    Some(json!([["categories:fabric"]])),
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(projects.total_hits, 0);
        },
    )
    .await;
}

use actix_http::StatusCode;
use common::api_v3::ApiV3;
use common::database::*;

use common::dummy_data::DUMMY_CATEGORIES;

use ariadne::ids::base62_impl::{parse_base62, to_base62};
use common::environment::TestEnvironment;
use common::environment::with_test_environment;
use common::search::setup_search_projects;
use futures::stream::StreamExt;
use labrinth::{models::projects::DependencyType, search::SearchRequest};
use serde_json::json;

use crate::common::api_common::Api;
use crate::common::api_common::ApiProject;

pub mod common;

#[actix_rt::test]
async fn normalized_titles_have_title_relevance() {
    with_test_environment(
        Some(10),
        |test_env: TestEnvironment<ApiV3>| async move {
            let project = &test_env.dummy.project_alpha;
            let resp = test_env
                .api
                .edit_project(
                    &project.project_id,
                    json!({
                        "name": "wWaypoints",
                        "summary": "A test project"
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let resp = test_env.api.reset_search_index().await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            for (query, expected_score) in [
                ("Waypoint", 4.0),
                ("Waypoints", 4.0),
                ("way", 4.0),
                ("wWaypoints", 5.0),
                ("wwaypoints", 5.0),
                ("alpha", 3.0),
                ("user", 2.0),
                ("test", 1.0),
            ] {
                let results = test_env
                    .db
                    .search_backend
                    .search_for_project_raw(&SearchRequest {
                        query: Some(query.to_string()),
                        offset: None,
                        index: Some("relevance".to_string()),
                        limit: None,
                        show_metadata: true,
                        new_filters: None,
                        facets: None,
                        filters: None,
                        version: None,
                    })
                    .await
                    .unwrap();
                let hit = results
                    .hits
                    .iter()
                    .find(|hit| hit.project_id == project.project_id)
                    .unwrap_or_else(|| panic!("missing project for {query}"));
                assert_eq!(
                    hit.search_metadata.as_ref().unwrap()["score"].as_f64(),
                    Some(expected_score),
                    "unexpected relevance for {query}"
                );
            }
        },
    )
    .await;
}

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
            let dependency_project_id = id_conversion
                .iter()
                .find_map(|(project_id, test_id)| {
                    (*test_id == 1).then_some(to_base62(*project_id))
                })
                .unwrap();

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
                (json!([["environment:server_only"]]), vec![0, 2, 3]),
                (
                    json!([["environment:client_or_server_prefers_both"]]),
                    vec![6, 7],
                ),
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
                (
                    json!([[format!(
                        "dependency_project_ids:{dependency_project_id}"
                    )]]),
                    vec![7],
                ),
                (
                    json!([[format!(
                        "compatible_dependency_project_ids:{dependency_project_id}"
                    )]]),
                    vec![7],
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
                                Some(&format!("&{test_name}")),
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

            let projects = api
                .search_deserialized(
                    Some(&format!("&{test_name}")),
                    Some(json!([[format!(
                        "dependency_project_ids:{dependency_project_id}"
                    )]])),
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(projects.total_hits, 1);
            assert_eq!(projects.hits[0].dependency_project_ids.len(), 1);
            assert_eq!(
                projects.hits[0].dependency_project_ids[0],
                dependency_project_id
            );
            assert_eq!(
                projects.hits[0].compatible_dependency_project_ids.len(),
                1
            );
            assert_eq!(
                projects.hits[0].compatible_dependency_project_ids[0],
                dependency_project_id
            );
            assert_eq!(projects.hits[0].dependencies.len(), 1);
            assert_eq!(
                projects.hits[0].dependencies[0].project_id,
                dependency_project_id
            );
            assert_eq!(
                projects.hits[0].dependencies[0].dependency_type,
                DependencyType::Required
            );
            assert!(
                projects.hits[0].dependencies[0]
                    .slug
                    .as_ref()
                    .unwrap()
                    .contains("searchable-project-1")
            );
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

            // When we reindex, the deleted project should be gone
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

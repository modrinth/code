use crate::assert_status;
use crate::common::api_common::{
    Api, ApiProject, ApiVersion, AppendsOptionalPat,
};
use crate::common::api_v2::ApiV2;
use crate::common::database::{ADMIN_USER_PAT, ENEMY_USER_PAT, USER_USER_PAT};
use crate::common::dummy_data::TestFile;
use crate::common::environment::{TestEnvironment, with_test_environment};
use actix_http::StatusCode;
use actix_web::test::{self, TestRequest};
use labrinth::models::ids::ProjectId;
use serde_json::{Value, json};
use std::collections::HashSet;

fn query(parameters: &[(&str, Value)]) -> String {
    let parameters = parameters
        .iter()
        .map(|(name, value)| {
            let value = match value {
                Value::String(value) => value.clone(),
                value => value.to_string(),
            };
            format!("{name}={}", urlencoding::encode(&value))
        })
        .collect::<Vec<_>>()
        .join("&");
    format!("/v2/versions?{parameters}")
}

async fn versions(
    api: &ApiV2,
    parameters: &[(&str, Value)],
    pat: Option<&str>,
) -> Vec<Value> {
    let response = api
        .call(
            TestRequest::get()
                .uri(&query(parameters))
                .append_pat(pat)
                .to_request(),
        )
        .await;
    assert_status!(&response, StatusCode::OK);
    test::read_body_json(response).await
}

fn ids(versions: &[Value]) -> HashSet<String> {
    versions
        .iter()
        .map(|version| version["id"].as_str().unwrap().to_owned())
        .collect()
}

async fn add_version(
    api: &ApiV2,
    project: ProjectId,
    number: &str,
    loader: &str,
    mc: &str,
) -> String {
    let patch = serde_json::from_value(json!([
        {"op": "replace", "path": "/loaders", "value": [loader]},
        {"op": "replace", "path": "/game_versions", "value": [mc]},
        {"op": "add", "path": "/changelog", "value": "bulk listing changelog"}
    ]))
    .unwrap();
    api.add_public_version_deserialized_common(
        project,
        number,
        TestFile::build_random_jar(),
        None,
        Some(patch),
        USER_USER_PAT,
    )
    .await
    .id
    .to_string()
}

#[actix_rt::test]
async fn bulk_project_versions_filter_each_project_without_collapsing_releases()
{
    with_test_environment(None, |env: TestEnvironment<ApiV2>| async move {
        let api = &env.api;
        let alpha = &env.dummy.project_alpha;
        let beta = &env.dummy.project_beta;
        let response = api
            .edit_project(
                &beta.project_id,
                json!({"status": "approved"}),
                ADMIN_USER_PAT,
            )
            .await;
        assert_status!(&response, StatusCode::NO_CONTENT);
        let mut expected = HashSet::new();
        let mut fabric = HashSet::new();
        for project in [alpha.project_id_parsed, beta.project_id_parsed] {
            for (number, loader, mc) in [
                ("bulk-fabric-1", "fabric", "1.20.1"),
                ("bulk-fabric-2", "fabric", "1.20.1"),
                ("bulk-forge", "forge", "1.20.1"),
                ("bulk-other-mc", "fabric", "1.20.2"),
            ] {
                let id = add_version(api, project, number, loader, mc).await;
                if mc == "1.20.1" {
                    if loader == "fabric" {
                        fabric.insert(id.clone());
                    }
                    expected.insert(id);
                }
            }
        }
        for id in [&alpha.version_id, &beta.version_id] {
            let response = api
                .edit_version(id, json!({"status": "draft"}), USER_USER_PAT)
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);
        }
        let selectors = json!([
            alpha.project_id,
            alpha.project_slug,
            beta.project_slug,
            "missing-project"
        ]);
        let result = versions(
            api,
            &[
                ("project_ids", selectors.clone()),
                ("game_versions", json!(["1.20.1"])),
                ("loaders", json!(["fabric", "forge"])),
            ],
            None,
        )
        .await;
        assert_eq!(result.len(), 6);
        assert_eq!(ids(&result), expected);
        let result = versions(
            api,
            &[
                ("project_ids", selectors.clone()),
                ("game_versions", json!(["1.20.1"])),
                ("loaders", json!(["fabric"])),
            ],
            None,
        )
        .await;
        assert_eq!(result.len(), 4);
        assert_eq!(ids(&result), fabric);
        let result = versions(
            api,
            &[
                ("project_ids", selectors),
                ("game_versions", json!(["1.20.2"])),
            ],
            None,
        )
        .await;
        assert_eq!(result.len(), 2);
        assert!(ids(&result).is_disjoint(&expected));
    })
    .await;
}

#[actix_rt::test]
async fn bulk_project_versions_featured_modpacks_fall_back_to_matching_loader()
{
    with_test_environment(None, |env: TestEnvironment<ApiV2>| async move {
        let api = &env.api;
        let (project, initial_versions) = api
            .add_public_project(
                "bulk-featured-modpack",
                None,
                None,
                USER_USER_PAT,
            )
            .await;
        assert!(initial_versions.is_empty());

        let mut fabric_id = None;
        for (number, loader, featured) in [
            ("bulk-featured-forge", "forge", true),
            ("bulk-unfeatured-fabric", "fabric", false),
        ] {
            let version = api
                .add_public_version_deserialized_common(
                    project.id,
                    number,
                    TestFile::build_random_mrpack(),
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            let id = version.id.to_string();
            let response = api
                .edit_version(
                    &id,
                    json!({
                        "loaders": [loader],
                        "featured": featured,
                        "version_type": "release"
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);
            if loader == "fabric" {
                fabric_id = Some(id);
            }
        }

        let result = versions(
            api,
            &[
                ("project_ids", json!([project.id])),
                ("loaders", json!(["fabric"])),
                ("featured", json!(true)),
            ],
            None,
        )
        .await;
        assert_eq!(result.len(), 1);
        assert_eq!(ids(&result), HashSet::from([fabric_id.unwrap()]));
        assert_eq!(result[0]["loaders"], json!(["fabric"]));
        assert_eq!(result[0]["featured"], json!(false));
        assert_eq!(result[0]["version_type"], json!("release"));
    })
    .await;
}

#[actix_rt::test]
async fn bulk_project_versions_validate_selectors_and_preserve_ids_compatibility()
 {
    with_test_environment(None, |env: TestEnvironment<ApiV2>| async move {
        let api = &env.api;
        let project = &env.dummy.project_alpha;
        for parameters in [
            vec![],
            vec![("ids", json!([])), ("project_ids", json!([]))],
            vec![("project_ids", json!("not-json"))],
            vec![("project_ids", json!({"id": project.project_id}))],
            vec![("project_ids", json!([1]))],
            vec![("project_ids", json!(vec![project.project_id.clone(); 101]))],
            vec![("project_ids", json!([])), ("loaders", json!("not-json"))],
            vec![("project_ids", json!([])), ("game_versions", json!([1]))],
            vec![("project_ids", json!([])), ("loaders", json!({}))],
            vec![
                ("project_ids", json!([])),
                ("game_versions", json!("not-json")),
            ],
        ] {
            let response = api
                .call(TestRequest::get().uri(&query(&parameters)).to_request())
                .await;
            assert_status!(&response, StatusCode::BAD_REQUEST);
        }
        assert!(
            versions(api, &[("project_ids", json!([]))], None)
                .await
                .is_empty()
        );
        let result = versions(
            api,
            &[("project_ids", json!(vec![project.project_id.clone(); 100]))],
            None,
        )
        .await;
        assert_eq!(result.len(), 1);
        assert_eq!(ids(&result), HashSet::from([project.version_id.clone()]));
        let baseline =
            versions(api, &[("ids", json!([project.version_id]))], None).await;
        let filtered = versions(
            api,
            &[
                ("ids", json!([project.version_id])),
                ("loaders", json!("not-json")),
                ("game_versions", json!("not-json")),
                ("featured", json!(false)),
                ("version_type", json!("alpha")),
                ("limit", json!(0)),
                ("offset", json!(100)),
            ],
            None,
        )
        .await;
        assert_eq!(baseline, filtered);
        assert_eq!(baseline.len(), 1);
    })
    .await;
}

#[actix_rt::test]
async fn bulk_project_versions_paginate_globally_without_a_default_limit() {
    with_test_environment(None, |env: TestEnvironment<ApiV2>| async move {
        let api = &env.api;
        let alpha = &env.dummy.project_alpha;
        let beta = &env.dummy.project_beta;
        let response = api
            .edit_project(
                &beta.project_id,
                json!({"status": "approved"}),
                ADMIN_USER_PAT,
            )
            .await;
        assert_status!(&response, StatusCode::NO_CONTENT);
        let mut expected = HashSet::new();
        for index in 0..101 {
            let project = if index % 2 == 0 {
                alpha.project_id_parsed
            } else {
                beta.project_id_parsed
            };
            expected.insert(
                add_version(
                    api,
                    project,
                    &format!("bulk-page-{index}"),
                    "fabric",
                    "1.20.1",
                )
                .await,
            );
        }
        for id in [&alpha.version_id, &beta.version_id] {
            let response = api
                .edit_version(id, json!({"status": "draft"}), USER_USER_PAT)
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);
        }
        let selectors = json!([beta.project_id, alpha.project_id]);
        let all =
            versions(api, &[("project_ids", selectors.clone())], None).await;
        assert_eq!(all.len(), 101);
        assert_eq!(ids(&all), expected);
        assert!(
            all.windows(2)
                .all(|pair| pair[0]["date_published"].as_str().unwrap()
                    >= pair[1]["date_published"].as_str().unwrap())
        );
        assert!(
			all.iter()
				.all(|version| version["changelog"] == "bulk listing changelog")
		);
        let page = versions(
            api,
            &[
                ("project_ids", selectors.clone()),
                ("offset", json!(1)),
                ("limit", json!(3)),
            ],
            None,
        )
        .await;
        assert_eq!(page, all[1..4]);
        let suffix = versions(
            api,
            &[("project_ids", selectors.clone()), ("offset", json!(99))],
            None,
        )
        .await;
        assert_eq!(suffix, all[99..]);
        for parameters in [
            vec![("project_ids", selectors.clone()), ("limit", json!(0))],
            vec![("project_ids", selectors.clone()), ("offset", json!(101))],
        ] {
            assert!(versions(api, &parameters, None).await.is_empty());
        }
        for selector in ["project_ids", "ids"] {
            let value = if selector == "ids" {
                json!([all[0]["id"]])
            } else {
                selectors.clone()
            };
            let result = versions(
                api,
                &[(selector, value), ("include_changelog", json!(false))],
                None,
            )
            .await;
            assert!(!result.is_empty());
            assert!(result.iter().all(|version| {
                version.get("changelog").is_none_or(Value::is_null)
            }));
        }
    })
    .await;
}

#[actix_rt::test]
async fn bulk_project_versions_apply_visibility_before_global_pagination() {
    with_test_environment(None, |env: TestEnvironment<ApiV2>| async move {
        let api = &env.api;
        let alpha = &env.dummy.project_alpha;
        let beta = &env.dummy.project_beta;
        let hidden = add_version(
            api,
            alpha.project_id_parsed,
            "bulk-hidden",
            "fabric",
            "1.20.1",
        )
        .await;
        let response = api
            .edit_version(&hidden, json!({"status": "draft"}), USER_USER_PAT)
            .await;
        assert_status!(&response, StatusCode::NO_CONTENT);
        let response = api
            .edit_project(
                &beta.project_id,
                json!({"status": "processing"}),
                ADMIN_USER_PAT,
            )
            .await;
        assert_status!(&response, StatusCode::NO_CONTENT);
        let response = api
            .edit_version(
                &beta.version_id,
                json!({"status": "listed"}),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&response, StatusCode::NO_CONTENT);
        let selectors = json!([alpha.project_id, beta.project_id]);
        for pat in [None, ENEMY_USER_PAT] {
            let result =
                versions(api, &[("project_ids", selectors.clone())], pat).await;
            assert_eq!(ids(&result), HashSet::from([alpha.version_id.clone()]));
            let page = versions(
                api,
                &[("project_ids", selectors.clone()), ("limit", json!(1))],
                pat,
            )
            .await;
            assert_eq!(page, result);
        }
        let result =
            versions(api, &[("project_ids", selectors)], USER_USER_PAT).await;
        assert_eq!(
            ids(&result),
            HashSet::from([
                alpha.version_id.clone(),
                beta.version_id.clone(),
                hidden
            ]),
            "the owner must see listed versions from both projects and their draft version"
        );
    })
    .await;
}

use std::collections::HashMap;

use actix_http::StatusCode;
use actix_web::test;
use chrono::{Duration, Utc};
use common::api_v3::ApiV3;
use common::database::*;
use common::dummy_data::DUMMY_CATEGORIES;

use common::environment::{with_test_environment, with_test_environment_all, TestEnvironment};
use common::permissions::{PermissionsTest, PermissionsTestContext};
use common::search::setup_search_projects;
use futures::StreamExt;
use labrinth::database::models::project_item::{PROJECTS_NAMESPACE, PROJECTS_SLUGS_NAMESPACE};
use labrinth::models::ids::base62_impl::parse_base62;
use labrinth::models::projects::{Project, ProjectId};
use labrinth::models::teams::ProjectPermissions;
use labrinth::util::actix::{MultipartSegment, MultipartSegmentData};
use serde_json::json;

use crate::common::api_common::models::CommonItemType;
use crate::common::api_common::request_data::ProjectCreationRequestData;
use crate::common::api_common::{ApiProject, ApiTeams, ApiVersion, AppendsOptionalPat};
use crate::common::dummy_data::{DummyImage, TestFile};
mod common;

#[actix_rt::test]
async fn test_get_project() {
    // Test setup and dummy data
    with_test_environment_all(None, |test_env| async move {
        let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
        let beta_project_id = &test_env.dummy.as_ref().unwrap().project_beta.project_id;
        let alpha_project_slug = &test_env.dummy.as_ref().unwrap().project_alpha.project_slug;
        let alpha_version_id = &test_env.dummy.as_ref().unwrap().project_alpha.version_id;

        // Perform request on dummy data
        let req = test::TestRequest::get()
            .uri(&format!("/v3/project/{alpha_project_id}"))
            .append_pat(USER_USER_PAT)
            .to_request();
        let resp = test_env.call(req).await;
        let status = resp.status();
        let body: serde_json::Value = test::read_body_json(resp).await;

        assert_eq!(status, 200);
        assert_eq!(body["id"], json!(alpha_project_id));
        assert_eq!(body["slug"], json!(alpha_project_slug));
        let versions = body["versions"].as_array().unwrap();
        assert_eq!(versions[0], json!(alpha_version_id));

        // Confirm that the request was cached
        let mut redis_pool = test_env.db.redis_pool.connect().await.unwrap();
        assert_eq!(
            redis_pool
                .get(PROJECTS_SLUGS_NAMESPACE, alpha_project_slug)
                .await
                .unwrap()
                .and_then(|x| x.parse::<i64>().ok()),
            Some(parse_base62(alpha_project_id).unwrap() as i64)
        );

        let cached_project = redis_pool
            .get(
                PROJECTS_NAMESPACE,
                &parse_base62(alpha_project_id).unwrap().to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        let cached_project: serde_json::Value = serde_json::from_str(&cached_project).unwrap();
        assert_eq!(cached_project["inner"]["slug"], json!(alpha_project_slug));

        // Make the request again, this time it should be cached
        let req = test::TestRequest::get()
            .uri(&format!("/v3/project/{alpha_project_id}"))
            .append_pat(USER_USER_PAT)
            .to_request();
        let resp = test_env.call(req).await;
        let status = resp.status();
        assert_eq!(status, 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["id"], json!(alpha_project_id));
        assert_eq!(body["slug"], json!(alpha_project_slug));

        // Request should fail on non-existent project
        let req = test::TestRequest::get()
            .uri("/v3/project/nonexistent")
            .append_pat(USER_USER_PAT)
            .to_request();

        let resp = test_env.call(req).await;
        assert_eq!(resp.status(), 404);

        // Similarly, request should fail on non-authorized user, on a yet-to-be-approved or hidden project, with a 404 (hiding the existence of the project)
        let req = test::TestRequest::get()
            .uri(&format!("/v3/project/{beta_project_id}"))
            .append_pat(ENEMY_USER_PAT)
            .to_request();

        let resp = test_env.call(req).await;
        assert_eq!(resp.status(), 404);
    })
    .await;
}

#[actix_rt::test]
async fn test_add_remove_project() {
    // Test setup and dummy data
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        let api = &test_env.api;

        // Generate test project data.
        let mut json_data = api
            .get_public_project_creation_data_json("demo", Some(&TestFile::BasicMod))
            .await;

        // Basic json
        let json_segment = MultipartSegment {
            name: "data".to_string(),
            filename: None,
            content_type: Some("application/json".to_string()),
            data: MultipartSegmentData::Text(serde_json::to_string(&json_data).unwrap()),
        };

        // Basic json, with a different file
        json_data["initial_versions"][0]["file_parts"][0] = json!("basic-mod-different.jar");
        let json_diff_file_segment = MultipartSegment {
            data: MultipartSegmentData::Text(serde_json::to_string(&json_data).unwrap()),
            ..json_segment.clone()
        };

        // Basic json, with a different file, and a different slug
        json_data["slug"] = json!("new_demo");
        json_data["initial_versions"][0]["file_parts"][0] = json!("basic-mod-different.jar");
        let json_diff_slug_file_segment = MultipartSegment {
            data: MultipartSegmentData::Text(serde_json::to_string(&json_data).unwrap()),
            ..json_segment.clone()
        };

        let basic_mod_file = TestFile::BasicMod;
        let basic_mod_different_file = TestFile::BasicModDifferent;

        // Basic file
        let file_segment = MultipartSegment {
            // 'Basic'
            name: basic_mod_file.filename(),
            filename: Some(basic_mod_file.filename()),
            content_type: basic_mod_file.content_type(),
            data: MultipartSegmentData::Binary(basic_mod_file.bytes()),
        };

        // Differently named file, with the SAME content (for hash testing)
        let file_diff_name_segment = MultipartSegment {
            // 'Different'
            name: basic_mod_different_file.filename(),
            filename: Some(basic_mod_different_file.filename()),
            content_type: basic_mod_different_file.content_type(),
            // 'Basic'
            data: MultipartSegmentData::Binary(basic_mod_file.bytes()),
        };

        // Differently named file, with different content
        let file_diff_name_content_segment = MultipartSegment {
            // 'Different'
            name: basic_mod_different_file.filename(),
            filename: Some(basic_mod_different_file.filename()),
            content_type: basic_mod_different_file.content_type(),
            data: MultipartSegmentData::Binary(basic_mod_different_file.bytes()),
        };

        // Add a project- simple, should work.
        let resp = api
            .create_project(
                ProjectCreationRequestData {
                    slug: "demo".to_string(),
                    segment_data: vec![json_segment.clone(), file_segment.clone()],
                    jar: None, // File not needed at this point
                },
                USER_USER_PAT,
            )
            .await;

        let status = resp.status();
        assert_eq!(status, 200);

        // Get the project we just made, and confirm that it's correct
        let project = api
            .get_project_deserialized_common("demo", USER_USER_PAT)
            .await;
        assert!(project.versions.len() == 1);
        let uploaded_version_id = project.versions[0];

        // Checks files to ensure they were uploaded and correctly identify the file
        let hash = sha1::Sha1::from(basic_mod_file.bytes())
            .digest()
            .to_string();
        let version = api
            .get_version_from_hash_deserialized_common(&hash, "sha1", USER_USER_PAT)
            .await;
        assert_eq!(version.id, uploaded_version_id);

        // Reusing with a different slug and the same file should fail
        // Even if that file is named differently
        let resp = api
            .create_project(
                ProjectCreationRequestData {
                    slug: "demo".to_string(),
                    segment_data: vec![
                        json_diff_slug_file_segment.clone(),
                        file_diff_name_segment.clone(),
                    ],
                    jar: None, // File not needed at this point
                },
                USER_USER_PAT,
            )
            .await;
        assert_eq!(resp.status(), 400);

        // Reusing with the same slug and a different file should fail
        let resp = api
            .create_project(
                ProjectCreationRequestData {
                    slug: "demo".to_string(),
                    segment_data: vec![
                        json_diff_file_segment.clone(),
                        file_diff_name_content_segment.clone(),
                    ],
                    jar: None, // File not needed at this point
                },
                USER_USER_PAT,
            )
            .await;
        assert_eq!(resp.status(), 400);

        // Different slug, different file should succeed
        let resp = api
            .create_project(
                ProjectCreationRequestData {
                    slug: "demo".to_string(),
                    segment_data: vec![
                        json_diff_slug_file_segment.clone(),
                        file_diff_name_content_segment.clone(),
                    ],
                    jar: None, // File not needed at this point
                },
                USER_USER_PAT,
            )
            .await;
        assert_eq!(resp.status(), 200);

        // Get
        let project = api
            .get_project_deserialized_common("demo", USER_USER_PAT)
            .await;
        let id = project.id.to_string();

        // Remove the project
        let resp = test_env.api.remove_project("demo", USER_USER_PAT).await;
        assert_eq!(resp.status(), 204);

        // Confirm that the project is gone from the cache
        let mut redis_pool = test_env.db.redis_pool.connect().await.unwrap();
        assert_eq!(
            redis_pool
                .get(PROJECTS_SLUGS_NAMESPACE, "demo")
                .await
                .unwrap()
                .and_then(|x| x.parse::<i64>().ok()),
            None
        );
        assert_eq!(
            redis_pool
                .get(PROJECTS_SLUGS_NAMESPACE, &id)
                .await
                .unwrap()
                .and_then(|x| x.parse::<i64>().ok()),
            None
        );

        // Old slug no longer works
        let resp = api.get_project("demo", USER_USER_PAT).await;
        assert_eq!(resp.status(), 404);
    })
    .await;
}

#[actix_rt::test]
pub async fn test_patch_project() {
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        let api = &test_env.api;

        let alpha_project_slug = &test_env.dummy.as_ref().unwrap().project_alpha.project_slug;
        let beta_project_slug = &test_env.dummy.as_ref().unwrap().project_beta.project_slug;

        // First, we do some patch requests that should fail.
        // Failure because the user is not authorized.
        let resp = api
            .edit_project(
                alpha_project_slug,
                json!({
                    "name": "Test_Add_Project project - test 1",
                }),
                ENEMY_USER_PAT,
            )
            .await;
        assert_eq!(resp.status(), 401);

        // Failure because we are setting URL fields to invalid urls.
        for url_type in ["issues", "source", "wiki", "discord"] {
            let resp = api
                .edit_project(
                    alpha_project_slug,
                    json!({
                        "link_urls": {
                            url_type: "not a url",
                        },
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(resp.status(), 400);
        }

        // Failure because these are illegal requested statuses for a normal user.
        for req in ["unknown", "processing", "withheld", "scheduled"] {
            let resp = api
                .edit_project(
                    alpha_project_slug,
                    json!({
                        "requested_status": req,
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(resp.status(), 400);
        }

        // Failure because these should not be able to be set by a non-mod
        for key in ["moderation_message", "moderation_message_body"] {
            let resp = api
                .edit_project(
                    alpha_project_slug,
                    json!({
                        key: "test",
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(resp.status(), 401);

            // (should work for a mod, though)
            let resp = api
                .edit_project(
                    alpha_project_slug,
                    json!({
                        key: "test",
                    }),
                    MOD_USER_PAT,
                )
                .await;
            assert_eq!(resp.status(), 204);
        }

        // Failed patch to alpha slug:
        // - slug collision with beta
        // - too short slug
        // - too long slug
        // - not url safe slug
        // - not url safe slug
        for slug in [
            beta_project_slug,
            "a",
            &"a".repeat(100),
            "not url safe%&^!#$##!@#$%^&*()",
        ] {
            let resp = api
                .edit_project(
                    alpha_project_slug,
                    json!({
                        "slug": slug, // the other dummy project has this slug
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(resp.status(), 400);
        }

        // Not allowed to directly set status, as 'beta_project_slug' (the other project) is "processing" and cannot have its status changed like this.
        let resp = api
            .edit_project(
                beta_project_slug,
                json!({
                    "status": "private"
                }),
                USER_USER_PAT,
            )
            .await;
        assert_eq!(resp.status(), 401);

        // Sucessful request to patch many fields.
        let resp = api
            .edit_project(
                alpha_project_slug,
                json!({
                    "slug": "newslug",
                    "categories": [DUMMY_CATEGORIES[0]],
                    "license_id": "MIT",
                    "link_urls":
                        {
                            "patreon": "https://patreon.com",
                            "issues": "https://github.com",
                            "discord": "https://discord.gg",
                            "wiki": "https://wiki.com"
                        }
                }),
                USER_USER_PAT,
            )
            .await;
        assert_eq!(resp.status(), 204);

        // Old slug no longer works
        let resp = api.get_project(alpha_project_slug, USER_USER_PAT).await;
        assert_eq!(resp.status(), 404);

        // New slug does work
        let project = api.get_project_deserialized("newslug", USER_USER_PAT).await;

        assert_eq!(project.slug.unwrap(), "newslug");
        assert_eq!(project.categories, vec![DUMMY_CATEGORIES[0]]);
        assert_eq!(project.license.id, "MIT");

        let link_urls = project.link_urls;
        assert_eq!(link_urls.len(), 4);
        assert_eq!(link_urls["patreon"].platform, "patreon");
        assert_eq!(link_urls["patreon"].url, "https://patreon.com");
        assert!(link_urls["patreon"].donation);
        assert_eq!(link_urls["issues"].platform, "issues");
        assert_eq!(link_urls["issues"].url, "https://github.com");
        assert!(!link_urls["issues"].donation);
        assert_eq!(link_urls["discord"].platform, "discord");
        assert_eq!(link_urls["discord"].url, "https://discord.gg");
        assert!(!link_urls["discord"].donation);
        assert_eq!(link_urls["wiki"].platform, "wiki");
        assert_eq!(link_urls["wiki"].url, "https://wiki.com");
        assert!(!link_urls["wiki"].donation);

        // Unset the set link_urls
        let resp = api
            .edit_project(
                "newslug",
                json!({
                    "link_urls":
                        {
                            "issues": null,
                        }
                }),
                USER_USER_PAT,
            )
            .await;
        assert_eq!(resp.status(), 204);
        let project = api.get_project_deserialized("newslug", USER_USER_PAT).await;
        assert_eq!(project.link_urls.len(), 3);
        assert!(!project.link_urls.contains_key("issues"));
    })
    .await;
}

#[actix_rt::test]
pub async fn test_patch_v3() {
    // Hits V3-specific patchable fields
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        let api = &test_env.api;

        let alpha_project_slug = &test_env.dummy.as_ref().unwrap().project_alpha.project_slug;

        // Sucessful request to patch many fields.
        let resp = api
            .edit_project(
                alpha_project_slug,
                json!({
                    "name": "New successful title",
                    "summary": "New successful summary",
                    "description": "New successful description",
                }),
                USER_USER_PAT,
            )
            .await;
        assert_eq!(resp.status(), 204);

        let project = api
            .get_project_deserialized(alpha_project_slug, USER_USER_PAT)
            .await;

        assert_eq!(project.name, "New successful title");
        assert_eq!(project.summary, "New successful summary");
        assert_eq!(project.description, "New successful description");
    })
    .await;
}

#[actix_rt::test]
pub async fn test_bulk_edit_categories() {
    with_test_environment_all(None, |test_env| async move {
        let api = &test_env.api;
        let alpha_project_id: &str = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
        let beta_project_id: &str = &test_env.dummy.as_ref().unwrap().project_beta.project_id;

        let resp = api
            .edit_project_bulk(
                &[alpha_project_id, beta_project_id],
                json!({
                    "categories": [DUMMY_CATEGORIES[0], DUMMY_CATEGORIES[3]],
                    "add_categories": [DUMMY_CATEGORIES[1], DUMMY_CATEGORIES[2]],
                    "remove_categories": [DUMMY_CATEGORIES[3]],
                    "additional_categories": [DUMMY_CATEGORIES[4], DUMMY_CATEGORIES[6]],
                    "add_additional_categories": [DUMMY_CATEGORIES[5]],
                    "remove_additional_categories": [DUMMY_CATEGORIES[6]],
                }),
                ADMIN_USER_PAT,
            )
            .await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let alpha_body = api
            .get_project_deserialized_common(alpha_project_id, ADMIN_USER_PAT)
            .await;
        assert_eq!(alpha_body.categories, DUMMY_CATEGORIES[0..=2]);
        assert_eq!(alpha_body.additional_categories, DUMMY_CATEGORIES[4..=5]);

        let beta_body = api
            .get_project_deserialized_common(beta_project_id, ADMIN_USER_PAT)
            .await;
        assert_eq!(beta_body.categories, alpha_body.categories);
        assert_eq!(
            beta_body.additional_categories,
            alpha_body.additional_categories,
        );
    })
    .await;
}

#[actix_rt::test]
pub async fn test_bulk_edit_links() {
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        let api = &test_env.api;
        let alpha_project_id: &str = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
        let beta_project_id: &str = &test_env.dummy.as_ref().unwrap().project_beta.project_id;

        // Sets links for issue, source, wiki, and patreon for all projects
        // The first loop, sets issue, the second, clears it for all projects.
        for issues in [Some("https://www.issues.com"), None] {
            let resp = api
                .edit_project_bulk(
                    &[alpha_project_id, beta_project_id],
                    json!({
                        "link_urls": {
                            "issues": issues,
                            "wiki": "https://wiki.com",
                            "patreon": "https://patreon.com",
                        },
                    }),
                    ADMIN_USER_PAT,
                )
                .await;
            assert_eq!(resp.status(), StatusCode::NO_CONTENT);

            let alpha_body = api
                .get_project_deserialized(alpha_project_id, ADMIN_USER_PAT)
                .await;
            if let Some(issues) = issues {
                assert_eq!(alpha_body.link_urls.len(), 3);
                assert_eq!(alpha_body.link_urls["issues"].url, issues);
            } else {
                assert_eq!(alpha_body.link_urls.len(), 2);
                assert!(!alpha_body.link_urls.contains_key("issues"));
            }
            assert_eq!(alpha_body.link_urls["wiki"].url, "https://wiki.com");
            assert_eq!(alpha_body.link_urls["patreon"].url, "https://patreon.com");

            let beta_body = api
                .get_project_deserialized(beta_project_id, ADMIN_USER_PAT)
                .await;
            assert_eq!(beta_body.categories, alpha_body.categories);
            assert_eq!(
                beta_body.additional_categories,
                alpha_body.additional_categories,
            );
        }
    })
    .await;
}

#[actix_rt::test]
async fn delete_project_with_report() {
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        let api = &test_env.api;
        let alpha_project_id: &str = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
        let beta_project_id: &str = &test_env.dummy.as_ref().unwrap().project_beta.project_id;

        // Create a report for the project
        let resp = api
            .create_report(
                "copyright",
                alpha_project_id,
                CommonItemType::Project,
                "Hey! This is my project, copied without permission!",
                ENEMY_USER_PAT, // Enemy makes a report
            )
            .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let value = test::read_body_json::<serde_json::Value, _>(resp).await;
        let alpha_report_id = value["id"].as_str().unwrap();

        // Confirm existence
        let resp = api
            .get_report(
                alpha_report_id,
                ENEMY_USER_PAT, // Enemy makes a report
            )
            .await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Do the same for beta
        let resp = api
            .create_report(
                "copyright",
                beta_project_id,
                CommonItemType::Project,
                "Hey! This is my project, copied without permission!",
                ENEMY_USER_PAT, // Enemy makes a report
            )
            .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let value = test::read_body_json::<serde_json::Value, _>(resp).await;
        let beta_report_id = value["id"].as_str().unwrap();

        // Delete the project
        let resp = api.remove_project(alpha_project_id, USER_USER_PAT).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        // Confirm that the project is gone from the cache
        let mut redis_pool = test_env.db.redis_pool.connect().await.unwrap();
        assert_eq!(
            redis_pool
                .get(PROJECTS_SLUGS_NAMESPACE, "demo")
                .await
                .unwrap()
                .and_then(|x| x.parse::<i64>().ok()),
            None
        );
        assert_eq!(
            redis_pool
                .get(PROJECTS_SLUGS_NAMESPACE, alpha_project_id)
                .await
                .unwrap()
                .and_then(|x| x.parse::<i64>().ok()),
            None
        );

        // Report for alpha no longer exists
        let resp = api
            .get_report(
                alpha_report_id,
                ENEMY_USER_PAT, // Enemy makes a report
            )
            .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // Confirm that report for beta still exists
        let resp = api
            .get_report(
                beta_report_id,
                ENEMY_USER_PAT, // Enemy makes a report
            )
            .await;
        assert_eq!(resp.status(), StatusCode::OK);
    })
    .await;
}

#[actix_rt::test]
async fn permissions_patch_project_v3() {
    with_test_environment(Some(8), |test_env: TestEnvironment<ApiV3>| async move {
        let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
        let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;

        let api = &test_env.api;
        // TODO: This should be a separate test from v3
        // - only a couple of these fields are v3-specific
        // once we have permissions/scope tests setup to not just take closures, we can split this up

        // For each permission covered by EDIT_DETAILS, ensure the permission is required
        let edit_details = ProjectPermissions::EDIT_DETAILS;
        let test_pairs = [
            // Body, status, requested_status tested separately
            ("slug", json!("")), // generated in the test to not collide slugs
            ("name", json!("randomname")),
            ("description", json!("randomdescription")),
            ("categories", json!(["combat", "economy"])),
            ("additional_categories", json!(["decoration"])),
            (
                "links",
                json!({
                    "issues": "https://issues.com",
                    "source": "https://source.com",
                }),
            ),
            ("license_id", json!("MIT")),
        ];

        futures::stream::iter(test_pairs)
            .map(|(key, value)| {
                let test_env = test_env.clone();
                async move {
                    let req_gen = |ctx: PermissionsTestContext| {
                        let value = value.clone();
                        async move {
                            api.edit_project(
                                &ctx.project_id.unwrap(),
                                json!({
                                    key: if key == "slug" {
                                        json!(generate_random_name("randomslug"))
                                    } else {
                                        value.clone()
                                    },
                                }),
                                ctx.test_pat.as_deref(),
                            )
                            .await
                        }
                    };
                    PermissionsTest::new(&test_env)
                        .simple_project_permissions_test(edit_details, req_gen)
                        .await
                        .into_iter();
                }
            })
            .buffer_unordered(4)
            .collect::<Vec<_>>()
            .await;

        // Test with status and requested_status
        // This requires a project with a version, so we use alpha_project_id
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project(
                &ctx.project_id.unwrap(),
                json!({
                    "status": "private",
                    "requested_status": "private",
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();

        // Bulk patch projects
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project_bulk(
                &[&ctx.project_id.unwrap()],
                json!({
                    "name": "randomname",
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();

        // Edit body
        // Cannot bulk edit body
        let edit_body = ProjectPermissions::EDIT_BODY;
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project(
                &ctx.project_id.unwrap(),
                json!({
                    "description": "new description!",
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .simple_project_permissions_test(edit_body, req_gen)
            .await
            .unwrap();
    })
    .await;
}

// Not covered by PATCH /project
#[actix_rt::test]
async fn permissions_edit_details() {
    with_test_environment_all(None, |test_env| async move {
        let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
        let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;
        let beta_project_id = &test_env.dummy.as_ref().unwrap().project_beta.project_id;
        let beta_team_id = &test_env.dummy.as_ref().unwrap().project_beta.team_id;
        let beta_version_id = &test_env.dummy.as_ref().unwrap().project_beta.version_id;

        let edit_details = ProjectPermissions::EDIT_DETAILS;
        let api = &test_env.api;

        // Approve beta version as private so we can schedule it
        let req = test::TestRequest::patch()
            .uri(&format!("/v3/version/{beta_version_id}"))
            .append_pat(MOD_USER_PAT)
            .set_json(json!({
                "status": "unlisted"
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_eq!(resp.status(), 204);

        // Schedule version
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.schedule_project(
                beta_version_id,
                "archived",
                Utc::now() + Duration::days(1),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(beta_project_id, beta_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();

        // Icon edit
        // Uses alpha project to delete this icon
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project_icon(
                &ctx.project_id.unwrap(),
                Some(DummyImage::SmallIcon.get_icon_data()),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();

        // Icon delete
        // Uses alpha project to delete added icon
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project_icon(&ctx.project_id.unwrap(), None, ctx.test_pat.as_deref())
                .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();

        // Add gallery item
        // Uses alpha project to add gallery item so we can get its url
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.add_gallery_item(
                &ctx.project_id.unwrap(),
                DummyImage::SmallIcon.get_icon_data(),
                true,
                None,
                None,
                None,
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();
        // Get project, as we need the gallery image url
        let req = test::TestRequest::get()
            .uri(&format!("/v3/project/{alpha_project_id}"))
            .append_pat(USER_USER_PAT)
            .to_request();
        let resp = test_env.call(req).await;
        let project: serde_json::Value = test::read_body_json(resp).await;
        let gallery_url = project["gallery"][0]["url"].as_str().unwrap();

        // Edit gallery item
        // Uses alpha project to edit gallery item
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_gallery_item(
                &ctx.project_id.unwrap(),
                gallery_url,
                vec![("description".to_string(), "new caption!".to_string())]
                    .into_iter()
                    .collect(),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();

        // Remove gallery item
        // Uses alpha project to remove gallery item
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.remove_gallery_item(
                &ctx.project_id.unwrap(),
                gallery_url,
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();
    })
    .await;
}

#[actix_rt::test]
async fn permissions_upload_version() {
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
        let alpha_version_id = &test_env.dummy.as_ref().unwrap().project_alpha.version_id;
        let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;
        let alpha_file_hash = &test_env.dummy.as_ref().unwrap().project_alpha.file_hash;

        let api = &test_env.api;

        let upload_version = ProjectPermissions::UPLOAD_VERSION;
        // Upload version with basic-mod.jar
        let req_gen = |ctx: PermissionsTestContext| async move {
            let project_id = ctx.project_id.unwrap();
            let project_id = ProjectId(parse_base62(&project_id).unwrap());
            api.add_public_version(
                project_id,
                "1.0.0",
                TestFile::BasicMod,
                None,
                None,
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .simple_project_permissions_test(upload_version, req_gen)
            .await
            .unwrap();

        // Upload file to existing version
        // Uses alpha project, as it has an existing version
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.upload_file_to_version(
                alpha_version_id,
                &TestFile::BasicModDifferent,
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(upload_version, req_gen)
            .await
            .unwrap();

        // Patch version
        // Uses alpha project, as it has an existing version
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_version(
                alpha_version_id,
                json!({
                    "name": "Basic Mod",
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(upload_version, req_gen)
            .await
            .unwrap();

        // Delete version file
        // Uses alpha project, as it has an existing version
        let delete_version = ProjectPermissions::DELETE_VERSION;
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.remove_version_file(alpha_file_hash, ctx.test_pat.as_deref())
                .await
        };

        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(delete_version, req_gen)
            .await
            .unwrap();

        // Delete version
        // Uses alpha project, as it has an existing version
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.remove_version(alpha_version_id, ctx.test_pat.as_deref())
                .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(delete_version, req_gen)
            .await
            .unwrap();
    })
    .await;
}

#[actix_rt::test]
async fn permissions_manage_invites() {
    // Add member, remove member, edit member
    with_test_environment_all(None, |test_env| async move {
        let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
        let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;

        let api = &test_env.api;
        let manage_invites = ProjectPermissions::MANAGE_INVITES;

        // Add member
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.add_user_to_team(
                &ctx.team_id.unwrap(),
                MOD_USER_ID,
                Some(ProjectPermissions::empty()),
                None,
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(manage_invites, req_gen)
            .await
            .unwrap();

        // Edit member
        let edit_member = ProjectPermissions::EDIT_MEMBER;
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_team_member(
                &ctx.team_id.unwrap(),
                MOD_USER_ID,
                json!({
                    "permissions": 0,
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_member, req_gen)
            .await
            .unwrap();

        // remove member
        // requires manage_invites if they have not yet accepted the invite
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.remove_from_team(&ctx.team_id.unwrap(), MOD_USER_ID, ctx.test_pat.as_deref())
                .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(manage_invites, req_gen)
            .await
            .unwrap();

        // re-add member for testing
        let req = test::TestRequest::post()
            .uri(&format!("/v3/team/{}/members", alpha_team_id))
            .append_pat(ADMIN_USER_PAT)
            .set_json(json!({
                "user_id": MOD_USER_ID,
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_eq!(resp.status(), 204);

        // Accept invite
        let req = test::TestRequest::post()
            .uri(&format!("/v3/team/{}/join", alpha_team_id))
            .append_pat(MOD_USER_PAT)
            .to_request();
        let resp = test_env.call(req).await;
        assert_eq!(resp.status(), 204);

        // remove existing member (requires remove_member)
        let remove_member = ProjectPermissions::REMOVE_MEMBER;
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.remove_from_team(&ctx.team_id.unwrap(), MOD_USER_ID, ctx.test_pat.as_deref())
                .await
        };

        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(remove_member, req_gen)
            .await
            .unwrap();
    })
    .await;
}

#[actix_rt::test]
async fn permissions_delete_project() {
    // Add member, remove member, edit member
    with_test_environment_all(None, |test_env| async move {
        let delete_project = ProjectPermissions::DELETE_PROJECT;
        let api = &test_env.api;
        // Delete project
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.remove_project(&ctx.project_id.unwrap(), ctx.test_pat.as_deref())
                .await
        };
        PermissionsTest::new(&test_env)
            .simple_project_permissions_test(delete_project, req_gen)
            .await
            .unwrap();

        test_env.cleanup().await;
    })
    .await;
}

#[actix_rt::test]
async fn project_permissions_consistency_test() {
    with_test_environment_all(Some(10), |test_env| async move {
        // Test that the permissions are consistent with each other
        // For example, if we get the projectpermissions directly, from an organization's defaults, overriden, etc, they should all be correct & consistent
        let api = &test_env.api;
        // Full project permissions test with EDIT_DETAILS
        let success_permissions = ProjectPermissions::EDIT_DETAILS;
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project(
                &ctx.project_id.unwrap(),
                json!({
                    "categories": [],
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .full_project_permissions_test(success_permissions, req_gen)
            .await
            .unwrap();

        // We do a test with more specific permissions, to ensure that *exactly* the permissions at each step are as expected
        let success_permissions = ProjectPermissions::EDIT_DETAILS
            | ProjectPermissions::REMOVE_MEMBER
            | ProjectPermissions::DELETE_VERSION
            | ProjectPermissions::VIEW_PAYOUTS;
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project(
                &ctx.project_id.unwrap(),
                json!({
                    "categories": [],
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .full_project_permissions_test(success_permissions, req_gen)
            .await
            .unwrap();
    })
    .await;
}

#[actix_rt::test]
async fn align_search_projects() {
    // Test setup and dummy data
    with_test_environment(Some(10), |test_env: TestEnvironment<ApiV3>| async move {
        setup_search_projects(&test_env).await;

        let api = &test_env.api;
        let test_name = test_env.db.database_name.clone();

        let projects = api
            .search_deserialized(
                Some(&format!("\"&{test_name}\"")),
                Some(json!([["categories:fabric"]])),
                USER_USER_PAT,
            )
            .await;

        for mut project in projects.hits {
            let project_model = api
                .get_project(&project.id.to_string(), USER_USER_PAT)
                .await;
            assert_eq!(project_model.status(), 200);
            let mut project_model: Project = test::read_body_json(project_model).await;

            // Body/description is huge- don't store it in search, so it's OK if they differ here
            // (Search should return "")
            project_model.description = "".into();

            // Aggregate project loader fields will not match exactly,
            // because the search will only return the matching version, whereas the project returns the aggregate.
            // So, we remove them from both.
            project_model.fields = HashMap::new();
            project.fields = HashMap::new();

            let project_model = serde_json::to_value(project_model).unwrap();
            let searched_project_serialized = serde_json::to_value(project).unwrap();
            assert_eq!(project_model, searched_project_serialized);
        }
    })
    .await
}

// Route tests:
// TODO: Missing routes on projects
// TODO: using permissions/scopes, can we SEE projects existence that we are not allowed to? (ie 401 instead of 404)

// Permissions:
// TODO: permissions VIEW_PAYOUTS currently is unused. Add tests when it is used.
// TODO: permissions VIEW_ANALYTICS currently is unused. Add tests when it is used.

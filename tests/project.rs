use actix_http::StatusCode;
use actix_web::test;
use bytes::Bytes;
use chrono::{Duration, Utc};
use common::actix::MultipartSegment;
use common::environment::with_test_environment;
use common::permissions::{PermissionsTest, PermissionsTestContext};
use futures::StreamExt;
use labrinth::database::models::project_item::{PROJECTS_NAMESPACE, PROJECTS_SLUGS_NAMESPACE};
use labrinth::models::ids::base62_impl::parse_base62;
use labrinth::models::teams::ProjectPermissions;
use serde_json::json;

use crate::common::database::*;

use crate::common::dummy_data::DUMMY_CATEGORIES;
use crate::common::{actix::AppendsMultipart, environment::TestEnvironment};

// importing common module.
mod common;

#[actix_rt::test]
async fn test_get_project() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build(None).await;
    let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
    let beta_project_id = &test_env.dummy.as_ref().unwrap().project_beta.project_id;
    let alpha_project_slug = &test_env.dummy.as_ref().unwrap().project_alpha.project_slug;
    let alpha_version_id = &test_env.dummy.as_ref().unwrap().project_alpha.version_id;

    // Perform request on dummy data
    let req = test::TestRequest::get()
        .uri(&format!("/v2/project/{alpha_project_id}"))
        .append_header(("Authorization", USER_USER_PAT))
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
    assert_eq!(
        test_env
            .db
            .redis_pool
            .get::<i64, _>(PROJECTS_SLUGS_NAMESPACE, alpha_project_slug)
            .await
            .unwrap(),
        Some(parse_base62(alpha_project_id).unwrap() as i64)
    );

    let cached_project = test_env
        .db
        .redis_pool
        .get::<String, _>(PROJECTS_NAMESPACE, parse_base62(alpha_project_id).unwrap())
        .await
        .unwrap()
        .unwrap();
    let cached_project: serde_json::Value = serde_json::from_str(&cached_project).unwrap();
    assert_eq!(cached_project["inner"]["slug"], json!(alpha_project_slug));

    // Make the request again, this time it should be cached
    let req = test::TestRequest::get()
        .uri(&format!("/v2/project/{alpha_project_id}"))
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    let status = resp.status();
    assert_eq!(status, 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], json!(alpha_project_id));
    assert_eq!(body["slug"], json!(alpha_project_slug));

    // Request should fail on non-existent project
    let req = test::TestRequest::get()
        .uri("/v2/project/nonexistent")
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();

    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 404);

    // Similarly, request should fail on non-authorized user, on a yet-to-be-approved or hidden project, with a 404 (hiding the existence of the project)
    let req = test::TestRequest::get()
        .uri(&format!("/v2/project/{beta_project_id}"))
        .append_header(("Authorization", ENEMY_USER_PAT))
        .to_request();

    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 404);

    // Cleanup test db
    test_env.cleanup().await;
}

#[actix_rt::test]
async fn test_add_remove_project() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build(None).await;
    let api = &test_env.v2;

    // Generate test project data.
    let mut json_data = json!(
        {
            "title": "Test_Add_Project project",
            "slug": "demo",
            "description": "Example description.",
            "body": "Example body.",
            "client_side": "required",
            "server_side": "optional",
            "initial_versions": [{
                "file_parts": ["basic-mod.jar"],
                "version_number": "1.2.3",
                "version_title": "start",
                "dependencies": [],
                "game_versions": ["1.20.1"] ,
                "release_channel": "release",
                "loaders": ["fabric"],
                "featured": true
            }],
            "categories": [],
            "license_id": "MIT"
        }
    );

    // Basic json
    let json_segment = common::actix::MultipartSegment {
        name: "data".to_string(),
        filename: None,
        content_type: Some("application/json".to_string()),
        data: common::actix::MultipartSegmentData::Text(serde_json::to_string(&json_data).unwrap()),
    };

    // Basic json, with a different file
    json_data["initial_versions"][0]["file_parts"][0] = json!("basic-mod-different.jar");
    let json_diff_file_segment = common::actix::MultipartSegment {
        data: common::actix::MultipartSegmentData::Text(serde_json::to_string(&json_data).unwrap()),
        ..json_segment.clone()
    };

    // Basic json, with a different file, and a different slug
    json_data["slug"] = json!("new_demo");
    json_data["initial_versions"][0]["file_parts"][0] = json!("basic-mod-different.jar");
    let json_diff_slug_file_segment = common::actix::MultipartSegment {
        data: common::actix::MultipartSegmentData::Text(serde_json::to_string(&json_data).unwrap()),
        ..json_segment.clone()
    };

    // Basic file
    let file_segment = common::actix::MultipartSegment {
        name: "basic-mod.jar".to_string(),
        filename: Some("basic-mod.jar".to_string()),
        content_type: Some("application/java-archive".to_string()),
        data: common::actix::MultipartSegmentData::Binary(
            include_bytes!("../tests/files/basic-mod.jar").to_vec(),
        ),
    };

    // Differently named file, with the same content (for hash testing)
    let file_diff_name_segment = common::actix::MultipartSegment {
        name: "basic-mod-different.jar".to_string(),
        filename: Some("basic-mod-different.jar".to_string()),
        content_type: Some("application/java-archive".to_string()),
        data: common::actix::MultipartSegmentData::Binary(
            include_bytes!("../tests/files/basic-mod.jar").to_vec(),
        ),
    };

    // Differently named file, with different content
    let file_diff_name_content_segment = common::actix::MultipartSegment {
        name: "basic-mod-different.jar".to_string(),
        filename: Some("basic-mod-different.jar".to_string()),
        content_type: Some("application/java-archive".to_string()),
        data: common::actix::MultipartSegmentData::Binary(
            include_bytes!("../tests/files/basic-mod-different.jar").to_vec(),
        ),
    };

    // Add a project- simple, should work.
    let req = test::TestRequest::post()
        .uri("/v2/project")
        .append_header(("Authorization", USER_USER_PAT))
        .set_multipart(vec![json_segment.clone(), file_segment.clone()])
        .to_request();
    let resp = test_env.call(req).await;

    let status = resp.status();
    assert_eq!(status, 200);

    // Get the project we just made, and confirm that it's correct
    let project = api.get_project_deserialized("demo", USER_USER_PAT).await;
    assert!(project.versions.len() == 1);
    let uploaded_version_id = project.versions[0];

    // Checks files to ensure they were uploaded and correctly identify the file
    let hash = sha1::Sha1::from(include_bytes!("../tests/files/basic-mod.jar"))
        .digest()
        .to_string();
    let version = api
        .get_version_from_hash_deserialized(&hash, "sha1", USER_USER_PAT)
        .await;
    assert_eq!(version.id, uploaded_version_id);

    // Reusing with a different slug and the same file should fail
    // Even if that file is named differently
    let req = test::TestRequest::post()
        .uri("/v2/project")
        .append_header(("Authorization", USER_USER_PAT))
        .set_multipart(vec![
            json_diff_slug_file_segment.clone(), // Different slug, different file name
            file_diff_name_segment.clone(),      // Different file name, same content
        ])
        .to_request();

    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 400);

    // Reusing with the same slug and a different file should fail
    let req = test::TestRequest::post()
        .uri("/v2/project")
        .append_header(("Authorization", USER_USER_PAT))
        .set_multipart(vec![
            json_diff_file_segment.clone(), // Same slug, different file name
            file_diff_name_content_segment.clone(), // Different file name, different content
        ])
        .to_request();

    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 400);

    // Different slug, different file should succeed
    let req = test::TestRequest::post()
        .uri("/v2/project")
        .append_header(("Authorization", USER_USER_PAT))
        .set_multipart(vec![
            json_diff_slug_file_segment.clone(), // Different slug, different file name
            file_diff_name_content_segment.clone(), // Different file name, same content
        ])
        .to_request();

    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 200);

    // Get
    let project = api.get_project_deserialized("demo", USER_USER_PAT).await;
    let id = project.id.to_string();

    // Remove the project
    let resp = test_env.v2.remove_project("demo", USER_USER_PAT).await;
    assert_eq!(resp.status(), 204);

    // Confirm that the project is gone from the cache
    assert_eq!(
        test_env
            .db
            .redis_pool
            .get::<i64, _>(PROJECTS_SLUGS_NAMESPACE, "demo")
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        test_env
            .db
            .redis_pool
            .get::<i64, _>(PROJECTS_SLUGS_NAMESPACE, id)
            .await
            .unwrap(),
        None
    );

    // Old slug no longer works
    let resp = api.get_project("demo", USER_USER_PAT).await;
    assert_eq!(resp.status(), 404);

    // Cleanup test db
    test_env.cleanup().await;
}

#[actix_rt::test]
pub async fn test_patch_project() {
    let test_env = TestEnvironment::build(None).await;
    let api = &test_env.v2;

    let alpha_project_slug = &test_env.dummy.as_ref().unwrap().project_alpha.project_slug;
    let beta_project_slug = &test_env.dummy.as_ref().unwrap().project_beta.project_slug;

    // First, we do some patch requests that should fail.
    // Failure because the user is not authorized.
    let resp = api
        .edit_project(
            alpha_project_slug,
            json!({
                "title": "Test_Add_Project project - test 1",
            }),
            ENEMY_USER_PAT,
        )
        .await;
    assert_eq!(resp.status(), 401);

    // Failure because we are setting URL fields to invalid urls.
    for url_type in ["issues_url", "source_url", "wiki_url", "discord_url"] {
        let resp = api
            .edit_project(
                alpha_project_slug,
                json!({
                    url_type: "w.fake.url",
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
                "title": "New successful title",
                "description": "New successful description",
                "body": "New successful body",
                "categories": [DUMMY_CATEGORIES[0]],
                "license_id": "MIT",
                "issues_url": "https://github.com",
                "discord_url": "https://discord.gg",
                "wiki_url": "https://wiki.com",
                "client_side": "optional",
                "server_side": "required",
                "donation_urls": [{
                    "id": "patreon",
                    "platform": "Patreon",
                    "url": "https://patreon.com"
                }]
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
    assert_eq!(project.slug, Some("newslug".to_string()));
    assert_eq!(project.title, "New successful title");
    assert_eq!(project.description, "New successful description");
    assert_eq!(project.body, "New successful body");
    assert_eq!(project.categories, vec![DUMMY_CATEGORIES[0]]);
    assert_eq!(project.license.id, "MIT");
    assert_eq!(project.issues_url, Some("https://github.com".to_string()));
    assert_eq!(project.discord_url, Some("https://discord.gg".to_string()));
    assert_eq!(project.wiki_url, Some("https://wiki.com".to_string()));
    assert_eq!(project.client_side.to_string(), "optional");
    assert_eq!(project.server_side.to_string(), "required");
    assert_eq!(project.donation_urls.unwrap()[0].url, "https://patreon.com");

    // Cleanup test db
    test_env.cleanup().await;
}

#[actix_rt::test]
pub async fn test_bulk_edit_categories() {
    with_test_environment(|test_env| async move {
        let api = &test_env.v2;
        let alpha_project_id: &str = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
        let beta_project_id: &str = &test_env.dummy.as_ref().unwrap().project_beta.project_id;

        let resp = api
            .edit_project_bulk(
                [alpha_project_id, beta_project_id],
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
            .get_project_deserialized(alpha_project_id, ADMIN_USER_PAT)
            .await;
        assert_eq!(alpha_body.categories, DUMMY_CATEGORIES[0..=2]);
        assert_eq!(alpha_body.additional_categories, DUMMY_CATEGORIES[4..=5]);

        let beta_body = api
            .get_project_deserialized(beta_project_id, ADMIN_USER_PAT)
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
async fn permissions_patch_project() {
    let test_env = TestEnvironment::build(Some(8)).await;
    let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
    let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;

    // For each permission covered by EDIT_DETAILS, ensure the permission is required
    let edit_details = ProjectPermissions::EDIT_DETAILS;
    let test_pairs = [
        // Body, status, requested_status tested separately
        ("slug", json!("")), // generated in the test to not collide slugs
        ("title", json!("randomname")),
        ("description", json!("randomdescription")),
        ("categories", json!(["combat", "economy"])),
        ("client_side", json!("unsupported")),
        ("server_side", json!("unsupported")),
        ("additional_categories", json!(["decoration"])),
        ("issues_url", json!("https://issues.com")),
        ("source_url", json!("https://source.com")),
        ("wiki_url", json!("https://wiki.com")),
        (
            "donation_urls",
            json!([{
                "id": "paypal",
                "platform": "Paypal",
                "url": "https://paypal.com"
            }]),
        ),
        ("discord_url", json!("https://discord.com")),
        ("license_id", json!("MIT")),
    ];

    futures::stream::iter(test_pairs)
        .map(|(key, value)| {
            let test_env = test_env.clone();
            async move {
                let req_gen = |ctx: &PermissionsTestContext| {
                    test::TestRequest::patch()
                        .uri(&format!("/v2/project/{}", ctx.project_id.unwrap()))
                        .set_json(json!({
                            key: if key == "slug" {
                                json!(generate_random_name("randomslug"))
                            } else {
                                value.clone()
                            },
                        }))
                };

                PermissionsTest::new(&test_env)
                    .simple_project_permissions_test(edit_details, req_gen)
                    .await
                    .unwrap();
            }
        })
        .buffer_unordered(4)
        .collect::<Vec<_>>()
        .await;

    // Test with status and requested_status
    // This requires a project with a version, so we use alpha_project_id
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::patch()
            .uri(&format!("/v2/project/{}", ctx.project_id.unwrap()))
            .set_json(json!({
                "status": "private",
                "requested_status": "private",
            }))
    };
    PermissionsTest::new(&test_env)
        .with_existing_project(alpha_project_id, alpha_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(edit_details, req_gen)
        .await
        .unwrap();

    // Bulk patch projects
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::patch()
            .uri(&format!(
                "/v2/projects?ids=[{uri}]",
                uri = urlencoding::encode(&format!("\"{}\"", ctx.project_id.unwrap()))
            ))
            .set_json(json!({
                "name": "randomname",
            }))
    };
    PermissionsTest::new(&test_env)
        .simple_project_permissions_test(edit_details, req_gen)
        .await
        .unwrap();

    // Edit body
    // Cannot bulk edit body
    let edit_body = ProjectPermissions::EDIT_BODY;
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::patch()
            .uri(&format!("/v2/project/{}", ctx.project_id.unwrap()))
            .set_json(json!({
                "body": "new body!",
            }))
    };
    PermissionsTest::new(&test_env)
        .simple_project_permissions_test(edit_body, req_gen)
        .await
        .unwrap();

    test_env.cleanup().await;
}

// Not covered by PATCH /project
#[actix_rt::test]
async fn permissions_edit_details() {
    let test_env = TestEnvironment::build(None).await;

    let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
    let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;
    let beta_project_id = &test_env.dummy.as_ref().unwrap().project_beta.project_id;
    let beta_team_id = &test_env.dummy.as_ref().unwrap().project_beta.team_id;
    let beta_version_id = &test_env.dummy.as_ref().unwrap().project_beta.version_id;
    let edit_details = ProjectPermissions::EDIT_DETAILS;

    // Approve beta version as private so we can schedule it
    let req = test::TestRequest::patch()
        .uri(&format!("/v2/version/{beta_version_id}"))
        .append_header(("Authorization", MOD_USER_PAT))
        .set_json(json!({
            "status": "unlisted"
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // Schedule version
    let req_gen = |_: &PermissionsTestContext| {
        test::TestRequest::post()
            .uri(&format!("/v2/version/{beta_version_id}/schedule")) // beta_version_id is an *approved* version, so we can schedule it
            .set_json(json!(
                {
                    "requested_status": "archived",
                    "time": Utc::now() + Duration::days(1),
                }
            ))
    };
    PermissionsTest::new(&test_env)
        .with_existing_project(beta_project_id, beta_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(edit_details, req_gen)
        .await
        .unwrap();

    // Icon edit
    // Uses alpha project to delete this icon
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::patch()
            .uri(&format!(
                "/v2/project/{}/icon?ext=png",
                ctx.project_id.unwrap()
            ))
            .set_payload(Bytes::from(
                include_bytes!("../tests/files/200x200.png") as &[u8]
            ))
    };
    PermissionsTest::new(&test_env)
        .with_existing_project(alpha_project_id, alpha_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(edit_details, req_gen)
        .await
        .unwrap();

    // Icon delete
    // Uses alpha project to delete added icon
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::delete().uri(&format!(
            "/v2/project/{}/icon?ext=png",
            ctx.project_id.unwrap()
        ))
    };
    PermissionsTest::new(&test_env)
        .with_existing_project(alpha_project_id, alpha_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(edit_details, req_gen)
        .await
        .unwrap();

    // Add gallery item
    // Uses alpha project to add gallery item so we can get its url
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::post()
            .uri(&format!(
                "/v2/project/{}/gallery?ext=png&featured=true",
                ctx.project_id.unwrap()
            ))
            .set_payload(Bytes::from(
                include_bytes!("../tests/files/200x200.png") as &[u8]
            ))
    };
    PermissionsTest::new(&test_env)
        .with_existing_project(alpha_project_id, alpha_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(edit_details, req_gen)
        .await
        .unwrap();
    // Get project, as we need the gallery image url
    let req = test::TestRequest::get()
        .uri(&format!("/v2/project/{alpha_project_id}"))
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    let project: serde_json::Value = test::read_body_json(resp).await;
    let gallery_url = project["gallery"][0]["url"].as_str().unwrap();

    // Edit gallery item
    // Uses alpha project to edit gallery item
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::patch().uri(&format!(
            "/v2/project/{}/gallery?url={gallery_url}",
            ctx.project_id.unwrap()
        ))
    };
    PermissionsTest::new(&test_env)
        .with_existing_project(alpha_project_id, alpha_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(edit_details, req_gen)
        .await
        .unwrap();

    // Remove gallery item
    // Uses alpha project to remove gallery item
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::delete().uri(&format!(
            "/v2/project/{}/gallery?url={gallery_url}",
            ctx.project_id.unwrap()
        ))
    };
    PermissionsTest::new(&test_env)
        .with_existing_project(alpha_project_id, alpha_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(edit_details, req_gen)
        .await
        .unwrap();
}

#[actix_rt::test]
async fn permissions_upload_version() {
    let test_env = TestEnvironment::build(None).await;
    let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
    let alpha_version_id = &test_env.dummy.as_ref().unwrap().project_alpha.version_id;
    let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;
    let alpha_file_hash = &test_env.dummy.as_ref().unwrap().project_alpha.file_hash;

    let upload_version = ProjectPermissions::UPLOAD_VERSION;

    // Upload version with basic-mod.jar
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::post().uri("/v2/version").set_multipart([
            MultipartSegment {
                name: "data".to_string(),
                filename: None,
                content_type: Some("application/json".to_string()),
                data: common::actix::MultipartSegmentData::Text(
                    serde_json::to_string(&json!({
                        "project_id": ctx.project_id.unwrap(),
                        "file_parts": ["basic-mod.jar"],
                        "version_number": "1.0.0",
                        "version_title": "1.0.0",
                        "version_type": "release",
                        "dependencies": [],
                        "game_versions": ["1.20.1"],
                        "loaders": ["fabric"],
                        "featured": false,

                    }))
                    .unwrap(),
                ),
            },
            MultipartSegment {
                name: "basic-mod.jar".to_string(),
                filename: Some("basic-mod.jar".to_string()),
                content_type: Some("application/java-archive".to_string()),
                data: common::actix::MultipartSegmentData::Binary(
                    include_bytes!("../tests/files/basic-mod.jar").to_vec(),
                ),
            },
        ])
    };
    PermissionsTest::new(&test_env)
        .simple_project_permissions_test(upload_version, req_gen)
        .await
        .unwrap();

    // Upload file to existing version
    // Uses alpha project, as it has an existing version
    let req_gen = |_: &PermissionsTestContext| {
        test::TestRequest::post()
            .uri(&format!("/v2/version/{}/file", alpha_version_id))
            .set_multipart([
                MultipartSegment {
                    name: "data".to_string(),
                    filename: None,
                    content_type: Some("application/json".to_string()),
                    data: common::actix::MultipartSegmentData::Text(
                        serde_json::to_string(&json!({
                            "file_parts": ["basic-mod-different.jar"],
                        }))
                        .unwrap(),
                    ),
                },
                MultipartSegment {
                    name: "basic-mod-different.jar".to_string(),
                    filename: Some("basic-mod-different.jar".to_string()),
                    content_type: Some("application/java-archive".to_string()),
                    data: common::actix::MultipartSegmentData::Binary(
                        include_bytes!("../tests/files/basic-mod-different.jar").to_vec(),
                    ),
                },
            ])
    };
    PermissionsTest::new(&test_env)
        .with_existing_project(alpha_project_id, alpha_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(upload_version, req_gen)
        .await
        .unwrap();

    // Patch version
    // Uses alpha project, as it has an existing version
    let req_gen = |_: &PermissionsTestContext| {
        test::TestRequest::patch()
            .uri(&format!("/v2/version/{}", alpha_version_id))
            .set_json(json!({
                "name": "Basic Mod",
            }))
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
    let req_gen = |_: &PermissionsTestContext| {
        test::TestRequest::delete().uri(&format!("/v2/version_file/{}", alpha_file_hash))
    };

    PermissionsTest::new(&test_env)
        .with_existing_project(alpha_project_id, alpha_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(delete_version, req_gen)
        .await
        .unwrap();

    // Delete version
    // Uses alpha project, as it has an existing version
    let req_gen = |_: &PermissionsTestContext| {
        test::TestRequest::delete().uri(&format!("/v2/version/{}", alpha_version_id))
    };
    PermissionsTest::new(&test_env)
        .with_existing_project(alpha_project_id, alpha_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(delete_version, req_gen)
        .await
        .unwrap();

    test_env.cleanup().await;
}

#[actix_rt::test]
async fn permissions_manage_invites() {
    // Add member, remove member, edit member
    let test_env = TestEnvironment::build(None).await;
    let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
    let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;

    let manage_invites = ProjectPermissions::MANAGE_INVITES;

    // Add member
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::post()
            .uri(&format!("/v2/team/{}/members", ctx.team_id.unwrap()))
            .set_json(json!({
                "user_id": MOD_USER_ID,
                "permissions": 0,
            }))
    };
    PermissionsTest::new(&test_env)
        .with_existing_project(alpha_project_id, alpha_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(manage_invites, req_gen)
        .await
        .unwrap();

    // Edit member
    let edit_member = ProjectPermissions::EDIT_MEMBER;
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::patch()
            .uri(&format!(
                "/v2/team/{}/members/{MOD_USER_ID}",
                ctx.team_id.unwrap()
            ))
            .set_json(json!({
                "permissions": 0,
            }))
    };
    PermissionsTest::new(&test_env)
        .with_existing_project(alpha_project_id, alpha_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(edit_member, req_gen)
        .await
        .unwrap();

    // remove member
    // requires manage_invites if they have not yet accepted the invite
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::delete().uri(&format!(
            "/v2/team/{}/members/{MOD_USER_ID}",
            ctx.team_id.unwrap()
        ))
    };
    PermissionsTest::new(&test_env)
        .with_existing_project(alpha_project_id, alpha_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(manage_invites, req_gen)
        .await
        .unwrap();

    // re-add member for testing
    let req = test::TestRequest::post()
        .uri(&format!("/v2/team/{}/members", alpha_team_id))
        .append_header(("Authorization", ADMIN_USER_PAT))
        .set_json(json!({
            "user_id": MOD_USER_ID,
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // Accept invite
    let req = test::TestRequest::post()
        .uri(&format!("/v2/team/{}/join", alpha_team_id))
        .append_header(("Authorization", MOD_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // remove existing member (requires remove_member)
    let remove_member = ProjectPermissions::REMOVE_MEMBER;
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::delete().uri(&format!(
            "/v2/team/{}/members/{MOD_USER_ID}",
            ctx.team_id.unwrap()
        ))
    };

    PermissionsTest::new(&test_env)
        .with_existing_project(alpha_project_id, alpha_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_project_permissions_test(remove_member, req_gen)
        .await
        .unwrap();

    test_env.cleanup().await;
}

#[actix_rt::test]
async fn permissions_delete_project() {
    // Add member, remove member, edit member
    let test_env = TestEnvironment::build(None).await;

    let delete_project = ProjectPermissions::DELETE_PROJECT;

    // Delete project
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::delete().uri(&format!("/v2/project/{}", ctx.project_id.unwrap()))
    };
    PermissionsTest::new(&test_env)
        .simple_project_permissions_test(delete_project, req_gen)
        .await
        .unwrap();

    test_env.cleanup().await;
}

#[actix_rt::test]
async fn project_permissions_consistency_test() {
    let test_env = TestEnvironment::build(Some(8)).await;

    // Test that the permissions are consistent with each other
    // For example, if we get the projectpermissions directly, from an organization's defaults, overriden, etc, they should all be correct & consistent

    // Full project permissions test with EDIT_DETAILS
    let success_permissions = ProjectPermissions::EDIT_DETAILS;
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::patch()
            .uri(&format!("/v2/project/{}", ctx.project_id.unwrap()))
            .set_json(json!({
                "title": "Example title - changed.",
            }))
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
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::patch()
            .uri(&format!("/v2/project/{}", ctx.project_id.unwrap()))
            .set_json(json!({
                "title": "Example title - changed.",
            }))
    };
    PermissionsTest::new(&test_env)
        .full_project_permissions_test(success_permissions, req_gen)
        .await
        .unwrap();

    test_env.cleanup().await;
}

// Route tests:
// TODO: Missing routes on projects
// TODO: using permissions/scopes, can we SEE projects existence that we are not allowed to? (ie 401 instead of 404)

// Permissions:
// TODO: permissions VIEW_PAYOUTS currently is unused. Add tests when it is used.
// TODO: permissions VIEW_ANALYTICS currently is unused. Add tests when it is used.

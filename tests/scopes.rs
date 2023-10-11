use actix_web::test::{self, TestRequest};
use bytes::Bytes;
use chrono::{Duration, Utc};
use common::actix::AppendsMultipart;
use labrinth::models::pats::Scopes;
use serde_json::json;

use crate::common::{database::*, environment::TestEnvironment, scopes::ScopeTest};

// importing common module.
mod common;

// For each scope, we (using test_scope):
// - create a PAT with a given set of scopes for a function
// - create a PAT with all other scopes for a function
// - test the function with the PAT with the given scopes
// - test the function with the PAT with all other scopes

// Test for users, emails, and payout scopes (not user auth scope or notifs)
#[actix_rt::test]
async fn user_scopes() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build_with_dummy().await;

    // User reading
    let read_user = Scopes::USER_READ;
    let req_gen = || TestRequest::get().uri("/v2/user");
    let (_, success) = ScopeTest::new(&test_env)
        .test(req_gen, read_user)
        .await
        .unwrap();
    assert!(success["email"].as_str().is_none()); // email should not be present
    assert!(success["payout_data"].as_object().is_none()); // payout should not be present

    // Email reading
    let read_email = Scopes::USER_READ | Scopes::USER_READ_EMAIL;
    let req_gen = || TestRequest::get().uri("/v2/user");
    let (_, success) = ScopeTest::new(&test_env)
        .test(req_gen, read_email)
        .await
        .unwrap();
    assert_eq!(success["email"], json!("user@modrinth.com")); // email should be present

    // Payout reading
    let read_payout = Scopes::USER_READ | Scopes::PAYOUTS_READ;
    let req_gen = || TestRequest::get().uri("/v2/user");
    let (_, success) = ScopeTest::new(&test_env)
        .test(req_gen, read_payout)
        .await
        .unwrap();
    assert!(success["payout_data"].as_object().is_some()); // payout should be present

    // User writing
    // We use the Admin PAT for this test, on the 'user' user
    let write_user = Scopes::USER_WRITE;
    let req_gen = || {
        TestRequest::patch().uri("/v2/user/user").set_json(json!( {
            // Do not include 'username', as to not change the rest of the tests
            "name": "NewName",
            "bio": "New bio",
            "location": "New location",
            "role": "admin",
            "badges": 5,
            // Do not include payout info, different scope
        }))
    };
    ScopeTest::new(&test_env)
        .with_user_id(ADMIN_USER_ID_PARSED)
        .test(req_gen, write_user)
        .await
        .unwrap();

    // User deletion
    // (The failure is first, and this is the last test for this test function, we can delete it and use the same PAT for both tests)
    let delete_user = Scopes::USER_DELETE;
    let req_gen = || TestRequest::delete().uri("/v2/user/enemy");
    ScopeTest::new(&test_env)
        .with_user_id(ENEMY_USER_ID_PARSED)
        .test(req_gen, delete_user)
        .await
        .unwrap();

    // Cleanup test db
    test_env.cleanup().await;
}

// Notifications
#[actix_rt::test]
pub async fn notifications_scopes() {
    let test_env = TestEnvironment::build_with_dummy().await;
    let alpha_team_id = &test_env.dummy.as_ref().unwrap().alpha_team_id.clone();

    // We will invite user 'friend' to project team, and use that as a notification
    // Get notifications
    let req = TestRequest::post()
        .uri(&format!("/v2/team/{alpha_team_id}/members"))
        .append_header(("Authorization", USER_USER_PAT))
        .set_json(json!( {
            "user_id": FRIEND_USER_ID // friend
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // Notification get
    let read_notifications = Scopes::NOTIFICATION_READ;
    let req_gen =
        || test::TestRequest::get().uri(&format!("/v2/user/{FRIEND_USER_ID}/notifications"));
    let (_, success) = ScopeTest::new(&test_env)
        .with_user_id(FRIEND_USER_ID_PARSED)
        .test(req_gen, read_notifications)
        .await
        .unwrap();
    let notification_id = success.as_array().unwrap()[0]["id"].as_str().unwrap();

    let req_gen = || {
        test::TestRequest::get().uri(&format!(
            "/v2/notifications?ids=[{uri}]",
            uri = urlencoding::encode(&format!("\"{notification_id}\""))
        ))
    };
    ScopeTest::new(&test_env)
        .with_user_id(FRIEND_USER_ID_PARSED)
        .test(req_gen, read_notifications)
        .await
        .unwrap();

    let req_gen = || test::TestRequest::get().uri(&format!("/v2/notification/{notification_id}"));
    ScopeTest::new(&test_env)
        .with_user_id(FRIEND_USER_ID_PARSED)
        .test(req_gen, read_notifications)
        .await
        .unwrap();

    // Notification mark as read
    let write_notifications = Scopes::NOTIFICATION_WRITE;
    let req_gen = || {
        test::TestRequest::patch().uri(&format!(
            "/v2/notifications?ids=[{uri}]",
            uri = urlencoding::encode(&format!("\"{notification_id}\""))
        ))
    };
    ScopeTest::new(&test_env)
        .with_user_id(FRIEND_USER_ID_PARSED)
        .test(req_gen, write_notifications)
        .await
        .unwrap();

    let req_gen = || test::TestRequest::patch().uri(&format!("/v2/notification/{notification_id}"));
    ScopeTest::new(&test_env)
        .with_user_id(FRIEND_USER_ID_PARSED)
        .test(req_gen, write_notifications)
        .await
        .unwrap();

    // Notification delete
    let req_gen =
        || test::TestRequest::delete().uri(&format!("/v2/notification/{notification_id}"));
    ScopeTest::new(&test_env)
        .with_user_id(FRIEND_USER_ID_PARSED)
        .test(req_gen, write_notifications)
        .await
        .unwrap();

    // Mass notification delete
    // We invite mod, get the notification ID, and do mass delete using that
    let req = test::TestRequest::post()
        .uri(&format!("/v2/team/{alpha_team_id}/members"))
        .append_header(("Authorization", USER_USER_PAT))
        .set_json(json!( {
            "user_id": MOD_USER_ID // mod
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);
    let read_notifications = Scopes::NOTIFICATION_READ;
    let req_gen = || test::TestRequest::get().uri(&format!("/v2/user/{MOD_USER_ID}/notifications"));
    let (_, success) = ScopeTest::new(&test_env)
        .with_user_id(MOD_USER_ID_PARSED)
        .test(req_gen, read_notifications)
        .await
        .unwrap();
    let notification_id = success.as_array().unwrap()[0]["id"].as_str().unwrap();

    let req_gen = || {
        test::TestRequest::delete().uri(&format!(
            "/v2/notifications?ids=[{uri}]",
            uri = urlencoding::encode(&format!("\"{notification_id}\""))
        ))
    };
    ScopeTest::new(&test_env)
        .with_user_id(MOD_USER_ID_PARSED)
        .test(req_gen, write_notifications)
        .await
        .unwrap();

    // Cleanup test db
    test_env.cleanup().await;
}

// Project version creation scopes
#[actix_rt::test]
pub async fn project_version_create_scopes() {
    let test_env = TestEnvironment::build_with_dummy().await;

    // Create project
    let create_project = Scopes::PROJECT_CREATE;
    let json_data = json!(
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
    let json_segment = common::actix::MultipartSegment {
        name: "data".to_string(),
        filename: None,
        content_type: Some("application/json".to_string()),
        data: common::actix::MultipartSegmentData::Text(serde_json::to_string(&json_data).unwrap()),
    };
    let file_segment = common::actix::MultipartSegment {
        name: "basic-mod.jar".to_string(),
        filename: Some("basic-mod.jar".to_string()),
        content_type: Some("application/java-archive".to_string()),
        data: common::actix::MultipartSegmentData::Binary(
            include_bytes!("../tests/files/basic-mod.jar").to_vec(),
        ),
    };

    let req_gen = || {
        test::TestRequest::post()
            .uri("/v2/project")
            .set_multipart(vec![json_segment.clone(), file_segment.clone()])
    };
    let (_, success) = ScopeTest::new(&test_env)
        .test(req_gen, create_project)
        .await
        .unwrap();
    let project_id = success["id"].as_str().unwrap();

    // Add version to project
    let create_version = Scopes::VERSION_CREATE;
    let json_data = json!(
            {
                "project_id": project_id,
                "file_parts": ["basic-mod-different.jar"],
                "version_number": "1.2.3.4",
                "version_title": "start",
                "dependencies": [],
                "game_versions": ["1.20.1"] ,
                "release_channel": "release",
                "loaders": ["fabric"],
                "featured": true
            }
    );
    let json_segment = common::actix::MultipartSegment {
        name: "data".to_string(),
        filename: None,
        content_type: Some("application/json".to_string()),
        data: common::actix::MultipartSegmentData::Text(serde_json::to_string(&json_data).unwrap()),
    };
    let file_segment = common::actix::MultipartSegment {
        name: "basic-mod-different.jar".to_string(),
        filename: Some("basic-mod.jar".to_string()),
        content_type: Some("application/java-archive".to_string()),
        data: common::actix::MultipartSegmentData::Binary(
            include_bytes!("../tests/files/basic-mod-different.jar").to_vec(),
        ),
    };

    let req_gen = || {
        test::TestRequest::post()
            .uri("/v2/version")
            .set_multipart(vec![json_segment.clone(), file_segment.clone()])
    };
    ScopeTest::new(&test_env)
        .test(req_gen, create_version)
        .await
        .unwrap();

    // Cleanup test db
    test_env.cleanup().await;
}

// Project management scopes
#[actix_rt::test]
pub async fn project_version_reads_scopes() {
    let test_env = TestEnvironment::build_with_dummy().await;
    let beta_project_id = &test_env.dummy.as_ref().unwrap().beta_project_id.clone();
    let beta_version_id = &test_env.dummy.as_ref().unwrap().beta_version_id.clone();
    let alpha_team_id = &test_env.dummy.as_ref().unwrap().alpha_team_id.clone();
    let beta_file_hash = &test_env.dummy.as_ref().unwrap().beta_file_hash.clone();

    // Project reading
    // Uses 404 as the expected failure code (or 200 and an empty list for mass reads)
    let read_project = Scopes::PROJECT_READ;
    let req_gen = || test::TestRequest::get().uri(&format!("/v2/project/{beta_project_id}"));
    ScopeTest::new(&test_env)
        .with_failure_code(404)
        .test(req_gen, read_project)
        .await
        .unwrap();

    let req_gen =
        || test::TestRequest::get().uri(&format!("/v2/project/{beta_project_id}/dependencies"));
    ScopeTest::new(&test_env)
        .with_failure_code(404)
        .test(req_gen, read_project)
        .await
        .unwrap();

    let req_gen = || {
        test::TestRequest::get().uri(&format!(
            "/v2/projects?ids=[{uri}]",
            uri = urlencoding::encode(&format!("\"{beta_project_id}\""))
        ))
    };
    let (failure, success) = ScopeTest::new(&test_env)
        .with_failure_code(200)
        .test(req_gen, read_project)
        .await
        .unwrap();
    assert!(failure.as_array().unwrap().is_empty());
    assert!(!success.as_array().unwrap().is_empty());

    // Team project reading
    let req_gen =
        || test::TestRequest::get().uri(&format!("/v2/project/{beta_project_id}/members"));
    ScopeTest::new(&test_env)
        .with_failure_code(404)
        .test(req_gen, read_project)
        .await
        .unwrap();

    // Get team members
    // In this case, as these are public endpoints, logging in only is relevant to showing permissions
    // So for our test project (with 1 user, 'user') we will check the permissions before and after having the scope.
    let req_gen = || test::TestRequest::get().uri(&format!("/v2/team/{alpha_team_id}/members"));
    let (failure, success) = ScopeTest::new(&test_env)
        .with_failure_code(200)
        .test(req_gen, read_project)
        .await
        .unwrap();
    assert!(!failure.as_array().unwrap()[0].as_object().unwrap()["permissions"].is_number());
    assert!(success.as_array().unwrap()[0].as_object().unwrap()["permissions"].is_number());

    let req_gen = || {
        test::TestRequest::get().uri(&format!(
            "/v2/teams?ids=[{uri}]",
            uri = urlencoding::encode(&format!("\"{alpha_team_id}\""))
        ))
    };
    let (failure, success) = ScopeTest::new(&test_env)
        .with_failure_code(200)
        .test(req_gen, read_project)
        .await
        .unwrap();
    assert!(!failure.as_array().unwrap()[0].as_array().unwrap()[0]
        .as_object()
        .unwrap()["permissions"]
        .is_number());
    assert!(success.as_array().unwrap()[0].as_array().unwrap()[0]
        .as_object()
        .unwrap()["permissions"]
        .is_number());

    // User project reading
    // Test user has two projects, one public and one private
    let req_gen = || test::TestRequest::get().uri(&format!("/v2/user/{USER_USER_ID}/projects"));
    let (failure, success) = ScopeTest::new(&test_env)
        .with_failure_code(200)
        .test(req_gen, read_project)
        .await
        .unwrap();
    assert!(!failure
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["status"] == "processing"));
    assert!(success
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["status"] == "processing"));

    // Project metadata reading
    let req_gen = || {
        test::TestRequest::get().uri(&format!(
            "/maven/maven/modrinth/{beta_project_id}/maven-metadata.xml"
        ))
    };
    ScopeTest::new(&test_env)
        .with_failure_code(404)
        .test(req_gen, read_project)
        .await
        .unwrap();

    // Version reading
    // First, set version to hidden (which is when the scope is required to read it)
    let read_version = Scopes::VERSION_READ;
    let req = test::TestRequest::patch()
        .uri(&format!("/v2/version/{beta_version_id}"))
        .append_header(("Authorization", USER_USER_PAT))
        .set_json(json!({
            "status": "draft"
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    let req_gen = || test::TestRequest::get().uri(&format!("/v2/version_file/{beta_file_hash}"));
    ScopeTest::new(&test_env)
        .with_failure_code(404)
        .test(req_gen, read_version)
        .await
        .unwrap();

    let req_gen =
        || test::TestRequest::get().uri(&format!("/v2/version_file/{beta_file_hash}/download"));
    ScopeTest::new(&test_env)
        .with_failure_code(404)
        .test(req_gen, read_version)
        .await
        .unwrap();

    // TODO: Should this be /POST? Looks like /GET
    // TODO: this scope doesn't actually affect anything, because the Project::get_id contained within disallows hidden versions, which is the point of this scope
    // let req_gen = || {
    //     test::TestRequest::post()
    //     .uri(&format!("/v2/version_file/{beta_file_hash}/update"))
    //     .set_json(json!({}))
    // };
    // ScopeTest::new(&test_env).with_failure_code(404).test(req_gen, read_version).await.unwrap();

    // TODO: Should this be /POST? Looks like /GET
    let req_gen = || {
        test::TestRequest::post()
            .uri("/v2/version_files")
            .set_json(json!({
                "hashes": [beta_file_hash]
            }))
    };
    let (failure, success) = ScopeTest::new(&test_env)
        .with_failure_code(200)
        .test(req_gen, read_version)
        .await
        .unwrap();
    assert!(!failure.as_object().unwrap().contains_key(beta_file_hash));
    assert!(success.as_object().unwrap().contains_key(beta_file_hash));

    // Update version file
    // TODO: Should this be /POST? Looks like /GET
    // TODO: this scope doesn't actually affect anything, because the Project::get_id contained within disallows hidden versions, which is the point of this scope

    // let req_gen = || {
    //     test::TestRequest::post()
    //     .uri(&format!("/v2/version_files/update_individual"))
    //     .set_json(json!({
    //         "hashes": [{
    //             "hash": beta_file_hash,
    //         }]
    //     }))
    // };
    // let (failure, success) = ScopeTest::new(&test_env).with_failure_code(200).test(req_gen, read_version).await.unwrap();
    // assert!(!failure.as_object().unwrap().contains_key(beta_file_hash));
    // assert!(success.as_object().unwrap().contains_key(beta_file_hash));

    // Update version file
    // TODO: this scope doesn't actually affect anything, because the Project::get_id contained within disallows hidden versions, which is the point of this scope
    // let req_gen = || {
    //     test::TestRequest::post()
    //     .uri(&format!("/v2/version_files/update"))
    //     .set_json(json!({
    //         "hashes": [beta_file_hash]
    //     }))
    // };
    // let (failure, success) = ScopeTest::new(&test_env).with_failure_code(200).test(req_gen, read_version).await.unwrap();
    // assert!(!failure.as_object().unwrap().contains_key(beta_file_hash));
    // assert!(success.as_object().unwrap().contains_key(beta_file_hash));

    // Both project and version reading
    let read_project_and_version = Scopes::PROJECT_READ | Scopes::VERSION_READ;
    let req_gen =
        || test::TestRequest::get().uri(&format!("/v2/project/{beta_project_id}/version"));
    ScopeTest::new(&test_env)
        .with_failure_code(404)
        .test(req_gen, read_project_and_version)
        .await
        .unwrap();

    // TODO: fails for the same reason as above
    // let req_gen = || {
    //     test::TestRequest::get()
    //     .uri(&format!("/v2/project/{beta_project_id}/version/{beta_version_id}"))
    // };
    // ScopeTest::new(&test_env).with_failure_code(404).test(req_gen, read_project_and_version).await.unwrap();

    // Cleanup test db
    test_env.cleanup().await;
}

// Project writing
#[actix_rt::test]
pub async fn project_write_scopes() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build_with_dummy().await;
    let beta_project_id = &test_env.dummy.as_ref().unwrap().beta_project_id.clone();
    let alpha_team_id = &test_env.dummy.as_ref().unwrap().alpha_team_id.clone();

    // Projects writing
    let write_project = Scopes::PROJECT_WRITE;
    let req_gen = || {
        test::TestRequest::patch()
            .uri(&format!("/v2/project/{beta_project_id}"))
            .set_json(json!(
                {
                    "title": "test_project_version_write_scopes Title",
                }
            ))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_project)
        .await
        .unwrap();

    let req_gen = || {
        test::TestRequest::patch()
            .uri(&format!(
                "/v2/projects?ids=[{uri}]",
                uri = urlencoding::encode(&format!("\"{beta_project_id}\""))
            ))
            .set_json(json!(
                {
                    "description": "test_project_version_write_scopes Description",
                }
            ))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_project)
        .await
        .unwrap();

    // Approve beta as private so we can schedule it
    let req = test::TestRequest::patch()
        .uri(&format!("/v2/project/{beta_project_id}"))
        .append_header(("Authorization", MOD_USER_PAT))
        .set_json(json!({
            "status": "private"
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    let req_gen = || {
        test::TestRequest::post()
            .uri(&format!("/v2/project/{beta_project_id}/schedule")) // beta_project_id is an unpublished can schedule it
            .set_json(json!(
                {
                    "requested_status": "private",
                    "time": Utc::now() + Duration::days(1),
                }
            ))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_project)
        .await
        .unwrap();

    // Icons and gallery images
    let req_gen = || {
        test::TestRequest::patch()
            .uri(&format!("/v2/project/{beta_project_id}/icon?ext=png"))
            .set_payload(Bytes::from(
                include_bytes!("../tests/files/200x200.png") as &[u8]
            ))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_project)
        .await
        .unwrap();

    let req_gen =
        || test::TestRequest::delete().uri(&format!("/v2/project/{beta_project_id}/icon"));
    ScopeTest::new(&test_env)
        .test(req_gen, write_project)
        .await
        .unwrap();

    let req_gen = || {
        test::TestRequest::post()
            .uri(&format!(
                "/v2/project/{beta_project_id}/gallery?ext=png&featured=true"
            ))
            .set_payload(Bytes::from(
                include_bytes!("../tests/files/200x200.png") as &[u8]
            ))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_project)
        .await
        .unwrap();

    // Get project, as we need the gallery image url
    let req_gen = test::TestRequest::get()
        .uri(&format!("/v2/project/{beta_project_id}"))
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req_gen).await;
    let project: serde_json::Value = test::read_body_json(resp).await;
    let gallery_url = project["gallery"][0]["url"].as_str().unwrap();

    let req_gen = || {
        test::TestRequest::patch().uri(&format!(
            "/v2/project/{beta_project_id}/gallery?url={gallery_url}"
        ))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_project)
        .await
        .unwrap();

    let req_gen = || {
        test::TestRequest::delete().uri(&format!(
            "/v2/project/{beta_project_id}/gallery?url={gallery_url}"
        ))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_project)
        .await
        .unwrap();

    // Team scopes - add user 'friend'
    let req_gen = || {
        test::TestRequest::post()
            .uri(&format!("/v2/team/{alpha_team_id}/members"))
            .set_json(json!({
                "user_id": FRIEND_USER_ID
            }))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_project)
        .await
        .unwrap();

    // Accept team invite as 'friend'
    let req_gen = || test::TestRequest::post().uri(&format!("/v2/team/{alpha_team_id}/join"));
    ScopeTest::new(&test_env)
        .with_user_id(FRIEND_USER_ID_PARSED)
        .test(req_gen, write_project)
        .await
        .unwrap();

    // Patch 'friend' user
    let req_gen = || {
        test::TestRequest::patch()
            .uri(&format!(
                "/v2/team/{alpha_team_id}/members/{FRIEND_USER_ID}"
            ))
            .set_json(json!({
                "permissions": 1
            }))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_project)
        .await
        .unwrap();

    // Transfer ownership to 'friend'
    let req_gen = || {
        test::TestRequest::patch()
            .uri(&format!("/v2/team/{alpha_team_id}/owner"))
            .set_json(json!({
                "user_id": FRIEND_USER_ID
            }))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_project)
        .await
        .unwrap();

    // Now as 'friend', delete 'user'
    let req_gen = || {
        test::TestRequest::delete().uri(&format!("/v2/team/{alpha_team_id}/members/{USER_USER_ID}"))
    };
    ScopeTest::new(&test_env)
        .with_user_id(FRIEND_USER_ID_PARSED)
        .test(req_gen, write_project)
        .await
        .unwrap();

    // Delete project
    // TODO: this route is currently broken,
    // because the Project::get_id contained within Project::remove doesnt include hidden versions, meaning that if there
    // is a hidden version, it will fail to delete the project (with a 500 error, as the versions of a project are not all deleted)
    // let delete_version = Scopes::PROJECT_DELETE;
    // let req_gen = || {
    //     test::TestRequest::delete()
    //     .uri(&format!("/v2/project/{beta_project_id}"))
    // };
    // ScopeTest::new(&test_env).test(req_gen, delete_version).await.unwrap();

    // Cleanup test db
    test_env.cleanup().await;
}

// Version write
#[actix_rt::test]
pub async fn version_write_scopes() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build_with_dummy().await;
    let alpha_version_id = &test_env.dummy.as_ref().unwrap().beta_version_id.clone();
    let beta_version_id = &test_env.dummy.as_ref().unwrap().beta_version_id.clone();
    let alpha_file_hash = &test_env.dummy.as_ref().unwrap().beta_file_hash.clone();

    let write_version = Scopes::VERSION_WRITE;

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
    let req_gen = || {
        test::TestRequest::post()
            .uri(&format!("/v2/version/{beta_version_id}/schedule")) // beta_version_id is an *approved* version, so we can schedule it
            .set_json(json!(
                {
                    "requested_status": "archived",
                    "time": Utc::now() + Duration::days(1),
                }
            ))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_version)
        .await
        .unwrap();

    // Patch version
    let req_gen = || {
        test::TestRequest::patch()
            .uri(&format!("/v2/version/{alpha_version_id}"))
            .set_json(json!(
                {
                    "version_title": "test_version_write_scopes Title",
                }
            ))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_version)
        .await
        .unwrap();

    // Generate test project data.
    // Basic json
    let json_segment = common::actix::MultipartSegment {
        name: "data".to_string(),
        filename: None,
        content_type: Some("application/json".to_string()),
        data: common::actix::MultipartSegmentData::Text(
            serde_json::to_string(&json!(
                {
                    "file_types": {
                        "simple-zip.zip": "required-resource-pack"
                    },
                }
            ))
            .unwrap(),
        ),
    };

    // Differently named file, with different content
    let content_segment = common::actix::MultipartSegment {
        name: "simple-zip.zip".to_string(),
        filename: Some("simple-zip.zip".to_string()),
        content_type: Some("application/zip".to_string()),
        data: common::actix::MultipartSegmentData::Binary(
            include_bytes!("../tests/files/simple-zip.zip").to_vec(),
        ),
    };

    // Upload version file
    let req_gen = || {
        test::TestRequest::post()
            .uri(&format!("/v2/version/{alpha_version_id}/file"))
            .set_multipart(vec![json_segment.clone(), content_segment.clone()])
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_version)
        .await
        .unwrap();

    //  Delete version file
    // TODO: Should this scope be VERSION_DELETE?
    let req_gen = || {
        test::TestRequest::delete().uri(&format!("/v2/version_file/{alpha_file_hash}"))
        // Delete from alpha_version_id, as we uploaded to alpha_version_id and it needs another file
    };
    ScopeTest::new(&test_env)
        .test(req_gen, write_version)
        .await
        .unwrap();

    // Delete version
    let delete_version = Scopes::VERSION_DELETE;
    let req_gen = || test::TestRequest::delete().uri(&format!("/v2/version/{alpha_version_id}"));
    ScopeTest::new(&test_env)
        .test(req_gen, delete_version)
        .await
        .unwrap();

    // Cleanup test db
    test_env.cleanup().await;
}

// Report scopes
#[actix_rt::test]
pub async fn report_scopes() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build_with_dummy().await;
    let beta_project_id = &test_env.dummy.as_ref().unwrap().beta_project_id.clone();

    // Create report
    let report_create = Scopes::REPORT_CREATE;
    let req_gen = || {
        test::TestRequest::post().uri("/v2/report").set_json(json!({
            "report_type": "copyright",
            "item_id": beta_project_id,
            "item_type": "project",
            "body": "This is a reupload of my mod, ",
        }))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, report_create)
        .await
        .unwrap();

    // Get reports
    let report_read = Scopes::REPORT_READ;
    let req_gen = || test::TestRequest::get().uri("/v2/report");
    let (_, success) = ScopeTest::new(&test_env)
        .test(req_gen, report_read)
        .await
        .unwrap();
    let report_id = success.as_array().unwrap()[0]["id"].as_str().unwrap();

    let req_gen = || test::TestRequest::get().uri(&format!("/v2/report/{}", report_id));
    ScopeTest::new(&test_env)
        .test(req_gen, report_read)
        .await
        .unwrap();

    let req_gen = || {
        test::TestRequest::get().uri(&format!(
            "/v2/reports?ids=[{}]",
            urlencoding::encode(&format!("\"{}\"", report_id))
        ))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, report_read)
        .await
        .unwrap();

    // Edit report
    let report_edit = Scopes::REPORT_WRITE;
    let req_gen = || {
        test::TestRequest::patch()
            .uri(&format!("/v2/report/{}", report_id))
            .set_json(json!({
                "body": "This is a reupload of my mod, G8!",
            }))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, report_edit)
        .await
        .unwrap();

    // Delete report
    // We use a moderator PAT here, as only moderators can delete reports
    let report_delete = Scopes::REPORT_DELETE;
    let req_gen = || test::TestRequest::delete().uri(&format!("/v2/report/{}", report_id));
    ScopeTest::new(&test_env)
        .with_user_id(MOD_USER_ID_PARSED)
        .test(req_gen, report_delete)
        .await
        .unwrap();

    // Cleanup test db
    test_env.cleanup().await;
}

// Thread scopes
#[actix_rt::test]
pub async fn thread_scopes() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build_with_dummy().await;
    let alpha_thread_id = &test_env.dummy.as_ref().unwrap().alpha_thread_id.clone();
    let beta_thread_id = &test_env.dummy.as_ref().unwrap().beta_thread_id.clone();

    // Thread read
    let thread_read = Scopes::THREAD_READ;
    let req_gen = || test::TestRequest::get().uri(&format!("/v2/thread/{alpha_thread_id}"));
    ScopeTest::new(&test_env)
        .test(req_gen, thread_read)
        .await
        .unwrap();

    let req_gen = || {
        test::TestRequest::get().uri(&format!(
            "/v2/threads?ids=[{}]",
            urlencoding::encode(&format!("\"{}\"", "U"))
        ))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, thread_read)
        .await
        .unwrap();

    // Thread write (to also push to moderator inbox)
    let thread_write = Scopes::THREAD_WRITE;
    let req_gen = || {
        test::TestRequest::post()
            .uri(&format!("/v2/thread/{beta_thread_id}"))
            .set_json(json!({
                "body": {
                    "type": "text",
                    "body": "test_thread_scopes Body"
                }
            }))
    };
    ScopeTest::new(&test_env)
        .with_user_id(USER_USER_ID_PARSED)
        .test(req_gen, thread_write)
        .await
        .unwrap();

    // Check moderation inbox
    // Uses moderator PAT, as only moderators can see the moderation inbox
    let req_gen = || test::TestRequest::get().uri("/v2/thread/inbox");
    let (_, success) = ScopeTest::new(&test_env)
        .with_user_id(MOD_USER_ID_PARSED)
        .test(req_gen, thread_read)
        .await
        .unwrap();
    let thread = success.as_array().unwrap()[0].as_object().unwrap();
    let thread_id = thread["id"].as_str().unwrap();

    // Moderator 'read' thread
    // Uses moderator PAT, as only moderators can see the moderation inbox
    let req_gen = || test::TestRequest::post().uri(&format!("/v2/thread/{thread_id}/read"));
    ScopeTest::new(&test_env)
        .with_user_id(MOD_USER_ID_PARSED)
        .test(req_gen, thread_read)
        .await
        .unwrap();

    // Delete that message
    // First, get message id
    let req_gen = test::TestRequest::get()
        .uri(&format!("/v2/thread/{thread_id}"))
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req_gen).await;
    let success: serde_json::Value = test::read_body_json(resp).await;
    let thread_messages = success.as_object().unwrap()["messages"].as_array().unwrap();
    let thread_message_id = thread_messages[0].as_object().unwrap()["id"]
        .as_str()
        .unwrap();
    let req_gen = || test::TestRequest::delete().uri(&format!("/v2/message/{thread_message_id}"));
    ScopeTest::new(&test_env)
        .with_user_id(MOD_USER_ID_PARSED)
        .test(req_gen, thread_write)
        .await
        .unwrap();

    // Cleanup test db
    test_env.cleanup().await;
}

// Pat scopes
#[actix_rt::test]
pub async fn pat_scopes() {
    let test_env = TestEnvironment::build_with_dummy().await;

    // Pat create
    let pat_create = Scopes::PAT_CREATE;
    let req_gen = || {
        test::TestRequest::post().uri("/v2/pat").set_json(json!({
            "scopes": 1,
            "name": "test_pat_scopes Name",
            "expires": Utc::now() + Duration::days(1),
        }))
    };
    let (_, success) = ScopeTest::new(&test_env)
        .test(req_gen, pat_create)
        .await
        .unwrap();
    let pat_id = success["id"].as_str().unwrap();

    // Pat write
    let pat_write = Scopes::PAT_WRITE;
    let req_gen = || {
        test::TestRequest::patch()
            .uri(&format!("/v2/pat/{pat_id}"))
            .set_json(json!({}))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, pat_write)
        .await
        .unwrap();

    // Pat read
    let pat_read = Scopes::PAT_READ;
    let req_gen = || test::TestRequest::get().uri("/v2/pat");
    ScopeTest::new(&test_env)
        .test(req_gen, pat_read)
        .await
        .unwrap();

    // Pat delete
    let pat_delete = Scopes::PAT_DELETE;
    let req_gen = || test::TestRequest::delete().uri(&format!("/v2/pat/{pat_id}"));
    ScopeTest::new(&test_env)
        .test(req_gen, pat_delete)
        .await
        .unwrap();

    // Cleanup test db
    test_env.cleanup().await;
}

// Collection scopes
#[actix_rt::test]
pub async fn collections_scopes() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build_with_dummy().await;
    let alpha_project_id = &test_env.dummy.as_ref().unwrap().alpha_project_id.clone();

    // Create collection
    let collection_create = Scopes::COLLECTION_CREATE;
    let req_gen = || {
        test::TestRequest::post()
            .uri("/v2/collection")
            .set_json(json!({
                "title": "Test Collection",
                "description": "Test Collection Description",
                "projects": [alpha_project_id]
            }))
    };
    let (_, success) = ScopeTest::new(&test_env)
        .test(req_gen, collection_create)
        .await
        .unwrap();
    let collection_id = success["id"].as_str().unwrap();

    // Patch collection
    // Collections always initialize to public, so we do patch before Get testing
    let collection_write = Scopes::COLLECTION_WRITE;
    let req_gen = || {
        test::TestRequest::patch()
            .uri(&format!("/v2/collection/{collection_id}"))
            .set_json(json!({
                "title": "Test Collection patch",
                "status": "private",
            }))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, collection_write)
        .await
        .unwrap();

    // Read collection
    let collection_read = Scopes::COLLECTION_READ;
    let req_gen = || test::TestRequest::get().uri(&format!("/v2/collection/{}", collection_id));
    ScopeTest::new(&test_env)
        .with_failure_code(404)
        .test(req_gen, collection_read)
        .await
        .unwrap();

    let req_gen = || {
        test::TestRequest::get().uri(&format!(
            "/v2/collections?ids=[{}]",
            urlencoding::encode(&format!("\"{}\"", collection_id))
        ))
    };
    let (failure, success) = ScopeTest::new(&test_env)
        .with_failure_code(200)
        .test(req_gen, collection_read)
        .await
        .unwrap();
    assert_eq!(failure.as_array().unwrap().len(), 0);
    assert_eq!(success.as_array().unwrap().len(), 1);

    let req_gen = || test::TestRequest::get().uri(&format!("/v2/user/{USER_USER_ID}/collections"));
    let (failure, success) = ScopeTest::new(&test_env)
        .with_failure_code(200)
        .test(req_gen, collection_read)
        .await
        .unwrap();
    assert_eq!(failure.as_array().unwrap().len(), 0);
    assert_eq!(success.as_array().unwrap().len(), 1);

    let req_gen = || {
        test::TestRequest::patch()
            .uri(&format!("/v2/collection/{collection_id}/icon?ext=png"))
            .set_payload(Bytes::from(
                include_bytes!("../tests/files/200x200.png") as &[u8]
            ))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, collection_write)
        .await
        .unwrap();

    let req_gen =
        || test::TestRequest::delete().uri(&format!("/v2/collection/{collection_id}/icon"));
    ScopeTest::new(&test_env)
        .test(req_gen, collection_write)
        .await
        .unwrap();

    // Cleanup test db
    test_env.cleanup().await;
}

// Organization scopes (and a couple PROJECT_WRITE scopes that are only allowed for orgs)
#[actix_rt::test]
pub async fn organization_scopes() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build_with_dummy().await;
    let beta_project_id = &test_env.dummy.as_ref().unwrap().beta_project_id.clone();

    // Create organization
    let organization_create = Scopes::ORGANIZATION_CREATE;
    let req_gen = || {
        test::TestRequest::post()
            .uri("/v2/organization")
            .set_json(json!({
                "title": "TestOrg",
                "description": "TestOrg Description",
            }))
    };
    let (_, success) = ScopeTest::new(&test_env)
        .test(req_gen, organization_create)
        .await
        .unwrap();
    let organization_id = success["id"].as_str().unwrap();

    // Patch organization
    let organization_edit = Scopes::ORGANIZATION_WRITE;
    let req_gen = || {
        test::TestRequest::patch()
            .uri(&format!("/v2/organization/{organization_id}"))
            .set_json(json!({
                "description": "TestOrg Patch Description",
            }))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, organization_edit)
        .await
        .unwrap();

    let req_gen = || {
        test::TestRequest::patch()
            .uri(&format!("/v2/organization/{organization_id}/icon?ext=png"))
            .set_payload(Bytes::from(
                include_bytes!("../tests/files/200x200.png") as &[u8]
            ))
    };
    ScopeTest::new(&test_env)
        .test(req_gen, organization_edit)
        .await
        .unwrap();

    let req_gen =
        || test::TestRequest::delete().uri(&format!("/v2/organization/{organization_id}/icon"));
    ScopeTest::new(&test_env)
        .test(req_gen, organization_edit)
        .await
        .unwrap();

    // add project
    let organization_project_edit = Scopes::PROJECT_WRITE | Scopes::ORGANIZATION_WRITE;
    let req_gen = || {
        test::TestRequest::post()
            .uri(&format!("/v2/organization/{organization_id}/projects"))
            .set_json(json!({
                "project_id": beta_project_id
            }))
    };
    ScopeTest::new(&test_env)
        .with_failure_scopes(Scopes::all() ^ Scopes::ORGANIZATION_WRITE)
        .test(req_gen, organization_project_edit)
        .await
        .unwrap();

    // Organization reads
    let organization_read = Scopes::ORGANIZATION_READ;
    let req_gen = || test::TestRequest::get().uri(&format!("/v2/organization/{organization_id}"));
    let (failure, success) = ScopeTest::new(&test_env)
        .with_failure_code(200)
        .test(req_gen, organization_read)
        .await
        .unwrap();
    assert!(
        failure.as_object().unwrap()["members"].as_array().unwrap()[0]
            .as_object()
            .unwrap()["permissions"]
            .is_null()
    );
    assert!(
        !success.as_object().unwrap()["members"].as_array().unwrap()[0]
            .as_object()
            .unwrap()["permissions"]
            .is_null()
    );

    let req_gen = || {
        test::TestRequest::get().uri(&format!(
            "/v2/organizations?ids=[{}]",
            urlencoding::encode(&format!("\"{}\"", organization_id))
        ))
    };

    let (failure, success) = ScopeTest::new(&test_env)
        .with_failure_code(200)
        .test(req_gen, organization_read)
        .await
        .unwrap();
    assert!(
        failure.as_array().unwrap()[0].as_object().unwrap()["members"]
            .as_array()
            .unwrap()[0]
            .as_object()
            .unwrap()["permissions"]
            .is_null()
    );
    assert!(
        !success.as_array().unwrap()[0].as_object().unwrap()["members"]
            .as_array()
            .unwrap()[0]
            .as_object()
            .unwrap()["permissions"]
            .is_null()
    );

    let organization_project_read = Scopes::PROJECT_READ | Scopes::ORGANIZATION_READ;
    let req_gen =
        || test::TestRequest::get().uri(&format!("/v2/organization/{organization_id}/projects"));
    let (failure, success) = ScopeTest::new(&test_env)
        .with_failure_code(200)
        .with_failure_scopes(Scopes::all() ^ Scopes::ORGANIZATION_READ)
        .test(req_gen, organization_project_read)
        .await
        .unwrap();
    assert!(failure.as_array().unwrap().is_empty());
    assert!(!success.as_array().unwrap().is_empty());

    // remove project (now that we've checked)
    let req_gen = || {
        test::TestRequest::delete().uri(&format!(
            "/v2/organization/{organization_id}/projects/{beta_project_id}"
        ))
    };
    ScopeTest::new(&test_env)
        .with_failure_scopes(Scopes::all() ^ Scopes::ORGANIZATION_WRITE)
        .test(req_gen, organization_project_edit)
        .await
        .unwrap();

    // Delete organization
    let organization_delete = Scopes::ORGANIZATION_DELETE;
    let req_gen =
        || test::TestRequest::delete().uri(&format!("/v2/organization/{organization_id}"));
    ScopeTest::new(&test_env)
        .test(req_gen, organization_delete)
        .await
        .unwrap();

    // Cleanup test db
    test_env.cleanup().await;
}

// TODO: Analytics scopes

// TODO: User authentication, and Session scopes

// TODO: Some hash/version files functions

// TODO: Meta pat stuff

// TODO: Image scopes

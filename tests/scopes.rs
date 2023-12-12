use actix_web::test::{self, TestRequest};
use bytes::Bytes;
use chrono::{Duration, Utc};
use common::api_v3::request_data::{
    get_public_project_creation_data, get_public_version_creation_data,
};
use common::api_v3::ApiV3;
use common::dummy_data::TestFile;
use common::environment::{with_test_environment, with_test_environment_all, TestEnvironment};
use common::{database::*, scopes::ScopeTest};
use labrinth::models::ids::base62_impl::parse_base62;
use labrinth::models::pats::Scopes;
use labrinth::models::projects::ProjectId;
use labrinth::util::actix::{AppendsMultipart, MultipartSegment, MultipartSegmentData};
use serde_json::json;

use crate::common::api_common::{ApiTeams, AppendsOptionalPat};
use crate::common::dummy_data::DummyImage;

// For each scope, we (using test_scope):
// - create a PAT with a given set of scopes for a function
// - create a PAT with all other scopes for a function
// - test the function with the PAT with the given scopes
// - test the function with the PAT with all other scopes

mod common;

// Test for users, emails, and payout scopes (not user auth scope or notifs)
#[actix_rt::test]
async fn user_scopes() {
    // Test setup and dummy data
    with_test_environment_all(None, |test_env| async move {
        // User reading
        let read_user = Scopes::USER_READ;
        let req_gen = || TestRequest::get().uri("/v3/user");
        let (_, success) = ScopeTest::new(&test_env)
            .test(req_gen, read_user)
            .await
            .unwrap();
        assert!(success["email"].as_str().is_none()); // email should not be present
        assert!(success["payout_data"].as_object().is_none()); // payout should not be present

        // Email reading
        let read_email = Scopes::USER_READ | Scopes::USER_READ_EMAIL;
        let req_gen = || TestRequest::get().uri("/v3/user");
        let (_, success) = ScopeTest::new(&test_env)
            .test(req_gen, read_email)
            .await
            .unwrap();
        assert_eq!(success["email"], json!("user@modrinth.com")); // email should be present

        // Payout reading
        let read_payout = Scopes::USER_READ | Scopes::PAYOUTS_READ;
        let req_gen = || TestRequest::get().uri("/v3/user");
        let (_, success) = ScopeTest::new(&test_env)
            .test(req_gen, read_payout)
            .await
            .unwrap();
        assert!(success["payout_data"].as_object().is_some()); // payout should be present

        // User writing
        // We use the Admin PAT for this test, on the 'user' user
        let write_user = Scopes::USER_WRITE;
        let req_gen = || {
            TestRequest::patch().uri("/v3/user/user").set_json(json!( {
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
        let req_gen = || TestRequest::delete().uri("/v3/user/enemy");
        ScopeTest::new(&test_env)
            .with_user_id(ENEMY_USER_ID_PARSED)
            .test(req_gen, delete_user)
            .await
            .unwrap();
    })
    .await;
}

// Notifications
#[actix_rt::test]
pub async fn notifications_scopes() {
    with_test_environment_all(None, |test_env| async move {
        let alpha_team_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_alpha
            .team_id
            .clone();

        // We will invite user 'friend' to project team, and use that as a notification
        // Get notifications
        let resp = test_env
            .api
            .add_user_to_team(alpha_team_id, FRIEND_USER_ID, None, None, USER_USER_PAT)
            .await;
        assert_eq!(resp.status(), 204);

        // Notification get
        let read_notifications = Scopes::NOTIFICATION_READ;
        let req_gen =
            || test::TestRequest::get().uri(&format!("/v3/user/{FRIEND_USER_ID}/notifications"));
        let (_, success) = ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, read_notifications)
            .await
            .unwrap();
        let notification_id = success[0]["id"].as_str().unwrap();

        let req_gen = || {
            test::TestRequest::get().uri(&format!(
                "/v3/notifications?ids=[{uri}]",
                uri = urlencoding::encode(&format!("\"{notification_id}\""))
            ))
        };
        ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, read_notifications)
            .await
            .unwrap();

        let req_gen =
            || test::TestRequest::get().uri(&format!("/v3/notification/{notification_id}"));
        ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, read_notifications)
            .await
            .unwrap();

        // Notification mark as read
        let write_notifications = Scopes::NOTIFICATION_WRITE;
        let req_gen = || {
            test::TestRequest::patch().uri(&format!(
                "/v3/notifications?ids=[{uri}]",
                uri = urlencoding::encode(&format!("\"{notification_id}\""))
            ))
        };
        ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, write_notifications)
            .await
            .unwrap();

        let req_gen =
            || test::TestRequest::patch().uri(&format!("/v3/notification/{notification_id}"));
        ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, write_notifications)
            .await
            .unwrap();

        // Notification delete
        let req_gen =
            || test::TestRequest::delete().uri(&format!("/v3/notification/{notification_id}"));
        ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, write_notifications)
            .await
            .unwrap();

        // Mass notification delete
        // We invite mod, get the notification ID, and do mass delete using that
        let resp = test_env
            .api
            .add_user_to_team(alpha_team_id, MOD_USER_ID, None, None, USER_USER_PAT)
            .await;
        assert_eq!(resp.status(), 204);
        let read_notifications = Scopes::NOTIFICATION_READ;
        let req_gen =
            || test::TestRequest::get().uri(&format!("/v3/user/{MOD_USER_ID}/notifications"));
        let (_, success) = ScopeTest::new(&test_env)
            .with_user_id(MOD_USER_ID_PARSED)
            .test(req_gen, read_notifications)
            .await
            .unwrap();
        let notification_id = success[0]["id"].as_str().unwrap();

        let req_gen = || {
            test::TestRequest::delete().uri(&format!(
                "/v3/notifications?ids=[{uri}]",
                uri = urlencoding::encode(&format!("\"{notification_id}\""))
            ))
        };
        ScopeTest::new(&test_env)
            .with_user_id(MOD_USER_ID_PARSED)
            .test(req_gen, write_notifications)
            .await
            .unwrap();
    })
    .await;
}

// Project version creation scopes
#[actix_rt::test]
pub async fn project_version_create_scopes_v3() {
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        // TODO: If possible, find a way to use generic api functions with the Permissions/Scopes test, then this can be recombined with the V2 version of this test
        // let api = &test_env.api;

        // Create project
        let create_project = Scopes::PROJECT_CREATE;
        let req_gen = || {
            let creation_data =
                get_public_project_creation_data("demo", Some(TestFile::BasicMod), None);
            test::TestRequest::post()
                .uri("/v3/project")
                .set_multipart(creation_data.segment_data)
        };
        let (_, success) = ScopeTest::new(&test_env)
            .test(req_gen, create_project)
            .await
            .unwrap();
        let project_id = success["id"].as_str().unwrap();
        let project_id = ProjectId(parse_base62(project_id).unwrap());

        // Add version to project
        let create_version = Scopes::VERSION_CREATE;
        let req_gen = || {
            let creation_data = get_public_version_creation_data(
                project_id,
                "1.2.3.4",
                TestFile::BasicModDifferent,
                None,
                None,
            );

            test::TestRequest::post()
                .uri("/v3/version")
                .set_multipart(creation_data.segment_data)
        };
        ScopeTest::new(&test_env)
            .test(req_gen, create_version)
            .await
            .unwrap();
    })
    .await;
}

// Project management scopes
#[actix_rt::test]
pub async fn project_version_reads_scopes() {
    with_test_environment_all(None, |test_env| async move {
        let beta_project_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_beta
            .project_id
            .clone();
        let beta_version_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_beta
            .version_id
            .clone();
        let alpha_team_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_alpha
            .team_id
            .clone();
        let beta_file_hash = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_beta
            .file_hash
            .clone();

        // Project reading
        // Uses 404 as the expected failure code (or 200 and an empty list for mass reads)
        let read_project = Scopes::PROJECT_READ;
        let req_gen = || test::TestRequest::get().uri(&format!("/v3/project/{beta_project_id}"));
        ScopeTest::new(&test_env)
            .with_failure_code(404)
            .test(req_gen, read_project)
            .await
            .unwrap();

        let req_gen =
            || test::TestRequest::get().uri(&format!("/v3/project/{beta_project_id}/dependencies"));
        ScopeTest::new(&test_env)
            .with_failure_code(404)
            .test(req_gen, read_project)
            .await
            .unwrap();

        let req_gen = || {
            test::TestRequest::get().uri(&format!(
                "/v3/projects?ids=[{uri}]",
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
            || test::TestRequest::get().uri(&format!("/v3/project/{beta_project_id}/members"));
        ScopeTest::new(&test_env)
            .with_failure_code(404)
            .test(req_gen, read_project)
            .await
            .unwrap();

        // Get team members
        // In this case, as these are public endpoints, logging in only is relevant to showing permissions
        // So for our test project (with 1 user, 'user') we will check the permissions before and after having the scope.
        let req_gen = || test::TestRequest::get().uri(&format!("/v3/team/{alpha_team_id}/members"));
        let (failure, success) = ScopeTest::new(&test_env)
            .with_failure_code(200)
            .test(req_gen, read_project)
            .await
            .unwrap();
        assert!(!failure[0]["permissions"].is_number());
        assert!(success[0]["permissions"].is_number());

        let req_gen = || {
            test::TestRequest::get().uri(&format!(
                "/v3/teams?ids=[{uri}]",
                uri = urlencoding::encode(&format!("\"{alpha_team_id}\""))
            ))
        };
        let (failure, success) = ScopeTest::new(&test_env)
            .with_failure_code(200)
            .test(req_gen, read_project)
            .await
            .unwrap();
        assert!(!failure[0][0]["permissions"].is_number());
        assert!(success[0][0]["permissions"].is_number());

        // User project reading
        // Test user has two projects, one public and one private
        let req_gen = || test::TestRequest::get().uri(&format!("/v3/user/{USER_USER_ID}/projects"));
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
            .uri(&format!("/v3/version/{beta_version_id}"))
            .append_pat(USER_USER_PAT)
            .set_json(json!({
                "status": "draft"
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_eq!(resp.status(), 204);

        let req_gen =
            || test::TestRequest::get().uri(&format!("/v3/version_file/{beta_file_hash}"));
        ScopeTest::new(&test_env)
            .with_failure_code(404)
            .test(req_gen, read_version)
            .await
            .unwrap();

        let req_gen =
            || test::TestRequest::get().uri(&format!("/v3/version_file/{beta_file_hash}/download"));
        ScopeTest::new(&test_env)
            .with_failure_code(404)
            .test(req_gen, read_version)
            .await
            .unwrap();

        // TODO: Should this be /POST? Looks like /GET
        // TODO: this scope doesn't actually affect anything, because the Project::get_id contained within disallows hidden versions, which is the point of this scope
        // let req_gen = || {
        //     test::TestRequest::post()
        //     .uri(&format!("/v3/version_file/{beta_file_hash}/update"))
        //     .set_json(json!({}))
        // };
        // ScopeTest::new(&test_env).with_failure_code(404).test(req_gen, read_version).await.unwrap();

        // TODO: Should this be /POST? Looks like /GET
        let req_gen = || {
            test::TestRequest::post()
                .uri("/v3/version_files")
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
        //     .uri(&format!("/v3/version_files/update_individual"))
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
        //     .uri(&format!("/v3/version_files/update"))
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
            || test::TestRequest::get().uri(&format!("/v3/project/{beta_project_id}/version"));
        ScopeTest::new(&test_env)
            .with_failure_code(404)
            .test(req_gen, read_project_and_version)
            .await
            .unwrap();

        // TODO: fails for the same reason as above
        // let req_gen = || {
        //     test::TestRequest::get()
        //     .uri(&format!("/v3/project/{beta_project_id}/version/{beta_version_id}"))
        // };
        // ScopeTest::new(&test_env).with_failure_code(404).test(req_gen, read_project_and_version).await.unwrap();
    })
    .await;
}

// Project writing
#[actix_rt::test]
pub async fn project_write_scopes() {
    // Test setup and dummy data
    with_test_environment_all(None, |test_env| async move {
        let beta_project_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_beta
            .project_id
            .clone();
        let alpha_team_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_alpha
            .team_id
            .clone();

        let test_icon = DummyImage::SmallIcon;

        // Projects writing
        let write_project = Scopes::PROJECT_WRITE;
        let req_gen = || {
            test::TestRequest::patch()
                .uri(&format!("/v3/project/{beta_project_id}"))
                .set_json(json!(
                    {
                        "name": "test_project_version_write_scopes Title",
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
                    "/v3/projects?ids=[{uri}]",
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
            .uri(&format!("/v3/project/{beta_project_id}"))
            .append_pat(MOD_USER_PAT)
            .set_json(json!({
                "status": "private"
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_eq!(resp.status(), 204);

        let req_gen = || {
            test::TestRequest::post()
                .uri(&format!("/v3/project/{beta_project_id}/schedule")) // beta_project_id is an unpublished can schedule it
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
                .uri(&format!(
                    "/v3/project/{beta_project_id}/icon?ext={ext}",
                    ext = test_icon.extension()
                ))
                .set_payload(test_icon.bytes())
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        let req_gen =
            || test::TestRequest::delete().uri(&format!("/v3/project/{beta_project_id}/icon"));
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        let req_gen = || {
            test::TestRequest::post()
                .uri(&format!(
                    "/v3/project/{beta_project_id}/gallery?ext={ext}&featured=true",
                    ext = test_icon.extension()
                ))
                .set_payload(test_icon.bytes())
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        // Get project, as we need the gallery image url
        let req_gen = test::TestRequest::get()
            .uri(&format!("/v3/project/{beta_project_id}"))
            .append_pat(USER_USER_PAT)
            .to_request();
        let resp = test_env.call(req_gen).await;
        let project: serde_json::Value = test::read_body_json(resp).await;
        let gallery_url = project["gallery"][0]["url"].as_str().unwrap();

        let req_gen = || {
            test::TestRequest::patch().uri(&format!(
                "/v3/project/{beta_project_id}/gallery?url={gallery_url}"
            ))
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        let req_gen = || {
            test::TestRequest::delete().uri(&format!(
                "/v3/project/{beta_project_id}/gallery?url={gallery_url}"
            ))
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        // Team scopes - add user 'friend'
        let req_gen = || {
            test::TestRequest::post()
                .uri(&format!("/v3/team/{alpha_team_id}/members"))
                .set_json(json!({
                    "user_id": FRIEND_USER_ID
                }))
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        // Accept team invite as 'friend'
        let req_gen = || test::TestRequest::post().uri(&format!("/v3/team/{alpha_team_id}/join"));
        ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, write_project)
            .await
            .unwrap();

        // Patch 'friend' user
        let req_gen = || {
            test::TestRequest::patch()
                .uri(&format!(
                    "/v3/team/{alpha_team_id}/members/{FRIEND_USER_ID}"
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
                .uri(&format!("/v3/team/{alpha_team_id}/owner"))
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
            test::TestRequest::delete()
                .uri(&format!("/v3/team/{alpha_team_id}/members/{USER_USER_ID}"))
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
        //     .uri(&format!("/v3/project/{beta_project_id}"))
        // };
        // ScopeTest::new(&test_env).test(req_gen, delete_version).await.unwrap();
    })
    .await;
}

// Version write
#[actix_rt::test]
pub async fn version_write_scopes() {
    // Test setup and dummy data
    with_test_environment_all(None, |test_env| async move {
        let alpha_version_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_alpha
            .version_id
            .clone();
        let beta_version_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_beta
            .version_id
            .clone();
        let alpha_file_hash = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_alpha
            .file_hash
            .clone();

        let basic_zip = TestFile::BasicZip;

        let write_version = Scopes::VERSION_WRITE;

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
        let req_gen = || {
            test::TestRequest::post()
                .uri(&format!("/v3/version/{beta_version_id}/schedule")) // beta_version_id is an *approved* version, so we can schedule it
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
                .uri(&format!("/v3/version/{alpha_version_id}"))
                .set_json(json!(
                    {
                        "name": "test_version_write_scopes Title",
                    }
                ))
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_version)
            .await
            .unwrap();

        // Generate test project data.
        // Basic json
        let json_segment = MultipartSegment {
            name: "data".to_string(),
            filename: None,
            content_type: Some("application/json".to_string()),
            data: MultipartSegmentData::Text(
                serde_json::to_string(&json!(
                    {
                        "file_types": {
                            basic_zip.filename(): "required-resource-pack"
                        },
                    }
                ))
                .unwrap(),
            ),
        };

        // Differently named file, with different content
        let content_segment = MultipartSegment {
            name: basic_zip.filename(),
            filename: Some(basic_zip.filename()),
            content_type: basic_zip.content_type(),
            data: MultipartSegmentData::Binary(basic_zip.bytes()),
        };

        // Upload version file
        let req_gen = || {
            test::TestRequest::post()
                .uri(&format!("/v3/version/{alpha_version_id}/file"))
                .set_multipart(vec![json_segment.clone(), content_segment.clone()])
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_version)
            .await
            .unwrap();

        //  Delete version file
        // TODO: Should this scope be VERSION_DELETE?
        let req_gen = || {
            test::TestRequest::delete().uri(&format!("/v3/version_file/{alpha_file_hash}"))
            // Delete from alpha_version_id, as we uploaded to alpha_version_id and it needs another file
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_version)
            .await
            .unwrap();

        // Delete version
        let delete_version = Scopes::VERSION_DELETE;
        let req_gen =
            || test::TestRequest::delete().uri(&format!("/v3/version/{alpha_version_id}"));
        ScopeTest::new(&test_env)
            .test(req_gen, delete_version)
            .await
            .unwrap();
    })
    .await;
}

// Report scopes
#[actix_rt::test]
pub async fn report_scopes() {
    // Test setup and dummy data
    with_test_environment_all(None, |test_env| async move {
        let beta_project_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_beta
            .project_id
            .clone();

        // Create report
        let report_create = Scopes::REPORT_CREATE;
        let req_gen = || {
            test::TestRequest::post().uri("/v3/report").set_json(json!({
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
        let req_gen = || test::TestRequest::get().uri("/v3/report");
        let (_, success) = ScopeTest::new(&test_env)
            .test(req_gen, report_read)
            .await
            .unwrap();
        let report_id = success[0]["id"].as_str().unwrap();

        let req_gen = || test::TestRequest::get().uri(&format!("/v3/report/{}", report_id));
        ScopeTest::new(&test_env)
            .test(req_gen, report_read)
            .await
            .unwrap();

        let req_gen = || {
            test::TestRequest::get().uri(&format!(
                "/v3/reports?ids=[{}]",
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
                .uri(&format!("/v3/report/{}", report_id))
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
        let req_gen = || test::TestRequest::delete().uri(&format!("/v3/report/{}", report_id));
        ScopeTest::new(&test_env)
            .with_user_id(MOD_USER_ID_PARSED)
            .test(req_gen, report_delete)
            .await
            .unwrap();
    })
    .await;
}

// Thread scopes
#[actix_rt::test]
pub async fn thread_scopes() {
    // Test setup and dummy data
    with_test_environment_all(None, |test_env| async move {
        let alpha_thread_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_alpha
            .thread_id
            .clone();
        let beta_thread_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_beta
            .thread_id
            .clone();

        // Thread read
        let thread_read = Scopes::THREAD_READ;
        let req_gen = || test::TestRequest::get().uri(&format!("/v3/thread/{alpha_thread_id}"));
        ScopeTest::new(&test_env)
            .test(req_gen, thread_read)
            .await
            .unwrap();

        let req_gen = || {
            test::TestRequest::get().uri(&format!(
                "/v3/threads?ids=[{}]",
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
                .uri(&format!("/v3/thread/{beta_thread_id}"))
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
        let req_gen = || test::TestRequest::get().uri("/v3/thread/inbox");
        let (_, success) = ScopeTest::new(&test_env)
            .with_user_id(MOD_USER_ID_PARSED)
            .test(req_gen, thread_read)
            .await
            .unwrap();
        let thread_id: &str = success[0]["id"].as_str().unwrap();

        // Moderator 'read' thread
        // Uses moderator PAT, as only moderators can see the moderation inbox
        let req_gen = || test::TestRequest::post().uri(&format!("/v3/thread/{thread_id}/read"));
        ScopeTest::new(&test_env)
            .with_user_id(MOD_USER_ID_PARSED)
            .test(req_gen, thread_read)
            .await
            .unwrap();

        // Delete that message
        // First, get message id
        let req_gen = test::TestRequest::get()
            .uri(&format!("/v3/thread/{thread_id}"))
            .append_pat(USER_USER_PAT)
            .to_request();
        let resp = test_env.call(req_gen).await;
        let success: serde_json::Value = test::read_body_json(resp).await;
        let thread_message_id = success["messages"][0]["id"].as_str().unwrap();

        let req_gen =
            || test::TestRequest::delete().uri(&format!("/v3/message/{thread_message_id}"));
        ScopeTest::new(&test_env)
            .with_user_id(MOD_USER_ID_PARSED)
            .test(req_gen, thread_write)
            .await
            .unwrap();
    })
    .await;
}

// Pat scopes
#[actix_rt::test]
pub async fn pat_scopes() {
    with_test_environment_all(None, |test_env| async move {
        // Pat create
        let pat_create = Scopes::PAT_CREATE;
        let req_gen = || {
            test::TestRequest::post()
                .uri("/_internal/pat")
                .set_json(json!({
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
                .uri(&format!("/_internal/pat/{pat_id}"))
                .set_json(json!({}))
        };
        ScopeTest::new(&test_env)
            .test(req_gen, pat_write)
            .await
            .unwrap();

        // Pat read
        let pat_read = Scopes::PAT_READ;
        let req_gen = || test::TestRequest::get().uri("/_internal/pat");
        ScopeTest::new(&test_env)
            .test(req_gen, pat_read)
            .await
            .unwrap();

        // Pat delete
        let pat_delete = Scopes::PAT_DELETE;
        let req_gen = || test::TestRequest::delete().uri(&format!("/_internal/pat/{pat_id}"));
        ScopeTest::new(&test_env)
            .test(req_gen, pat_delete)
            .await
            .unwrap();
    })
    .await;
}

// Collection scopes
#[actix_rt::test]
pub async fn collections_scopes() {
    // Test setup and dummy data
    with_test_environment_all(None, |test_env| async move {
        let alpha_project_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_alpha
            .project_id
            .clone();

        let small_icon = DummyImage::SmallIcon;

        // Create collection
        let collection_create = Scopes::COLLECTION_CREATE;
        let req_gen = || {
            test::TestRequest::post()
                .uri("/v3/collection")
                .set_json(json!({
                    "name": "Test Collection",
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
                .uri(&format!("/v3/collection/{collection_id}"))
                .set_json(json!({
                    "name": "Test Collection patch",
                    "status": "private",
                }))
        };
        ScopeTest::new(&test_env)
            .test(req_gen, collection_write)
            .await
            .unwrap();

        // Read collection
        let collection_read = Scopes::COLLECTION_READ;
        let req_gen = || test::TestRequest::get().uri(&format!("/v3/collection/{}", collection_id));
        ScopeTest::new(&test_env)
            .with_failure_code(404)
            .test(req_gen, collection_read)
            .await
            .unwrap();

        let req_gen = || {
            test::TestRequest::get().uri(&format!(
                "/v3/collections?ids=[{}]",
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

        let req_gen =
            || test::TestRequest::get().uri(&format!("/v3/user/{USER_USER_ID}/collections"));
        let (failure, success) = ScopeTest::new(&test_env)
            .with_failure_code(200)
            .test(req_gen, collection_read)
            .await
            .unwrap();
        assert_eq!(failure.as_array().unwrap().len(), 0);
        assert_eq!(success.as_array().unwrap().len(), 1);

        let req_gen = || {
            test::TestRequest::patch()
                .uri(&format!(
                    "/v3/collection/{collection_id}/icon?ext={ext}",
                    ext = small_icon.extension()
                ))
                .set_payload(Bytes::from(small_icon.bytes()))
        };
        ScopeTest::new(&test_env)
            .test(req_gen, collection_write)
            .await
            .unwrap();

        let req_gen =
            || test::TestRequest::delete().uri(&format!("/v3/collection/{collection_id}/icon"));
        ScopeTest::new(&test_env)
            .test(req_gen, collection_write)
            .await
            .unwrap();
    })
    .await;
}

// Organization scopes (and a couple PROJECT_WRITE scopes that are only allowed for orgs)
#[actix_rt::test]
pub async fn organization_scopes() {
    // Test setup and dummy data
    with_test_environment_all(None, |test_env| async move {
        let beta_project_id = &test_env
            .dummy
            .as_ref()
            .unwrap()
            .project_beta
            .project_id
            .clone();

        let icon = DummyImage::SmallIcon;

        // Create organization
        let organization_create = Scopes::ORGANIZATION_CREATE;
        let req_gen = || {
            test::TestRequest::post()
                .uri("/v3/organization")
                .set_json(json!({
                    "name": "TestOrg",
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
                .uri(&format!("/v3/organization/{organization_id}"))
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
                .uri(&format!(
                    "/v3/organization/{organization_id}/icon?ext={ext}",
                    ext = icon.extension()
                ))
                .set_payload(Bytes::from(icon.bytes()))
        };
        ScopeTest::new(&test_env)
            .test(req_gen, organization_edit)
            .await
            .unwrap();

        let req_gen =
            || test::TestRequest::delete().uri(&format!("/v3/organization/{organization_id}/icon"));
        ScopeTest::new(&test_env)
            .test(req_gen, organization_edit)
            .await
            .unwrap();

        // add project
        let organization_project_edit = Scopes::PROJECT_WRITE | Scopes::ORGANIZATION_WRITE;
        let req_gen = || {
            test::TestRequest::post()
                .uri(&format!("/v3/organization/{organization_id}/projects"))
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
        let req_gen =
            || test::TestRequest::get().uri(&format!("/v3/organization/{organization_id}"));
        let (failure, success) = ScopeTest::new(&test_env)
            .with_failure_code(200)
            .test(req_gen, organization_read)
            .await
            .unwrap();
        assert!(failure["members"][0]["permissions"].is_null());
        assert!(!success["members"][0]["permissions"].is_null());

        let req_gen = || {
            test::TestRequest::get().uri(&format!(
                "/v3/organizations?ids=[{}]",
                urlencoding::encode(&format!("\"{}\"", organization_id))
            ))
        };

        let (failure, success) = ScopeTest::new(&test_env)
            .with_failure_code(200)
            .test(req_gen, organization_read)
            .await
            .unwrap();
        assert!(failure[0]["members"][0]["permissions"].is_null());
        assert!(!success[0]["members"][0]["permissions"].is_null());

        let organization_project_read = Scopes::PROJECT_READ | Scopes::ORGANIZATION_READ;
        let req_gen = || {
            test::TestRequest::get().uri(&format!("/v3/organization/{organization_id}/projects"))
        };
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
                "/v3/organization/{organization_id}/projects/{beta_project_id}"
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
            || test::TestRequest::delete().uri(&format!("/v3/organization/{organization_id}"));
        ScopeTest::new(&test_env)
            .test(req_gen, organization_delete)
            .await
            .unwrap();
    })
    .await;
}

// TODO: Analytics scopes

// TODO: User authentication, and Session scopes

// TODO: Some hash/version files functions

// TODO: Image scopes

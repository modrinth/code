use std::collections::HashMap;

use crate::common::api_common::{
    ApiProject, ApiTeams, ApiUser, ApiVersion, AppendsOptionalPat,
};
use crate::common::dummy_data::{
    DummyImage, DummyProjectAlpha, DummyProjectBeta,
};
use actix_http::StatusCode;
use actix_web::test;
use ariadne::ids::UserId;
use ariadne::ids::base62_impl::parse_base62;
use chrono::{Duration, Utc};
use common::api_common::Api;
use common::api_common::models::CommonItemType;
use common::api_v3::ApiV3;
use common::api_v3::request_data::get_public_project_creation_data;
use common::dummy_data::TestFile;
use common::environment::{
    TestEnvironment, with_test_environment, with_test_environment_all,
};
use common::{database::*, scopes::ScopeTest};
use labrinth::models::ids::ProjectId;
use labrinth::models::pats::Scopes;
use serde_json::json;
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
        let api = &test_env.api;
        // User reading
        let read_user = Scopes::USER_READ;
        let req_gen = |pat: Option<String>| async move {
            api.get_current_user(pat.as_deref()).await
        };
        let (_, success) = ScopeTest::new(&test_env)
            .test(req_gen, read_user)
            .await
            .unwrap();
        assert!(success["email"].as_str().is_none()); // email should not be present
        assert!(success["payout_data"].as_object().is_none()); // payout should not be present

        // Email reading
        let read_email = Scopes::USER_READ | Scopes::USER_READ_EMAIL;
        let req_gen = |pat: Option<String>| async move {
            api.get_current_user(pat.as_deref()).await
        };
        let (_, success) = ScopeTest::new(&test_env)
            .test(req_gen, read_email)
            .await
            .unwrap();
        assert_eq!(success["email"], json!("user@modrinth.com")); // email should be present

        // Payout reading
        let read_payout = Scopes::USER_READ | Scopes::PAYOUTS_READ;
        let req_gen = |pat: Option<String>| async move {
            api.get_current_user(pat.as_deref()).await
        };
        let (_, success) = ScopeTest::new(&test_env)
            .test(req_gen, read_payout)
            .await
            .unwrap();
        assert!(success["payout_data"].as_object().is_some()); // payout should be present

        // User writing
        // We use the Admin PAT for this test, on the 'user' user
        let write_user = Scopes::USER_WRITE;
        let req_gen = |pat: Option<String>| async move {
            api.edit_user(
                "user",
                json!( {
                    // Do not include 'username', as to not change the rest of the tests
                    "name": "NewName",
                    "bio": "New bio",
                    "location": "New location",
                    "role": "admin",
                    "badges": 5,
                    // Do not include payout info, different scope
                }),
                pat.as_deref(),
            )
            .await
        };
        ScopeTest::new(&test_env)
            .with_user_id(ADMIN_USER_ID_PARSED)
            .test(req_gen, write_user)
            .await
            .unwrap();

        // User deletion
        // (The failure is first, and this is the last test for this test function, we can delete it and use the same PAT for both tests)
        let delete_user = Scopes::USER_DELETE;
        let req_gen = |pat: Option<String>| async move {
            api.delete_user("enemy", pat.as_deref()).await
        };
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
        let api = &test_env.api;
        let alpha_team_id = &test_env.dummy.project_alpha.team_id;

        // We will invite user 'friend' to project team, and use that as a notification
        // Get notifications
        let resp = test_env
            .api
            .add_user_to_team(
                alpha_team_id,
                FRIEND_USER_ID,
                None,
                None,
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // Notification get
        let read_notifications = Scopes::NOTIFICATION_READ;
        let req_gen = |pat: Option<String>| async move {
            api.get_user_notifications(FRIEND_USER_ID, pat.as_deref())
                .await
        };
        let (_, success) = ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, read_notifications)
            .await
            .unwrap();
        let notification_id = success[0]["id"].as_str().unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.get_notifications(&[notification_id], pat.as_deref())
                .await
        };
        ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, read_notifications)
            .await
            .unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.get_notification(notification_id, pat.as_deref()).await
        };
        ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, read_notifications)
            .await
            .unwrap();

        // Notification mark as read
        let write_notifications = Scopes::NOTIFICATION_WRITE;
        let req_gen = |pat: Option<String>| async move {
            api.mark_notifications_read(&[notification_id], pat.as_deref())
                .await
        };
        ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, write_notifications)
            .await
            .unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.mark_notification_read(notification_id, pat.as_deref())
                .await
        };
        ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, write_notifications)
            .await
            .unwrap();

        // Notification delete
        let req_gen = |pat: Option<String>| async move {
            api.delete_notification(notification_id, pat.as_deref())
                .await
        };
        ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, write_notifications)
            .await
            .unwrap();

        // Mass notification delete
        // We invite mod, get the notification ID, and do mass delete using that
        let resp = test_env
            .api
            .add_user_to_team(
                alpha_team_id,
                MOD_USER_ID,
                None,
                None,
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);
        let read_notifications = Scopes::NOTIFICATION_READ;
        let req_gen = |pat: Option<String>| async move {
            api.get_user_notifications(MOD_USER_ID, pat.as_deref())
                .await
        };
        let (_, success) = ScopeTest::new(&test_env)
            .with_user_id(MOD_USER_ID_PARSED)
            .test(req_gen, read_notifications)
            .await
            .unwrap();
        let notification_id = success[0]["id"].as_str().unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.delete_notifications(&[notification_id], pat.as_deref())
                .await
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
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;

            // Create project
            let create_project = Scopes::PROJECT_CREATE;
            let req_gen = |pat: Option<String>| async move {
                let creation_data = get_public_project_creation_data(
                    "demo",
                    Some(TestFile::BasicMod),
                    None,
                );
                api.create_project(creation_data, pat.as_deref()).await
            };
            let (_, success) = ScopeTest::new(&test_env)
                .test(req_gen, create_project)
                .await
                .unwrap();
            let project_id = success["id"].as_str().unwrap();
            let project_id = ProjectId(parse_base62(project_id).unwrap());

            // Add version to project
            let create_version = Scopes::VERSION_CREATE;
            let req_gen = |pat: Option<String>| async move {
                api.add_public_version(
                    project_id,
                    "1.2.3.4",
                    TestFile::BasicModDifferent,
                    None,
                    None,
                    pat.as_deref(),
                )
                .await
            };
            ScopeTest::new(&test_env)
                .test(req_gen, create_version)
                .await
                .unwrap();
        },
    )
    .await;
}

// Project management scopes
#[actix_rt::test]
pub async fn project_version_reads_scopes() {
    with_test_environment_all(None, |test_env| async move {
        let api = &test_env.api;
        let DummyProjectAlpha {
            team_id: alpha_team_id,
            ..
        } = &test_env.dummy.project_alpha;
        let DummyProjectBeta {
            project_id: beta_project_id,
            version_id: beta_version_id,
            file_hash: beta_file_hash,
            ..
        } = &test_env.dummy.project_beta;

        // Project reading
        // Uses 404 as the expected failure code (or 200 and an empty list for mass reads)
        let read_project = Scopes::PROJECT_READ;
        let req_gen = |pat: Option<String>| async move {
            api.get_project(beta_project_id, pat.as_deref()).await
        };
        ScopeTest::new(&test_env)
            .with_failure_code(404)
            .test(req_gen, read_project)
            .await
            .unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.get_project_dependencies(beta_project_id, pat.as_deref())
                .await
        };
        ScopeTest::new(&test_env)
            .with_failure_code(404)
            .test(req_gen, read_project)
            .await
            .unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.get_projects(&[beta_project_id.as_str()], pat.as_deref())
                .await
        };
        let (failure, success) = ScopeTest::new(&test_env)
            .with_failure_code(200)
            .test(req_gen, read_project)
            .await
            .unwrap();
        assert!(failure.as_array().unwrap().is_empty());
        assert!(!success.as_array().unwrap().is_empty());

        // Team project reading
        let req_gen = |pat: Option<String>| async move {
            api.get_project_members(beta_project_id, pat.as_deref())
                .await
        };
        ScopeTest::new(&test_env)
            .with_failure_code(404)
            .test(req_gen, read_project)
            .await
            .unwrap();

        // Get team members
        // In this case, as these are public endpoints, logging in only is relevant to showing permissions
        // So for our test project (with 1 user, 'user') we will check the permissions before and after having the scope.
        let req_gen = |pat: Option<String>| async move {
            api.get_team_members(alpha_team_id, pat.as_deref()).await
        };
        let (failure, success) = ScopeTest::new(&test_env)
            .with_failure_code(200)
            .test(req_gen, read_project)
            .await
            .unwrap();
        assert!(!failure[0]["permissions"].is_number());
        assert!(success[0]["permissions"].is_number());

        let req_gen = |pat: Option<String>| async move {
            api.get_teams_members(&[alpha_team_id.as_str()], pat.as_deref())
                .await
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
        let req_gen = |pat: Option<String>| async move {
            api.get_user_projects(USER_USER_ID, pat.as_deref()).await
        };
        let (failure, success) = ScopeTest::new(&test_env)
            .with_failure_code(200)
            .test(req_gen, read_project)
            .await
            .unwrap();
        assert!(
            !failure
                .as_array()
                .unwrap()
                .iter()
                .any(|x| x["status"] == "processing")
        );
        assert!(
            success
                .as_array()
                .unwrap()
                .iter()
                .any(|x| x["status"] == "processing")
        );

        // Project metadata reading
        let req_gen = |pat: Option<String>| async move {
            let req = test::TestRequest::get()
                .uri(&format!(
                    "/maven/maven/modrinth/{beta_project_id}/maven-metadata.xml"
                ))
                .append_pat(pat.as_deref())
                .to_request();
            api.call(req).await
        };
        ScopeTest::new(&test_env)
            .with_failure_code(404)
            .test(req_gen, read_project)
            .await
            .unwrap();

        // Version reading
        // First, set version to hidden (which is when the scope is required to read it)
        let read_version = Scopes::VERSION_READ;
        let resp = test_env
            .api
            .edit_version(
                beta_version_id,
                json!({ "status": "draft" }),
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        let req_gen = |pat: Option<String>| async move {
            api.get_version_from_hash(beta_file_hash, "sha1", pat.as_deref())
                .await
        };
        ScopeTest::new(&test_env)
            .with_failure_code(404)
            .test(req_gen, read_version)
            .await
            .unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.download_version_redirect(
                beta_file_hash,
                "sha1",
                pat.as_deref(),
            )
            .await
        };
        ScopeTest::new(&test_env)
            .with_failure_code(404)
            .test(req_gen, read_version)
            .await
            .unwrap();

        // TODO: This scope currently fails still as the 'version' field of QueryProject only allows public versions.
        // TODO: This will be fixed when the 'extracts_versions' PR is merged.
        // let req_gen = |pat: Option<String>| async move {
        //     api.get_update_from_hash(beta_file_hash, "sha1", None, None, None, pat.as_deref())
        //         .await
        // };
        // ScopeTest::new(&test_env).with_failure_code(404).test(req_gen, read_version).await.unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.get_versions_from_hashes(
                &[beta_file_hash],
                "sha1",
                pat.as_deref(),
            )
            .await
        };
        let (failure, success) = ScopeTest::new(&test_env)
            .with_failure_code(200)
            .test(req_gen, read_version)
            .await
            .unwrap();
        assert!(!failure.as_object().unwrap().contains_key(beta_file_hash));
        assert!(success.as_object().unwrap().contains_key(beta_file_hash));

        // Update version file
        // TODO: This scope currently fails still as the 'version' field of QueryProject only allows public versions.
        // TODO: This will be fixed when the 'extracts_versions' PR is merged.
        // let req_gen = |pat : Option<String>| async move {
        //     api.update_files("sha1", vec![beta_file_hash.clone()], None, None, None, pat.as_deref()).await
        // };
        // let (failure, success) = ScopeTest::new(&test_env).with_failure_code(200).test(req_gen, read_version).await.unwrap();
        // assert!(!failure.as_object().unwrap().contains_key(beta_file_hash));
        // assert!(success.as_object().unwrap().contains_key(beta_file_hash));

        // Both project and version reading
        let read_project_and_version =
            Scopes::PROJECT_READ | Scopes::VERSION_READ;
        let req_gen = |pat: Option<String>| async move {
            api.get_project_versions(
                beta_project_id,
                None,
                None,
                None,
                None,
                None,
                None,
                pat.as_deref(),
            )
            .await
        };
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
        let api = &test_env.api;
        let beta_project_id = &test_env.dummy.project_beta.project_id;
        let alpha_team_id = &test_env.dummy.project_alpha.team_id;

        // Projects writing
        let write_project = Scopes::PROJECT_WRITE;
        let req_gen = |pat: Option<String>| async move {
            api.edit_project(
                beta_project_id,
                json!(
                    {
                        "name": "test_project_version_write_scopes Title",
                    }
                ),
                pat.as_deref(),
            )
            .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.edit_project_bulk(
                &[beta_project_id.as_str()],
                json!(
                {
                    "description": "test_project_version_write_scopes Description"
                }),
                pat.as_deref(),
            )
            .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        // Icons and gallery images
        let req_gen = |pat: Option<String>| async move {
            api.edit_project_icon(
                beta_project_id,
                Some(DummyImage::SmallIcon.get_icon_data()),
                pat.as_deref(),
            )
            .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.edit_project_icon(beta_project_id, None, pat.as_deref())
                .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.add_gallery_item(
                beta_project_id,
                DummyImage::SmallIcon.get_icon_data(),
                true,
                None,
                None,
                None,
                pat.as_deref(),
            )
            .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        // Get project, as we need the gallery image url
        let resp = api.get_project(beta_project_id, USER_USER_PAT).await;
        let project: serde_json::Value = test::read_body_json(resp).await;
        let gallery_url = project["gallery"][0]["url"].as_str().unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.edit_gallery_item(beta_project_id, gallery_url, HashMap::new(), pat.as_deref())
                .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.remove_gallery_item(beta_project_id, gallery_url, pat.as_deref())
                .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        // Team scopes - add user 'friend'
        let req_gen = |pat: Option<String>| async move {
            api.add_user_to_team(alpha_team_id, FRIEND_USER_ID, None, None, pat.as_deref())
                .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        // Accept team invite as 'friend'
        let req_gen =
            |pat: Option<String>| async move { api.join_team(alpha_team_id, pat.as_deref()).await };
        ScopeTest::new(&test_env)
            .with_user_id(FRIEND_USER_ID_PARSED)
            .test(req_gen, write_project)
            .await
            .unwrap();

        // Patch 'friend' user
        let req_gen = |pat: Option<String>| async move {
            api.edit_team_member(
                alpha_team_id,
                FRIEND_USER_ID,
                json!({
                    "permissions": 1
                }),
                pat.as_deref(),
            )
            .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        // Transfer ownership to 'friend'
        let req_gen = |pat: Option<String>| async move {
            api.transfer_team_ownership(alpha_team_id, FRIEND_USER_ID, pat.as_deref())
                .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_project)
            .await
            .unwrap();

        // Now as 'friend', delete 'user'
        let req_gen = |pat: Option<String>| async move {
            api.remove_from_team(alpha_team_id, USER_USER_ID, pat.as_deref())
                .await
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
        let api = &test_env.api;
        let DummyProjectAlpha {
            version_id: alpha_version_id,
            file_hash: alpha_file_hash,
            ..
        } = &test_env.dummy.project_alpha;

        let write_version = Scopes::VERSION_WRITE;

        // Patch version
        let req_gen = |pat: Option<String>| async move {
            api.edit_version(
                alpha_version_id,
                json!(
                    {
                        "name": "test_version_write_scopes Title",
                    }
                ),
                pat.as_deref(),
            )
            .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_version)
            .await
            .unwrap();

        // Upload version file
        let req_gen = |pat: Option<String>| async move {
            api.upload_file_to_version(
                alpha_version_id,
                &TestFile::BasicZip,
                pat.as_deref(),
            )
            .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_version)
            .await
            .unwrap();

        //  Delete version file. Notably, this uses 'VERSION_WRITE' instead of 'VERSION_DELETE' as it is writing to the version
        let req_gen = |pat: Option<String>| async move {
            api.remove_version_file(alpha_file_hash, pat.as_deref())
                .await
            // Delete from alpha_version_id, as we uploaded to alpha_version_id and it needs another file
        };
        ScopeTest::new(&test_env)
            .test(req_gen, write_version)
            .await
            .unwrap();

        // Delete version
        let delete_version = Scopes::VERSION_DELETE;
        let req_gen = |pat: Option<String>| async move {
            api.remove_version(alpha_version_id, pat.as_deref()).await
        };
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
        let api = &test_env.api;
        let beta_project_id = &test_env.dummy.project_beta.project_id;

        // Create report
        let report_create = Scopes::REPORT_CREATE;
        let req_gen = |pat: Option<String>| async move {
            api.create_report(
                "copyright",
                beta_project_id,
                CommonItemType::Project,
                "This is a reupload of my mod",
                pat.as_deref(),
            )
            .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, report_create)
            .await
            .unwrap();

        // Get reports
        let report_read = Scopes::REPORT_READ;
        let req_gen = |pat: Option<String>| async move {
            api.get_user_reports(pat.as_deref()).await
        };
        let (_, success) = ScopeTest::new(&test_env)
            .test(req_gen, report_read)
            .await
            .unwrap();
        let report_id = success[0]["id"].as_str().unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.get_report(report_id, pat.as_deref()).await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, report_read)
            .await
            .unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.get_reports(&[report_id], pat.as_deref()).await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, report_read)
            .await
            .unwrap();

        // Edit report
        let report_edit = Scopes::REPORT_WRITE;
        let req_gen = |pat: Option<String>| async move {
            api.edit_report(
                report_id,
                json!({ "body": "This is a reupload of my mod, G8!" }),
                pat.as_deref(),
            )
            .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, report_edit)
            .await
            .unwrap();

        // Delete report
        // We use a moderator PAT here, as only moderators can delete reports
        let report_delete = Scopes::REPORT_DELETE;
        let req_gen = |pat: Option<String>| async move {
            api.delete_report(report_id, pat.as_deref()).await
        };
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
        let api = &test_env.api;
        let alpha_thread_id = &test_env.dummy.project_alpha.thread_id;
        let beta_thread_id = &test_env.dummy.project_beta.thread_id;

        // Thread read
        let thread_read = Scopes::THREAD_READ;
        let req_gen = |pat: Option<String>| async move {
            api.get_thread(alpha_thread_id, pat.as_deref()).await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, thread_read)
            .await
            .unwrap();

        let req_gen = |pat: Option<String>| async move {
            api.get_threads(&[alpha_thread_id.as_str()], pat.as_deref())
                .await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, thread_read)
            .await
            .unwrap();

        // Thread write (to also push to moderator inbox)
        let thread_write = Scopes::THREAD_WRITE;
        let req_gen = |pat: Option<String>| async move {
            api.write_to_thread(
                beta_thread_id,
                "text",
                "test_thread_scopes Body",
                pat.as_deref(),
            )
            .await
        };
        ScopeTest::new(&test_env)
            .with_user_id(USER_USER_ID_PARSED)
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
        let api = &test_env.api;
        // Pat create
        let pat_create = Scopes::PAT_CREATE;
        let req_gen = |pat: Option<String>| async move {
            let req = test::TestRequest::post()
                .uri("/_internal/pat")
                .set_json(json!({
                    "scopes": 1,
                    "name": "test_pat_scopes Name",
                    "expires": Utc::now() + Duration::days(1),
                }))
                .append_pat(pat.as_deref())
                .to_request();
            api.call(req).await
        };
        let (_, success) = ScopeTest::new(&test_env)
            .test(req_gen, pat_create)
            .await
            .unwrap();
        let pat_id = success["id"].as_str().unwrap();

        // Pat write
        let pat_write = Scopes::PAT_WRITE;
        let req_gen = |pat: Option<String>| async move {
            let req = test::TestRequest::patch()
                .uri(&format!("/_internal/pat/{pat_id}"))
                .set_json(json!({}))
                .append_pat(pat.as_deref())
                .to_request();
            api.call(req).await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, pat_write)
            .await
            .unwrap();

        // Pat read
        let pat_read = Scopes::PAT_READ;
        let req_gen = |pat: Option<String>| async move {
            let req = test::TestRequest::get()
                .uri("/_internal/pat")
                .append_pat(pat.as_deref())
                .to_request();
            api.call(req).await
        };
        ScopeTest::new(&test_env)
            .test(req_gen, pat_read)
            .await
            .unwrap();

        // Pat delete
        let pat_delete = Scopes::PAT_DELETE;
        let req_gen = |pat: Option<String>| async move {
            let req = test::TestRequest::delete()
                .uri(&format!("/_internal/pat/{pat_id}"))
                .append_pat(pat.as_deref())
                .to_request();
            api.call(req).await
        };
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
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let alpha_project_id = &test_env.dummy.project_alpha.project_id;

            // Create collection
            let collection_create = Scopes::COLLECTION_CREATE;
            let req_gen = |pat: Option<String>| async move {
                api.create_collection(
                    "Test Collection",
                    "Test Collection Description",
                    &[alpha_project_id.as_str()],
                    pat.as_deref(),
                )
                .await
            };
            let (_, success) = ScopeTest::new(&test_env)
                .test(req_gen, collection_create)
                .await
                .unwrap();
            let collection_id = success["id"].as_str().unwrap();

            // Patch collection
            // Collections always initialize to public, so we do patch before Get testing
            let collection_write = Scopes::COLLECTION_WRITE;
            let req_gen = |pat: Option<String>| async move {
                api.edit_collection(
                    collection_id,
                    json!({
                        "name": "Test Collection patch",
                        "status": "private",
                    }),
                    pat.as_deref(),
                )
                .await
            };
            ScopeTest::new(&test_env)
                .test(req_gen, collection_write)
                .await
                .unwrap();

            // Read collection
            let collection_read = Scopes::COLLECTION_READ;
            let req_gen = |pat: Option<String>| async move {
                api.get_collection(collection_id, pat.as_deref()).await
            };
            ScopeTest::new(&test_env)
                .with_failure_code(404)
                .test(req_gen, collection_read)
                .await
                .unwrap();

            let req_gen = |pat: Option<String>| async move {
                api.get_collections(&[collection_id], pat.as_deref()).await
            };
            let (failure, success) = ScopeTest::new(&test_env)
                .with_failure_code(200)
                .test(req_gen, collection_read)
                .await
                .unwrap();
            assert_eq!(failure.as_array().unwrap().len(), 0);
            assert_eq!(success.as_array().unwrap().len(), 1);

            let req_gen = |pat: Option<String>| async move {
                api.get_user_collections(USER_USER_ID, pat.as_deref()).await
            };
            let (failure, success) = ScopeTest::new(&test_env)
                .with_failure_code(200)
                .test(req_gen, collection_read)
                .await
                .unwrap();
            assert_eq!(failure.as_array().unwrap().len(), 0);
            assert_eq!(success.as_array().unwrap().len(), 1);

            let req_gen = |pat: Option<String>| async move {
                api.edit_collection_icon(
                    collection_id,
                    Some(DummyImage::SmallIcon.get_icon_data()),
                    pat.as_deref(),
                )
                .await
            };
            ScopeTest::new(&test_env)
                .test(req_gen, collection_write)
                .await
                .unwrap();

            let req_gen = |pat: Option<String>| async move {
                api.edit_collection_icon(collection_id, None, pat.as_deref())
                    .await
            };
            ScopeTest::new(&test_env)
                .test(req_gen, collection_write)
                .await
                .unwrap();
        },
    )
    .await;
}

// Organization scopes (and a couple PROJECT_WRITE scopes that are only allowed for orgs)
#[actix_rt::test]
pub async fn organization_scopes() {
    // Test setup and dummy data
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let beta_project_id = &test_env.dummy.project_beta.project_id;

            // Create organization
            let organization_create = Scopes::ORGANIZATION_CREATE;
            let req_gen = |pat: Option<String>| async move {
                api.create_organization(
                    "Test Org",
                    "TestOrg",
                    "TestOrg Description",
                    pat.as_deref(),
                )
                .await
            };
            let (_, success) = ScopeTest::new(&test_env)
                .test(req_gen, organization_create)
                .await
                .unwrap();
            let organization_id = success["id"].as_str().unwrap();

            // Patch organization
            let organization_edit = Scopes::ORGANIZATION_WRITE;
            let req_gen = |pat: Option<String>| async move {
                api.edit_organization(
                    organization_id,
                    json!({
                        "description": "TestOrg Patch Description",
                    }),
                    pat.as_deref(),
                )
                .await
            };
            ScopeTest::new(&test_env)
                .test(req_gen, organization_edit)
                .await
                .unwrap();

            let req_gen = |pat: Option<String>| async move {
                api.edit_organization_icon(
                    organization_id,
                    Some(DummyImage::SmallIcon.get_icon_data()),
                    pat.as_deref(),
                )
                .await
            };
            ScopeTest::new(&test_env)
                .test(req_gen, organization_edit)
                .await
                .unwrap();

            let req_gen = |pat: Option<String>| async move {
                api.edit_organization_icon(
                    organization_id,
                    None,
                    pat.as_deref(),
                )
                .await
            };
            ScopeTest::new(&test_env)
                .test(req_gen, organization_edit)
                .await
                .unwrap();

            // add project
            let organization_project_edit =
                Scopes::PROJECT_WRITE | Scopes::ORGANIZATION_WRITE;
            let req_gen = |pat: Option<String>| async move {
                api.organization_add_project(
                    organization_id,
                    beta_project_id,
                    pat.as_deref(),
                )
                .await
            };
            ScopeTest::new(&test_env)
                .with_failure_scopes(Scopes::all() ^ Scopes::ORGANIZATION_WRITE)
                .test(req_gen, organization_project_edit)
                .await
                .unwrap();

            // Organization reads
            let organization_read = Scopes::ORGANIZATION_READ;
            let req_gen = |pat: Option<String>| async move {
                api.get_organization(organization_id, pat.as_deref()).await
            };
            let (failure, success) = ScopeTest::new(&test_env)
                .with_failure_code(200)
                .test(req_gen, organization_read)
                .await
                .unwrap();
            assert!(failure["members"][0]["permissions"].is_null());
            assert!(!success["members"][0]["permissions"].is_null());

            let req_gen = |pat: Option<String>| async move {
                api.get_organizations(&[organization_id], pat.as_deref())
                    .await
            };

            let (failure, success) = ScopeTest::new(&test_env)
                .with_failure_code(200)
                .test(req_gen, organization_read)
                .await
                .unwrap();
            assert!(failure[0]["members"][0]["permissions"].is_null());
            assert!(!success[0]["members"][0]["permissions"].is_null());

            let organization_project_read =
                Scopes::PROJECT_READ | Scopes::ORGANIZATION_READ;
            let req_gen = |pat: Option<String>| async move {
                api.get_organization_projects(organization_id, pat.as_deref())
                    .await
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
            let req_gen = |pat: Option<String>| async move {
                api.organization_remove_project(
                    organization_id,
                    beta_project_id,
                    UserId(USER_USER_ID_PARSED as u64),
                    pat.as_deref(),
                )
                .await
            };
            ScopeTest::new(&test_env)
                .with_failure_scopes(Scopes::all() ^ Scopes::ORGANIZATION_WRITE)
                .test(req_gen, organization_project_edit)
                .await
                .unwrap();

            // Delete organization
            let organization_delete = Scopes::ORGANIZATION_DELETE;
            let req_gen = |pat: Option<String>| async move {
                api.delete_organization(organization_id, pat.as_deref())
                    .await
            };
            ScopeTest::new(&test_env)
                .test(req_gen, organization_delete)
                .await
                .unwrap();
        },
    )
    .await;
}

// TODO: Analytics scopes

// TODO: User authentication, and Session scopes

// TODO: Some hash/version files functions

// TODO: Image scopes

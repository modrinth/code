use actix_web::test;
use labrinth::models::teams::{OrganizationPermissions, ProjectPermissions};
use serde_json::json;

use crate::common::database::*;

use crate::common::environment::TestEnvironment;

// importing common module.
mod common;

#[actix_rt::test]
async fn test_get_team() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build(None).await;
    let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
    let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;
    let zeta_organization_id = &test_env
        .dummy
        .as_ref()
        .unwrap()
        .organization_zeta
        .organization_id;
    let zeta_team_id = &test_env.dummy.as_ref().unwrap().organization_zeta.team_id;

    // Perform tests for an organization team and a project team
    for (team_association_id, team_association, team_id) in [
        (alpha_project_id, "project", alpha_team_id),
        (zeta_organization_id, "organization", zeta_team_id),
    ] {
        // A non-member of the team should get basic info but not be able to see private data
        for uri in [
            format!("/v2/team/{team_id}/members"),
            format!("/v2/{team_association}/{team_association_id}/members"),
        ] {
            let req = test::TestRequest::get()
                .uri(&uri)
                .append_header(("Authorization", FRIEND_USER_PAT))
                .to_request();

            let resp = test_env.call(req).await;
            assert_eq!(resp.status(), 200);
            let value: serde_json::Value = test::read_body_json(resp).await;
            assert_eq!(value[0]["user"]["id"], USER_USER_ID);
            assert!(value[0]["permissions"].is_null());
        }

        // A non-accepted member of the team should:
        // - not be able to see private data about the team, but see all members including themselves
        // - should not appear in the team members list to enemy users
        let req = test::TestRequest::post()
            .uri(&format!("/v2/team/{team_id}/members"))
            .append_header(("Authorization", USER_USER_PAT))
            .set_json(&json!({
                "user_id": FRIEND_USER_ID,
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_eq!(resp.status(), 204);

        for uri in [
            format!("/v2/team/{team_id}/members"),
            format!("/v2/{team_association}/{team_association_id}/members"),
        ] {
            let req = test::TestRequest::get()
                .uri(&uri)
                .append_header(("Authorization", FRIEND_USER_PAT))
                .to_request();
            let resp = test_env.call(req).await;
            assert_eq!(resp.status(), 200);
            let value: serde_json::Value = test::read_body_json(resp).await;
            let members = value.as_array().unwrap();
            assert!(members.len() == 2); // USER_USER_ID and FRIEND_USER_ID should be in the team
            let user_user = members
                .iter()
                .find(|x| x["user"]["id"] == USER_USER_ID)
                .unwrap();
            let friend_user = members
                .iter()
                .find(|x| x["user"]["id"] == FRIEND_USER_ID)
                .unwrap();
            assert_eq!(user_user["user"]["id"], USER_USER_ID);
            assert!(user_user["permissions"].is_null()); // Should not see private data of the team
            assert_eq!(friend_user["user"]["id"], FRIEND_USER_ID);
            assert!(friend_user["permissions"].is_null());

            let req = test::TestRequest::get()
                .uri(&uri)
                .append_header(("Authorization", ENEMY_USER_PAT))
                .to_request();
            let resp = test_env.call(req).await;
            assert_eq!(resp.status(), 200);
            let value: serde_json::Value = test::read_body_json(resp).await;
            let members = value.as_array().unwrap();
            assert_eq!(members.len(), 1); // Only USER_USER_ID should be in the team
            assert_eq!(members[0]["user"]["id"], USER_USER_ID);
            assert!(members[0]["permissions"].is_null());
        }
        // An accepted member of the team should appear in the team members list
        // and should be able to see private data about the team
        let req = test::TestRequest::post()
            .uri(&format!("/v2/team/{team_id}/join"))
            .append_header(("Authorization", FRIEND_USER_PAT))
            .to_request();
        let resp = test_env.call(req).await;
        assert_eq!(resp.status(), 204);

        for uri in [
            format!("/v2/team/{team_id}/members"),
            format!("/v2/{team_association}/{team_association_id}/members"),
        ] {
            let req = test::TestRequest::get()
                .uri(&uri)
                .append_header(("Authorization", FRIEND_USER_PAT))
                .to_request();
            let resp = test_env.call(req).await;
            assert_eq!(resp.status(), 200);
            let value: serde_json::Value = test::read_body_json(resp).await;
            let members = value.as_array().unwrap();
            assert!(members.len() == 2); // USER_USER_ID and FRIEND_USER_ID should be in the team
            let user_user = members
                .iter()
                .find(|x| x["user"]["id"] == USER_USER_ID)
                .unwrap();
            let friend_user = members
                .iter()
                .find(|x| x["user"]["id"] == FRIEND_USER_ID)
                .unwrap();
            assert_eq!(user_user["user"]["id"], USER_USER_ID);
            assert!(!user_user["permissions"].is_null()); // SHOULD see private data of the team
            assert_eq!(friend_user["user"]["id"], FRIEND_USER_ID);
            assert!(!friend_user["permissions"].is_null());
        }
    }

    // Cleanup test db
    test_env.cleanup().await;
}

#[actix_rt::test]
async fn test_get_team_project_orgs() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build(None).await;
    let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
    let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;
    let zeta_organization_id = &test_env
        .dummy
        .as_ref()
        .unwrap()
        .organization_zeta
        .organization_id;
    let zeta_team_id = &test_env.dummy.as_ref().unwrap().organization_zeta.team_id;

    // Attach alpha to zeta
    let req = test::TestRequest::post()
        .uri(&format!("/v2/organization/{zeta_organization_id}/projects"))
        .append_header(("Authorization", USER_USER_PAT))
        .set_json(json!({
            "project_id": alpha_project_id,
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 200);

    // Invite and add friend to zeta
    let req = test::TestRequest::post()
        .uri(&format!("/v2/team/{zeta_team_id}/members"))
        .append_header(("Authorization", USER_USER_PAT))
        .set_json(json!({
            "user_id": FRIEND_USER_ID,
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    let req = test::TestRequest::post()
        .uri(&format!("/v2/team/{zeta_team_id}/join"))
        .append_header(("Authorization", FRIEND_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // The team members route from teams (on a project's team):
    // - the members of the project team specifically
    // - not the ones from the organization
    let req = test::TestRequest::get()
        .uri(&format!("/v2/team/{alpha_team_id}/members"))
        .append_header(("Authorization", FRIEND_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 200);
    let value: serde_json::Value = test::read_body_json(resp).await;
    let members = value.as_array().unwrap();
    assert_eq!(members.len(), 1);

    // The team members route from project should show:
    // - the members of the project team including the ones from the organization
    let req = test::TestRequest::get()
        .uri(&format!("/v2/project/{alpha_project_id}/members"))
        .append_header(("Authorization", FRIEND_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 200);
    let value: serde_json::Value = test::read_body_json(resp).await;
    let members = value.as_array().unwrap();
    assert_eq!(members.len(), 2);

    // Cleanup test db
    test_env.cleanup().await;
}

// edit team member (Varying permissions, varying roles)
#[actix_rt::test]
async fn test_patch_project_team_member() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build(None).await;
    let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;

    // Edit team as admin/mod but not a part of the team should be OK
    let req = test::TestRequest::patch()
        .uri(&format!("/v2/team/{alpha_team_id}/members/{USER_USER_ID}"))
        .set_json(json!({}))
        .append_header(("Authorization", ADMIN_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // As a non-owner with full permissions, attempt to edit the owner's permissions/roles
    let req = test::TestRequest::patch()
        .uri(&format!("/v2/team/{alpha_team_id}/members/{USER_USER_ID}"))
        .append_header(("Authorization", ADMIN_USER_PAT))
        .set_json(json!({
            "role": "member"
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 400);

    let req = test::TestRequest::patch()
        .uri(&format!("/v2/team/{alpha_team_id}/members/{USER_USER_ID}"))
        .append_header(("Authorization", ADMIN_USER_PAT))
        .set_json(json!({
            "permissions": 0
        }))
        .to_request();
    let resp = test_env.call(req).await;

    assert_eq!(resp.status(), 400);

    // Should not be able to edit organization permissions of a project team
    let req = test::TestRequest::patch()
        .uri(&format!("/v2/team/{alpha_team_id}/members/{USER_USER_ID}"))
        .append_header(("Authorization", USER_USER_PAT))
        .set_json(json!({
            "organization_permissions": 0
        }))
        .to_request();
    let resp = test_env.call(req).await;

    assert_eq!(resp.status(), 400);

    // Should not be able to add permissions to a user that the adding-user does not have
    // (true for both project and org)

    // first, invite friend
    let req = test::TestRequest::post()
        .uri(&format!("/v2/team/{alpha_team_id}/members"))
        .append_header(("Authorization", USER_USER_PAT))
        .set_json(&json!({
            "user_id": FRIEND_USER_ID,
            "permissions": (ProjectPermissions::EDIT_MEMBER | ProjectPermissions::EDIT_BODY).bits(),
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // accept
    let req = test::TestRequest::post()
        .uri(&format!("/v2/team/{alpha_team_id}/join"))
        .append_header(("Authorization", FRIEND_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // try to add permissions
    let req = test::TestRequest::patch()
    .uri(&format!("/v2/team/{alpha_team_id}/members/{FRIEND_USER_ID}"))
    .append_header(("Authorization", FRIEND_USER_PAT))
        .set_json(json!({
            "permissions": (ProjectPermissions::EDIT_MEMBER | ProjectPermissions::EDIT_DETAILS).bits()
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 400);

    // Cannot set a user to Owner
    let req = test::TestRequest::patch()
        .uri(&format!(
            "/v2/team/{alpha_team_id}/members/{FRIEND_USER_ID}"
        ))
        .append_header(("Authorization", USER_USER_PAT))
        .set_json(json!({
            "role": "Owner"
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 400);

    // Cannot set payouts outside of 0 and 5000
    for payout in [-1, 5001] {
        let req = test::TestRequest::patch()
            .uri(&format!(
                "/v2/team/{alpha_team_id}/members/{FRIEND_USER_ID}"
            ))
            .append_header(("Authorization", USER_USER_PAT))
            .set_json(json!({
                "payouts_split": payout
            }))
            .to_request();
        let resp = test_env.call(req).await;

        assert_eq!(resp.status(), 400);
    }

    // Successful patch
    let req = test::TestRequest::patch()
        .uri(&format!(
            "/v2/team/{alpha_team_id}/members/{FRIEND_USER_ID}"
        ))
        .append_header(("Authorization", FRIEND_USER_PAT))
        .set_json(json!({
            "payouts_split": 51,
            "permissions": ProjectPermissions::EDIT_MEMBER.bits(), // reduces permissions
            "role": "member",
            "ordering": 5
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // Check results
    let req = test::TestRequest::get()
        .uri(&format!("/v2/team/{alpha_team_id}/members"))
        .append_header(("Authorization", FRIEND_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 200);
    let value: serde_json::Value = test::read_body_json(resp).await;
    let member = value
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["user"]["id"] == FRIEND_USER_ID)
        .unwrap();
    assert_eq!(member["payouts_split"], 51.0);
    assert_eq!(
        member["permissions"],
        ProjectPermissions::EDIT_MEMBER.bits()
    );
    assert_eq!(member["role"], "member");
    assert_eq!(member["ordering"], 5);

    // Cleanup test db
    test_env.cleanup().await;
}

// edit team member (Varying permissions, varying roles)
#[actix_rt::test]
async fn test_patch_organization_team_member() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build(None).await;
    let zeta_team_id = &test_env.dummy.as_ref().unwrap().organization_zeta.team_id;

    // Edit team as admin/mod but not a part of the team should be OK
    let req = test::TestRequest::patch()
        .uri(&format!("/v2/team/{zeta_team_id}/members/{USER_USER_ID}"))
        .set_json(json!({}))
        .append_header(("Authorization", ADMIN_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // As a non-owner with full permissions, attempt to edit the owner's permissions/roles
    let req = test::TestRequest::patch()
        .uri(&format!("/v2/team/{zeta_team_id}/members/{USER_USER_ID}"))
        .append_header(("Authorization", ADMIN_USER_PAT))
        .set_json(json!({
            "role": "member"
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 400);

    let req = test::TestRequest::patch()
        .uri(&format!("/v2/team/{zeta_team_id}/members/{USER_USER_ID}"))
        .append_header(("Authorization", ADMIN_USER_PAT))
        .set_json(json!({
            "permissions": 0
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 400);

    // Should not be able to add permissions to a user that the adding-user does not have
    // (true for both project and org)

    // first, invite friend
    let req = test::TestRequest::post()
        .uri(&format!("/v2/team/{zeta_team_id}/members"))
        .append_header(("Authorization", USER_USER_PAT))
        .set_json(&json!({
            "user_id": FRIEND_USER_ID,
            "organization_permissions": (OrganizationPermissions::EDIT_MEMBER | OrganizationPermissions::EDIT_MEMBER_DEFAULT_PERMISSIONS).bits(),
        })).to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // accept
    let req = test::TestRequest::post()
        .uri(&format!("/v2/team/{zeta_team_id}/join"))
        .append_header(("Authorization", FRIEND_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // try to add permissions- fails, as we do not have EDIT_DETAILS
    let req = test::TestRequest::patch()
    .uri(&format!("/v2/team/{zeta_team_id}/members/{FRIEND_USER_ID}"))
    .append_header(("Authorization", FRIEND_USER_PAT))
        .set_json(json!({
            "organization_permissions": (OrganizationPermissions::EDIT_MEMBER | OrganizationPermissions::EDIT_DETAILS).bits()
        }))
        .to_request();
    let resp = test_env.call(req).await;

    assert_eq!(resp.status(), 400);

    // Cannot set a user to Owner
    let req = test::TestRequest::patch()
        .uri(&format!("/v2/team/{zeta_team_id}/members/{FRIEND_USER_ID}"))
        .append_header(("Authorization", USER_USER_PAT))
        .set_json(json!({
            "role": "Owner"
        }))
        .to_request();
    let resp = test_env.call(req).await;

    assert_eq!(resp.status(), 400);

    // Cannot set payouts outside of 0 and 5000
    for payout in [-1, 5001] {
        let req = test::TestRequest::patch()
            .uri(&format!("/v2/team/{zeta_team_id}/members/{FRIEND_USER_ID}"))
            .append_header(("Authorization", USER_USER_PAT))
            .set_json(json!({
                "payouts_split": payout
            }))
            .to_request();
        let resp = test_env.call(req).await;
        assert_eq!(resp.status(), 400);
    }

    // Successful patch
    let req = test::TestRequest::patch()
        .uri(&format!("/v2/team/{zeta_team_id}/members/{FRIEND_USER_ID}"))
        .append_header(("Authorization", FRIEND_USER_PAT))
        .set_json(json!({
            "payouts_split": 51,
            "organization_permissions": (OrganizationPermissions::EDIT_MEMBER).bits(), // reduces permissions
            "permissions": (ProjectPermissions::EDIT_MEMBER).bits(),
            "role": "member",
            "ordering": 5
        }))
        .to_request();
    let resp = test_env.call(req).await;

    assert_eq!(resp.status(), 204);

    // Check results
    let req = test::TestRequest::get()
        .uri(&format!("/v2/team/{zeta_team_id}/members"))
        .append_header(("Authorization", FRIEND_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 200);
    let value: serde_json::Value = test::read_body_json(resp).await;
    let member = value
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["user"]["id"] == FRIEND_USER_ID)
        .unwrap();
    assert_eq!(member["payouts_split"], 51.0);
    assert_eq!(
        member["organization_permissions"],
        OrganizationPermissions::EDIT_MEMBER.bits()
    );
    assert_eq!(
        member["permissions"],
        ProjectPermissions::EDIT_MEMBER.bits()
    );
    assert_eq!(member["role"], "member");
    assert_eq!(member["ordering"], 5);

    // Cleanup test db
    test_env.cleanup().await;
}

// trasnfer ownership (requires being owner, etc)
#[actix_rt::test]
async fn transfer_ownership() {
    // Test setup and dummy data
    let test_env = TestEnvironment::build(None).await;
    let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;

    // Cannot set friend as owner (not a member)
    let req = test::TestRequest::patch()
        .uri(&format!("/v2/team/{alpha_team_id}/owner"))
        .set_json(json!({
            "user_id": FRIEND_USER_ID
        }))
        .append_header(("Authorization", USER_USER_ID))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 401);

    // first, invite friend
    let req = test::TestRequest::post()
        .uri(&format!("/v2/team/{alpha_team_id}/members"))
        .append_header(("Authorization", USER_USER_PAT))
        .set_json(json!({
            "user_id": FRIEND_USER_ID,
        }))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // accept
    let req = test::TestRequest::post()
        .uri(&format!("/v2/team/{alpha_team_id}/join"))
        .append_header(("Authorization", FRIEND_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // Cannot set ourselves as owner
    let req = test::TestRequest::patch()
        .uri(&format!("/v2/team/{alpha_team_id}/owner"))
        .set_json(json!({
            "user_id": FRIEND_USER_ID
        }))
        .append_header(("Authorization", FRIEND_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 401);

    // Can set friend as owner
    let req = test::TestRequest::patch()
        .uri(&format!("/v2/team/{alpha_team_id}/owner"))
        .set_json(json!({
            "user_id": FRIEND_USER_ID
        }))
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 204);

    // Check
    let req = test::TestRequest::get()
        .uri(&format!("/v2/team/{alpha_team_id}/members"))
        .set_json(json!({
            "user_id": FRIEND_USER_ID
        }))
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 200);
    let value: serde_json::Value = test::read_body_json(resp).await;
    let friend_member = value
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["user"]["id"] == FRIEND_USER_ID)
        .unwrap();
    assert_eq!(friend_member["role"], "Owner");
    assert_eq!(
        friend_member["permissions"],
        ProjectPermissions::all().bits()
    );
    let user_member = value
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["user"]["id"] == USER_USER_ID)
        .unwrap();
    assert_eq!(user_member["role"], "Member");
    assert_eq!(user_member["permissions"], ProjectPermissions::all().bits());

    // Confirm that user, a user who still has full permissions, cannot then remove the owner
    let req = test::TestRequest::delete()
        .uri(&format!(
            "/v2/team/{alpha_team_id}/members/{FRIEND_USER_ID}"
        ))
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();

    let resp = test_env.call(req).await;
    assert_eq!(resp.status(), 401);

    // Cleanup test db
    test_env.cleanup().await;
}

// This test is currently not working.
// #[actix_rt::test]
// pub async fn no_acceptance_permissions() {
//     // Adding a user to a project team in an organization, when that user is in the organization but not the team,
//     // should have those permissions apply regardless of whether the user has accepted the invite or not.

//     // This is because project-team permission overrriding must be possible, and this overriding can decrease the number of permissions a user has.

//     let test_env = TestEnvironment::build(None).await;
//     let api = &test_env.v2;

//     let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;
//     let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
//     let zeta_organization_id = &test_env.dummy.as_ref().unwrap().zeta_organization_id;
//     let zeta_team_id = &test_env.dummy.as_ref().unwrap().zeta_team_id;

//     // Link alpha team to zeta org
//     let resp = api.organization_add_project(zeta_organization_id, alpha_project_id, USER_USER_PAT).await;
//     assert_eq!(resp.status(), 200);

//     // Invite friend to zeta team with all project default permissions
//     let resp = api.add_user_to_team(&zeta_team_id, FRIEND_USER_ID, Some(ProjectPermissions::all()), Some(OrganizationPermissions::all()), USER_USER_PAT).await;
//     assert_eq!(resp.status(), 204);

//     // Accept invite to zeta team
//     let resp = api.join_team(&zeta_team_id, FRIEND_USER_PAT).await;
//     assert_eq!(resp.status(), 204);

//     // Attempt, as friend, to edit details of alpha project (should succeed, org invite accepted)
//     let resp = api.edit_project(alpha_project_id, json!({
//         "title": "new name"
//     }), FRIEND_USER_PAT).await;
//     assert_eq!(resp.status(), 204);

//     // Invite friend to alpha team with *no* project permissions
//     let resp = api.add_user_to_team(&alpha_team_id, FRIEND_USER_ID, Some(ProjectPermissions::empty()), None, USER_USER_PAT).await;
//     assert_eq!(resp.status(), 204);

//     // Do not accept invite to alpha team

//     // Attempt, as friend, to edit details of alpha project (should fail now, even though user has not accepted invite)
//     let resp = api.edit_project(alpha_project_id, json!({
//         "title": "new name"
//     }), FRIEND_USER_PAT).await;
//     assert_eq!(resp.status(), 401);

//     test_env.cleanup().await;
// }

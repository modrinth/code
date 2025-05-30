use crate::common::{api_common::ApiTeams, database::*};
use actix_http::StatusCode;
use common::{
    api_v3::ApiV3,
    environment::{
        TestEnvironment, with_test_environment, with_test_environment_all,
    },
};
use labrinth::models::teams::{OrganizationPermissions, ProjectPermissions};
use rust_decimal::Decimal;
use serde_json::json;

mod common;

#[actix_rt::test]
async fn test_get_team() {
    // Test setup and dummy data
    // Perform get_team related tests for a project team
    //TODO: This needs to consider organizations now as well
    with_test_environment_all(None, |test_env| async move {
        let api = &test_env.api;
        let alpha_project_id = &test_env.dummy.project_alpha.project_id;
        let alpha_team_id = &test_env.dummy.project_alpha.team_id;

        // A non-member of the team should get basic info but not be able to see private data
        let members = api
            .get_team_members_deserialized_common(
                alpha_team_id,
                FRIEND_USER_PAT,
            )
            .await;
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].user.id.0, USER_USER_ID_PARSED as u64);
        assert!(members[0].permissions.is_none());

        let members = api
            .get_project_members_deserialized_common(
                alpha_project_id,
                FRIEND_USER_PAT,
            )
            .await;
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].user.id.0, USER_USER_ID_PARSED as u64);

        // A non-accepted member of the team should:
        // - not be able to see private data about the team, but see all members including themselves
        // - should not appear in the team members list to enemy users
        let resp = api
            .add_user_to_team(
                alpha_team_id,
                FRIEND_USER_ID,
                None,
                None,
                USER_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // Team check directly
        let members = api
            .get_team_members_deserialized_common(
                alpha_team_id,
                FRIEND_USER_PAT,
            )
            .await;
        assert!(members.len() == 2); // USER_USER_ID and FRIEND_USER_ID should be in the team
        let user_user = members
            .iter()
            .find(|x| x.user.id.0 == USER_USER_ID_PARSED as u64)
            .unwrap();
        let friend_user = members
            .iter()
            .find(|x| x.user.id.0 == FRIEND_USER_ID_PARSED as u64)
            .unwrap();
        assert_eq!(user_user.user.id.0, USER_USER_ID_PARSED as u64);
        assert!(user_user.permissions.is_none()); // Should not see private data of the team
        assert_eq!(friend_user.user.id.0, FRIEND_USER_ID_PARSED as u64);
        assert!(friend_user.permissions.is_none());

        // team check via association
        let members = api
            .get_project_members_deserialized_common(
                alpha_project_id,
                FRIEND_USER_PAT,
            )
            .await;
        assert!(members.len() == 2); // USER_USER_ID and FRIEND_USER_ID should be in the team
        let user_user = members
            .iter()
            .find(|x| x.user.id.0 == USER_USER_ID_PARSED as u64)
            .unwrap();
        let friend_user = members
            .iter()
            .find(|x| x.user.id.0 == FRIEND_USER_ID_PARSED as u64)
            .unwrap();
        assert_eq!(user_user.user.id.0, USER_USER_ID_PARSED as u64);
        assert!(user_user.permissions.is_none()); // Should not see private data of the team
        assert_eq!(friend_user.user.id.0, FRIEND_USER_ID_PARSED as u64);
        assert!(friend_user.permissions.is_none());

        // enemy team check directly
        let members = api
            .get_team_members_deserialized_common(alpha_team_id, ENEMY_USER_PAT)
            .await;
        assert_eq!(members.len(), 1); // Only USER_USER_ID should be in the team

        // enemy team check via association
        let members = api
            .get_project_members_deserialized_common(
                alpha_project_id,
                ENEMY_USER_PAT,
            )
            .await;
        assert_eq!(members.len(), 1); // Only USER_USER_ID should be in the team

        // An accepted member of the team should appear in the team members list
        // and should be able to see private data about the team
        let resp = api.join_team(alpha_team_id, FRIEND_USER_PAT).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // Team check directly
        let members = api
            .get_team_members_deserialized_common(
                alpha_team_id,
                FRIEND_USER_PAT,
            )
            .await;
        assert!(members.len() == 2); // USER_USER_ID and FRIEND_USER_ID should be in the team
        let user_user = members
            .iter()
            .find(|x| x.user.id.0 == USER_USER_ID_PARSED as u64)
            .unwrap();
        let friend_user = members
            .iter()
            .find(|x| x.user.id.0 == FRIEND_USER_ID_PARSED as u64)
            .unwrap();
        assert_eq!(user_user.user.id.0, USER_USER_ID_PARSED as u64);
        assert!(user_user.permissions.is_some()); // SHOULD see private data of the team
        assert_eq!(friend_user.user.id.0, FRIEND_USER_ID_PARSED as u64);
        assert!(friend_user.permissions.is_some());

        // team check via association
        let members = api
            .get_project_members_deserialized_common(
                alpha_project_id,
                FRIEND_USER_PAT,
            )
            .await;
        assert!(members.len() == 2); // USER_USER_ID and FRIEND_USER_ID should be in the team
        let user_user = members
            .iter()
            .find(|x| x.user.id.0 == USER_USER_ID_PARSED as u64)
            .unwrap();
        let friend_user = members
            .iter()
            .find(|x| x.user.id.0 == FRIEND_USER_ID_PARSED as u64)
            .unwrap();
        assert_eq!(user_user.user.id.0, USER_USER_ID_PARSED as u64);
        assert!(user_user.permissions.is_some()); // SHOULD see private data of the team
        assert_eq!(friend_user.user.id.0, FRIEND_USER_ID_PARSED as u64);
        assert!(friend_user.permissions.is_some());
    })
    .await;
}

#[actix_rt::test]
async fn test_get_team_organization() {
    // Test setup and dummy data
    // Perform get_team related tests for an organization team
    //TODO: This needs to consider users in organizations now and how they perceive as well
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let zeta_organization_id =
                &test_env.dummy.organization_zeta.organization_id;
            let zeta_team_id = &test_env.dummy.organization_zeta.team_id;

            // A non-member of the team should get basic info but not be able to see private data
            let members = api
                .get_team_members_deserialized_common(
                    zeta_team_id,
                    FRIEND_USER_PAT,
                )
                .await;
            assert_eq!(members.len(), 1);
            assert_eq!(members[0].user.id.0, USER_USER_ID_PARSED as u64);
            assert!(members[0].permissions.is_none());

            let members = api
                .get_organization_members_deserialized_common(
                    zeta_organization_id,
                    FRIEND_USER_PAT,
                )
                .await;
            assert_eq!(members.len(), 1);
            assert_eq!(members[0].user.id.0, USER_USER_ID_PARSED as u64);

            // A non-accepted member of the team should:
            // - not be able to see private data about the team, but see all members including themselves
            // - should not appear in the team members list to enemy users
            let resp = api
                .add_user_to_team(
                    zeta_team_id,
                    FRIEND_USER_ID,
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Team check directly
            let members = api
                .get_team_members_deserialized_common(
                    zeta_team_id,
                    FRIEND_USER_PAT,
                )
                .await;
            assert!(members.len() == 2); // USER_USER_ID and FRIEND_USER_ID should be in the team
            let user_user = members
                .iter()
                .find(|x| x.user.id.0 == USER_USER_ID_PARSED as u64)
                .unwrap();
            let friend_user = members
                .iter()
                .find(|x| x.user.id.0 == FRIEND_USER_ID_PARSED as u64)
                .unwrap();
            assert_eq!(user_user.user.id.0, USER_USER_ID_PARSED as u64);
            assert!(user_user.permissions.is_none()); // Should not see private data of the team
            assert_eq!(friend_user.user.id.0, FRIEND_USER_ID_PARSED as u64);
            assert!(friend_user.permissions.is_none());

            // team check via association
            let members = api
                .get_organization_members_deserialized_common(
                    zeta_organization_id,
                    FRIEND_USER_PAT,
                )
                .await;

            assert!(members.len() == 2); // USER_USER_ID and FRIEND_USER_ID should be in the team
            let user_user = members
                .iter()
                .find(|x| x.user.id.0 == USER_USER_ID_PARSED as u64)
                .unwrap();
            let friend_user = members
                .iter()
                .find(|x| x.user.id.0 == FRIEND_USER_ID_PARSED as u64)
                .unwrap();
            assert_eq!(user_user.user.id.0, USER_USER_ID_PARSED as u64);
            assert!(user_user.permissions.is_none()); // Should not see private data of the team
            assert_eq!(friend_user.user.id.0, FRIEND_USER_ID_PARSED as u64);
            assert!(friend_user.permissions.is_none());

            // enemy team check directly
            let members = api
                .get_team_members_deserialized_common(
                    zeta_team_id,
                    ENEMY_USER_PAT,
                )
                .await;
            assert_eq!(members.len(), 1); // Only USER_USER_ID should be in the team

            // enemy team check via association
            let members = api
                .get_organization_members_deserialized_common(
                    zeta_organization_id,
                    ENEMY_USER_PAT,
                )
                .await;
            assert_eq!(members.len(), 1); // Only USER_USER_ID should be in the team

            // An accepted member of the team should appear in the team members list
            // and should be able to see private data about the team
            let resp = api.join_team(zeta_team_id, FRIEND_USER_PAT).await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Team check directly
            let members = api
                .get_team_members_deserialized_common(
                    zeta_team_id,
                    FRIEND_USER_PAT,
                )
                .await;
            assert!(members.len() == 2); // USER_USER_ID and FRIEND_USER_ID should be in the team
            let user_user = members
                .iter()
                .find(|x| x.user.id.0 == USER_USER_ID_PARSED as u64)
                .unwrap();
            let friend_user = members
                .iter()
                .find(|x| x.user.id.0 == FRIEND_USER_ID_PARSED as u64)
                .unwrap();
            assert_eq!(user_user.user.id.0, USER_USER_ID_PARSED as u64);
            assert!(user_user.permissions.is_some()); // SHOULD see private data of the team
            assert_eq!(friend_user.user.id.0, FRIEND_USER_ID_PARSED as u64);
            assert!(friend_user.permissions.is_some());

            // team check via association
            let members = api
                .get_organization_members_deserialized_common(
                    zeta_organization_id,
                    FRIEND_USER_PAT,
                )
                .await;
            assert!(members.len() == 2); // USER_USER_ID and FRIEND_USER_ID should be in the team
            let user_user = members
                .iter()
                .find(|x| x.user.id.0 == USER_USER_ID_PARSED as u64)
                .unwrap();
            let friend_user = members
                .iter()
                .find(|x| x.user.id.0 == FRIEND_USER_ID_PARSED as u64)
                .unwrap();
            assert_eq!(user_user.user.id.0, USER_USER_ID_PARSED as u64);
            assert!(user_user.permissions.is_some()); // SHOULD see private data of the team
            assert_eq!(friend_user.user.id.0, FRIEND_USER_ID_PARSED as u64);
            assert!(friend_user.permissions.is_some());
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_get_team_project_orgs() {
    // Test setup and dummy data
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let alpha_project_id = &test_env.dummy.project_alpha.project_id;
            let alpha_team_id = &test_env.dummy.project_alpha.team_id;
            let zeta_organization_id =
                &test_env.dummy.organization_zeta.organization_id;
            let zeta_team_id = &test_env.dummy.organization_zeta.team_id;

            // Attach alpha to zeta
            let resp = test_env
                .api
                .organization_add_project(
                    zeta_organization_id,
                    alpha_project_id,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::OK);

            // Invite and add friend to zeta
            let resp = test_env
                .api
                .add_user_to_team(
                    zeta_team_id,
                    FRIEND_USER_ID,
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let resp =
                test_env.api.join_team(zeta_team_id, FRIEND_USER_PAT).await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // The team members route from teams (on a project's team):
            // - the members of the project team specifically
            // - not the ones from the organization
            // - Remember: the owner of an org will not be included in the org's team members list
            let members = test_env
                .api
                .get_team_members_deserialized_common(
                    alpha_team_id,
                    FRIEND_USER_PAT,
                )
                .await;
            assert_eq!(members.len(), 0);

            // The team members route from project should show the same!
            let members = test_env
                .api
                .get_project_members_deserialized_common(
                    alpha_project_id,
                    FRIEND_USER_PAT,
                )
                .await;
            assert_eq!(members.len(), 0);
        },
    )
    .await;
}

// edit team member (Varying permissions, varying roles)
#[actix_rt::test]
async fn test_patch_project_team_member() {
    // Test setup and dummy data
    with_test_environment_all(None, |test_env| async move {
        let api = &test_env.api;

        let alpha_team_id = &test_env.dummy.project_alpha.team_id;

        // Edit team as admin/mod but not a part of the team should be StatusCode::OK
        let resp = api.edit_team_member(alpha_team_id, USER_USER_ID, json!({}), ADMIN_USER_PAT).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // As a non-owner with full permissions, attempt to edit the owner's permissions
        let resp = api.edit_team_member(alpha_team_id, USER_USER_ID, json!({
            "permissions": 0
        }), ADMIN_USER_PAT).await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // Should not be able to edit organization permissions of a project team
        let resp = api.edit_team_member(alpha_team_id, USER_USER_ID, json!({
            "organization_permissions": 0
        }), USER_USER_PAT).await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // Should not be able to add permissions to a user that the adding-user does not have
        // (true for both project and org)

        // first, invite friend
        let resp = api.add_user_to_team(alpha_team_id, FRIEND_USER_ID,
            Some(ProjectPermissions::EDIT_MEMBER | ProjectPermissions::EDIT_BODY),
            None, USER_USER_PAT).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // accept
        let resp = api.join_team(alpha_team_id, FRIEND_USER_PAT).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // try to add permissions
        let resp = api.edit_team_member(alpha_team_id, FRIEND_USER_ID, json!({
            "permissions": (ProjectPermissions::EDIT_MEMBER | ProjectPermissions::EDIT_DETAILS).bits()
        }), FRIEND_USER_PAT).await; // should this be friend_user_pat
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // Cannot set payouts outside of 0 and 5000
        for payout in [-1, 5001] {
            let resp = api.edit_team_member(alpha_team_id, FRIEND_USER_ID, json!({
                "payouts_split": payout
            }), USER_USER_PAT).await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);
        }

        // Successful patch
        let resp = api.edit_team_member(alpha_team_id, FRIEND_USER_ID, json!({
                "payouts_split": 51,
                "permissions": ProjectPermissions::EDIT_MEMBER.bits(), // reduces permissions
                "role": "membe2r",
                "ordering": 5
        }), FRIEND_USER_PAT).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // Check results
        let members = api.get_team_members_deserialized_common(alpha_team_id, FRIEND_USER_PAT).await;
        let member = members.iter().find(|x| x.user.id.0 == FRIEND_USER_ID_PARSED as u64).unwrap();
        assert_eq!(member.payouts_split, Decimal::from_f64_retain(51.0));
        assert_eq!(member.permissions.unwrap(), ProjectPermissions::EDIT_MEMBER);
        assert_eq!(member.role, "membe2r");
        assert_eq!(member.ordering, 5);
    }).await;
}

// edit team member (Varying permissions, varying roles)
#[actix_rt::test]
async fn test_patch_organization_team_member() {
    // Test setup and dummy data
    with_test_environment(None, |test_env: TestEnvironment<ApiV3>| async move {
        let zeta_team_id = &test_env.dummy.organization_zeta.team_id;

        // Edit team as admin/mod but not a part of the team should be StatusCode::OK
        let resp = test_env
            .api
            .edit_team_member(zeta_team_id, USER_USER_ID, json!({}), ADMIN_USER_PAT)
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // As a non-owner with full permissions, attempt to edit the owner's permissions
        let resp = test_env
            .api
            .edit_team_member(zeta_team_id, USER_USER_ID, json!({ "permissions": 0 }), ADMIN_USER_PAT)
            .await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // Should not be able to add permissions to a user that the adding-user does not have
        // (true for both project and org)

        // first, invite friend
        let resp = test_env
            .api
            .add_user_to_team(zeta_team_id, FRIEND_USER_ID, None, Some(OrganizationPermissions::EDIT_MEMBER | OrganizationPermissions::EDIT_MEMBER_DEFAULT_PERMISSIONS), USER_USER_PAT)
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // accept
        let resp = test_env.api.join_team(zeta_team_id, FRIEND_USER_PAT).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // try to add permissions- fails, as we do not have EDIT_DETAILS
        let resp = test_env
            .api
            .edit_team_member(zeta_team_id, FRIEND_USER_ID, json!({ "organization_permissions": (OrganizationPermissions::EDIT_MEMBER | OrganizationPermissions::EDIT_DETAILS).bits() }), FRIEND_USER_PAT)
            .await;
        assert_status!(&resp, StatusCode::BAD_REQUEST);

        // Cannot set payouts outside of 0 and 5000
        for payout in [-1, 5001] {
            let resp = test_env
                .api
                .edit_team_member(zeta_team_id, FRIEND_USER_ID, json!({ "payouts_split": payout }), USER_USER_PAT)
                .await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);
        }

        // Successful patch
        let resp = test_env
            .api
            .edit_team_member(
                zeta_team_id,
                FRIEND_USER_ID,
                json!({
                    "payouts_split": 51,
                    "organization_permissions": OrganizationPermissions::EDIT_MEMBER.bits(), // reduces permissions
                    "permissions": (ProjectPermissions::EDIT_MEMBER).bits(),
                    "role": "very-cool-member",
                    "ordering": 5
                }),
                FRIEND_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // Check results
        let members = test_env
            .api
            .get_team_members_deserialized(zeta_team_id, FRIEND_USER_PAT)
            .await;
        let member = members
            .iter()
            .find(|x| x.user.id.0 == FRIEND_USER_ID_PARSED as u64)
            .unwrap();
        assert_eq!(member.payouts_split.unwrap(), Decimal::from_f64_retain(51.0_f64).unwrap());
        assert_eq!(
            member.organization_permissions,
            Some(OrganizationPermissions::EDIT_MEMBER)
        );
        assert_eq!(
            member.permissions,
            Some(ProjectPermissions::EDIT_MEMBER)
        );
        assert_eq!(member.role, "very-cool-member");
        assert_eq!(member.ordering, 5);

    }).await;
}

// trasnfer ownership (requires being owner, etc)
#[actix_rt::test]
async fn transfer_ownership_v3() {
    // Test setup and dummy data
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;

            let alpha_team_id = &test_env.dummy.project_alpha.team_id;

            // Cannot set friend as owner (not a member)
            let resp = api
                .transfer_team_ownership(
                    alpha_team_id,
                    FRIEND_USER_ID,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);
            let resp = api
                .transfer_team_ownership(
                    alpha_team_id,
                    FRIEND_USER_ID,
                    FRIEND_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::UNAUTHORIZED);

            // first, invite friend
            let resp = api
                .add_user_to_team(
                    alpha_team_id,
                    FRIEND_USER_ID,
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // still cannot set friend as owner (not accepted)
            let resp = api
                .transfer_team_ownership(
                    alpha_team_id,
                    FRIEND_USER_ID,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);

            // accept
            let resp = api.join_team(alpha_team_id, FRIEND_USER_PAT).await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Cannot set ourselves as owner if we are not owner
            let resp = api
                .transfer_team_ownership(
                    alpha_team_id,
                    FRIEND_USER_ID,
                    FRIEND_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::UNAUTHORIZED);

            // Can set friend as owner
            let resp = api
                .transfer_team_ownership(
                    alpha_team_id,
                    FRIEND_USER_ID,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Check
            let members = api
                .get_team_members_deserialized(alpha_team_id, USER_USER_PAT)
                .await;
            let friend_member = members
                .iter()
                .find(|x| x.user.id.0 == FRIEND_USER_ID_PARSED as u64)
                .unwrap();
            assert_eq!(friend_member.role, "Member"); // her role does not actually change, but is_owner is set to true
            assert!(friend_member.is_owner);
            assert_eq!(
                friend_member.permissions.unwrap(),
                ProjectPermissions::all()
            );

            let user_member = members
                .iter()
                .find(|x| x.user.id.0 == USER_USER_ID_PARSED as u64)
                .unwrap();
            assert_eq!(user_member.role, "Member"); // We are the 'owner', but we are not actually the owner!
            assert!(!user_member.is_owner);
            assert_eq!(
                user_member.permissions.unwrap(),
                ProjectPermissions::all()
            );

            // Confirm that user, a user who still has full permissions, cannot then remove the owner
            let resp = api
                .remove_from_team(alpha_team_id, FRIEND_USER_ID, USER_USER_PAT)
                .await;
            assert_status!(&resp, StatusCode::UNAUTHORIZED);

            // V3 only- confirm the owner can change their role without losing ownership
            let resp = api
                .edit_team_member(
                    alpha_team_id,
                    FRIEND_USER_ID,
                    json!({
                        "role": "Member"
                    }),
                    FRIEND_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let members = api
                .get_team_members_deserialized(alpha_team_id, USER_USER_PAT)
                .await;
            let friend_member = members
                .iter()
                .find(|x| x.user.id.0 == FRIEND_USER_ID_PARSED as u64)
                .unwrap();
            assert_eq!(friend_member.role, "Member");
            assert!(friend_member.is_owner);
        },
    )
    .await;
}

// This test is currently not working.
// #[actix_rt::test]
// pub async fn no_acceptance_permissions() {
//     // Adding a user to a project team in an organization, when that user is in the organization but not the team,
//     // should have those permissions apply regardless of whether the user has accepted the invite or not.

//     // This is because project-team permission overrriding must be possible, and this overriding can decrease the number of permissions a user has.

//     let test_env = TestEnvironment::build(None).await;
//     let api = &test_env.api;

//     let alpha_team_id = &test_env.dummy.project_alpha.team_id;
//     let alpha_project_id = &test_env.dummy.project_alpha.project_id;
//     let zeta_organization_id = &test_env.dummy.zeta_organization_id;
//     let zeta_team_id = &test_env.dummy.zeta_team_id;

//     // Link alpha team to zeta org
//     let resp = api.organization_add_project(zeta_organization_id, alpha_project_id, USER_USER_PAT).await;
//     assert_status!(&resp, StatusCode::OK);

//     // Invite friend to zeta team with all project default permissions
//     let resp = api.add_user_to_team(&zeta_team_id, FRIEND_USER_ID, Some(ProjectPermissions::all()), Some(OrganizationPermissions::all()), USER_USER_PAT).await;
//     assert_status!(&resp, StatusCode::NO_CONTENT);

//     // Accept invite to zeta team
//     let resp = api.join_team(&zeta_team_id, FRIEND_USER_PAT).await;
//     assert_status!(&resp, StatusCode::NO_CONTENT);

//     // Attempt, as friend, to edit details of alpha project (should succeed, org invite accepted)
//     let resp = api.edit_project(alpha_project_id, json!({
//         "title": "new name"
//     }), FRIEND_USER_PAT).await;
//     assert_status!(&resp, StatusCode::NO_CONTENT);

//     // Invite friend to alpha team with *no* project permissions
//     let resp = api.add_user_to_team(&alpha_team_id, FRIEND_USER_ID, Some(ProjectPermissions::empty()), None, USER_USER_PAT).await;
//     assert_status!(&resp, StatusCode::NO_CONTENT);

//     // Do not accept invite to alpha team

//     // Attempt, as friend, to edit details of alpha project (should fail now, even though user has not accepted invite)
//     let resp = api.edit_project(alpha_project_id, json!({
//         "title": "new name"
//     }), FRIEND_USER_PAT).await;
//     assert_status!(&resp, StatusCode::UNAUTHORIZED);

//     test_env.cleanup().await;
// }

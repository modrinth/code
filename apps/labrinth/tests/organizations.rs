use crate::common::{
    api_common::{ApiProject, ApiTeams},
    database::{
        ADMIN_USER_PAT, ENEMY_USER_ID_PARSED, ENEMY_USER_PAT,
        FRIEND_USER_ID_PARSED, MOD_USER_ID, MOD_USER_PAT, USER_USER_ID,
        USER_USER_ID_PARSED, generate_random_name,
    },
    dummy_data::{
        DummyImage, DummyOrganizationZeta, DummyProjectAlpha, DummyProjectBeta,
    },
};
use actix_http::StatusCode;
use ariadne::ids::UserId;
use common::{
    api_v3::ApiV3,
    database::{FRIEND_USER_ID, FRIEND_USER_PAT, USER_USER_PAT},
    environment::{
        TestEnvironment, with_test_environment, with_test_environment_all,
    },
    permissions::{PermissionsTest, PermissionsTestContext},
};
use labrinth::models::teams::{OrganizationPermissions, ProjectPermissions};
use serde_json::json;

mod common;

#[actix_rt::test]
async fn create_organization() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let zeta_organization_slug =
                &test_env.dummy.organization_zeta.organization_id;

            // Failed creations title:
            // - too short title
            // - too long title
            for title in ["a", &"a".repeat(100)] {
                let resp = api
                    .create_organization(
                        title,
                        "theta",
                        "theta_description",
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::BAD_REQUEST);
            }

            // Failed creations slug:
            // - slug collision with zeta
            // - too short slug
            // - too long slug
            // - not url safe slug
            for slug in [
                zeta_organization_slug,
                "a",
                &"a".repeat(100),
                "not url safe%&^!#$##!@#$%^&*()",
            ] {
                let resp = api
                    .create_organization(
                        "Theta Org",
                        slug,
                        "theta_description",
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::BAD_REQUEST);
            }

            // Failed creations description:
            // - too short desc
            // - too long desc
            for description in ["a", &"a".repeat(300)] {
                let resp = api
                    .create_organization(
                        "Theta Org",
                        "theta",
                        description,
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::BAD_REQUEST);
            }

            // Create 'theta' organization
            let resp = api
                .create_organization(
                    "Theta Org",
                    "theta",
                    "not url safe%&^!#$##!@#$%^&",
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::OK);

            // Get organization using slug
            let theta = api
                .get_organization_deserialized("theta", USER_USER_PAT)
                .await;
            assert_eq!(theta.name, "Theta Org");
            assert_eq!(theta.slug, "theta");
            assert_eq!(theta.description, "not url safe%&^!#$##!@#$%^&");
            assert_status!(&resp, StatusCode::OK);

            // Get created team
            let members = api
                .get_organization_members_deserialized("theta", USER_USER_PAT)
                .await;

            // Should only be one member, which is USER_USER_ID, and is the owner with full permissions
            assert_eq!(members[0].user.id.to_string(), USER_USER_ID);
            assert_eq!(
                members[0].organization_permissions,
                Some(OrganizationPermissions::all())
            );
            assert_eq!(members[0].role, "Member");
            assert!(members[0].is_owner);
        },
    )
    .await;
}

#[actix_rt::test]
async fn get_project_organization() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let zeta_organization_id =
                &test_env.dummy.organization_zeta.organization_id;
            let alpha_project_id = &test_env.dummy.project_alpha.project_id;

            // ADd alpha project to zeta organization
            let resp = api
                .organization_add_project(
                    zeta_organization_id,
                    alpha_project_id,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::OK);

            // Get project organization
            let zeta = api
                .get_project_organization_deserialized(
                    alpha_project_id,
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(zeta.id.to_string(), zeta_organization_id.to_string());
        },
    )
    .await;
}

#[actix_rt::test]
async fn patch_organization() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;

            let zeta_organization_id =
                &test_env.dummy.organization_zeta.organization_id;

            // Create 'theta' organization
            let resp = api
                .create_organization(
                    "Theta Org",
                    "theta",
                    "theta_description",
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::OK);

            // Failed patch to theta title:
            // - too short title
            // - too long title
            for title in ["a", &"a".repeat(100)] {
                let resp = api
                    .edit_organization(
                        "theta",
                        json!({
                            "name": title,
                        }),
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::BAD_REQUEST);
            }

            // Failed patch to zeta slug:
            // - slug collision with theta
            // - too short slug
            // - too long slug
            // - not url safe slug
            for title in [
                "theta",
                "a",
                &"a".repeat(100),
                "not url safe%&^!#$##!@#$%^&*()",
            ] {
                let resp = api
                    .edit_organization(
                        zeta_organization_id,
                        json!({
                            "slug": title,
                            "description": "theta_description"
                        }),
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::BAD_REQUEST);
            }

            // Failed patch to zeta description:
            // - too short description
            // - too long description
            for description in ["a", &"a".repeat(300)] {
                let resp = api
                    .edit_organization(
                        zeta_organization_id,
                        json!({
                            "description": description
                        }),
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::BAD_REQUEST);
            }

            // Successful patch to many fields
            let resp = api
                .edit_organization(
                    zeta_organization_id,
                    json!({
                        "name": "new_title",
                        "slug": "new_slug",
                        "description": "not url safe%&^!#$##!@#$%^&" // not-URL-safe description should still work
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Get project using new slug
            let new_title = api
                .get_organization_deserialized("new_slug", USER_USER_PAT)
                .await;
            assert_eq!(new_title.name, "new_title");
            assert_eq!(new_title.slug, "new_slug");
            assert_eq!(new_title.description, "not url safe%&^!#$##!@#$%^&");
        },
    )
    .await;
}

// add/remove icon
#[actix_rt::test]
async fn add_remove_icon() {
    with_test_environment(
        Some(10),
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let zeta_organization_id =
                &test_env.dummy.organization_zeta.organization_id;

            // Get project
            let resp = test_env
                .api
                .get_organization_deserialized(
                    zeta_organization_id,
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(resp.icon_url, None);

            // Icon edit
            // Uses alpha organization to delete this icon
            let resp = api
                .edit_organization_icon(
                    zeta_organization_id,
                    Some(DummyImage::SmallIcon.get_icon_data()),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Get project
            let zeta_org = api
                .get_organization_deserialized(
                    zeta_organization_id,
                    USER_USER_PAT,
                )
                .await;
            assert!(zeta_org.icon_url.is_some());

            // Icon delete
            // Uses alpha organization to delete added icon
            let resp = api
                .edit_organization_icon(
                    zeta_organization_id,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Get project
            let zeta_org = api
                .get_organization_deserialized(
                    zeta_organization_id,
                    USER_USER_PAT,
                )
                .await;
            assert!(zeta_org.icon_url.is_none());
        },
    )
    .await;
}

// delete org
#[actix_rt::test]
async fn delete_org() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let zeta_organization_id =
                &test_env.dummy.organization_zeta.organization_id;

            let resp = api
                .delete_organization(zeta_organization_id, USER_USER_PAT)
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Get organization, which should no longer exist
            let resp = api
                .get_organization(zeta_organization_id, USER_USER_PAT)
                .await;
            assert_status!(&resp, StatusCode::NOT_FOUND);
        },
    )
    .await;
}

// add/remove organization projects
#[actix_rt::test]
async fn add_remove_organization_projects() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let alpha_project_id: &str =
                &test_env.dummy.project_alpha.project_id;
            let alpha_project_slug: &str =
                &test_env.dummy.project_alpha.project_slug;
            let zeta_organization_id: &str =
                &test_env.dummy.organization_zeta.organization_id;

            // user's page should show alpha project
            // It may contain more than one project, depending on dummy data, but should contain the alpha project
            let projects = test_env
                .api
                .get_user_projects_deserialized_common(
                    USER_USER_ID,
                    USER_USER_PAT,
                )
                .await;
            assert!(
                projects
                    .iter()
                    .any(|p| p.id.to_string() == alpha_project_id)
            );

            // Add/remove project to organization, first by ID, then by slug
            for alpha in [alpha_project_id, alpha_project_slug] {
                let resp = test_env
                    .api
                    .organization_add_project(
                        zeta_organization_id,
                        alpha,
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::OK);

                // Get organization projects
                let projects = test_env
                    .api
                    .get_organization_projects_deserialized(
                        zeta_organization_id,
                        USER_USER_PAT,
                    )
                    .await;
                assert_eq!(projects[0].id.to_string(), alpha_project_id);
                assert_eq!(
                    projects[0].slug,
                    Some(alpha_project_slug.to_string())
                );

                // Currently, intended behaviour is that user's page should NOT show organization projects.
                // It may contain other projects, depending on dummy data, but should not contain the alpha project
                let projects = test_env
                    .api
                    .get_user_projects_deserialized_common(
                        USER_USER_ID,
                        USER_USER_PAT,
                    )
                    .await;
                assert!(
                    !projects
                        .iter()
                        .any(|p| p.id.to_string() == alpha_project_id)
                );

                // Remove project from organization
                let resp = test_env
                    .api
                    .organization_remove_project(
                        zeta_organization_id,
                        alpha,
                        UserId(USER_USER_ID_PARSED as u64),
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::OK);

                // Get user's projects as user - should be 1, the alpha project,
                // as we gave back ownership to the user when we removed it from the organization
                // So user's page should show the alpha project (and possibly others)
                let projects = test_env
                    .api
                    .get_user_projects_deserialized_common(
                        USER_USER_ID,
                        USER_USER_PAT,
                    )
                    .await;
                assert!(
                    projects
                        .iter()
                        .any(|p| p.id.to_string() == alpha_project_id)
                );

                // Get organization projects
                let projects = test_env
                    .api
                    .get_organization_projects_deserialized(
                        zeta_organization_id,
                        USER_USER_PAT,
                    )
                    .await;
                assert!(projects.is_empty());
            }
        },
    )
    .await;
}

// Like above, but specifically regarding ownership transferring
#[actix_rt::test]
async fn add_remove_organization_project_ownership_to_user() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let DummyProjectAlpha {
                project_id: alpha_project_id,
                team_id: alpha_team_id,
                ..
            } = &test_env.dummy.project_alpha;
            let DummyProjectBeta {
                project_id: beta_project_id,
                team_id: beta_team_id,
                ..
            } = &test_env.dummy.project_beta;
            let DummyOrganizationZeta {
                organization_id: zeta_organization_id,
                team_id: zeta_team_id,
                ..
            } = &test_env.dummy.organization_zeta;

            // Add friend to alpha, beta, and zeta
            for (team, organization) in [
                (alpha_team_id, false),
                (beta_team_id, false),
                (zeta_team_id, true),
            ] {
                let org_permissions = if organization {
                    Some(OrganizationPermissions::all())
                } else {
                    None
                };
                let resp = test_env
                    .api
                    .add_user_to_team(
                        team,
                        FRIEND_USER_ID,
                        Some(ProjectPermissions::all()),
                        org_permissions,
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::NO_CONTENT);

                // Accept invites
                let resp = test_env.api.join_team(team, FRIEND_USER_PAT).await;
                assert_status!(&resp, StatusCode::NO_CONTENT);
            }

            // For each team, confirm there are two members, but only one owner of the project, and it is USER_USER_ID
            for team in [alpha_team_id, beta_team_id, zeta_team_id] {
                let members = test_env
                    .api
                    .get_team_members_deserialized(team, USER_USER_PAT)
                    .await;
                assert_eq!(members.len(), 2);
                let user_member =
                    members.iter().filter(|m| m.is_owner).collect::<Vec<_>>();
                assert_eq!(user_member.len(), 1);
                assert_eq!(user_member[0].user.id.to_string(), USER_USER_ID);
            }

            // Transfer ownership of beta project to FRIEND
            let resp = test_env
                .api
                .transfer_team_ownership(
                    beta_team_id,
                    FRIEND_USER_ID,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Confirm there are still two users, but now FRIEND_USER_ID is the owner
            let members = test_env
                .api
                .get_team_members_deserialized(beta_team_id, USER_USER_PAT)
                .await;
            assert_eq!(members.len(), 2);
            let user_member =
                members.iter().filter(|m| m.is_owner).collect::<Vec<_>>();
            assert_eq!(user_member.len(), 1);
            assert_eq!(user_member[0].user.id.to_string(), FRIEND_USER_ID);

            // Add alpha, beta to zeta organization
            for (project_id, pat) in [
                (alpha_project_id, USER_USER_PAT),
                (beta_project_id, FRIEND_USER_PAT),
            ] {
                let resp = test_env
                    .api
                    .organization_add_project(
                        zeta_organization_id,
                        project_id,
                        pat,
                    )
                    .await;
                assert_status!(&resp, StatusCode::OK);

                // Get and confirm it has been added
                let project = test_env
                    .api
                    .get_project_deserialized(project_id, pat)
                    .await;
                assert_eq!(
                    &project.organization.unwrap().to_string(),
                    zeta_organization_id
                );
            }

            // Alpha project should have:
            // - 1 member, FRIEND_USER_ID
            //      -> User was removed entirely as a team_member as it is now the owner of the organization
            // - No owner.
            //      -> For alpha, user was removed as owner when it was added to the organization
            //      -> Friend was never an owner of the alpha project
            let members = test_env
                .api
                .get_team_members_deserialized(alpha_team_id, USER_USER_PAT)
                .await;
            assert_eq!(members.len(), 1);
            assert_eq!(members[0].user.id.to_string(), FRIEND_USER_ID);
            let user_member =
                members.iter().filter(|m| m.is_owner).collect::<Vec<_>>();
            assert_eq!(user_member.len(), 0);

            // Beta project should have:
            // - No members
            // -> User was removed entirely as a team_member as it is now the owner of the organization
            // -> Friend was made owner of the beta project, but was removed as a member when it was added to the organization
            // If you are owner of a projeect, you are removed from the team when it is added to an organization,
            // so that your former permissions are not overriding the organization permissions by default.
            let members = test_env
                .api
                .get_team_members_deserialized(beta_team_id, USER_USER_PAT)
                .await;
            assert!(members.is_empty());

            // Transfer ownership of zeta organization to FRIEND
            let resp = test_env
                .api
                .transfer_team_ownership(
                    zeta_team_id,
                    FRIEND_USER_ID,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Confirm there are no members of the alpha project OR the beta project
            // - Friend was removed as a member of these projects when ownership was transferred to them
            for team_id in [alpha_team_id, beta_team_id] {
                let members = test_env
                    .api
                    .get_team_members_deserialized(team_id, USER_USER_PAT)
                    .await;
                assert!(members.is_empty());
            }

            // As user, cannot add friend to alpha project, as they are the org owner
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
            assert_status!(&resp, StatusCode::BAD_REQUEST);

            // As friend, can add user to alpha project, as they are not the org owner
            let resp = test_env
                .api
                .add_user_to_team(
                    alpha_team_id,
                    USER_USER_ID,
                    None,
                    None,
                    FRIEND_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // At this point, friend owns the org
            // Alpha member has user as a member, but not as an owner
            // Neither project has an owner, as they are owned by the org

            // Remove project from organization with a user that is not an organization member
            // This should fail as we cannot give a project to a user that is not a member of the organization
            let resp = test_env
                .api
                .organization_remove_project(
                    zeta_organization_id,
                    alpha_project_id,
                    UserId(ENEMY_USER_ID_PARSED as u64),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);

            // Set user's permissions within the project that it is a member of to none (for a later test)
            let resp = test_env
                .api
                .edit_team_member(
                    alpha_team_id,
                    USER_USER_ID,
                    json!({
                        "project_permissions": 0,
                    }),
                    FRIEND_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Remove project from organization with a user that is an organization member, and a project member
            // This should succeed
            let resp = test_env
                .api
                .organization_remove_project(
                    zeta_organization_id,
                    alpha_project_id,
                    UserId(USER_USER_ID_PARSED as u64),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::OK);

            // Remove project from organization with a user that is an organization member, but not a project member
            // This should succeed
            let resp = test_env
                .api
                .organization_remove_project(
                    zeta_organization_id,
                    beta_project_id,
                    UserId(USER_USER_ID_PARSED as u64),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::OK);

            // For each of alpha and beta, confirm:
            // - There is one member of each project, the owner, USER_USER_ID
            // - In addition to being the owner, they have full permissions (even though they were set to none earlier)
            // - They no longer have an attached organization
            for team_id in [alpha_team_id, beta_team_id] {
                let members = test_env
                    .api
                    .get_team_members_deserialized(team_id, USER_USER_PAT)
                    .await;
                assert_eq!(members.len(), 1);
                let user_member =
                    members.iter().filter(|m| m.is_owner).collect::<Vec<_>>();
                assert_eq!(user_member.len(), 1);
                assert_eq!(user_member[0].user.id.to_string(), USER_USER_ID);
                assert_eq!(
                    user_member[0].permissions.unwrap(),
                    ProjectPermissions::all()
                );
            }

            for project_id in [alpha_project_id, beta_project_id] {
                let project = test_env
                    .api
                    .get_project_deserialized(project_id, USER_USER_PAT)
                    .await;
                assert!(project.organization.is_none());
            }
        },
    )
    .await;
}

#[actix_rt::test]
async fn delete_organization_means_all_projects_to_org_owner() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let DummyProjectAlpha {
                project_id: alpha_project_id,
                team_id: alpha_team_id,
                ..
            } = &test_env.dummy.project_alpha;
            let DummyProjectBeta {
                project_id: beta_project_id,
                team_id: beta_team_id,
                ..
            } = &test_env.dummy.project_beta;
            let DummyOrganizationZeta {
                organization_id: zeta_organization_id,
                team_id: zeta_team_id,
                ..
            } = &test_env.dummy.organization_zeta;

            // Create random project from enemy, ensure it wont get affected
            let (enemy_project, _) = test_env
                .api
                .add_public_project("enemy_project", None, None, ENEMY_USER_PAT)
                .await;

            // Add FRIEND
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

            // Accept invite
            let resp =
                test_env.api.join_team(zeta_team_id, FRIEND_USER_PAT).await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Confirm there is only one owner of the project, and it is USER_USER_ID
            let members = test_env
                .api
                .get_team_members_deserialized(alpha_team_id, USER_USER_PAT)
                .await;
            let user_member =
                members.iter().filter(|m| m.is_owner).collect::<Vec<_>>();
            assert_eq!(user_member.len(), 1);
            assert_eq!(user_member[0].user.id.to_string(), USER_USER_ID);

            // Add alpha to zeta organization
            let resp = test_env
                .api
                .organization_add_project(
                    zeta_organization_id,
                    alpha_project_id,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::OK);

            // Add beta to zeta organization
            test_env
                .api
                .organization_add_project(
                    zeta_organization_id,
                    beta_project_id,
                    USER_USER_PAT,
                )
                .await;

            // Add friend as a member of the beta project
            let resp = test_env
                .api
                .add_user_to_team(
                    beta_team_id,
                    FRIEND_USER_ID,
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Try to accept invite
            // This returns a failure, because since beta and FRIEND are in the organizations,
            // they can be added to the project without an invite
            let resp =
                test_env.api.join_team(beta_team_id, FRIEND_USER_PAT).await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);

            // Confirm there is NO owner of the project, as it is owned by the organization
            let members = test_env
                .api
                .get_team_members_deserialized(alpha_team_id, USER_USER_PAT)
                .await;
            let user_member =
                members.iter().filter(|m| m.is_owner).collect::<Vec<_>>();
            assert_eq!(user_member.len(), 0);

            // Transfer ownership of zeta organization to FRIEND
            let resp = test_env
                .api
                .transfer_team_ownership(
                    zeta_team_id,
                    FRIEND_USER_ID,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Confirm there is NO owner of the project, as it is owned by the organization
            let members = test_env
                .api
                .get_team_members_deserialized(alpha_team_id, USER_USER_PAT)
                .await;
            let user_member =
                members.iter().filter(|m| m.is_owner).collect::<Vec<_>>();
            assert_eq!(user_member.len(), 0);

            // Delete organization
            let resp = test_env
                .api
                .delete_organization(zeta_organization_id, FRIEND_USER_PAT)
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Confirm there is only one owner of the alpha project, and it is now FRIEND_USER_ID
            let members = test_env
                .api
                .get_team_members_deserialized(alpha_team_id, USER_USER_PAT)
                .await;
            let user_member =
                members.iter().filter(|m| m.is_owner).collect::<Vec<_>>();
            assert_eq!(user_member.len(), 1);
            assert_eq!(user_member[0].user.id.to_string(), FRIEND_USER_ID);

            // Confirm there is only one owner of the beta project, and it is now FRIEND_USER_ID
            let members = test_env
                .api
                .get_team_members_deserialized(beta_team_id, USER_USER_PAT)
                .await;
            let user_member =
                members.iter().filter(|m| m.is_owner).collect::<Vec<_>>();
            assert_eq!(user_member.len(), 1);
            assert_eq!(user_member[0].user.id.to_string(), FRIEND_USER_ID);

            // Confirm there is only one member of the enemy project, and it is STILL ENEMY_USER_ID
            let enemy_project = test_env
                .api
                .get_project_deserialized(
                    &enemy_project.id.to_string(),
                    ENEMY_USER_PAT,
                )
                .await;
            let members = test_env
                .api
                .get_team_members_deserialized(
                    &enemy_project.team_id.to_string(),
                    ENEMY_USER_PAT,
                )
                .await;
            let user_member =
                members.iter().filter(|m| m.is_owner).collect::<Vec<_>>();
            assert_eq!(user_member.len(), 1);
            assert_eq!(
                user_member[0].user.id.to_string(),
                ENEMY_USER_ID_PARSED.to_string()
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn permissions_patch_organization() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            // For each permission covered by EDIT_DETAILS, ensure the permission is required
            let api = &test_env.api;
            let edit_details = OrganizationPermissions::EDIT_DETAILS;
            let test_pairs = [
                ("name", json!("")), // generated in the test to not collide slugs
                ("description", json!("New description")),
            ];

            for (key, value) in test_pairs {
                let req_gen = |ctx: PermissionsTestContext| {
                    let value = value.clone();
                    async move {
                        api.edit_organization(
                            &ctx.organization_id.unwrap(),
                            json!({
                                key: if key == "name" {
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
                    .simple_organization_permissions_test(edit_details, req_gen)
                    .await
                    .unwrap();
            }
        },
    )
    .await;
}

// Not covered by PATCH /organization
#[actix_rt::test]
async fn permissions_edit_details() {
    with_test_environment(
        Some(12),
        |test_env: TestEnvironment<ApiV3>| async move {
            let zeta_organization_id =
                &test_env.dummy.organization_zeta.organization_id;
            let zeta_team_id = &test_env.dummy.organization_zeta.team_id;

            let api = &test_env.api;
            let edit_details = OrganizationPermissions::EDIT_DETAILS;

            // Icon edit
            // Uses alpha organization to delete this icon
            let req_gen = |ctx: PermissionsTestContext| async move {
                api.edit_organization_icon(
                    &ctx.organization_id.unwrap(),
                    Some(DummyImage::SmallIcon.get_icon_data()),
                    ctx.test_pat.as_deref(),
                )
                .await
            };
            PermissionsTest::new(&test_env)
                .with_existing_organization(zeta_organization_id, zeta_team_id)
                .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
                .simple_organization_permissions_test(edit_details, req_gen)
                .await
                .unwrap();

            // Icon delete
            // Uses alpha project to delete added icon
            let req_gen = |ctx: PermissionsTestContext| async move {
                api.edit_organization_icon(
                    &ctx.organization_id.unwrap(),
                    None,
                    ctx.test_pat.as_deref(),
                )
                .await
            };
            PermissionsTest::new(&test_env)
                .with_existing_organization(zeta_organization_id, zeta_team_id)
                .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
                .simple_organization_permissions_test(edit_details, req_gen)
                .await
                .unwrap();
        },
    )
    .await;
}

#[actix_rt::test]
async fn permissions_manage_invites() {
    // Add member, remove member, edit member
    with_test_environment_all(None, |test_env| async move {
        let api = &test_env.api;

        let zeta_organization_id =
            &test_env.dummy.organization_zeta.organization_id;
        let zeta_team_id = &test_env.dummy.organization_zeta.team_id;

        let manage_invites = OrganizationPermissions::MANAGE_INVITES;

        // Add member
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.add_user_to_team(
                &ctx.team_id.unwrap(),
                MOD_USER_ID,
                Some(ProjectPermissions::empty()),
                Some(OrganizationPermissions::empty()),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_organization(zeta_organization_id, zeta_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_organization_permissions_test(manage_invites, req_gen)
            .await
            .unwrap();

        // Edit member
        let edit_member = OrganizationPermissions::EDIT_MEMBER;
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_team_member(
                &ctx.team_id.unwrap(),
                MOD_USER_ID,
                json!({
                    "organization_permissions": 0,
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_organization(zeta_organization_id, zeta_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_organization_permissions_test(edit_member, req_gen)
            .await
            .unwrap();

        // remove member
        // requires manage_invites if they have not yet accepted the invite
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.remove_from_team(
                &ctx.team_id.unwrap(),
                MOD_USER_ID,
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_organization(zeta_organization_id, zeta_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_organization_permissions_test(manage_invites, req_gen)
            .await
            .unwrap();

        // re-add member for testing
        let resp = api
            .add_user_to_team(
                zeta_team_id,
                MOD_USER_ID,
                None,
                None,
                ADMIN_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);
        let resp = api.join_team(zeta_team_id, MOD_USER_PAT).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // remove existing member (requires remove_member)
        let remove_member = OrganizationPermissions::REMOVE_MEMBER;
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.remove_from_team(
                &ctx.team_id.unwrap(),
                MOD_USER_ID,
                ctx.test_pat.as_deref(),
            )
            .await
        };

        PermissionsTest::new(&test_env)
            .with_existing_organization(zeta_organization_id, zeta_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_organization_permissions_test(remove_member, req_gen)
            .await
            .unwrap();
    })
    .await;
}

#[actix_rt::test]
async fn permissions_add_remove_project() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;

            let alpha_project_id = &test_env.dummy.project_alpha.project_id;
            let alpha_team_id = &test_env.dummy.project_alpha.team_id;
            let zeta_organization_id =
                &test_env.dummy.organization_zeta.organization_id;
            let zeta_team_id = &test_env.dummy.organization_zeta.team_id;

            let add_project = OrganizationPermissions::ADD_PROJECT;

            // First, we add FRIEND_USER_ID to the alpha project and transfer ownership to them
            // This is because the ownership of a project is needed to add it to an organization
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
            let resp = api.join_team(alpha_team_id, FRIEND_USER_PAT).await;
            assert_status!(&resp, StatusCode::NO_CONTENT);
            let resp = api
                .transfer_team_ownership(
                    alpha_team_id,
                    FRIEND_USER_ID,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Now, FRIEND_USER_ID owns the alpha project
            // Add alpha project to zeta organization
            let req_gen = |ctx: PermissionsTestContext| async move {
                api.organization_add_project(
                    &ctx.organization_id.unwrap(),
                    alpha_project_id,
                    ctx.test_pat.as_deref(),
                )
                .await
            };
            PermissionsTest::new(&test_env)
                .with_existing_organization(zeta_organization_id, zeta_team_id)
                .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
                .simple_organization_permissions_test(add_project, req_gen)
                .await
                .unwrap();

            // Remove alpha project from zeta organization
            let remove_project = OrganizationPermissions::REMOVE_PROJECT;
            let req_gen = |ctx: PermissionsTestContext| async move {
                api.organization_remove_project(
                    &ctx.organization_id.unwrap(),
                    alpha_project_id,
                    UserId(FRIEND_USER_ID_PARSED as u64),
                    ctx.test_pat.as_deref(),
                )
                .await
            };
            PermissionsTest::new(&test_env)
                .with_existing_organization(zeta_organization_id, zeta_team_id)
                .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
                .simple_organization_permissions_test(remove_project, req_gen)
                .await
                .unwrap();
        },
    )
    .await;
}

#[actix_rt::test]
async fn permissions_delete_organization() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let delete_organization =
                OrganizationPermissions::DELETE_ORGANIZATION;

            // Now, FRIEND_USER_ID owns the alpha project
            // Add alpha project to zeta organization
            let req_gen = |ctx: PermissionsTestContext| async move {
                api.delete_organization(
                    &ctx.organization_id.unwrap(),
                    ctx.test_pat.as_deref(),
                )
                .await
            };
            PermissionsTest::new(&test_env)
                .simple_organization_permissions_test(
                    delete_organization,
                    req_gen,
                )
                .await
                .unwrap();
        },
    )
    .await;
}

#[actix_rt::test]
async fn permissions_add_default_project_permissions() {
    with_test_environment_all(None, |test_env| async move {
        let zeta_organization_id =
            &test_env.dummy.organization_zeta.organization_id;
        let zeta_team_id = &test_env.dummy.organization_zeta.team_id;

        let api = &test_env.api;

        // Add member
        let add_member_default_permissions =
            OrganizationPermissions::MANAGE_INVITES
                | OrganizationPermissions::EDIT_MEMBER_DEFAULT_PERMISSIONS;

        // Failure test should include MANAGE_INVITES, as it is required to add
        // default permissions on an invited user, but should still fail without EDIT_MEMBER_DEFAULT_PERMISSIONS
        let failure_with_add_member = (OrganizationPermissions::all()
            ^ add_member_default_permissions)
            | OrganizationPermissions::MANAGE_INVITES;

        let req_gen = |ctx: PermissionsTestContext| async move {
            api.add_user_to_team(
                &ctx.team_id.unwrap(),
                MOD_USER_ID,
                Some(
                    ProjectPermissions::UPLOAD_VERSION
                        | ProjectPermissions::DELETE_VERSION,
                ),
                Some(OrganizationPermissions::empty()),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_organization(zeta_organization_id, zeta_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .with_failure_permissions(None, Some(failure_with_add_member))
            .simple_organization_permissions_test(
                add_member_default_permissions,
                req_gen,
            )
            .await
            .unwrap();

        // Now that member is added, modify default permissions
        let modify_member_default_permission =
            OrganizationPermissions::EDIT_MEMBER
                | OrganizationPermissions::EDIT_MEMBER_DEFAULT_PERMISSIONS;

        // Failure test should include MANAGE_INVITES, as it is required to add
        // default permissions on an invited user, but should still fail without EDIT_MEMBER_DEFAULT_PERMISSIONS
        let failure_with_modify_member = (OrganizationPermissions::all()
            ^ add_member_default_permissions)
            | OrganizationPermissions::EDIT_MEMBER;

        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_team_member(
                &ctx.team_id.unwrap(),
                MOD_USER_ID,
                json!({
                    "permissions": ProjectPermissions::EDIT_DETAILS.bits(),
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_organization(zeta_organization_id, zeta_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .with_failure_permissions(None, Some(failure_with_modify_member))
            .simple_organization_permissions_test(
                modify_member_default_permission,
                req_gen,
            )
            .await
            .unwrap();
    })
    .await;
}

#[actix_rt::test]
async fn permissions_organization_permissions_consistency_test() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            // Ensuring that permission are as we expect them to be
            // Full organization permissions test
            let success_permissions = OrganizationPermissions::EDIT_DETAILS;
            let req_gen = |ctx: PermissionsTestContext| async move {
                api.edit_organization(
                    &ctx.organization_id.unwrap(),
                    json!({
                        "description": "Example description - changed.",
                    }),
                    ctx.test_pat.as_deref(),
                )
                .await
            };
            PermissionsTest::new(&test_env)
                .full_organization_permissions_tests(
                    success_permissions,
                    req_gen,
                )
                .await
                .unwrap();
        },
    )
    .await;
}

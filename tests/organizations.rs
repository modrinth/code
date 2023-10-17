use crate::common::{
    database::{generate_random_name, ADMIN_USER_PAT, MOD_USER_ID, MOD_USER_PAT, USER_USER_ID},
    dummy_data::DummyImage,
    environment::TestEnvironment,
    request_data::get_icon_data,
};
use actix_web::test;
use bytes::Bytes;
use common::{
    database::{FRIEND_USER_ID, FRIEND_USER_PAT, USER_USER_PAT},
    permissions::{PermissionsTest, PermissionsTestContext},
};
use labrinth::models::teams::{OrganizationPermissions, ProjectPermissions};
use serde_json::json;

mod common;

#[actix_rt::test]
async fn create_organization() {
    let test_env = TestEnvironment::build(None).await;
    let api = &test_env.v2;
    let zeta_organization_slug = &test_env
        .dummy
        .as_ref()
        .unwrap()
        .organization_zeta
        .organization_id;

    // Failed creations title:
    // - slug collision with zeta
    // - too short slug
    // - too long slug
    // - not url safe slug
    for title in [
        zeta_organization_slug,
        "a",
        &"a".repeat(100),
        "not url safe%&^!#$##!@#$%^&*()",
    ] {
        let resp = api
            .create_organization(title, "theta_description", USER_USER_PAT)
            .await;
        assert_eq!(resp.status(), 400);
    }

    // Failed creations description:
    // - too short slug
    // - too long slug
    for description in ["a", &"a".repeat(300)] {
        let resp = api
            .create_organization("theta", description, USER_USER_PAT)
            .await;
        assert_eq!(resp.status(), 400);
    }

    // Create 'theta' organization
    let resp = api
        .create_organization("theta", "not url safe%&^!#$##!@#$%^&", USER_USER_PAT)
        .await;
    assert_eq!(resp.status(), 200);

    // Get organization using slug
    let theta = api
        .get_organization_deserialized("theta", USER_USER_PAT)
        .await;
    assert_eq!(theta.title, "theta");
    assert_eq!(theta.description, "not url safe%&^!#$##!@#$%^&");
    assert_eq!(resp.status(), 200);

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
    assert_eq!(members[0].role, "Owner");

    test_env.cleanup().await;
}

#[actix_rt::test]
async fn patch_organization() {
    let test_env = TestEnvironment::build(None).await;
    let api = &test_env.v2;

    let zeta_organization_id = &test_env
        .dummy
        .as_ref()
        .unwrap()
        .organization_zeta
        .organization_id;

    // Create 'theta' organization
    let resp = api
        .create_organization("theta", "theta_description", USER_USER_PAT)
        .await;
    assert_eq!(resp.status(), 200);

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
                    "title": title,
                    "description": "theta_description"
                }),
                USER_USER_PAT,
            )
            .await;
        assert_eq!(resp.status(), 400);
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
        assert_eq!(resp.status(), 400);
    }

    // Successful patch to many fields
    let resp = api
        .edit_organization(
            zeta_organization_id,
            json!({
                "title": "new_title",
                "description": "not url safe%&^!#$##!@#$%^&" // not-URL-safe description should still work
            }),
            USER_USER_PAT,
        )
        .await;
    assert_eq!(resp.status(), 204);

    // Get project using new slug
    let new_title = api
        .get_organization_deserialized("new_title", USER_USER_PAT)
        .await;
    assert_eq!(new_title.title, "new_title");
    assert_eq!(new_title.description, "not url safe%&^!#$##!@#$%^&");

    test_env.cleanup().await;
}

// add/remove icon
#[actix_rt::test]
async fn add_remove_icon() {
    let test_env = TestEnvironment::build(None).await;
    let api = &test_env.v2;
    let zeta_organization_id = &test_env
        .dummy
        .as_ref()
        .unwrap()
        .organization_zeta
        .organization_id;

    // Get project
    let resp = test_env
        .v2
        .get_organization_deserialized(zeta_organization_id, USER_USER_PAT)
        .await;
    assert_eq!(resp.icon_url, None);

    // Icon edit
    // Uses alpha organization to delete this icon
    let resp = api
        .edit_organization_icon(
            zeta_organization_id,
            Some(get_icon_data(DummyImage::SmallIcon)),
            USER_USER_PAT,
        )
        .await;
    assert_eq!(resp.status(), 204);

    // Get project
    let zeta_org = api
        .get_organization_deserialized(zeta_organization_id, USER_USER_PAT)
        .await;
    assert!(zeta_org.icon_url.is_some());

    // Icon delete
    // Uses alpha organization to delete added icon
    let resp = api
        .edit_organization_icon(zeta_organization_id, None, USER_USER_PAT)
        .await;
    assert_eq!(resp.status(), 204);

    // Get project
    let zeta_org = api
        .get_organization_deserialized(zeta_organization_id, USER_USER_PAT)
        .await;
    assert!(zeta_org.icon_url.is_none());

    test_env.cleanup().await;
}

// delete org
#[actix_rt::test]
async fn delete_org() {
    let test_env = TestEnvironment::build(None).await;
    let api = &test_env.v2;
    let zeta_organization_id = &test_env
        .dummy
        .as_ref()
        .unwrap()
        .organization_zeta
        .organization_id;

    let resp = api
        .delete_organization(zeta_organization_id, USER_USER_PAT)
        .await;
    assert_eq!(resp.status(), 204);

    // Get organization, which should no longer exist
    let resp = api
        .get_organization(zeta_organization_id, USER_USER_PAT)
        .await;
    assert_eq!(resp.status(), 404);

    test_env.cleanup().await;
}

// add/remove organization projects
#[actix_rt::test]
async fn add_remove_organization_projects() {
    let test_env = TestEnvironment::build(None).await;
    let alpha_project_id: &str = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
    let alpha_project_slug: &str = &test_env.dummy.as_ref().unwrap().project_alpha.project_slug;
    let zeta_organization_id: &str = &test_env
        .dummy
        .as_ref()
        .unwrap()
        .organization_zeta
        .organization_id;

    // Add/remove project to organization, first by ID, then by slug
    for alpha in [alpha_project_id, alpha_project_slug] {
        let resp = test_env
            .v2
            .organization_add_project(zeta_organization_id, alpha, USER_USER_PAT)
            .await;
        assert_eq!(resp.status(), 200);

        // Get organization projects
        let projects = test_env
            .v2
            .get_organization_projects_deserialized(zeta_organization_id, USER_USER_PAT)
            .await;
        assert_eq!(projects[0].id.to_string(), alpha_project_id);
        assert_eq!(projects[0].slug, Some(alpha_project_slug.to_string()));

        // Remove project from organization
        let resp = test_env
            .v2
            .organization_remove_project(zeta_organization_id, alpha, USER_USER_PAT)
            .await;
        assert_eq!(resp.status(), 200);

        // Get organization projects
        let projects = test_env
            .v2
            .get_organization_projects_deserialized(zeta_organization_id, USER_USER_PAT)
            .await;
        assert!(projects.is_empty());
    }

    test_env.cleanup().await;
}

#[actix_rt::test]
async fn permissions_patch_organization() {
    let test_env = TestEnvironment::build(Some(8)).await;

    // For each permission covered by EDIT_DETAILS, ensure the permission is required
    let edit_details = OrganizationPermissions::EDIT_DETAILS;
    let test_pairs = [
        ("title", json!("")), // generated in the test to not collide slugs
        ("description", json!("New description")),
    ];

    for (key, value) in test_pairs {
        let req_gen = |ctx: &PermissionsTestContext| {
            test::TestRequest::patch()
                .uri(&format!(
                    "/v2/organization/{}",
                    ctx.organization_id.unwrap()
                ))
                .set_json(json!({
                    key: if key == "title" {
                        json!(generate_random_name("randomslug"))
                    } else {
                        value.clone()
                    },
                }))
        };
        PermissionsTest::new(&test_env)
            .simple_organization_permissions_test(edit_details, req_gen)
            .await
            .unwrap();
    }

    test_env.cleanup().await;
}

// Not covered by PATCH /organization
#[actix_rt::test]
async fn permissions_edit_details() {
    let test_env = TestEnvironment::build(None).await;

    let zeta_organization_id = &test_env
        .dummy
        .as_ref()
        .unwrap()
        .organization_zeta
        .organization_id;
    let zeta_team_id = &test_env.dummy.as_ref().unwrap().organization_zeta.team_id;

    let edit_details = OrganizationPermissions::EDIT_DETAILS;

    // Icon edit
    // Uses alpha organization to delete this icon
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::patch()
            .uri(&format!(
                "/v2/organization/{}/icon?ext=png",
                ctx.organization_id.unwrap()
            ))
            .set_payload(Bytes::from(
                include_bytes!("../tests/files/200x200.png") as &[u8]
            ))
    };
    PermissionsTest::new(&test_env)
        .with_existing_organization(zeta_organization_id, zeta_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_organization_permissions_test(edit_details, req_gen)
        .await
        .unwrap();

    // Icon delete
    // Uses alpha project to delete added icon
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::delete().uri(&format!(
            "/v2/organization/{}/icon?ext=png",
            ctx.organization_id.unwrap()
        ))
    };
    PermissionsTest::new(&test_env)
        .with_existing_organization(zeta_organization_id, zeta_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_organization_permissions_test(edit_details, req_gen)
        .await
        .unwrap();
}

#[actix_rt::test]
async fn permissions_manage_invites() {
    // Add member, remove member, edit member
    let test_env = TestEnvironment::build(None).await;
    let api = &test_env.v2;

    let zeta_organization_id = &test_env
        .dummy
        .as_ref()
        .unwrap()
        .organization_zeta
        .organization_id;
    let zeta_team_id = &test_env.dummy.as_ref().unwrap().organization_zeta.team_id;

    let manage_invites = OrganizationPermissions::MANAGE_INVITES;

    // Add member
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::post()
            .uri(&format!("/v2/team/{}/members", ctx.team_id.unwrap()))
            .set_json(json!({
                "user_id": MOD_USER_ID,
                "permissions": 0,
                "organization_permissions": 0,
            }))
    };
    PermissionsTest::new(&test_env)
        .with_existing_organization(zeta_organization_id, zeta_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_organization_permissions_test(manage_invites, req_gen)
        .await
        .unwrap();

    // Edit member
    let edit_member = OrganizationPermissions::EDIT_MEMBER;
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::patch()
            .uri(&format!(
                "/v2/team/{}/members/{MOD_USER_ID}",
                ctx.team_id.unwrap()
            ))
            .set_json(json!({
                "organization_permissions": 0,
            }))
    };
    PermissionsTest::new(&test_env)
        .with_existing_organization(zeta_organization_id, zeta_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_organization_permissions_test(edit_member, req_gen)
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
        .with_existing_organization(zeta_organization_id, zeta_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_organization_permissions_test(manage_invites, req_gen)
        .await
        .unwrap();

    // re-add member for testing
    let resp = api
        .add_user_to_team(zeta_team_id, MOD_USER_ID, None, None, ADMIN_USER_PAT)
        .await;
    assert_eq!(resp.status(), 204);
    let resp = api.join_team(zeta_team_id, MOD_USER_PAT).await;
    assert_eq!(resp.status(), 204);

    // remove existing member (requires remove_member)
    let remove_member = OrganizationPermissions::REMOVE_MEMBER;
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::delete().uri(&format!(
            "/v2/team/{}/members/{MOD_USER_ID}",
            ctx.team_id.unwrap()
        ))
    };

    PermissionsTest::new(&test_env)
        .with_existing_organization(zeta_organization_id, zeta_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_organization_permissions_test(remove_member, req_gen)
        .await
        .unwrap();

    test_env.cleanup().await;
}

#[actix_rt::test]
async fn permissions_add_remove_project() {
    let test_env = TestEnvironment::build(None).await;
    let api = &test_env.v2;

    let alpha_project_id = &test_env.dummy.as_ref().unwrap().project_alpha.project_id;
    let alpha_team_id = &test_env.dummy.as_ref().unwrap().project_alpha.team_id;
    let zeta_organization_id = &test_env
        .dummy
        .as_ref()
        .unwrap()
        .organization_zeta
        .organization_id;
    let zeta_team_id = &test_env.dummy.as_ref().unwrap().organization_zeta.team_id;

    let add_project = OrganizationPermissions::ADD_PROJECT;

    // First, we add FRIEND_USER_ID to the alpha project and transfer ownership to them
    // This is because the ownership of a project is needed to add it to an organization
    let resp = api
        .add_user_to_team(alpha_team_id, FRIEND_USER_ID, None, None, USER_USER_PAT)
        .await;
    assert_eq!(resp.status(), 204);
    let resp = api.join_team(alpha_team_id, FRIEND_USER_PAT).await;
    assert_eq!(resp.status(), 204);
    let resp = api
        .transfer_team_ownership(alpha_team_id, FRIEND_USER_ID, USER_USER_PAT)
        .await;
    assert_eq!(resp.status(), 204);

    // Now, FRIEND_USER_ID owns the alpha project
    // Add alpha project to zeta organization
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::post()
            .uri(&format!(
                "/v2/organization/{}/projects",
                ctx.organization_id.unwrap()
            ))
            .set_json(json!({
                "project_id": alpha_project_id,
            }))
    };
    PermissionsTest::new(&test_env)
        .with_existing_organization(zeta_organization_id, zeta_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_organization_permissions_test(add_project, req_gen)
        .await
        .unwrap();

    // Remove alpha project from zeta organization
    let remove_project = OrganizationPermissions::REMOVE_PROJECT;
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::delete().uri(&format!(
            "/v2/organization/{}/projects/{alpha_project_id}",
            ctx.organization_id.unwrap()
        ))
    };
    PermissionsTest::new(&test_env)
        .with_existing_organization(zeta_organization_id, zeta_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .simple_organization_permissions_test(remove_project, req_gen)
        .await
        .unwrap();

    test_env.cleanup().await;
}

#[actix_rt::test]
async fn permissions_delete_organization() {
    let test_env = TestEnvironment::build(None).await;
    let delete_organization = OrganizationPermissions::DELETE_ORGANIZATION;

    // Now, FRIEND_USER_ID owns the alpha project
    // Add alpha project to zeta organization
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::delete().uri(&format!(
            "/v2/organization/{}",
            ctx.organization_id.unwrap()
        ))
    };
    PermissionsTest::new(&test_env)
        .simple_organization_permissions_test(delete_organization, req_gen)
        .await
        .unwrap();

    test_env.cleanup().await;
}

#[actix_rt::test]
async fn permissions_add_default_project_permissions() {
    let test_env = TestEnvironment::build(None).await;
    let zeta_organization_id = &test_env
        .dummy
        .as_ref()
        .unwrap()
        .organization_zeta
        .organization_id;
    let zeta_team_id = &test_env.dummy.as_ref().unwrap().organization_zeta.team_id;

    // Add member
    let add_member_default_permissions = OrganizationPermissions::MANAGE_INVITES
        | OrganizationPermissions::EDIT_MEMBER_DEFAULT_PERMISSIONS;

    // Failure test should include MANAGE_INVITES, as it is required to add
    // default permissions on an invited user, but should still fail without EDIT_MEMBER_DEFAULT_PERMISSIONS
    let failure_with_add_member = (OrganizationPermissions::all() ^ add_member_default_permissions)
        | OrganizationPermissions::MANAGE_INVITES;

    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::post()
            .uri(&format!("/v2/team/{}/members", ctx.team_id.unwrap()))
            .set_json(json!({
                "user_id": MOD_USER_ID,
                // do not set permissions as it will be set to default, which is non-zero
                "organization_permissions": 0,
            }))
    };
    PermissionsTest::new(&test_env)
        .with_existing_organization(zeta_organization_id, zeta_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .with_failure_permissions(None, Some(failure_with_add_member))
        .simple_organization_permissions_test(add_member_default_permissions, req_gen)
        .await
        .unwrap();

    // Now that member is added, modify default permissions
    let modify_member_default_permission = OrganizationPermissions::EDIT_MEMBER
        | OrganizationPermissions::EDIT_MEMBER_DEFAULT_PERMISSIONS;

    // Failure test should include MANAGE_INVITES, as it is required to add
    // default permissions on an invited user, but should still fail without EDIT_MEMBER_DEFAULT_PERMISSIONS
    let failure_with_modify_member = (OrganizationPermissions::all()
        ^ add_member_default_permissions)
        | OrganizationPermissions::EDIT_MEMBER;

    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::patch()
            .uri(&format!(
                "/v2/team/{}/members/{MOD_USER_ID}",
                ctx.team_id.unwrap()
            ))
            .set_json(json!({
                "permissions": ProjectPermissions::EDIT_DETAILS.bits(),
            }))
    };
    PermissionsTest::new(&test_env)
        .with_existing_organization(zeta_organization_id, zeta_team_id)
        .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
        .with_failure_permissions(None, Some(failure_with_modify_member))
        .simple_organization_permissions_test(modify_member_default_permission, req_gen)
        .await
        .unwrap();

    test_env.cleanup().await;
}

#[actix_rt::test]
async fn permissions_organization_permissions_consistency_test() {
    let test_env = TestEnvironment::build(None).await;
    // Ensuring that permission are as we expect them to be
    // Full organization permissions test
    let success_permissions = OrganizationPermissions::EDIT_DETAILS;
    let req_gen = |ctx: &PermissionsTestContext| {
        test::TestRequest::patch()
            .uri(&format!(
                "/v2/organization/{}",
                ctx.organization_id.unwrap()
            ))
            .set_json(json!({
                "description": "Example description - changed.",
            }))
    };
    PermissionsTest::new(&test_env)
        .full_organization_permissions_tests(success_permissions, req_gen)
        .await
        .unwrap();

    test_env.cleanup().await;
}

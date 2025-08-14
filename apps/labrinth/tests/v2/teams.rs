use actix_http::StatusCode;
use labrinth::models::teams::ProjectPermissions;
use serde_json::json;

use crate::{
    assert_status,
    common::{
        api_common::ApiTeams,
        api_v2::ApiV2,
        database::{
            FRIEND_USER_ID, FRIEND_USER_ID_PARSED, FRIEND_USER_PAT,
            USER_USER_ID_PARSED, USER_USER_PAT,
        },
        environment::{TestEnvironment, with_test_environment},
    },
};

// trasnfer ownership (requires being owner, etc)
#[actix_rt::test]
async fn transfer_ownership_v2() {
    // Test setup and dummy data
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
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
            assert_eq!(friend_member.role, "Owner");
            assert_eq!(
                friend_member.permissions.unwrap(),
                ProjectPermissions::all()
            );

            let user_member = members
                .iter()
                .find(|x| x.user.id.0 == USER_USER_ID_PARSED as u64)
                .unwrap();
            assert_eq!(user_member.role, "Member");
            assert_eq!(
                user_member.permissions.unwrap(),
                ProjectPermissions::all()
            );

            // Confirm that user, a user who still has full permissions, cannot then remove the owner
            let resp = api
                .remove_from_team(alpha_team_id, FRIEND_USER_ID, USER_USER_PAT)
                .await;
            assert_status!(&resp, StatusCode::UNAUTHORIZED);

            // V2 only- confirm the owner changing the role to member does nothing
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
            assert_eq!(friend_member.role, "Owner");
        },
    )
    .await;
}

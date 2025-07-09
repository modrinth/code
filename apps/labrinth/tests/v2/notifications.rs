use crate::common::{
    api_common::ApiTeams,
    api_v2::ApiV2,
    database::{FRIEND_USER_ID, FRIEND_USER_PAT, USER_USER_PAT},
    environment::{TestEnvironment, with_test_environment},
};

#[actix_rt::test]
pub async fn get_user_notifications_after_team_invitation_returns_notification()
{
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
            let alpha_team_id = test_env.dummy.project_alpha.team_id.clone();
            let api = test_env.api;
            api.add_user_to_team(
                &alpha_team_id,
                FRIEND_USER_ID,
                None,
                None,
                USER_USER_PAT,
            )
            .await;

            let notifications = api
                .get_user_notifications_deserialized(
                    FRIEND_USER_ID,
                    FRIEND_USER_PAT,
                )
                .await;
            assert_eq!(1, notifications.len());

            // Check to make sure type_ is correct
            assert_eq!(notifications[0].type_.as_ref().unwrap(), "team_invite");
        },
    )
    .await;
}

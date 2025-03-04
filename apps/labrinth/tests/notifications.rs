use common::{
    database::{FRIEND_USER_ID, FRIEND_USER_PAT, USER_USER_PAT},
    environment::with_test_environment_all,
};

use crate::common::api_common::ApiTeams;

mod common;

#[actix_rt::test]
pub async fn get_user_notifications_after_team_invitation_returns_notification()
{
    with_test_environment_all(None, |test_env| async move {
        let alpha_team_id = test_env.dummy.project_alpha.team_id.clone();
        let api = test_env.api;
        api.get_user_notifications_deserialized_common(
            FRIEND_USER_ID,
            FRIEND_USER_PAT,
        )
        .await;

        api.add_user_to_team(
            &alpha_team_id,
            FRIEND_USER_ID,
            None,
            None,
            USER_USER_PAT,
        )
        .await;

        let notifications = api
            .get_user_notifications_deserialized_common(
                FRIEND_USER_ID,
                FRIEND_USER_PAT,
            )
            .await;
        assert_eq!(1, notifications.len());
    })
    .await;
}

#[actix_rt::test]
pub async fn get_user_notifications_after_reading_indicates_notification_read()
{
    with_test_environment_all(None, |test_env| async move {
        test_env.generate_friend_user_notification().await;
        let api = test_env.api;
        let notifications = api
            .get_user_notifications_deserialized_common(
                FRIEND_USER_ID,
                FRIEND_USER_PAT,
            )
            .await;
        assert_eq!(1, notifications.len());
        let notification_id = notifications[0].id.to_string();

        api.mark_notification_read(&notification_id, FRIEND_USER_PAT)
            .await;

        let notifications = api
            .get_user_notifications_deserialized_common(
                FRIEND_USER_ID,
                FRIEND_USER_PAT,
            )
            .await;
        assert_eq!(1, notifications.len());
        assert!(notifications[0].read);
    })
    .await;
}

#[actix_rt::test]
pub async fn get_user_notifications_after_deleting_does_not_show_notification()
{
    with_test_environment_all(None, |test_env| async move {
        test_env.generate_friend_user_notification().await;
        let api = test_env.api;
        let notifications = api
            .get_user_notifications_deserialized_common(
                FRIEND_USER_ID,
                FRIEND_USER_PAT,
            )
            .await;
        assert_eq!(1, notifications.len());
        let notification_id = notifications[0].id.to_string();

        api.delete_notification(&notification_id, FRIEND_USER_PAT)
            .await;

        let notifications = api
            .get_user_notifications_deserialized_common(
                FRIEND_USER_ID,
                FRIEND_USER_PAT,
            )
            .await;
        assert_eq!(0, notifications.len());
    })
    .await;
}

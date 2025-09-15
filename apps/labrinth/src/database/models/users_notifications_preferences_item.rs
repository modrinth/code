use super::ids::*;
use crate::database::models::DatabaseError;
use crate::models::v3::notifications::{NotificationChannel, NotificationType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserNotificationPreference {
    pub id: i64,
    pub user_id: Option<DBUserId>,
    pub channel: NotificationChannel,
    pub notification_type: NotificationType,
    pub enabled: bool,
}

struct UserNotificationPreferenceQueryResult {
    id: i64,
    user_id: Option<i64>,
    channel: String,
    notification_type: String,
    enabled: bool,
}

impl From<UserNotificationPreferenceQueryResult>
    for UserNotificationPreference
{
    fn from(r: UserNotificationPreferenceQueryResult) -> Self {
        UserNotificationPreference {
            id: r.id,
            user_id: r.user_id.map(DBUserId),
            channel: NotificationChannel::from_str_or_default(&r.channel),
            notification_type: NotificationType::from_str_or_default(
                &r.notification_type,
            ),
            enabled: r.enabled,
        }
    }
}

impl UserNotificationPreference {
    pub async fn get_user_or_default(
        user_id: DBUserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<UserNotificationPreference>, DatabaseError> {
        Self::get_many_users_or_default(&[user_id], exec).await
    }

    pub async fn get_many_users_or_default(
        user_ids: &[DBUserId],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<UserNotificationPreference>, DatabaseError> {
        let results = sqlx::query!(
            r#"
            SELECT
              COALESCE(unp.id, dnp.id) "id!",
              unp.user_id,
              dnp.channel "channel!",
              dnp.notification_type "notification_type!",
              COALESCE(unp.enabled, dnp.enabled, false) "enabled!"
            FROM users_notifications_preferences dnp
            LEFT JOIN users_notifications_preferences unp
              ON unp.channel = dnp.channel
              AND unp.notification_type = dnp.notification_type
              AND unp.user_id = ANY($1::bigint[])
            "#,
            &user_ids.iter().map(|x| x.0).collect::<Vec<_>>(),
        )
        .fetch_all(exec)
        .await?;

        let preferences = results
            .into_iter()
            .map(|r| UserNotificationPreference {
                id: r.id,
                user_id: r.user_id.map(DBUserId),
                channel: NotificationChannel::from_str_or_default(&r.channel),
                notification_type: NotificationType::from_str_or_default(
                    &r.notification_type,
                ),
                enabled: r.enabled,
            })
            .collect();

        Ok(preferences)
    }

    /// Inserts the row into the table and updates its ID.
    pub async fn insert(
        &mut self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let id = sqlx::query_scalar!(
            "
            INSERT INTO users_notifications_preferences (
                user_id, channel, notification_type, enabled
            )
            VALUES ($1, $2, $3, $4)
            RETURNING id
            ",
            self.user_id.map(|x| x.0),
            self.channel.as_str(),
            self.notification_type.as_str(),
            self.enabled,
        )
        .fetch_one(exec)
        .await?;

        self.id = id;

        Ok(())
    }
}

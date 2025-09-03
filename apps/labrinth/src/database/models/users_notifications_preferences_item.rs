use super::ids::*;
use crate::database::models::DatabaseError;
use crate::models::v3::notifications::{NotificationChannel, NotificationType};

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

macro_rules! select_user_notification_preferences_with_predicate {
    ($predicate:literal $(, $($param0:expr $(, $param:expr)* $(,)?)?)?) => {
        sqlx::query_as!(
            UserNotificationPreferenceQueryResult,
            r#"
            SELECT
                id, user_id, channel, notification_type, enabled
            FROM users_notifications_preferences
            "#
                + $predicate
            $($(, $param0 $(, $param)* )?)?
        )
    };
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
    pub async fn get_id(
        id: i64,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<UserNotificationPreference>, DatabaseError> {
        let results = select_user_notification_preferences_with_predicate!(
            "WHERE id = $1",
            id
        )
        .fetch_optional(exec)
        .await?;

        Ok(results.map(|r| r.into()))
    }

    pub async fn get_user_or_default(
        user_id: DBUserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<UserNotificationPreference>, DatabaseError> {
        let results = sqlx::query!(
            r#"
            SELECT
              COALESCE(unp.id, dnp.id) "id!",
              unp.user_id,
              dnp.channel "channel!",
              dnp.notification_type "notification_type!",
              COALESCE(unp.enabled, COALESCE(dnp.enabled, false)) "enabled!"
            FROM users_notifications_preferences dnp
            LEFT JOIN users_notifications_preferences unp
              ON unp.channel = dnp.channel
              AND unp.notification_type = dnp.notification_type
              AND unp.user_id = $1
            "#,
            user_id.0
        )
        .fetch_all(exec)
        .await?;

        Ok(results
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
            .collect())
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

    /// Updates semantically mutable columns of the row.
    pub async fn update(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            UPDATE users_notifications_preferences
            SET
              user_id = $2,
              channel = $3,
              notification_type = $4,
              enabled = $5
            WHERE id = $1
            ",
            self.id,
            self.user_id.map(|x| x.0),
            self.channel.as_str(),
            self.notification_type.as_str(),
            self.enabled,
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}

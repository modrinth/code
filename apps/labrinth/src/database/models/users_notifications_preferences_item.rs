use super::ids::*;
use crate::database::models::DatabaseError;
use crate::models::v3::notifications::{NotificationChannel, NotificationType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub struct UserNotificationPreference {
    pub id: i64,
    pub user_id: Option<DBUserId>,
    pub channel: NotificationChannel,
    pub notification_type: NotificationType,
    pub enabled: bool,
}

struct UserNotificationPreferenceQueryResult {
    id: i64,
    user_id: i64,
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
            user_id: DBUserId(r.user_id),
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
        let user_id = user_id.0;
        sqlx::query!(
            "
            SELECT
              unp.user_id,
              unp.channel,
              unp.notification_type,
              COALESCE(unp.enabled, COALESCE(dnp.enabled, false)) enabled
            FROM users_notifications_preferences dnp
            LEFT JOIN users_notifications_preferences unp
              ON unp.channel = dnp.channel
              AND unp.notification_type = dnp.notification_type
              AND unp.user_id = $1
            ",
            user_id.0
        )
        .fetch_all(exec)
        .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// Inserts the row into the table and updates its ID.
    pub async fn insert(
        &mut self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let id = sqlx::query_scalar!(
            "
            INSERT INTO users_notifications_preferences (
                notification_id, user_id, channel, delivery_priority, status, next_attempt, attempt_count
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            ",
            self.notification_id.0,
            self.user_id.0,
            self.channel.as_str(),
            self.delivery_priority,
            self.status.as_str(),
            self.next_attempt,
            self.attempt_count,
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
            UPDATE notifications_deliveries
            SET
              delivery_priority = $2,
              status = $3,
              next_attempt = $4,
              attempt_count = $5
            WHERE id = $1
            ",
            self.id,
            self.delivery_priority,
            self.status.as_str(),
            self.next_attempt,
            self.attempt_count,
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}

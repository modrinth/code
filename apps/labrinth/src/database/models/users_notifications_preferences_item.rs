use super::ids::*;
use crate::database::models::DatabaseError;
use crate::database::redis::RedisPool;
use crate::models::v3::notifications::{NotificationChannel, NotificationType};
use serde::{Deserialize, Serialize};

const USER_NOTIFICATION_PREFERENCES_NAMESPACE: &str =
    "user_notification_preferences";

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
        redis: &RedisPool,
    ) -> Result<Vec<UserNotificationPreference>, DatabaseError> {
        let mut redis = redis.connect().await?;

        let cached_preferences = redis
            .get_deserialized_from_json(
                USER_NOTIFICATION_PREFERENCES_NAMESPACE,
                &user_id.0.to_string(),
            )
            .await?;

        if let Some(preferences) = cached_preferences {
            return Ok(preferences);
        }

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

        redis
            .set_serialized_to_json(
                USER_NOTIFICATION_PREFERENCES_NAMESPACE,
                &user_id.0.to_string(),
                &preferences,
                None,
            )
            .await?;

        Ok(preferences)
    }

    /// Inserts the row into the table and updates its ID.
    pub async fn insert(
        &mut self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        redis: &RedisPool,
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

        if let Some(user_id) = self.user_id {
            Self::clear_user_notification_preferences_cache(user_id, redis)
                .await?;
        }

        self.id = id;

        Ok(())
    }

    pub async fn clear_user_notification_preferences_cache(
        user_id: DBUserId,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;

        redis
            .delete(
                USER_NOTIFICATION_PREFERENCES_NAMESPACE,
                &user_id.0.to_string(),
            )
            .await?;

        Ok(())
    }
}

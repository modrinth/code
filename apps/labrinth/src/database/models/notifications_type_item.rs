use crate::database::models::DatabaseError;
use crate::database::redis::RedisPool;
use crate::models::v3::notifications::NotificationType;
use serde::{Deserialize, Serialize};

const NOTIFICATION_TYPES_NAMESPACE: &str = "notification_types";

#[derive(Serialize, Deserialize)]
pub struct NotificationTypeItem {
    pub name: NotificationType,
    pub delivery_priority: i32,
    pub expose_in_user_preferences: bool,
    pub expose_in_site_notifications: bool,
}

struct NotificationTypeQueryResult {
    name: String,
    delivery_priority: i32,
    expose_in_user_preferences: bool,
    expose_in_site_notifications: bool,
}

impl From<NotificationTypeQueryResult> for NotificationTypeItem {
    fn from(r: NotificationTypeQueryResult) -> Self {
        NotificationTypeItem {
            name: NotificationType::from_str_or_default(&r.name),
            delivery_priority: r.delivery_priority,
            expose_in_user_preferences: r.expose_in_user_preferences,
            expose_in_site_notifications: r.expose_in_site_notifications,
        }
    }
}

impl NotificationTypeItem {
    pub async fn list<'a, E>(
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<NotificationTypeItem>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let mut redis = redis.connect().await?;

        let cached_types = redis
            .get_deserialized_from_json(NOTIFICATION_TYPES_NAMESPACE, "all")
            .await?;

        if let Some(types) = cached_types {
            return Ok(types);
        }

        let results = sqlx::query_as!(
            NotificationTypeQueryResult,
            "SELECT * FROM notifications_types"
        )
        .fetch_all(exec)
        .await?;

        let types = results.into_iter().map(Into::into).collect();

        redis
            .set_serialized_to_json(
                NOTIFICATION_TYPES_NAMESPACE,
                "all",
                &types,
                None,
            )
            .await?;

        Ok(types)
    }
}

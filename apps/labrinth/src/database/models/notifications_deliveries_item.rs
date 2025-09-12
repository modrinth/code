use super::ids::*;
use crate::database::models::DatabaseError;
use crate::models::v3::notifications::{
    NotificationChannel, NotificationDeliveryStatus,
};
use chrono::{DateTime, Utc};

pub struct DBNotificationDelivery {
    pub id: i64,
    pub notification_id: DBNotificationId,
    pub user_id: DBUserId,
    pub channel: NotificationChannel,
    pub delivery_priority: i32,
    pub status: NotificationDeliveryStatus,
    pub next_attempt: DateTime<Utc>,
    pub attempt_count: i32,
}

struct NotificationDeliveryQueryResult {
    id: i64,
    notification_id: i64,
    user_id: i64,
    channel: String,
    delivery_priority: i32,
    status: String,
    next_attempt: DateTime<Utc>,
    attempt_count: i32,
}

macro_rules! select_notification_deliveries_with_predicate {
    ($predicate:literal $(, $($param0:expr $(, $param:expr)* $(,)?)?)?) => {
        sqlx::query_as!(
            NotificationDeliveryQueryResult,
            r#"
            SELECT
                id, notification_id, user_id, channel, delivery_priority, status, next_attempt, attempt_count
            FROM notifications_deliveries
            "#
                + $predicate
            $($(, $param0 $(, $param)* )?)?
        )
    };
}

impl From<NotificationDeliveryQueryResult> for DBNotificationDelivery {
    fn from(r: NotificationDeliveryQueryResult) -> Self {
        DBNotificationDelivery {
            id: r.id,
            notification_id: DBNotificationId(r.notification_id),
            user_id: DBUserId(r.user_id),
            channel: NotificationChannel::from_str_or_default(&r.channel),
            delivery_priority: r.delivery_priority,
            status: NotificationDeliveryStatus::from_str_or_default(&r.status),
            next_attempt: r.next_attempt,
            attempt_count: r.attempt_count,
        }
    }
}

impl DBNotificationDelivery {
    pub async fn get_all_user(
        user_id: DBUserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBNotificationDelivery>, DatabaseError> {
        let user_id = user_id.0;
        let results = select_notification_deliveries_with_predicate!(
            "WHERE user_id = $1",
            user_id
        )
        .fetch_all(exec)
        .await?;

        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    /// Returns deliveries that should be processed next for a given channel using a row-level
    /// `UPDATE` lock, barring the provided limit.
    pub async fn lock_channel_processable(
        channel: NotificationChannel,
        limit: i64,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBNotificationDelivery>, DatabaseError> {
        // This follows the `idx_notifications_deliveries_composite_queue` index.
        Ok(select_notification_deliveries_with_predicate!(
            "WHERE
              status = $3
              AND channel = $1
              AND next_attempt <= NOW()
            ORDER BY
              delivery_priority DESC,
              next_attempt ASC
            LIMIT $2
            FOR UPDATE
            SKIP LOCKED
            ",
            channel.as_str(),
            limit,
            NotificationDeliveryStatus::Pending.as_str()
        )
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
    }

    /// Inserts the row into the table and updates its ID.
    pub async fn insert(
        &mut self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let id = sqlx::query_scalar!(
            "
            INSERT INTO notifications_deliveries (
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

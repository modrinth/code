use super::ids::*;
use crate::database::{models::DatabaseError, redis::RedisPool};
use crate::models::notifications::{
    NotificationBody, NotificationChannel, NotificationDeliveryStatus,
    NotificationType,
};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};

const USER_NOTIFICATIONS_NAMESPACE: &str = "user_notifications";

pub struct NotificationBuilder {
    pub body: NotificationBody,
}

#[derive(Serialize, Deserialize)]
pub struct DBNotification {
    pub id: DBNotificationId,
    pub user_id: DBUserId,
    pub body: NotificationBody,
    pub read: bool,
    pub created: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct DBNotificationAction {
    pub id: NotificationActionId,
    pub notification_id: DBNotificationId,
    pub name: String,
    pub action_route_method: String,
    pub action_route: String,
}

impl NotificationBuilder {
    pub async fn insert(
        &self,
        user: DBUserId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        self.insert_many(vec![user], transaction, redis).await
    }

    pub async fn insert_many_payout_notifications(
        users: Vec<DBUserId>,
        dates_available: Vec<DateTime<Utc>>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let notification_ids =
            generate_many_notification_ids(users.len(), &mut *transaction)
                .await?;

        let users_raw_ids = users.iter().map(|x| x.0).collect::<Vec<_>>();
        let notification_ids =
            notification_ids.iter().map(|x| x.0).collect::<Vec<_>>();

        sqlx::query!(
            "
            WITH
              period_payouts AS (
                SELECT
                  ids.notification_id,
                  ids.user_id,
                  ids.date_available,
                  FLOOR(COALESCE(SUM(pv.amount), 0.0) * 100) :: BIGINT sum -- Convert to cents
                  FROM UNNEST($1::bigint[], $2::bigint[], $3::timestamptz[]) AS ids(notification_id, user_id, date_available)
                LEFT JOIN payouts_values pv ON pv.user_id = ids.user_id AND pv.date_available = ids.date_available
                GROUP BY ids.user_id, ids.notification_id, ids.date_available
              )
            INSERT INTO notifications (
                id, user_id, body
            )
            SELECT
              notification_id id,
              user_id,
              JSONB_BUILD_OBJECT(
                'type', 'payout_available',
                'date_available', to_jsonb(date_available),
                'amount', to_jsonb(sum)
              ) body
            FROM period_payouts
            WHERE sum >= 100
            ",
            &notification_ids[..],
            &users_raw_ids[..],
            &dates_available[..],
        )
        .execute(&mut **transaction)
        .await?;

        let notification_types = notification_ids
            .iter()
            .map(|_| NotificationType::PayoutAvailable.as_str())
            .collect::<Vec<_>>();

        NotificationBuilder::insert_many_deliveries(
            transaction,
            redis,
            &notification_ids,
            &users_raw_ids,
            &notification_types,
            &users,
        )
        .await?;

        Ok(())
    }

    pub async fn insert_many(
        &self,
        users: Vec<DBUserId>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let notification_ids =
            generate_many_notification_ids(users.len(), &mut *transaction)
                .await?;

        let body = serde_json::value::to_value(&self.body)?;
        let bodies = notification_ids
            .iter()
            .map(|_| body.clone())
            .collect::<Vec<_>>();

        let users_raw_ids = users.iter().map(|x| x.0).collect::<Vec<_>>();
        let notification_ids =
            notification_ids.iter().map(|x| x.0).collect::<Vec<_>>();

        sqlx::query!(
            "
            INSERT INTO notifications (
                id, user_id, body
            )
            SELECT * FROM UNNEST($1::bigint[], $2::bigint[], $3::jsonb[])
            ",
            &notification_ids[..],
            &users_raw_ids[..],
            &bodies[..],
        )
        .execute(&mut **transaction)
        .await?;

        let notification_types = notification_ids
            .iter()
            .map(|_| self.body.notification_type().as_str())
            .collect::<Vec<_>>();

        NotificationBuilder::insert_many_deliveries(
            transaction,
            redis,
            &notification_ids,
            &users_raw_ids,
            &notification_types,
            &users,
        )
        .await?;

        Ok(())
    }

    pub async fn insert_many_deliveries(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
        notification_ids: &[i64],
        users_raw_ids: &[i64],
        notification_types: &[&str],
        users: &[DBUserId],
    ) -> Result<(), DatabaseError> {
        let notification_channels = NotificationChannel::list()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>();

        // Insert required rows into `notifications_deliveries` by channel
        // and notification type, based on the user's preferences.
        let query = sqlx::query!(
            r#"
            WITH
              channels AS (
                SELECT channel FROM UNNEST($1::varchar[]) AS t(channel)
              ),
              delivery_candidates AS (
                SELECT
                  ids.notification_id,
                  ids.user_id,
                  channels.channel,
                  nt.delivery_priority,
                  uprefs.enabled user_enabled,
                  dprefs.enabled default_enabled
                FROM
                  UNNEST(
                    $2::bigint[],
                    $3::bigint[],
                    $4::varchar[]
                  ) AS ids(notification_id, user_id, notification_type)
                CROSS JOIN channels
                INNER JOIN
                  notifications_types nt ON nt.name = ids.notification_type
                LEFT JOIN users_notifications_preferences uprefs
                  ON uprefs.user_id = ids.user_id
                  AND uprefs.channel = channels.channel
                  AND uprefs.notification_type = ids.notification_type
                LEFT JOIN users_notifications_preferences dprefs
                  ON dprefs.user_id IS NULL
                  AND dprefs.channel = channels.channel
                  AND dprefs.notification_type = ids.notification_type
              )
            INSERT INTO notifications_deliveries
            (notification_id, user_id, channel, delivery_priority, status, next_attempt, attempt_count)
            SELECT
              dc.notification_id,
              dc.user_id,
              dc.channel,
              dc.delivery_priority,
              CASE
                -- User explicitly enabled
                WHEN user_enabled = TRUE THEN $5

                -- Is enabled by default, no preference by user
                WHEN user_enabled IS NULL AND default_enabled = TRUE THEN $5

                -- User explicitly disabled (regardless of default)
                WHEN user_enabled = FALSE THEN $6

                -- User set no preference, default disabled
                WHEN user_enabled IS NULL AND default_enabled = FALSE THEN $7

                -- At this point, user set no preference and there is no
                -- default set, so treat as disabled-by-default.
                ELSE $7
              END status,
              NOW() next_attempt,
              0 attempt_count
            FROM
              delivery_candidates dc
            "#,
            &notification_channels[..] as &[&str],
            &notification_ids[..],
            &users_raw_ids[..],
            &notification_types[..] as &[&str],
            NotificationDeliveryStatus::Pending.as_str(),
            NotificationDeliveryStatus::SkippedPreferences.as_str(),
            NotificationDeliveryStatus::SkippedDefault.as_str(),
        );

        query.execute(&mut **transaction).await?;

        DBNotification::clear_user_notifications_cache(users, redis).await?;

        Ok(())
    }
}

impl DBNotification {
    pub async fn get<'a, 'b, E>(
        id: DBNotificationId,
        executor: E,
    ) -> Result<Option<Self>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        Self::get_many(&[id], executor)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(
        notification_ids: &[DBNotificationId],
        exec: E,
    ) -> Result<Vec<DBNotification>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let notification_ids_parsed: Vec<i64> =
            notification_ids.iter().map(|x| x.0).collect();
        sqlx::query!(
            "
            SELECT n.id, n.user_id, n.name, n.text, n.link, n.created, n.read, n.type notification_type, n.body,
            JSONB_AGG(DISTINCT jsonb_build_object('id', na.id, 'notification_id', na.notification_id, 'name', na.name, 'action_route_method', na.action_route_method, 'action_route', na.action_route)) filter (where na.id is not null) actions
            FROM notifications n
            LEFT OUTER JOIN notifications_actions na on n.id = na.notification_id
            WHERE n.id = ANY($1)
            GROUP BY n.id, n.user_id
            ORDER BY n.created DESC;
            ",
            &notification_ids_parsed
        )
            .fetch(exec)
            .map_ok(|row| {
                let id = DBNotificationId(row.id);

                DBNotification {
                    id,
                    user_id: DBUserId(row.user_id),
                    read: row.read,
                    created: row.created,
                    body: row.body.clone().and_then(|x| serde_json::from_value(x).ok()).unwrap_or_else(|| {
                        if let Some(name) = row.name {
                            NotificationBody::LegacyMarkdown {
                                notification_type: row.notification_type,
                                name,
                                text: row.text.unwrap_or_default(),
                                link: row.link.unwrap_or_default(),
                                actions: serde_json::from_value(
                                    row.actions.unwrap_or_default(),
                                )
                                    .ok()
                                    .unwrap_or_default(),
                            }
                        } else {
                            NotificationBody::Unknown
                        }
                    }),
                }
            })
            .try_collect::<Vec<DBNotification>>()
            .await
    }

    pub async fn get_all_user<'a, E>(
        user_id: DBUserId,
        exec: E,
    ) -> Result<Vec<DBNotification>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let db_notifications = sqlx::query!(
            "
            SELECT n.id, n.user_id, n.name, n.text, n.link, n.created, n.read, n.type notification_type, n.body,
            JSONB_AGG(DISTINCT jsonb_build_object('id', na.id, 'notification_id', na.notification_id, 'name', na.name, 'action_route_method', na.action_route_method, 'action_route', na.action_route)) filter (where na.id is not null) actions
            FROM notifications n
            LEFT OUTER JOIN notifications_actions na on n.id = na.notification_id
            WHERE n.user_id = $1
            GROUP BY n.id, n.user_id
            ",
            user_id as DBUserId
        )
            .fetch(exec)
            .map_ok(|row| {
                let id = DBNotificationId(row.id);

                DBNotification {
                    id,
                    user_id: DBUserId(row.user_id),
                    read: row.read,
                    created: row.created,
                    body: row.body.clone().and_then(|x| serde_json::from_value(x).ok()).unwrap_or_else(|| {
                        if let Some(name) = row.name {
                            NotificationBody::LegacyMarkdown {
                                notification_type: row.notification_type,
                                name,
                                text: row.text.unwrap_or_default(),
                                link: row.link.unwrap_or_default(),
                                actions: serde_json::from_value(
                                    row.actions.unwrap_or_default(),
                                )
                                    .ok()
                                    .unwrap_or_default(),
                            }
                        } else {
                            NotificationBody::Unknown
                        }
                    }),
                }
            })
            .try_collect::<Vec<DBNotification>>()
            .await?;

        Ok(db_notifications)
    }

    /// Returns user notifications that are configured to be exposed on the website.
    pub async fn get_many_user_exposed_on_site<'a, E>(
        user_id: DBUserId,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBNotification>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let mut redis = redis.connect().await?;

        let cached_notifications: Option<Vec<DBNotification>> = redis
            .get_deserialized_from_json(
                USER_NOTIFICATIONS_NAMESPACE,
                &user_id.0.to_string(),
            )
            .await?;

        if let Some(notifications) = cached_notifications {
            return Ok(notifications);
        }

        let db_notifications = sqlx::query!(
            "
            SELECT n.id, n.user_id, n.name, n.text, n.link, n.created, n.read, n.type notification_type, n.body,
            JSONB_AGG(DISTINCT jsonb_build_object('id', na.id, 'notification_id', na.notification_id, 'name', na.name, 'action_route_method', na.action_route_method, 'action_route', na.action_route)) filter (where na.id is not null) actions
            FROM notifications n
            LEFT OUTER JOIN notifications_actions na on n.id = na.notification_id
            INNER JOIN notifications_types nt on nt.name = n.body ->> 'type'
            WHERE n.user_id = $1
              AND nt.expose_in_site_notifications = TRUE
            GROUP BY n.id, n.user_id
            ",
            user_id as DBUserId
        )
            .fetch(exec)
            .map_ok(|row| {
                let id = DBNotificationId(row.id);

                DBNotification {
                    id,
                    user_id: DBUserId(row.user_id),
                    read: row.read,
                    created: row.created,
                    body: row.body.clone().and_then(|x| serde_json::from_value(x).ok()).unwrap_or_else(|| {
                        if let Some(name) = row.name {
                            NotificationBody::LegacyMarkdown {
                                notification_type: row.notification_type,
                                name,
                                text: row.text.unwrap_or_default(),
                                link: row.link.unwrap_or_default(),
                                actions: serde_json::from_value(
                                    row.actions.unwrap_or_default(),
                                )
                                    .ok()
                                    .unwrap_or_default(),
                            }
                        } else {
                            NotificationBody::Unknown
                        }
                    }),
                }
            })
            .try_collect::<Vec<DBNotification>>()
            .await?;

        redis
            .set_serialized_to_json(
                USER_NOTIFICATIONS_NAMESPACE,
                user_id.0,
                &db_notifications,
                None,
            )
            .await?;

        Ok(db_notifications)
    }

    pub async fn read(
        id: DBNotificationId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        Self::read_many(&[id], transaction, redis).await
    }

    pub async fn read_many(
        notification_ids: &[DBNotificationId],
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        let notification_ids_parsed: Vec<i64> =
            notification_ids.iter().map(|x| x.0).collect();

        let affected_users = sqlx::query!(
            "
            UPDATE notifications
            SET read = TRUE
            WHERE id = ANY($1)
            RETURNING user_id
            ",
            &notification_ids_parsed
        )
        .fetch(&mut **transaction)
        .map_ok(|x| DBUserId(x.user_id))
        .try_collect::<Vec<_>>()
        .await?;

        DBNotification::clear_user_notifications_cache(
            affected_users.iter(),
            redis,
        )
        .await?;

        Ok(Some(()))
    }

    pub async fn remove(
        id: DBNotificationId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        Self::remove_many(&[id], transaction, redis).await
    }

    pub async fn remove_many(
        notification_ids: &[DBNotificationId],
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        let notification_ids_parsed: Vec<i64> =
            notification_ids.iter().map(|x| x.0).collect();

        sqlx::query!(
            "
            DELETE FROM notifications_deliveries
            WHERE notification_id = ANY($1)
            ",
            &notification_ids_parsed
        )
        .execute(&mut **transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM notifications_actions
            WHERE notification_id = ANY($1)
            ",
            &notification_ids_parsed
        )
        .execute(&mut **transaction)
        .await?;

        let affected_users = sqlx::query!(
            "
            DELETE FROM notifications
            WHERE id = ANY($1)
            RETURNING user_id
            ",
            &notification_ids_parsed
        )
        .fetch(&mut **transaction)
        .map_ok(|x| DBUserId(x.user_id))
        .try_collect::<Vec<_>>()
        .await?;

        DBNotification::clear_user_notifications_cache(
            affected_users.iter(),
            redis,
        )
        .await?;

        Ok(Some(()))
    }

    pub async fn clear_user_notifications_cache(
        user_ids: impl IntoIterator<Item = &DBUserId>,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;

        redis
            .delete_many(user_ids.into_iter().map(|id| {
                (USER_NOTIFICATIONS_NAMESPACE, Some(id.0.to_string()))
            }))
            .await?;

        Ok(())
    }
}

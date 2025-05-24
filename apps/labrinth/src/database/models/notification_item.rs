use super::ids::*;
use crate::database::{models::DatabaseError, redis::RedisPool};
use crate::models::notifications::NotificationBody;
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

        sqlx::query!(
            "
            INSERT INTO notifications (
                id, user_id, body
            )
            SELECT * FROM UNNEST($1::bigint[], $2::bigint[], $3::jsonb[])
            ",
            &notification_ids
                .into_iter()
                .map(|x| x.0)
                .collect::<Vec<_>>()[..],
            &users.iter().map(|x| x.0).collect::<Vec<_>>()[..],
            &bodies[..],
        )
        .execute(&mut **transaction)
        .await?;

        DBNotification::clear_user_notifications_cache(&users, redis).await?;

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
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
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

    pub async fn get_many_user<'a, E>(
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
            WHERE n.user_id = $1
            GROUP BY n.id, n.user_id;
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

use super::ids::*;
use crate::database::{models::DatabaseError, redis::RedisPool};
use crate::models::notifications::NotificationBody;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

const USER_NOTIFICATIONS_NAMESPACE: &str = "user_notifications";

pub struct NotificationBuilder {
    pub body: NotificationBody,
}

#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub body: NotificationBody,
    pub read: bool,
    pub created: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct NotificationAction {
    pub id: NotificationActionId,
    pub notification_id: NotificationId,
    pub title: String,
    pub action_route_method: String,
    pub action_route: String,
}

impl NotificationBuilder {
    pub async fn insert(
        &self,
        user: UserId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        self.insert_many(vec![user], transaction, redis).await
    }

    pub async fn insert_many(
        &self,
        users: Vec<UserId>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut notifications = Vec::new();
        for user in users {
            let id = generate_notification_id(&mut *transaction).await?;

            notifications.push(Notification {
                id,
                user_id: user,
                body: self.body.clone(),
                read: false,
                created: Utc::now(),
            });
        }

        Notification::insert_many(&notifications, transaction, redis).await?;

        Ok(())
    }
}

impl Notification {
    pub async fn insert_many(
        notifications: &[Notification],
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let notification_ids = notifications.iter().map(|n| n.id.0).collect_vec();
        let user_ids = notifications.iter().map(|n| n.user_id.0).collect_vec();
        let bodies = notifications
            .iter()
            .map(|n| Ok(serde_json::value::to_value(n.body.clone())?))
            .collect::<Result<Vec<_>, DatabaseError>>()?;
        sqlx::query!(
            "
            INSERT INTO notifications (
                id, user_id, body
            )
            SELECT * FROM UNNEST($1::bigint[], $2::bigint[], $3::jsonb[])
            ",
            &notification_ids[..],
            &user_ids[..],
            &bodies[..],
        )
        .execute(&mut **transaction)
        .await?;

        Notification::clear_user_notifications_cache(
            notifications.iter().map(|n| &n.user_id),
            redis,
        )
        .await?;

        Ok(())
    }

    pub async fn get<'a, 'b, E>(
        id: NotificationId,
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
        notification_ids: &[NotificationId],
        exec: E,
    ) -> Result<Vec<Notification>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let notification_ids_parsed: Vec<i64> = notification_ids.iter().map(|x| x.0).collect();
        sqlx::query!(
            "
            SELECT n.id, n.user_id, n.title, n.text, n.link, n.created, n.read, n.type notification_type, n.body,
            JSONB_AGG(DISTINCT jsonb_build_object('id', na.id, 'notification_id', na.notification_id, 'title', na.title, 'action_route_method', na.action_route_method, 'action_route', na.action_route)) filter (where na.id is not null) actions
            FROM notifications n
            LEFT OUTER JOIN notifications_actions na on n.id = na.notification_id
            WHERE n.id = ANY($1)
            GROUP BY n.id, n.user_id
            ORDER BY n.created DESC;
            ",
            &notification_ids_parsed
        )
            .fetch_many(exec)
            .try_filter_map(|e| async {
                Ok(e.right().map(|row| {
                    let id = NotificationId(row.id);

                    Notification {
                        id,
                        user_id: UserId(row.user_id),
                        read: row.read,
                        created: row.created,
                        body: row.body.clone().and_then(|x| serde_json::from_value(x).ok()).unwrap_or_else(|| {
                            if let Some(title) = row.title {
                                NotificationBody::LegacyMarkdown {
                                    notification_type: row.notification_type,
                                    title,
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
                }))
            })
            .try_collect::<Vec<Notification>>()
            .await
    }

    pub async fn get_many_user<'a, E>(
        user_id: UserId,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Notification>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let cached_notifications: Option<Vec<Notification>> = redis
            .get_deserialized_from_json(USER_NOTIFICATIONS_NAMESPACE, user_id.0)
            .await?;

        if let Some(notifications) = cached_notifications {
            return Ok(notifications);
        }

        let db_notifications = sqlx::query!(
            "
            SELECT n.id, n.user_id, n.title, n.text, n.link, n.created, n.read, n.type notification_type, n.body,
            JSONB_AGG(DISTINCT jsonb_build_object('id', na.id, 'notification_id', na.notification_id, 'title', na.title, 'action_route_method', na.action_route_method, 'action_route', na.action_route)) filter (where na.id is not null) actions
            FROM notifications n
            LEFT OUTER JOIN notifications_actions na on n.id = na.notification_id
            WHERE n.user_id = $1
            GROUP BY n.id, n.user_id;
            ",
            user_id as UserId
        )
            .fetch_many(exec)
            .try_filter_map(|e| async {
                Ok(e.right().map(|row| {
                    let id = NotificationId(row.id);

                    Notification {
                        id,
                        user_id: UserId(row.user_id),
                        read: row.read,
                        created: row.created,
                        body: row.body.clone().and_then(|x| serde_json::from_value(x).ok()).unwrap_or_else(|| {
                            if let Some(title) = row.title {
                                NotificationBody::LegacyMarkdown {
                                    notification_type: row.notification_type,
                                    title,
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
                }))
            })
            .try_collect::<Vec<Notification>>()
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
        id: NotificationId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        Self::read_many(&[id], transaction, redis).await
    }

    pub async fn read_many(
        notification_ids: &[NotificationId],
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        let notification_ids_parsed: Vec<i64> = notification_ids.iter().map(|x| x.0).collect();

        let affected_users = sqlx::query!(
            "
            UPDATE notifications
            SET read = TRUE
            WHERE id = ANY($1)
            RETURNING user_id
            ",
            &notification_ids_parsed
        )
        .fetch_many(&mut **transaction)
        .try_filter_map(|e| async { Ok(e.right().map(|x| UserId(x.user_id))) })
        .try_collect::<Vec<_>>()
        .await?;

        Notification::clear_user_notifications_cache(affected_users.iter(), redis).await?;

        Ok(Some(()))
    }

    pub async fn remove(
        id: NotificationId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        Self::remove_many(&[id], transaction, redis).await
    }

    pub async fn remove_many(
        notification_ids: &[NotificationId],
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        let notification_ids_parsed: Vec<i64> = notification_ids.iter().map(|x| x.0).collect();

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
        .fetch_many(&mut **transaction)
        .try_filter_map(|e| async { Ok(e.right().map(|x| UserId(x.user_id))) })
        .try_collect::<Vec<_>>()
        .await?;

        Notification::clear_user_notifications_cache(affected_users.iter(), redis).await?;

        Ok(Some(()))
    }

    pub async fn clear_user_notifications_cache(
        user_ids: impl IntoIterator<Item = &UserId>,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        redis
            .delete_many(
                user_ids
                    .into_iter()
                    .map(|id| (USER_NOTIFICATIONS_NAMESPACE, Some(id.0.to_string()))),
            )
            .await?;

        Ok(())
    }
}

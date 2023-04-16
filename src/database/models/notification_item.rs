use super::ids::*;
use crate::database::models::DatabaseError;
use crate::models::notifications::NotificationBody;
use chrono::{DateTime, Utc};
use serde::Deserialize;

pub struct NotificationBuilder {
    pub body: NotificationBody,
}

pub struct Notification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub body: NotificationBody,
    pub read: bool,
    pub created: DateTime<Utc>,
}

#[derive(Deserialize)]
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
    ) -> Result<(), DatabaseError> {
        self.insert_many(vec![user], transaction).await
    }

    pub async fn insert_many(
        &self,
        users: Vec<UserId>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        for user in users {
            let id = generate_notification_id(&mut *transaction).await?;

            Notification {
                id,
                user_id: user,
                body: self.body.clone(),
                read: false,
                created: Utc::now(),
            }
            .insert(&mut *transaction)
            .await?;
        }

        Ok(())
    }
}

impl Notification {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO notifications (
                id, user_id, body
            )
            VALUES (
                $1, $2, $3
            )
            ",
            self.id as NotificationId,
            self.user_id as UserId,
            serde_json::value::to_value(self.body.clone())?
        )
        .execute(&mut *transaction)
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
        use futures::stream::TryStreamExt;

        let notification_ids_parsed: Vec<i64> =
            notification_ids.iter().map(|x| x.0).collect();
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
    ) -> Result<Vec<Notification>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        sqlx::query!(
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
            .await
    }

    pub async fn remove(
        id: NotificationId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, sqlx::error::Error> {
        Self::remove_many(&[id], transaction).await
    }

    pub async fn remove_many(
        notification_ids: &[NotificationId],
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, sqlx::error::Error> {
        let notification_ids_parsed: Vec<i64> =
            notification_ids.iter().map(|x| x.0).collect();

        sqlx::query!(
            "
            DELETE FROM notifications_actions
            WHERE notification_id = ANY($1)
            ",
            &notification_ids_parsed
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM notifications
            WHERE id = ANY($1)
            ",
            &notification_ids_parsed
        )
        .execute(&mut *transaction)
        .await?;

        Ok(Some(()))
    }
}

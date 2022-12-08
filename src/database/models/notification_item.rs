use super::ids::*;
use crate::database::models::DatabaseError;
use chrono::{DateTime, Utc};
use serde::Deserialize;

pub struct NotificationBuilder {
    pub notification_type: Option<String>,
    pub title: String,
    pub text: String,
    pub link: String,
    pub actions: Vec<NotificationActionBuilder>,
}

pub struct NotificationActionBuilder {
    pub title: String,
    pub action_route: (String, String),
}

pub struct Notification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub notification_type: Option<String>,
    pub title: String,
    pub text: String,
    pub link: String,
    pub read: bool,
    pub created: DateTime<Utc>,
    pub actions: Vec<NotificationAction>,
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

            let mut actions = Vec::new();

            for action in &self.actions {
                actions.push(NotificationAction {
                    id: NotificationActionId(0),
                    notification_id: id,
                    title: action.title.clone(),
                    action_route_method: action.action_route.0.clone(),
                    action_route: action.action_route.1.clone(),
                })
            }

            Notification {
                id,
                user_id: user,
                notification_type: self.notification_type.clone(),
                title: self.title.clone(),
                text: self.text.clone(),
                link: self.link.clone(),
                read: false,
                created: Utc::now(),
                actions,
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
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO notifications (
                id, user_id, title, text, link, type
            )
            VALUES (
                $1, $2, $3, $4, $5, $6
            )
            ",
            self.id as NotificationId,
            self.user_id as UserId,
            &self.title,
            &self.text,
            &self.link,
            self.notification_type
        )
        .execute(&mut *transaction)
        .await?;

        for action in &self.actions {
            action.insert(&mut *transaction).await?;
        }

        Ok(())
    }

    pub async fn get<'a, 'b, E>(
        id: NotificationId,
        executor: E,
    ) -> Result<Option<Self>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT n.user_id, n.title, n.text, n.link, n.created, n.read, n.type notification_type,
            JSONB_AGG(DISTINCT jsonb_build_object('id', na.id, 'notification_id', na.notification_id, 'title', na.title, 'action_route_method', na.action_route_method, 'action_route', na.action_route)) filter (where na.id is not null) actions
            FROM notifications n
            LEFT OUTER JOIN notifications_actions na on n.id = na.notification_id
            WHERE n.id = $1
            GROUP BY n.id, n.user_id;
            ",
            id as NotificationId,
        )
            .fetch_optional(executor)
            .await?;

        if let Some(row) = result {
            Ok(Some(Notification {
                id,
                user_id: UserId(row.user_id),
                notification_type: row.notification_type,
                title: row.title,
                text: row.text,
                link: row.link,
                read: row.read,
                created: row.created,
                actions: serde_json::from_value(
                    row.actions.unwrap_or_default(),
                )
                .ok()
                .unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many<'a, E>(
        notification_ids: Vec<NotificationId>,
        exec: E,
    ) -> Result<Vec<Notification>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let notification_ids_parsed: Vec<i64> =
            notification_ids.into_iter().map(|x| x.0).collect();
        sqlx::query!(
            "
            SELECT n.id, n.user_id, n.title, n.text, n.link, n.created, n.read, n.type notification_type,
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
                        notification_type: row.notification_type,
                        title: row.title,
                        text: row.text,
                        link: row.link,
                        read: row.read,
                        created: row.created,
                        actions: serde_json::from_value(
                            row.actions.unwrap_or_default(),
                        )
                            .ok()
                            .unwrap_or_default(),
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
            SELECT n.id, n.user_id, n.title, n.text, n.link, n.created, n.read, n.type notification_type,
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
                        notification_type: row.notification_type,
                        title: row.title,
                        text: row.text,
                        link: row.link,
                        read: row.read,
                        created: row.created,
                        actions: serde_json::from_value(
                            row.actions.unwrap_or_default(),
                        )
                            .ok()
                            .unwrap_or_default(),
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
        sqlx::query!(
            "
            DELETE FROM notifications_actions
            WHERE notification_id = $1
            ",
            id as NotificationId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM notifications
            WHERE id = $1
            ",
            id as NotificationId,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(Some(()))
    }

    pub async fn remove_many(
        notification_ids: Vec<NotificationId>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, sqlx::error::Error> {
        let notification_ids_parsed: Vec<i64> =
            notification_ids.into_iter().map(|x| x.0).collect();

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

impl NotificationAction {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO notifications_actions (
                notification_id, title, action_route, action_route_method
            )
            VALUES (
                $1, $2, $3, $4
            )
            ",
            self.notification_id as NotificationId,
            &self.title,
            &self.action_route,
            &self.action_route_method
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }
}

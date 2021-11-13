use super::ids::*;
use crate::database::models::DatabaseError;

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
    pub created: chrono::DateTime<chrono::Utc>,
    pub actions: Vec<NotificationAction>,
}

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
                created: chrono::Utc::now(),
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
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let (notifications, actions) = futures::join!(
            sqlx::query!(
                "
                SELECT n.user_id, n.title, n.text, n.link, n.created, n.read, n.type notification_type
                FROM notifications n
                WHERE n.id = $1
                GROUP BY n.id, n.user_id;
                ",
                id as NotificationId,
            )
            .fetch_optional(executor),
            sqlx::query!(
                "
                SELECT id, title, notification_id, action_route, action_route_method
                FROM notifications_actions
                WHERE notification_id = $1
                ",
                id as NotificationId,
            ).fetch_all(executor),
        );

        if let Some(row) = notifications? {
            Ok(Some(Notification {
                id,
                user_id: UserId(row.user_id),
                notification_type: row.notification_type,
                title: row.title,
                text: row.text,
                link: row.link,
                read: row.read,
                created: row.created,
                actions: actions?
                    .into_iter()
                    .map(|x| NotificationAction {
                        id: NotificationActionId(x.id),
                        notification_id: NotificationId(x.notification_id),
                        title: x.title,
                        action_route_method: x.action_route_method,
                        action_route: x.action_route,
                    })
                    .collect(),
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
        futures::future::try_join_all(notification_ids.into_iter().map(|id| Self::get(id, exec)))
            .await
            .map(|x| x.into_iter().flatten().collect())
    }

    pub async fn get_many_user<'a, E>(
        user_id: UserId,
        exec: E,
    ) -> Result<Vec<Notification>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let notification_ids = sqlx::query!(
            "
            SELECT id
            FROM notifications
            WHERE user_id = $1
            ",
            user_id as UserId
        )
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|x| NotificationId(x.id))
        .collect();

        Self::get_many(notification_ids, exec).await
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
        let notification_ids_parsed: Vec<i64> = notification_ids.into_iter().map(|x| x.0).collect();

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

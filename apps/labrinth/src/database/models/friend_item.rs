use crate::database::models::UserId;
use chrono::{DateTime, Utc};

pub struct FriendItem {
    pub user_id: UserId,
    pub friend_id: UserId,
    pub created: DateTime<Utc>,
    pub accepted: bool,
}

impl FriendItem {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO friends (user_id, friend_id, created, accepted)
            VALUES ($1, $2, $3, $4)
            ",
            self.user_id.0,
            self.friend_id.0,
            self.created,
            self.accepted,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn get_friend<'a, E>(
        user_id: UserId,
        friend_id: UserId,
        exec: E,
    ) -> Result<Option<FriendItem>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let friend = sqlx::query!(
            "
            SELECT f.user_id, f.friend_id, f.created, f.accepted
            FROM friends f
            WHERE (f.user_id = $1 AND f.friend_id = $2) OR (f.user_id = $2 AND f.friend_id = $1)
            ",
            user_id.0,
            friend_id.0,
        )
        .fetch_optional(exec)
        .await?
            .map(|row| FriendItem {
                user_id: UserId(row.user_id),
                friend_id: UserId(row.friend_id),
                created: row.created,
                accepted: row.accepted,
            });

        Ok(friend)
    }

    pub async fn update_friend(
        user_id: UserId,
        friend_id: UserId,
        accepted: bool,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            UPDATE friends
            SET accepted = $3
            WHERE (user_id = $1 AND friend_id = $2) OR (user_id = $2 AND friend_id = $1)
            ",
            user_id.0,
            friend_id.0,
            accepted,
        )
            .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn get_user_friends<'a, E>(
        user_id: UserId,
        accepted: Option<bool>,
        exec: E,
    ) -> Result<Vec<FriendItem>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let friends = sqlx::query!(
            "
            SELECT f.user_id, f.friend_id, f.created, f.accepted
            FROM friends f
            WHERE f.user_id = $1 OR f.friend_id = $1
            ",
            user_id.0,
        )
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| FriendItem {
            user_id: UserId(row.user_id),
            friend_id: UserId(row.friend_id),
            created: row.created,
            accepted: row.accepted,
        })
        .filter(|x| accepted.map(|y| y == x.accepted).unwrap_or(true))
        .collect::<Vec<_>>();

        Ok(friends)
    }

    pub async fn remove(
        user_id: UserId,
        friend_id: UserId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            DELETE FROM friends
            WHERE (user_id = $1 AND friend_id = $2) OR (user_id = $2 AND friend_id = $1)
            ",
            user_id.0 as i64,
            friend_id.0 as i64,
        )
            .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

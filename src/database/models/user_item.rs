use super::ids::{ProjectId, UserId};
use crate::models::users::{Badges, RecipientType, RecipientWallet};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

pub struct User {
    pub id: UserId,
    pub github_id: Option<i64>,
    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created: DateTime<Utc>,
    pub role: String,
    pub badges: Badges,
    pub balance: Decimal,
    pub payout_wallet: Option<RecipientWallet>,
    pub payout_wallet_type: Option<RecipientType>,
    pub payout_address: Option<String>,
    pub flame_anvil_key: Option<String>,
}

impl User {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO users (
                id, github_id, username, name, email,
                avatar_url, bio, created
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8
            )
            ",
            self.id as UserId,
            self.github_id,
            &self.username,
            self.name.as_ref(),
            self.email.as_ref(),
            self.avatar_url.as_ref(),
            self.bio.as_ref(),
            self.created,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }
    pub async fn get<'a, 'b, E>(
        id: UserId,
        executor: E,
    ) -> Result<Option<Self>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT u.github_id, u.name, u.email,
                u.avatar_url, u.username, u.bio,
                u.created, u.role, u.badges,
                u.balance, u.payout_wallet, u.payout_wallet_type,
                u.payout_address, u.flame_anvil_key
            FROM users u
            WHERE u.id = $1
            ",
            id as UserId,
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            Ok(Some(User {
                id,
                github_id: row.github_id,
                name: row.name,
                email: row.email,
                avatar_url: row.avatar_url,
                username: row.username,
                bio: row.bio,
                created: row.created,
                role: row.role,
                badges: Badges::from_bits(row.badges as u64)
                    .unwrap_or_default(),
                balance: row.balance,
                payout_wallet: row
                    .payout_wallet
                    .map(|x| RecipientWallet::from_string(&x)),
                payout_wallet_type: row
                    .payout_wallet_type
                    .map(|x| RecipientType::from_string(&x)),
                payout_address: row.payout_address,
                flame_anvil_key: row.flame_anvil_key,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_from_github_id<'a, 'b, E>(
        github_id: u64,
        executor: E,
    ) -> Result<Option<Self>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT u.id, u.name, u.email,
                u.avatar_url, u.username, u.bio,
                u.created, u.role, u.badges,
                u.balance, u.payout_wallet, u.payout_wallet_type,
                u.payout_address, u.flame_anvil_key
            FROM users u
            WHERE u.github_id = $1
            ",
            github_id as i64,
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            Ok(Some(User {
                id: UserId(row.id),
                github_id: Some(github_id as i64),
                name: row.name,
                email: row.email,
                avatar_url: row.avatar_url,
                username: row.username,
                bio: row.bio,
                created: row.created,
                role: row.role,
                badges: Badges::from_bits(row.badges as u64)
                    .unwrap_or_default(),
                balance: row.balance,
                payout_wallet: row
                    .payout_wallet
                    .map(|x| RecipientWallet::from_string(&x)),
                payout_wallet_type: row
                    .payout_wallet_type
                    .map(|x| RecipientType::from_string(&x)),
                payout_address: row.payout_address,
                flame_anvil_key: row.flame_anvil_key,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_from_username<'a, 'b, E>(
        username: String,
        executor: E,
    ) -> Result<Option<Self>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT u.id, u.github_id, u.name, u.email,
                u.avatar_url, u.username, u.bio,
                u.created, u.role, u.badges,
                u.balance, u.payout_wallet, u.payout_wallet_type,
                u.payout_address, u.flame_anvil_key
            FROM users u
            WHERE LOWER(u.username) = LOWER($1)
            ",
            username
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            Ok(Some(User {
                id: UserId(row.id),
                github_id: row.github_id,
                name: row.name,
                email: row.email,
                avatar_url: row.avatar_url,
                username: row.username,
                bio: row.bio,
                created: row.created,
                role: row.role,
                badges: Badges::from_bits(row.badges as u64)
                    .unwrap_or_default(),
                balance: row.balance,
                payout_wallet: row
                    .payout_wallet
                    .map(|x| RecipientWallet::from_string(&x)),
                payout_wallet_type: row
                    .payout_wallet_type
                    .map(|x| RecipientType::from_string(&x)),
                payout_address: row.payout_address,
                flame_anvil_key: row.flame_anvil_key,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many<'a, E>(
        user_ids: Vec<UserId>,
        exec: E,
    ) -> Result<Vec<User>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let user_ids_parsed: Vec<i64> =
            user_ids.into_iter().map(|x| x.0).collect();
        let users = sqlx::query!(
            "
            SELECT u.id, u.github_id, u.name, u.email,
                u.avatar_url, u.username, u.bio,
                u.created, u.role, u.badges,
                u.balance, u.payout_wallet, u.payout_wallet_type,
                u.payout_address, u.flame_anvil_key
            FROM users u
            WHERE u.id = ANY($1)
            ",
            &user_ids_parsed
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|u| User {
                id: UserId(u.id),
                github_id: u.github_id,
                name: u.name,
                email: u.email,
                avatar_url: u.avatar_url,
                username: u.username,
                bio: u.bio,
                created: u.created,
                role: u.role,
                badges: Badges::from_bits(u.badges as u64).unwrap_or_default(),
                balance: u.balance,
                payout_wallet: u
                    .payout_wallet
                    .map(|x| RecipientWallet::from_string(&x)),
                payout_wallet_type: u
                    .payout_wallet_type
                    .map(|x| RecipientType::from_string(&x)),
                payout_address: u.payout_address,
                flame_anvil_key: u.flame_anvil_key,
            }))
        })
        .try_collect::<Vec<User>>()
        .await?;

        Ok(users)
    }

    pub async fn get_projects<'a, E>(
        user_id: UserId,
        exec: E,
    ) -> Result<Vec<ProjectId>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let projects = sqlx::query!(
            "
            SELECT m.id FROM mods m
            INNER JOIN team_members tm ON tm.team_id = m.team_id AND tm.accepted = TRUE
            WHERE tm.user_id = $1
            ORDER BY m.downloads DESC
            ",
            user_id as UserId,
        )
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|m| ProjectId(m.id))) })
        .try_collect::<Vec<ProjectId>>()
        .await?;

        Ok(projects)
    }

    pub async fn remove(
        id: UserId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, sqlx::error::Error> {
        let deleted_user: UserId = crate::models::users::DELETED_USER.into();

        sqlx::query!(
            "
            UPDATE team_members
            SET user_id = $1
            WHERE (user_id = $2 AND role = $3)
            ",
            deleted_user as UserId,
            id as UserId,
            crate::models::teams::OWNER_ROLE
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            UPDATE versions
            SET author_id = $1
            WHERE (author_id = $2)
            ",
            deleted_user as UserId,
            id as UserId,
        )
        .execute(&mut *transaction)
        .await?;

        use futures::TryStreamExt;
        let notifications: Vec<i64> = sqlx::query!(
            "
            SELECT n.id FROM notifications n
            WHERE n.user_id = $1
            ",
            id as UserId,
        )
        .fetch_many(&mut *transaction)
        .try_filter_map(|e| async { Ok(e.right().map(|m| m.id)) })
        .try_collect::<Vec<i64>>()
        .await?;

        sqlx::query!(
            "
            DELETE FROM notifications
            WHERE user_id = $1
            ",
            id as UserId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM reports
            WHERE user_id = $1
            ",
            id as UserId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM mod_follows
            WHERE follower_id = $1
            ",
            id as UserId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM notifications_actions
             WHERE notification_id = ANY($1)
            ",
            &notifications
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM team_members
            WHERE user_id = $1
            ",
            id as UserId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM payouts_values
            WHERE user_id = $1
            ",
            id as UserId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM historical_payouts
            WHERE user_id = $1
            ",
            id as UserId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM users
            WHERE id = $1
            ",
            id as UserId,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(Some(()))
    }

    pub async fn remove_full(
        id: UserId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, sqlx::error::Error> {
        use futures::TryStreamExt;
        let projects: Vec<ProjectId> = sqlx::query!(
            "
            SELECT m.id FROM mods m
            INNER JOIN team_members tm ON tm.team_id = m.team_id
            WHERE tm.user_id = $1 AND tm.role = $2
            ",
            id as UserId,
            crate::models::teams::OWNER_ROLE
        )
        .fetch_many(&mut *transaction)
        .try_filter_map(|e| async { Ok(e.right().map(|m| ProjectId(m.id))) })
        .try_collect::<Vec<ProjectId>>()
        .await?;

        for project_id in projects {
            let _result = super::project_item::Project::remove_full(
                project_id,
                transaction,
            )
            .await?;
        }

        let notifications: Vec<i64> = sqlx::query!(
            "
            SELECT n.id FROM notifications n
            WHERE n.user_id = $1
            ",
            id as UserId,
        )
        .fetch_many(&mut *transaction)
        .try_filter_map(|e| async { Ok(e.right().map(|m| m.id)) })
        .try_collect::<Vec<i64>>()
        .await?;

        sqlx::query!(
            "
            DELETE FROM notifications
            WHERE user_id = $1
            ",
            id as UserId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM notifications_actions
             WHERE notification_id = ANY($1)
            ",
            &notifications
        )
        .execute(&mut *transaction)
        .await?;

        let deleted_user: UserId = crate::models::users::DELETED_USER.into();

        sqlx::query!(
            "
            UPDATE versions
            SET author_id = $1
            WHERE (author_id = $2)
            ",
            deleted_user as UserId,
            id as UserId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM team_members
            WHERE user_id = $1
            ",
            id as UserId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM users
            WHERE id = $1
            ",
            id as UserId,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(Some(()))
    }

    pub async fn get_id_from_username_or_id<'a, 'b, E>(
        username_or_id: &str,
        executor: E,
    ) -> Result<Option<UserId>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let id_option =
            crate::models::ids::base62_impl::parse_base62(username_or_id).ok();

        if let Some(id) = id_option {
            let id = UserId(id as i64);

            let mut user_id = sqlx::query!(
                "
                SELECT id FROM users
                WHERE id = $1
                ",
                id as UserId
            )
            .fetch_optional(executor)
            .await?
            .map(|x| UserId(x.id));

            if user_id.is_none() {
                user_id = sqlx::query!(
                    "
                    SELECT id FROM users
                    WHERE LOWER(username) = LOWER($1)
                    ",
                    username_or_id
                )
                .fetch_optional(executor)
                .await?
                .map(|x| UserId(x.id));
            }

            Ok(user_id)
        } else {
            let id = sqlx::query!(
                "
                SELECT id FROM users
                WHERE LOWER(username) = LOWER($1)
                ",
                username_or_id
            )
            .fetch_optional(executor)
            .await?;

            Ok(id.map(|x| UserId(x.id)))
        }
    }
}

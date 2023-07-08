use super::ids::{ProjectId, UserId};
use crate::database::models::DatabaseError;
use crate::models::ids::base62_impl::{parse_base62, to_base62};
use crate::models::users::{Badges, RecipientType, RecipientWallet};
use chrono::{DateTime, Utc};
use redis::cmd;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

const USERS_NAMESPACE: &str = "users";
const USER_USERNAMES_NAMESPACE: &str = "users_usernames";
// const USERS_PROJECTS_NAMESPACE: &str = "users_projects";
const DEFAULT_EXPIRY: i64 = 1800; // 30 minutes

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct User {
    pub id: UserId,

    pub github_id: Option<i64>,
    pub discord_id: Option<i64>,
    pub gitlab_id: Option<i64>,
    pub google_id: Option<String>,
    pub steam_id: Option<i64>,
    pub microsoft_id: Option<String>,
    pub password: Option<String>,

    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created: DateTime<Utc>,
    pub role: String,
    pub badges: Badges,
    pub balance: Decimal,
    pub payout_wallet: Option<RecipientWallet>,
    pub payout_wallet_type: Option<RecipientType>,
    pub payout_address: Option<String>,
}

impl User {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO users (
                id, username, name, email,
                avatar_url, bio, created,
                github_id, discord_id, gitlab_id, google_id, steam_id, microsoft_id,
                email_verified, password
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7,
                $8, $9, $10, $11, $12, $13,
                $14, $15
            )
            ",
            self.id as UserId,
            &self.username,
            self.name.as_ref(),
            self.email.as_ref(),
            self.avatar_url.as_ref(),
            self.bio.as_ref(),
            self.created,
            self.github_id,
            self.discord_id,
            self.gitlab_id,
            self.google_id,
            self.steam_id,
            self.microsoft_id,
            self.email_verified,
            self.password,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn get<'a, 'b, E>(
        string: &str,
        executor: E,
        redis: &deadpool_redis::Pool,
    ) -> Result<Option<User>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        User::get_many(&[string], executor, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_id<'a, 'b, E>(
        id: UserId,
        executor: E,
        redis: &deadpool_redis::Pool,
    ) -> Result<Option<User>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        User::get_many(&[crate::models::ids::UserId::from(id)], executor, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many_ids<'a, E>(
        user_ids: &[UserId],
        exec: E,
        redis: &deadpool_redis::Pool,
    ) -> Result<Vec<User>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let ids = user_ids
            .iter()
            .map(|x| crate::models::ids::UserId::from(*x))
            .collect::<Vec<_>>();
        User::get_many(&ids, exec, redis).await
    }

    pub async fn get_many<'a, E, T: ToString>(
        users_strings: &[T],
        exec: E,
        redis: &deadpool_redis::Pool,
    ) -> Result<Vec<User>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::TryStreamExt;

        if users_strings.is_empty() {
            return Ok(Vec::new());
        }

        let mut redis = redis.get().await?;

        let mut found_users = Vec::new();
        let mut remaining_strings = users_strings
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();

        let mut user_ids = users_strings
            .iter()
            .flat_map(|x| parse_base62(&x.to_string()).map(|x| x as i64))
            .collect::<Vec<_>>();

        user_ids.append(
            &mut cmd("MGET")
                .arg(
                    users_strings
                        .iter()
                        .map(|x| {
                            format!(
                                "{}:{}",
                                USER_USERNAMES_NAMESPACE,
                                x.to_string().to_lowercase()
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                .query_async::<_, Vec<Option<i64>>>(&mut redis)
                .await?
                .into_iter()
                .flatten()
                .collect(),
        );

        if !user_ids.is_empty() {
            let users = cmd("MGET")
                .arg(
                    user_ids
                        .iter()
                        .map(|x| format!("{}:{}", USERS_NAMESPACE, x))
                        .collect::<Vec<_>>(),
                )
                .query_async::<_, Vec<Option<String>>>(&mut redis)
                .await?;

            for user in users {
                if let Some(user) = user.and_then(|x| serde_json::from_str::<User>(&x).ok()) {
                    remaining_strings
                        .retain(|x| &to_base62(user.id.0 as u64) != x && &user.username != x);
                    found_users.push(user);
                    continue;
                }
            }
        }

        if !remaining_strings.is_empty() {
            let user_ids_parsed: Vec<i64> = remaining_strings
                .iter()
                .flat_map(|x| parse_base62(&x.to_string()).ok())
                .map(|x| x as i64)
                .collect();
            let db_users: Vec<User> = sqlx::query!(
                "
                SELECT id, name, email,
                    avatar_url, username, bio,
                    created, role, badges,
                    balance, payout_wallet, payout_wallet_type, payout_address,
                    github_id, discord_id, gitlab_id, google_id, steam_id, microsoft_id,
                    email_verified, password
                FROM users
                WHERE id = ANY($1) OR LOWER(username) = ANY($2)
                ",
                &user_ids_parsed,
                &remaining_strings
                    .into_iter()
                    .map(|x| x.to_string().to_lowercase())
                    .collect::<Vec<_>>(),
            )
            .fetch_many(exec)
            .try_filter_map(|e| async {
                Ok(e.right().map(|u| User {
                    id: UserId(u.id),
                    github_id: u.github_id,
                    discord_id: u.discord_id,
                    gitlab_id: u.gitlab_id,
                    google_id: u.google_id,
                    steam_id: u.steam_id,
                    microsoft_id: u.microsoft_id,
                    name: u.name,
                    email: u.email,
                    email_verified: u.email_verified,
                    avatar_url: u.avatar_url,
                    username: u.username,
                    bio: u.bio,
                    created: u.created,
                    role: u.role,
                    badges: Badges::from_bits(u.badges as u64).unwrap_or_default(),
                    balance: u.balance,
                    payout_wallet: u.payout_wallet.map(|x| RecipientWallet::from_string(&x)),
                    payout_wallet_type: u
                        .payout_wallet_type
                        .map(|x| RecipientType::from_string(&x)),
                    payout_address: u.payout_address,
                    password: u.password,
                }))
            })
            .try_collect::<Vec<User>>()
            .await?;

            for user in db_users {
                cmd("SET")
                    .arg(format!("{}:{}", USERS_NAMESPACE, user.id.0))
                    .arg(serde_json::to_string(&user)?)
                    .arg("EX")
                    .arg(DEFAULT_EXPIRY)
                    .query_async::<_, ()>(&mut redis)
                    .await?;

                cmd("SET")
                    .arg(format!(
                        "{}:{}",
                        USER_USERNAMES_NAMESPACE,
                        user.username.to_lowercase()
                    ))
                    .arg(user.id.0)
                    .arg("EX")
                    .arg(DEFAULT_EXPIRY)
                    .query_async::<_, ()>(&mut redis)
                    .await?;
                found_users.push(user);
            }
        }

        Ok(found_users)
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

    pub async fn clear_caches(
        user_ids: &[(UserId, Option<String>)],
        redis: &deadpool_redis::Pool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.get().await?;
        let mut cmd = cmd("DEL");

        for (id, username) in user_ids {
            cmd.arg(format!("{}:{}", USERS_NAMESPACE, id.0));
            if let Some(username) = username {
                cmd.arg(format!(
                    "{}:{}",
                    USER_USERNAMES_NAMESPACE,
                    username.to_lowercase()
                ));
            }
        }

        cmd.query_async::<_, ()>(&mut redis).await?;

        Ok(())
    }

    pub async fn remove(
        id: UserId,
        full: bool,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &deadpool_redis::Pool,
    ) -> Result<Option<()>, DatabaseError> {
        let user = Self::get_id(id, &mut *transaction, redis).await?;

        if let Some(delete_user) = user {
            User::clear_caches(&[(id, Some(delete_user.username))], redis).await?;

            let deleted_user: UserId = crate::models::users::DELETED_USER.into();

            if full {
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
                    let _result =
                        super::project_item::Project::remove(project_id, transaction, redis)
                            .await?;
                }
            } else {
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
            }

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
                DELETE FROM notifications_actions
                 WHERE notification_id = ANY($1)
                ",
                &notifications
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
                r#"
                UPDATE threads_messages
                SET body = '{"type": "deleted"}', author_id = $2
                WHERE author_id = $1
                "#,
                id as UserId,
                deleted_user as UserId,
            )
            .execute(&mut *transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM threads_members
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
        } else {
            Ok(None)
        }
    }
}

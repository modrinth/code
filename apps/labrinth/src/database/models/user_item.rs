use super::ids::{DBProjectId, DBUserId};
use super::{DBCollectionId, DBReportId, DBThreadId};
use crate::database::models;
use crate::database::models::charge_item::DBCharge;
use crate::database::models::user_subscription_item::DBUserSubscription;
use crate::database::models::{DBOrganizationId, DatabaseError};
use crate::database::redis::RedisPool;
use crate::models::billing::ChargeStatus;
use crate::models::users::Badges;
use ariadne::ids::base62_impl::{parse_base62, to_base62};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::hash::Hash;

const USERS_NAMESPACE: &str = "users";
const USER_USERNAMES_NAMESPACE: &str = "users_usernames";
const USERS_PROJECTS_NAMESPACE: &str = "users_projects";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DBUser {
    pub id: DBUserId,

    pub github_id: Option<i64>,
    pub discord_id: Option<i64>,
    pub gitlab_id: Option<i64>,
    pub google_id: Option<String>,
    pub steam_id: Option<i64>,
    pub microsoft_id: Option<String>,
    pub password: Option<String>,

    pub paypal_id: Option<String>,
    pub paypal_country: Option<String>,
    pub paypal_email: Option<String>,
    pub venmo_handle: Option<String>,
    pub stripe_customer_id: Option<String>,

    pub totp_secret: Option<String>,

    pub username: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub avatar_url: Option<String>,
    pub raw_avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created: DateTime<Utc>,
    pub role: String,
    pub badges: Badges,

    pub allow_friend_requests: bool,
}

impl DBUser {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO users (
                id, username, email,
                avatar_url, raw_avatar_url, bio, created,
                github_id, discord_id, gitlab_id, google_id, steam_id, microsoft_id,
                email_verified, password, paypal_id, paypal_country, paypal_email,
                venmo_handle, stripe_customer_id, allow_friend_requests
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7,
                $8, $9, $10, $11, $12, $13,
                $14, $15, $16, $17, $18, $19, $20, $21
            )
            ",
            self.id as DBUserId,
            &self.username,
            self.email.as_ref(),
            self.avatar_url.as_ref(),
            self.raw_avatar_url.as_ref(),
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
            self.paypal_id,
            self.paypal_country,
            self.paypal_email,
            self.venmo_handle,
            self.stripe_customer_id,
            self.allow_friend_requests,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn get<'a, 'b, E>(
        string: &str,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<DBUser>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        DBUser::get_many(&[string], executor, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_id<'a, 'b, E>(
        id: DBUserId,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<DBUser>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        DBUser::get_many(&[ariadne::ids::UserId::from(id)], executor, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many_ids<'a, E>(
        user_ids: &[DBUserId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBUser>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let ids = user_ids
            .iter()
            .map(|x| ariadne::ids::UserId::from(*x))
            .collect::<Vec<_>>();
        DBUser::get_many(&ids, exec, redis).await
    }

    pub async fn get_many<
        'a,
        E,
        T: Display + Hash + Eq + PartialEq + Clone + Debug,
    >(
        users_strings: &[T],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBUser>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::TryStreamExt;

        let val = redis.get_cached_keys_with_slug(
            USERS_NAMESPACE,
            USER_USERNAMES_NAMESPACE,
            false,
            users_strings,
            |ids| async move {
                let user_ids: Vec<i64> = ids
                    .iter()
                    .flat_map(|x| parse_base62(&x.to_string()).ok())
                    .map(|x| x as i64)
                    .collect();
                let slugs = ids
                    .into_iter()
                    .map(|x| x.to_string().to_lowercase())
                    .collect::<Vec<_>>();

                let users = sqlx::query!(
                    "
                    SELECT id, email,
                        avatar_url, raw_avatar_url, username, bio,
                        created, role, badges,
                        github_id, discord_id, gitlab_id, google_id, steam_id, microsoft_id,
                        email_verified, password, totp_secret, paypal_id, paypal_country, paypal_email,
                        venmo_handle, stripe_customer_id, allow_friend_requests
                    FROM users
                    WHERE id = ANY($1) OR LOWER(username) = ANY($2)
                    ",
                    &user_ids,
                    &slugs,
                )
                    .fetch(exec)
                    .try_fold(DashMap::new(), |acc, u| {
                        let user = DBUser {
                            id: DBUserId(u.id),
                            github_id: u.github_id,
                            discord_id: u.discord_id,
                            gitlab_id: u.gitlab_id,
                            google_id: u.google_id,
                            steam_id: u.steam_id,
                            microsoft_id: u.microsoft_id,
                            email: u.email,
                            email_verified: u.email_verified,
                            avatar_url: u.avatar_url,
                            raw_avatar_url: u.raw_avatar_url,
                            username: u.username.clone(),
                            bio: u.bio,
                            created: u.created,
                            role: u.role,
                            badges: Badges::from_bits(u.badges as u64).unwrap_or_default(),
                            password: u.password,
                            paypal_id: u.paypal_id,
                            paypal_country: u.paypal_country,
                            paypal_email: u.paypal_email,
                            venmo_handle: u.venmo_handle,
                            stripe_customer_id: u.stripe_customer_id,
                            totp_secret: u.totp_secret,
                            allow_friend_requests: u.allow_friend_requests,
                        };

                        acc.insert(u.id, (Some(u.username), user));
                        async move { Ok(acc) }
                    })
                    .await?;

                Ok(users)
            }).await?;
        Ok(val)
    }

    pub async fn get_email<'a, E>(
        email: &str,
        exec: E,
    ) -> Result<Option<DBUserId>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let user_pass = sqlx::query!(
            "
            SELECT id FROM users
            WHERE email = $1
            ",
            email
        )
        .fetch_optional(exec)
        .await?;

        Ok(user_pass.map(|x| DBUserId(x.id)))
    }

    pub async fn get_projects<'a, E>(
        user_id: DBUserId,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBProjectId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let mut redis = redis.connect().await?;

        let cached_projects = redis
            .get_deserialized_from_json::<Vec<DBProjectId>>(
                USERS_PROJECTS_NAMESPACE,
                &user_id.0.to_string(),
            )
            .await?;

        if let Some(projects) = cached_projects {
            return Ok(projects);
        }

        let db_projects = sqlx::query!(
            "
            SELECT m.id FROM mods m
            INNER JOIN team_members tm ON tm.team_id = m.team_id AND tm.accepted = TRUE
            WHERE tm.user_id = $1
            ORDER BY m.downloads DESC
            ",
            user_id as DBUserId,
        )
        .fetch(exec)
        .map_ok(|m| DBProjectId(m.id))
        .try_collect::<Vec<DBProjectId>>()
        .await?;

        redis
            .set_serialized_to_json(
                USERS_PROJECTS_NAMESPACE,
                user_id.0,
                &db_projects,
                None,
            )
            .await?;

        Ok(db_projects)
    }

    pub async fn get_organizations<'a, E>(
        user_id: DBUserId,
        exec: E,
    ) -> Result<Vec<DBOrganizationId>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let orgs = sqlx::query!(
            "
            SELECT o.id FROM organizations o
            INNER JOIN team_members tm ON tm.team_id = o.team_id AND tm.accepted = TRUE
            WHERE tm.user_id = $1
            ",
            user_id as DBUserId,
        )
        .fetch(exec)
        .map_ok(|m| DBOrganizationId(m.id))
        .try_collect::<Vec<DBOrganizationId>>()
        .await?;

        Ok(orgs)
    }

    pub async fn get_collections<'a, E>(
        user_id: DBUserId,
        exec: E,
    ) -> Result<Vec<DBCollectionId>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let projects = sqlx::query!(
            "
            SELECT c.id FROM collections c
            WHERE c.user_id = $1
            ",
            user_id as DBUserId,
        )
        .fetch(exec)
        .map_ok(|m| DBCollectionId(m.id))
        .try_collect::<Vec<DBCollectionId>>()
        .await?;

        Ok(projects)
    }

    pub async fn get_follows<'a, E>(
        user_id: DBUserId,
        exec: E,
    ) -> Result<Vec<DBProjectId>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let projects = sqlx::query!(
            "
            SELECT mf.mod_id FROM mod_follows mf
            WHERE mf.follower_id = $1
            ",
            user_id as DBUserId,
        )
        .fetch(exec)
        .map_ok(|m| DBProjectId(m.mod_id))
        .try_collect::<Vec<DBProjectId>>()
        .await?;

        Ok(projects)
    }

    pub async fn get_reports<'a, E>(
        user_id: DBUserId,
        exec: E,
    ) -> Result<Vec<DBReportId>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let reports = sqlx::query!(
            "
            SELECT r.id FROM reports r
            WHERE r.user_id = $1
            ",
            user_id as DBUserId,
        )
        .fetch(exec)
        .map_ok(|m| DBReportId(m.id))
        .try_collect::<Vec<DBReportId>>()
        .await?;

        Ok(reports)
    }

    pub async fn get_backup_codes<'a, E>(
        user_id: DBUserId,
        exec: E,
    ) -> Result<Vec<String>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let codes = sqlx::query!(
            "
            SELECT code FROM user_backup_codes
            WHERE user_id = $1
            ",
            user_id as DBUserId,
        )
        .fetch(exec)
        .map_ok(|m| to_base62(m.code as u64))
        .try_collect::<Vec<String>>()
        .await?;

        Ok(codes)
    }

    pub async fn clear_caches(
        user_ids: &[(DBUserId, Option<String>)],
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;

        redis
            .delete_many(user_ids.iter().flat_map(|(id, username)| {
                [
                    (USERS_NAMESPACE, Some(id.0.to_string())),
                    (
                        USER_USERNAMES_NAMESPACE,
                        username.clone().map(|i| i.to_lowercase()),
                    ),
                ]
            }))
            .await?;
        Ok(())
    }

    pub async fn clear_project_cache(
        user_ids: &[DBUserId],
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;

        redis
            .delete_many(
                user_ids.iter().map(|id| {
                    (USERS_PROJECTS_NAMESPACE, Some(id.0.to_string()))
                }),
            )
            .await?;

        Ok(())
    }

    pub async fn remove(
        id: DBUserId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        let user = Self::get_id(id, &mut **transaction, redis).await?;

        if let Some(delete_user) = user {
            DBUser::clear_caches(&[(id, Some(delete_user.username))], redis)
                .await?;

            let deleted_user: DBUserId =
                crate::models::users::DELETED_USER.into();

            sqlx::query!(
                "
                UPDATE team_members
                SET user_id = $1
                WHERE (user_id = $2 AND is_owner = TRUE)
                ",
                deleted_user as DBUserId,
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                UPDATE versions
                SET author_id = $1
                WHERE (author_id = $2)
                ",
                deleted_user as DBUserId,
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            use futures::TryStreamExt;
            let notifications: Vec<i64> = sqlx::query!(
                "
                SELECT n.id FROM notifications n
                WHERE n.user_id = $1
                ",
                id as DBUserId,
            )
            .fetch(&mut **transaction)
            .map_ok(|m| m.id)
            .try_collect::<Vec<i64>>()
            .await?;

            sqlx::query!(
                "
                DELETE FROM notifications
                WHERE user_id = $1
                ",
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM notifications_actions
                 WHERE notification_id = ANY($1)
                ",
                &notifications
            )
            .execute(&mut **transaction)
            .await?;

            let user_collections = sqlx::query!(
                "
                SELECT id
                FROM collections
                WHERE user_id = $1
                ",
                id as DBUserId,
            )
            .fetch(&mut **transaction)
            .map_ok(|x| DBCollectionId(x.id))
            .try_collect::<Vec<_>>()
            .await?;

            for collection_id in user_collections {
                models::DBCollection::remove(collection_id, transaction, redis)
                    .await?;
            }

            let report_threads = sqlx::query!(
                "
                SELECT t.id
                FROM threads t
                INNER JOIN reports r ON t.report_id = r.id AND (r.user_id = $1 OR r.reporter = $1)
                WHERE report_id IS NOT NULL
                ",
                id as DBUserId,
            )
            .fetch(&mut **transaction)
            .map_ok(|x| DBThreadId(x.id))
            .try_collect::<Vec<_>>()
            .await?;

            for thread_id in report_threads {
                models::DBThread::remove_full(thread_id, transaction).await?;
            }

            sqlx::query!(
                "
                DELETE FROM reports
                WHERE user_id = $1 OR reporter = $1
                ",
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM mod_follows
                WHERE follower_id = $1
                ",
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM team_members
                WHERE user_id = $1
                ",
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM payouts_values
                WHERE user_id = $1
                ",
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM payouts
                WHERE user_id = $1
                ",
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                r#"
                UPDATE threads_messages
                SET body = '{"type": "deleted"}', author_id = $2
                WHERE author_id = $1
                "#,
                id as DBUserId,
                deleted_user as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM threads_members
                WHERE user_id = $1
                ",
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM sessions
                WHERE user_id = $1
                ",
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM pats
                WHERE user_id = $1
                ",
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM friends
                WHERE user_id = $1 OR friend_id = $1
                ",
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            let open_subscriptions =
                DBUserSubscription::get_all_user(id, &mut **transaction)
                    .await?;

            for x in open_subscriptions {
                let charge =
                    DBCharge::get_open_subscription(x.id, &mut **transaction)
                        .await?;
                if let Some(mut charge) = charge {
                    charge.status = ChargeStatus::Cancelled;
                    charge.due = Utc::now();
                    charge.user_id = deleted_user;

                    charge.upsert(transaction).await?;
                }
            }

            sqlx::query!(
                "
                UPDATE users_subscriptions
                SET user_id = $1
                WHERE user_id = $2
                ",
                deleted_user as DBUserId,
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM user_backup_codes
                WHERE user_id = $1
                ",
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM users
                WHERE id = $1
                ",
                id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;

            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}

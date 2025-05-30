use super::ids::*;
use crate::database::models::DatabaseError;
use crate::database::redis::RedisPool;
use ariadne::ids::base62_impl::parse_base62;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::hash::Hash;

const SESSIONS_NAMESPACE: &str = "sessions";
const SESSIONS_IDS_NAMESPACE: &str = "sessions_ids";
const SESSIONS_USERS_NAMESPACE: &str = "sessions_users";

pub struct SessionBuilder {
    pub session: String,
    pub user_id: DBUserId,

    pub os: Option<String>,
    pub platform: Option<String>,

    pub city: Option<String>,
    pub country: Option<String>,

    pub ip: String,
    pub user_agent: String,
}

impl SessionBuilder {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<DBSessionId, DatabaseError> {
        let id = generate_session_id(transaction).await?;

        sqlx::query!(
            "
            INSERT INTO sessions (
                id, session, user_id, os, platform,
                city, country, ip, user_agent
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9
            )
            ",
            id as DBSessionId,
            self.session,
            self.user_id as DBUserId,
            self.os,
            self.platform,
            self.city,
            self.country,
            self.ip,
            self.user_agent,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(id)
    }
}

#[derive(Deserialize, Serialize)]
pub struct DBSession {
    pub id: DBSessionId,
    pub session: String,
    pub user_id: DBUserId,

    pub created: DateTime<Utc>,
    pub last_login: DateTime<Utc>,
    pub expires: DateTime<Utc>,
    pub refresh_expires: DateTime<Utc>,

    pub os: Option<String>,
    pub platform: Option<String>,
    pub user_agent: String,

    pub city: Option<String>,
    pub country: Option<String>,
    pub ip: String,
}

impl DBSession {
    pub async fn get<
        'a,
        E,
        T: Display + Hash + Eq + PartialEq + Clone + Debug,
    >(
        id: T,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Option<DBSession>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        Self::get_many(&[id], exec, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_id<'a, 'b, E>(
        id: DBSessionId,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<DBSession>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        DBSession::get_many(
            &[crate::models::ids::SessionId::from(id)],
            executor,
            redis,
        )
        .await
        .map(|x| x.into_iter().next())
    }

    pub async fn get_many_ids<'a, E>(
        session_ids: &[DBSessionId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBSession>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let ids = session_ids
            .iter()
            .map(|x| crate::models::ids::SessionId::from(*x))
            .collect::<Vec<_>>();
        DBSession::get_many(&ids, exec, redis).await
    }

    pub async fn get_many<
        'a,
        E,
        T: Display + Hash + Eq + PartialEq + Clone + Debug,
    >(
        session_strings: &[T],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBSession>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::TryStreamExt;

        let val = redis.get_cached_keys_with_slug(
            SESSIONS_NAMESPACE,
            SESSIONS_IDS_NAMESPACE,
            true,
            session_strings,
            |ids| async move {
                let session_ids: Vec<i64> = ids
                    .iter()
                    .flat_map(|x| parse_base62(&x.to_string()).ok())
                    .map(|x| x as i64)
                    .collect();
                let slugs = ids
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>();
                let db_sessions = sqlx::query!(
                    "
                    SELECT id, user_id, session, created, last_login, expires, refresh_expires, os, platform,
                    city, country, ip, user_agent
                    FROM sessions
                    WHERE id = ANY($1) OR session = ANY($2)
                    ORDER BY created DESC
                    ",
                    &session_ids,
                    &slugs,
                )
                    .fetch(exec)
                    .try_fold(DashMap::new(), |acc, x| {
                        let session = DBSession {
                            id: DBSessionId(x.id),
                            session: x.session.clone(),
                            user_id: DBUserId(x.user_id),
                            created: x.created,
                            last_login: x.last_login,
                            expires: x.expires,
                            refresh_expires: x.refresh_expires,
                            os: x.os,
                            platform: x.platform,
                            city: x.city,
                            country: x.country,
                            ip: x.ip,
                            user_agent: x.user_agent,
                        };

                        acc.insert(x.id, (Some(x.session), session));

                        async move { Ok(acc) }
                    })
                    .await?;

                Ok(db_sessions)
            }).await?;

        Ok(val)
    }

    pub async fn get_user_sessions<'a, E>(
        user_id: DBUserId,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBSessionId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let mut redis = redis.connect().await?;

        let res = redis
            .get_deserialized_from_json::<Vec<i64>>(
                SESSIONS_USERS_NAMESPACE,
                &user_id.0.to_string(),
            )
            .await?;

        if let Some(res) = res {
            return Ok(res.into_iter().map(DBSessionId).collect());
        }

        use futures::TryStreamExt;
        let db_sessions: Vec<DBSessionId> = sqlx::query!(
            "
                SELECT id
                FROM sessions
                WHERE user_id = $1
                ORDER BY created DESC
                ",
            user_id.0,
        )
        .fetch(exec)
        .map_ok(|x| DBSessionId(x.id))
        .try_collect::<Vec<DBSessionId>>()
        .await?;

        redis
            .set_serialized_to_json(
                SESSIONS_USERS_NAMESPACE,
                user_id.0,
                &db_sessions,
                None,
            )
            .await?;

        Ok(db_sessions)
    }

    pub async fn clear_cache(
        clear_sessions: Vec<(
            Option<DBSessionId>,
            Option<String>,
            Option<DBUserId>,
        )>,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;

        if clear_sessions.is_empty() {
            return Ok(());
        }

        redis
            .delete_many(clear_sessions.into_iter().flat_map(
                |(id, session, user_id)| {
                    [
                        (SESSIONS_NAMESPACE, id.map(|i| i.0.to_string())),
                        (SESSIONS_IDS_NAMESPACE, session),
                        (
                            SESSIONS_USERS_NAMESPACE,
                            user_id.map(|i| i.0.to_string()),
                        ),
                    ]
                },
            ))
            .await?;
        Ok(())
    }

    pub async fn remove(
        id: DBSessionId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, sqlx::error::Error> {
        sqlx::query!(
            "
            DELETE FROM sessions WHERE id = $1
            ",
            id as DBSessionId,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(Some(()))
    }
}

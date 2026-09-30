use super::ids::*;
use crate::database::PgTransaction;

use crate::models::pats::Scopes;
use ariadne::ids::base62_impl::parse_base62;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use eyre::{Result, WrapErr};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use xredis::RedisPool;

const PATS_NAMESPACE: &str = "pats:v4";
const PATS_TOKENS_NAMESPACE: &str = "pats_tokens:v4";
const PATS_USERS_NAMESPACE: &str = "pats_users:v4";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DBPersonalAccessToken {
    pub id: DBPatId,
    pub name: String,
    pub access_token: String,
    pub scopes: Scopes,
    pub user_id: DBUserId,
    pub created: DateTime<Utc>,
    pub expires: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

impl DBPersonalAccessToken {
    pub async fn insert(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<()> {
        sqlx::query!(
            "
            INSERT INTO pats (
                id, name, access_token, scopes, user_id,
                expires
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6
            )
            ",
            self.id as DBPatId,
            self.name,
            self.access_token,
            self.scopes.bits() as i64,
            self.user_id as DBUserId,
            self.expires
        )
        .execute(&mut *transaction)
        .await
        .wrap_err("inserting personal access token")?;

        Ok(())
    }

    pub async fn get<
        'a,
        E,
        T: Display + Hash + Eq + PartialEq + Clone + Debug,
    >(
        id: T,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Option<DBPersonalAccessToken>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        Self::get_many(&[id], exec, redis)
            .await
            .wrap_err("fetching personal access token")
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many_ids<'a, E>(
        pat_ids: &[DBPatId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBPersonalAccessToken>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let ids = pat_ids
            .iter()
            .map(|x| crate::models::ids::PatId::from(*x))
            .collect::<Vec<_>>();
        DBPersonalAccessToken::get_many(&ids, exec, redis)
            .await
            .wrap_err("fetching personal access tokens by ID")
    }

    pub async fn get_many<
        'a,
        E,
        T: Display + Hash + Eq + PartialEq + Clone + Debug,
    >(
        pat_strings: &[T],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBPersonalAccessToken>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let val = redis
            .get_cached_keys_with_slug(
                PATS_NAMESPACE,
                PATS_TOKENS_NAMESPACE,
                true,
                pat_strings,
                |ids| async move {
                    let pat_ids: Vec<i64> = ids
                        .iter()
                        .filter_map(|x| parse_base62(&x.to_string()).ok())
                        .map(|x| x as i64)
                        .collect();
                    let slugs = ids.into_iter().map(|x| x.to_string()).collect::<Vec<_>>();

                    let pats = sqlx::query!(
                        "
                        SELECT id, name, access_token, scopes, user_id, created, expires, last_used
                        FROM pats
                        WHERE id = ANY($1) OR access_token = ANY($2)
                        ORDER BY created DESC
                        ",
                        &pat_ids,
                        &slugs,
                    )
                    .fetch(exec)
                    .map_err(eyre::Report::from)
                    .try_fold(DashMap::new(), |acc, x| {
                        let pat = DBPersonalAccessToken {
                            id: DBPatId(x.id),
                            name: x.name,
                            access_token: x.access_token.clone(),
                            scopes: Scopes::from_bits(x.scopes as u64).unwrap_or(Scopes::NONE),
                            user_id: DBUserId(x.user_id),
                            created: x.created,
                            expires: x.expires,
                            last_used: x.last_used,
                        };

                        acc.insert(x.id, (Some(x.access_token), pat));
                        async move { eyre::Ok(acc) }
                    })
                    .await
                    .wrap_err("fetching personal access tokens from database")?;
                    eyre::Ok(pats)
                },
            )
            .await
            .wrap_err("fetching personal access tokens")?;

        Ok(val)
    }

    pub async fn get_user_pats<'a, E>(
        user_id: DBUserId,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBPatId>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        {
            let mut redis = redis.connect().await.wrap_err(
                "connecting to Redis for user personal access tokens",
            )?;
            let key = redis.key().entity(PATS_USERS_NAMESPACE, user_id.0);

            let res = redis
                .get_deserialized::<Vec<i64>>(&key)
                .await
                .wrap_err("fetching user personal access tokens from cache")?;

            if let Some(res) = res {
                return Ok(res.into_iter().map(DBPatId).collect());
            }
        }

        let db_pats: Vec<DBPatId> = sqlx::query!(
            "
            SELECT id
            FROM pats
            WHERE user_id = $1
            ORDER BY created DESC
            ",
            user_id.0,
        )
        .fetch(exec)
        .map_ok(|x| DBPatId(x.id))
        .try_collect::<Vec<DBPatId>>()
        .await
        .wrap_err("fetching user personal access tokens from database")?;

        let mut redis = redis.connect().await.wrap_err(
            "connecting to Redis to cache user personal access tokens",
        )?;
        let key = redis.key().entity(PATS_USERS_NAMESPACE, user_id.0);

        redis
            .set_serialized(&key, &db_pats, None)
            .await
            .wrap_err("caching user personal access tokens")?;
        Ok(db_pats)
    }

    pub async fn clear_cache(
        clear_pats: Vec<(Option<DBPatId>, Option<String>, Option<DBUserId>)>,
        redis: &RedisPool,
    ) -> Result<()> {
        let mut redis = redis.connect().await.wrap_err(
            "connecting to Redis to clear personal access token cache",
        )?;

        if clear_pats.is_empty() {
            return Ok(());
        }

        let keys = clear_pats
            .into_iter()
            .flat_map(|(id, token, user_id)| {
                [
                    id.map(|id| redis.key().entity(PATS_NAMESPACE, id.0)),
                    token.map(|token| {
                        redis.key().entity(PATS_TOKENS_NAMESPACE, token)
                    }),
                    user_id.map(|user_id| {
                        redis.key().entity(PATS_USERS_NAMESPACE, user_id.0)
                    }),
                ]
                .into_iter()
                .flatten()
            })
            .collect::<Vec<_>>();
        redis
            .delete_many(&keys)
            .await
            .wrap_err("clearing personal access token cache")?;

        Ok(())
    }

    pub async fn remove(
        id: DBPatId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<Option<()>> {
        sqlx::query!(
            "
            DELETE FROM pats WHERE id = $1
            ",
            id as DBPatId,
        )
        .execute(&mut *transaction)
        .await
        .wrap_err("removing personal access token")?;

        Ok(Some(()))
    }
}

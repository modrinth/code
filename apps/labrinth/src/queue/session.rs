use crate::database::models::pat_item::DBPersonalAccessToken;
use crate::database::models::session_item::DBSession;
use crate::database::models::{
    DBOAuthAccessTokenId, DBPatId, DBSessionId, DBUserId, DatabaseError,
};
use crate::database::redis::RedisPool;
use crate::routes::internal::session::SessionMetadata;
use chrono::Utc;
use itertools::Itertools;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use tokio::sync::Mutex;

pub struct AuthQueue {
    session_queue: Mutex<HashMap<DBSessionId, SessionMetadata>>,
    pat_queue: Mutex<HashSet<DBPatId>>,
    oauth_access_token_queue: Mutex<HashSet<DBOAuthAccessTokenId>>,
}

impl Default for AuthQueue {
    fn default() -> Self {
        Self::new()
    }
}

// Batches session accessing transactions every 30 seconds
impl AuthQueue {
    pub fn new() -> Self {
        AuthQueue {
            session_queue: Mutex::new(HashMap::with_capacity(1000)),
            pat_queue: Mutex::new(HashSet::with_capacity(1000)),
            oauth_access_token_queue: Mutex::new(HashSet::with_capacity(1000)),
        }
    }
    pub async fn add_session(
        &self,
        id: DBSessionId,
        metadata: SessionMetadata,
    ) {
        self.session_queue.lock().await.insert(id, metadata);
    }

    pub async fn add_pat(&self, id: DBPatId) {
        self.pat_queue.lock().await.insert(id);
    }

    pub async fn add_oauth_access_token(
        &self,
        id: crate::database::models::DBOAuthAccessTokenId,
    ) {
        self.oauth_access_token_queue.lock().await.insert(id);
    }

    pub async fn take_sessions(&self) -> HashMap<DBSessionId, SessionMetadata> {
        let mut queue = self.session_queue.lock().await;
        let len = queue.len();

        std::mem::replace(&mut queue, HashMap::with_capacity(len))
    }

    pub async fn take_hashset<T>(queue: &Mutex<HashSet<T>>) -> HashSet<T> {
        let mut queue = queue.lock().await;
        let len = queue.len();

        std::mem::replace(&mut queue, HashSet::with_capacity(len))
    }

    pub async fn index(
        &self,
        pool: &PgPool,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let session_queue = self.take_sessions().await;
        let pat_queue = Self::take_hashset(&self.pat_queue).await;
        let oauth_access_token_queue =
            Self::take_hashset(&self.oauth_access_token_queue).await;

        if !session_queue.is_empty()
            || !pat_queue.is_empty()
            || !oauth_access_token_queue.is_empty()
        {
            let mut transaction = pool.begin().await?;
            let mut clear_cache_sessions = Vec::new();

            for (id, metadata) in session_queue {
                clear_cache_sessions.push((Some(id), None, None));

                sqlx::query!(
                    "
                    UPDATE sessions
                    SET last_login = $2, city = $3, country = $4, ip = $5, os = $6, platform = $7, user_agent = $8
                    WHERE (id = $1)
                    ",
                    id as DBSessionId,
                    Utc::now(),
                    metadata.city,
                    metadata.country,
                    metadata.ip,
                    metadata.os,
                    metadata.platform,
                    metadata.user_agent,
                )
                .execute(&mut *transaction)
                .await?;
            }

            use futures::TryStreamExt;
            let expired_ids = sqlx::query!(
                "
                SELECT id, session, user_id
                FROM sessions
                WHERE refresh_expires <= NOW()
                "
            )
            .fetch(&mut *transaction)
            .map_ok(|x| (DBSessionId(x.id), x.session, DBUserId(x.user_id)))
            .try_collect::<Vec<(DBSessionId, String, DBUserId)>>()
            .await?;

            for (id, session, user_id) in expired_ids {
                clear_cache_sessions.push((
                    Some(id),
                    Some(session),
                    Some(user_id),
                ));
                DBSession::remove(id, &mut transaction).await?;
            }

            DBSession::clear_cache(clear_cache_sessions, redis).await?;

            let ids = pat_queue.iter().map(|id| id.0).collect_vec();
            let clear_cache_pats = pat_queue
                .into_iter()
                .map(|id| (Some(id), None, None))
                .collect_vec();
            sqlx::query!(
                "
                UPDATE pats
                SET last_used = $2
                WHERE id IN
                (SELECT * FROM UNNEST($1::bigint[]))
                ",
                &ids[..],
                Utc::now(),
            )
            .execute(&mut *transaction)
            .await?;

            update_oauth_access_token_last_used(
                oauth_access_token_queue,
                &mut transaction,
            )
            .await?;

            transaction.commit().await?;
            DBPersonalAccessToken::clear_cache(clear_cache_pats, redis).await?;
        }

        Ok(())
    }
}

async fn update_oauth_access_token_last_used(
    oauth_access_token_queue: HashSet<DBOAuthAccessTokenId>,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), DatabaseError> {
    let ids = oauth_access_token_queue.iter().map(|id| id.0).collect_vec();
    sqlx::query!(
        "
        UPDATE oauth_access_tokens
        SET last_used = $2
        WHERE id IN
        (SELECT * FROM UNNEST($1::bigint[]))
        ",
        &ids[..],
        Utc::now()
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

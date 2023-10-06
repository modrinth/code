use crate::auth::session::SessionMetadata;
use crate::database::models::pat_item::PersonalAccessToken;
use crate::database::models::session_item::Session;
use crate::database::models::{DatabaseError, PatId, SessionId, UserId};
use crate::database::redis::RedisPool;
use chrono::Utc;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use tokio::sync::Mutex;

pub struct AuthQueue {
    session_queue: Mutex<HashMap<SessionId, SessionMetadata>>,
    pat_queue: Mutex<HashSet<PatId>>,
}

// Batches session accessing transactions every 30 seconds
impl AuthQueue {
    pub fn new() -> Self {
        AuthQueue {
            session_queue: Mutex::new(HashMap::with_capacity(1000)),
            pat_queue: Mutex::new(HashSet::with_capacity(1000)),
        }
    }
    pub async fn add_session(&self, id: SessionId, metadata: SessionMetadata) {
        self.session_queue.lock().await.insert(id, metadata);
    }

    pub async fn add_pat(&self, id: PatId) {
        self.pat_queue.lock().await.insert(id);
    }

    pub async fn take_sessions(&self) -> HashMap<SessionId, SessionMetadata> {
        let mut queue = self.session_queue.lock().await;
        let len = queue.len();

        std::mem::replace(&mut queue, HashMap::with_capacity(len))
    }

    pub async fn take_pats(&self) -> HashSet<PatId> {
        let mut queue = self.pat_queue.lock().await;
        let len = queue.len();

        std::mem::replace(&mut queue, HashSet::with_capacity(len))
    }

    pub async fn index(&self, pool: &PgPool, redis: &RedisPool) -> Result<(), DatabaseError> {
        let session_queue = self.take_sessions().await;
        let pat_queue = self.take_pats().await;

        if !session_queue.is_empty() || !pat_queue.is_empty() {
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
                    id as SessionId,
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
            .fetch_many(&mut *transaction)
            .try_filter_map(|e| async {
                Ok(e.right()
                    .map(|x| (SessionId(x.id), x.session, UserId(x.user_id))))
            })
            .try_collect::<Vec<(SessionId, String, UserId)>>()
            .await?;

            for (id, session, user_id) in expired_ids {
                clear_cache_sessions.push((Some(id), Some(session), Some(user_id)));
                Session::remove(id, &mut transaction).await?;
            }

            Session::clear_cache(clear_cache_sessions, redis).await?;

            let mut clear_cache_pats = Vec::new();

            for id in pat_queue {
                clear_cache_pats.push((Some(id), None, None));

                sqlx::query!(
                    "
                    UPDATE pats
                    SET last_used = $2
                    WHERE (id = $1)
                    ",
                    id as PatId,
                    Utc::now(),
                )
                .execute(&mut *transaction)
                .await?;
            }

            PersonalAccessToken::clear_cache(clear_cache_pats, redis).await?;

            transaction.commit().await?;
        }

        Ok(())
    }
}

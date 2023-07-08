use crate::auth::session::SessionMetadata;
use crate::database::models::session_item::Session;
use crate::database::models::{DatabaseError, SessionId, UserId};
use chrono::Utc;
use sqlx::PgPool;
use tokio::sync::Mutex;

pub struct SessionQueue {
    queue: Mutex<Vec<(SessionId, SessionMetadata)>>,
}

// Batches session accessing transactions every 30 seconds
impl SessionQueue {
    pub fn new() -> Self {
        SessionQueue {
            queue: Mutex::new(Vec::with_capacity(1000)),
        }
    }
    pub async fn add(&self, id: SessionId, metadata: SessionMetadata) {
        self.queue.lock().await.push((id, metadata));
    }

    pub async fn take(&self) -> Vec<(SessionId, SessionMetadata)> {
        let mut queue = self.queue.lock().await;
        let len = queue.len();

        std::mem::replace(&mut queue, Vec::with_capacity(len))
    }

    pub async fn index(
        &self,
        pool: &PgPool,
        redis: &deadpool_redis::Pool,
    ) -> Result<(), DatabaseError> {
        let queue = self.take().await;

        if !queue.is_empty() {
            let mut transaction = pool.begin().await?;
            let mut clear_cache_sessions = Vec::new();

            for (id, metadata) in queue {
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
                WHERE refresh_expires >= NOW()
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

            transaction.commit().await?;
        }

        Ok(())
    }
}

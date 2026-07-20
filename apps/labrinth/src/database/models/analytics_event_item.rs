use chrono::{DateTime, Utc};
use futures::{StreamExt, TryStreamExt};
use sqlx::types::Json;

use crate::{
    database::{
        models::{DBAnalyticsEventId, DatabaseError},
        redis::RedisPool,
    },
    models::v3::analytics_event::AnalyticsEventMeta,
};
use serde::{Deserialize, Serialize};

const ANALYTICS_EVENTS_NAMESPACE: &str = "analytics_events:v1";
const ANALYTICS_EVENTS_ALL_KEY: &str = "all";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBAnalyticsEvent {
    pub id: DBAnalyticsEventId,
    pub meta: AnalyticsEventMeta,
    pub starts: DateTime<Utc>,
    pub ends: DateTime<Utc>,
}

impl DBAnalyticsEvent {
    pub async fn insert(
        &self,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
			INSERT INTO analytics_events (id, meta, starts, ends)
			VALUES ($1, $2, $3, $4)
			",
            self.id as DBAnalyticsEventId,
            sqlx::types::Json(&self.meta) as Json<&AnalyticsEventMeta>,
            self.starts,
            self.ends,
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn update(
        &self,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<bool, DatabaseError> {
        let result = sqlx::query!(
            "
			UPDATE analytics_events
			SET meta = $2, starts = $3, ends = $4
			WHERE id = $1
			",
            self.id as DBAnalyticsEventId,
            sqlx::types::Json(&self.meta) as Json<&AnalyticsEventMeta>,
            self.starts,
            self.ends,
        )
        .execute(exec)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn remove(
        id: DBAnalyticsEventId,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<bool, DatabaseError> {
        let result = sqlx::query!(
            "
			DELETE FROM analytics_events
			WHERE id = $1
			",
            id as DBAnalyticsEventId,
        )
        .execute(exec)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_all(
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Vec<DBAnalyticsEvent>, DatabaseError> {
        let mut redis = redis.connect().await?;

        if let Some(events) = redis
            .get_deserialized(
                ANALYTICS_EVENTS_NAMESPACE,
                ANALYTICS_EVENTS_ALL_KEY,
            )
            .await?
        {
            return Ok(events);
        }

        let events = sqlx::query!(
            r#"
			SELECT id, meta AS "meta: Json<AnalyticsEventMeta>", starts, ends
			FROM analytics_events
			ORDER BY starts DESC
			"#
        )
        .fetch(exec)
        .map(|record| {
            let record = record?;

            Ok::<_, DatabaseError>(DBAnalyticsEvent {
                id: DBAnalyticsEventId(record.id),
                meta: record.meta.0,
                starts: record.starts,
                ends: record.ends,
            })
        })
        .try_collect::<Vec<_>>()
        .await?;

        redis
            .set_serialized(
                ANALYTICS_EVENTS_NAMESPACE,
                ANALYTICS_EVENTS_ALL_KEY,
                &events,
                None,
            )
            .await?;

        Ok(events)
    }

    pub async fn clear_cache(redis: &RedisPool) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;
        redis
            .delete(ANALYTICS_EVENTS_NAMESPACE, ANALYTICS_EVENTS_ALL_KEY)
            .await?;
        Ok(())
    }
}

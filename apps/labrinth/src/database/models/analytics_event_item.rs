use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use futures::{StreamExt, TryStreamExt};
use sqlx::types::Json;
use xredis::RedisPool;

use crate::{
    database::models::DBAnalyticsEventId,
    models::v3::analytics_event::AnalyticsEventMeta,
};
use serde::{Deserialize, Serialize};

const ANALYTICS_EVENTS_NAMESPACE: &str = "analytics_events:v4";
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
    ) -> Result<()> {
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
        .await
        .wrap_err("inserting analytics event")?;

        Ok(())
    }

    pub async fn update(
        &self,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<bool> {
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
        .await
        .wrap_err("updating analytics event")?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn remove(
        id: DBAnalyticsEventId,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<bool> {
        let result = sqlx::query!(
            "
			DELETE FROM analytics_events
			WHERE id = $1
			",
            id as DBAnalyticsEventId,
        )
        .execute(exec)
        .await
        .wrap_err("removing analytics event")?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_all(
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Vec<DBAnalyticsEvent>> {
        let mut redis = redis
            .connect()
            .await
            .wrap_err("connecting to redis for analytics events")?;
        let key = redis
            .key()
            .metadata(ANALYTICS_EVENTS_NAMESPACE, ANALYTICS_EVENTS_ALL_KEY);

        if let Some(events) = redis
            .get_deserialized(&key)
            .await
            .wrap_err("getting cached analytics events")?
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
            let record = record.wrap_err("reading analytics event record")?;

            eyre::Ok(DBAnalyticsEvent {
                id: DBAnalyticsEventId(record.id),
                meta: record.meta.0,
                starts: record.starts,
                ends: record.ends,
            })
        })
        .try_collect::<Vec<_>>()
        .await
        .wrap_err("fetching analytics events from database")?;

        redis
            .set_serialized(&key, &events, None)
            .await
            .wrap_err("caching analytics events")?;

        Ok(events)
    }

    pub async fn clear_cache(redis: &RedisPool) -> Result<()> {
        let mut redis = redis
            .connect()
            .await
            .wrap_err("connecting to redis to clear analytics event cache")?;
        let key = redis
            .key()
            .metadata(ANALYTICS_EVENTS_NAMESPACE, ANALYTICS_EVENTS_ALL_KEY);
        redis
            .delete(&key)
            .await
            .wrap_err("clearing analytics event cache")?;
        Ok(())
    }
}

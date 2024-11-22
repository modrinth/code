use crate::database::models::{DatabaseError, UserId};
use crate::database::redis::RedisPool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const USERS_STATUSES_NAMESPACE: &str = "users_statuses";
const STATUS_EXPIRY: i64 = 60 * 60 * 24; // 24 hours

#[derive(Serialize, Deserialize)]
pub struct UserStatusItem {
    pub id: UserId,
    pub profile_name: Option<String>,
    pub last_update: DateTime<Utc>,
}

impl UserStatusItem {
    pub async fn set(&self, redis: &RedisPool) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;

        redis
            .set_serialized_to_json(
                USERS_STATUSES_NAMESPACE,
                &self.id.0.to_string(),
                &self,
                Some(STATUS_EXPIRY),
            )
            .await?;
        Ok(())
    }

    pub async fn get(
        id: UserId,
        redis: &RedisPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let mut redis = redis.connect().await?;

        let res = redis
            .get_deserialized_from_json::<Self>(
                USERS_STATUSES_NAMESPACE,
                &id.0.to_string(),
            )
            .await?;

        Ok(res)
    }

    pub async fn get_many(
        ids: &[UserId],
        redis: &RedisPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        let mut redis = redis.connect().await?;

        let res = redis
            .get_many_deserialized_from_json::<Self>(
                USERS_STATUSES_NAMESPACE,
                &ids.iter().map(|x| x.0.to_string()).collect::<Vec<_>>(),
            )
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(res)
    }

    pub async fn remove(
        id: UserId,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;

        redis
            .delete(USERS_STATUSES_NAMESPACE, id.0.to_string())
            .await?;
        Ok(())
    }
}

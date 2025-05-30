use crate::database::models::{
    DBFileId, DBSharedInstanceId, DBSharedInstanceVersionId, DBUserId,
};
use crate::database::redis::RedisPool;
use dashmap::DashMap;
use futures_util::TryStreamExt;

//region shared_instances
pub struct DBSharedInstance {
    pub id: DBSharedInstanceId,
    pub title: String,
    pub owner_id: DBUserId,
    pub current_version_id: Option<DBSharedInstanceVersionId>,
}

struct SharedInstanceQueryResult {
    id: i64,
    title: String,
    owner_id: i64,
    current_version_id: Option<i64>,
}

impl From<SharedInstanceQueryResult> for DBSharedInstance {
    fn from(val: SharedInstanceQueryResult) -> Self {
        DBSharedInstance {
            id: DBSharedInstanceId(val.id),
            title: val.title,
            owner_id: DBUserId(val.owner_id),
            current_version_id: val
                .current_version_id
                .map(DBSharedInstanceVersionId),
        }
    }
}

impl DBSharedInstance {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO shared_instances (id, title, owner_id, current_version_id)
            VALUES ($1, $2, $3, $4)
            ",
            self.id as DBSharedInstanceId,
            self.title,
            self.owner_id as DBUserId,
            self.current_version_id.map(|x| x.0),
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn get(
        id: DBSharedInstanceId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<Self>, sqlx::Error> {
        let result = sqlx::query_as!(
            SharedInstanceQueryResult,
            "
            SELECT id, title, owner_id, current_version_id
            FROM shared_instances
            WHERE id = $1
            ",
            id.0,
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(Into::into))
    }
}
//endregion

//region shared_instance_users
const USERS_NAMESPACE: &str = "shared_instance_users";

pub struct DBSharedInstanceUser {
    pub user_id: DBUserId,
    pub shared_instance_id: DBSharedInstanceId,
}

impl DBSharedInstanceUser {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO shared_instance_users (user_id, shared_instance_id)
            VALUES ($1, $2)
            ",
            self.user_id as DBUserId,
            self.shared_instance_id as DBSharedInstanceId,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn get_from_instance(
        instance_id: DBSharedInstanceId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Vec<DBUserId>, super::DatabaseError> {
        Self::get_from_instance_many(&[instance_id], exec, redis).await
    }

    pub async fn get_from_instance_many(
        instance_ids: &[DBSharedInstanceId],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Vec<DBUserId>, super::DatabaseError> {
        if instance_ids.is_empty() {
            return Ok(vec![]);
        }

        let users = redis
            .get_cached_keys(
                USERS_NAMESPACE,
                &instance_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
                async |user_ids| {
                    let users = sqlx::query!(
                        "
                        SELECT shared_instance_id, user_id
                        FROM shared_instance_users
                        WHERE shared_instance_id = ANY($1)
                        ",
                        &user_ids
                    )
                    .fetch(exec)
                    .try_fold(
                        DashMap::new(),
                        |acc: DashMap<_, Vec<DBUserId>>, m| {
                            acc.entry(m.shared_instance_id)
                                .or_default()
                                .push(DBUserId(m.user_id));

                            async move { Ok(acc) }
                        },
                    )
                    .await?;

                    Ok(users)
                },
            )
            .await?;

        Ok(users.into_iter().flatten().collect())
    }

    pub async fn clear_cache(
        instance_id: DBSharedInstanceId,
        redis: &RedisPool,
    ) -> Result<(), super::DatabaseError> {
        let mut redis = redis.connect().await?;
        redis.delete(USERS_NAMESPACE, instance_id.0).await?;
        Ok(())
    }
}
//endregion

//region shared_instance_versions
pub struct DBSharedInstanceVersion {
    pub id: DBSharedInstanceVersionId,
    pub file_id: DBFileId,
    pub shared_instance_id: DBSharedInstanceId,
}

struct SharedInstanceVersionQueryResult {
    id: i64,
    file_id: i64,
    shared_instance_id: i64,
}

impl From<SharedInstanceVersionQueryResult> for DBSharedInstanceVersion {
    fn from(val: SharedInstanceVersionQueryResult) -> Self {
        DBSharedInstanceVersion {
            id: DBSharedInstanceVersionId(val.id),
            file_id: DBFileId(val.file_id),
            shared_instance_id: DBSharedInstanceId(val.shared_instance_id),
        }
    }
}

impl DBSharedInstanceVersion {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO shared_instance_versions (id, file_id, shared_instance_id)
            VALUES ($1, $2, $3)
            ",
            self.id as DBSharedInstanceVersionId,
            self.file_id as DBFileId,
            self.shared_instance_id as DBSharedInstanceId,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn get(
        id: DBSharedInstanceVersionId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<Self>, sqlx::Error> {
        let result = sqlx::query_as!(
            SharedInstanceVersionQueryResult,
            "
            SELECT id, file_id, shared_instance_id
            FROM shared_instance_versions
            WHERE id = $1
            ",
            id.0,
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(Into::into))
    }
}
//endregion

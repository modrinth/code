use crate::database::models::{
    DBSharedInstanceId, DBSharedInstanceVersionId, DBUserId,
};
use crate::database::redis::RedisPool;
use crate::models::shared_instances::SharedInstanceUserPermissions;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

//region shared_instances
pub struct DBSharedInstance {
    pub id: DBSharedInstanceId,
    pub title: String,
    pub owner_id: DBUserId,
    pub public: bool,
    pub current_version_id: Option<DBSharedInstanceVersionId>,
}

struct SharedInstanceQueryResult {
    id: i64,
    title: String,
    owner_id: i64,
    public: bool,
    current_version_id: Option<i64>,
}

impl From<SharedInstanceQueryResult> for DBSharedInstance {
    fn from(val: SharedInstanceQueryResult) -> Self {
        DBSharedInstance {
            id: DBSharedInstanceId(val.id),
            title: val.title,
            owner_id: DBUserId(val.owner_id),
            public: val.public,
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
            SELECT id, title, owner_id, public, current_version_id
            FROM shared_instances
            WHERE id = $1
            ",
            id.0,
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(Into::into))
    }

    pub async fn list_for_user(
        user: DBUserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let results = sqlx::query_as!(
            SharedInstanceQueryResult,
            r#"
            -- See https://github.com/launchbadge/sqlx/issues/1266 for why we need all the "as"
            SELECT
                id as "id!",
                title as "title!",
                public as "public!",
                owner_id as "owner_id!",
                current_version_id
            FROM shared_instances
            WHERE owner_id = $1
            UNION
            SELECT
                id as "id!",
                title as "title!",
                public as "public!",
                owner_id as "owner_id!",
                current_version_id
            FROM shared_instances
            JOIN shared_instance_users ON id = shared_instance_id
            WHERE user_id = $1
            "#,
            user.0,
        )
        .fetch_all(exec)
        .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}
//endregion

//region shared_instance_users
const USERS_NAMESPACE: &str = "shared_instance_users";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DBSharedInstanceUser {
    pub user_id: DBUserId,
    pub shared_instance_id: DBSharedInstanceId,
    pub permissions: SharedInstanceUserPermissions,
}

impl DBSharedInstanceUser {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO shared_instance_users (user_id, shared_instance_id, permissions)
            VALUES ($1, $2, $3)
            ",
            self.user_id as DBUserId,
            self.shared_instance_id as DBSharedInstanceId,
            self.permissions.bits() as i64,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn get_user_permissions(
        instance_id: DBSharedInstanceId,
        user_id: DBUserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<SharedInstanceUserPermissions>, super::DatabaseError>
    {
        let permissions = sqlx::query!(
            "
            SELECT permissions
            FROM shared_instance_users
            WHERE shared_instance_id = $1 AND user_id = $2
            ",
            instance_id as DBSharedInstanceId,
            user_id as DBUserId,
        )
        .fetch_optional(exec)
        .await?
        .map(|x| {
            SharedInstanceUserPermissions::from_bits(x.permissions as u64)
                .unwrap_or(SharedInstanceUserPermissions::empty())
        });

        Ok(permissions)
    }

    pub async fn get_from_instance(
        instance_id: DBSharedInstanceId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Vec<DBSharedInstanceUser>, super::DatabaseError> {
        Self::get_from_instance_many(&[instance_id], exec, redis).await
    }

    pub async fn get_from_instance_many(
        instance_ids: &[DBSharedInstanceId],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Vec<DBSharedInstanceUser>, super::DatabaseError> {
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
                        SELECT shared_instance_id, user_id, permissions
                        FROM shared_instance_users
                        WHERE shared_instance_id = ANY($1)
                        ",
                        &user_ids
                    )
                    .fetch(exec)
                    .try_fold(DashMap::new(), |acc: DashMap<_, Vec<_>>, m| {
                        acc.entry(m.shared_instance_id).or_default().push(
                            DBSharedInstanceUser {
                                user_id: DBUserId(m.user_id),
                                shared_instance_id: DBSharedInstanceId(
                                    m.shared_instance_id,
                                ),
                                permissions:
                                    SharedInstanceUserPermissions::from_bits(
                                        m.permissions as u64,
                                    )
                                    .unwrap_or(
                                        SharedInstanceUserPermissions::empty(),
                                    ),
                            },
                        );

                        async move { Ok(acc) }
                    })
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
    pub shared_instance_id: DBSharedInstanceId,
    pub size: u64,
    pub sha512: Vec<u8>,
    pub created: DateTime<Utc>,
}

struct SharedInstanceVersionQueryResult {
    id: i64,
    shared_instance_id: i64,
    size: i64,
    sha512: Vec<u8>,
    created: DateTime<Utc>,
}

impl From<SharedInstanceVersionQueryResult> for DBSharedInstanceVersion {
    fn from(val: SharedInstanceVersionQueryResult) -> Self {
        DBSharedInstanceVersion {
            id: DBSharedInstanceVersionId(val.id),
            shared_instance_id: DBSharedInstanceId(val.shared_instance_id),
            size: val.size as u64,
            sha512: val.sha512,
            created: val.created,
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
            INSERT INTO shared_instance_versions (id, shared_instance_id, size, sha512, created)
            VALUES ($1, $2, $3, $4, $5)
            ",
            self.id as DBSharedInstanceVersionId,
            self.shared_instance_id as DBSharedInstanceId,
            self.size as i64,
            self.sha512,
            self.created,
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
            SELECT id, shared_instance_id, size, sha512, created
            FROM shared_instance_versions
            WHERE id = $1
            ",
            id as DBSharedInstanceVersionId,
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(Into::into))
    }

    pub async fn get_for_instance(
        instance_id: DBSharedInstanceId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let results = sqlx::query_as!(
            SharedInstanceVersionQueryResult,
            "
            SELECT id, shared_instance_id, size, sha512, created
            FROM shared_instance_versions
            WHERE shared_instance_id = $1
            ORDER BY created DESC
            ",
            instance_id as DBSharedInstanceId,
        )
        .fetch_all(exec)
        .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}
//endregion

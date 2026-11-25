use super::ids::*;
use crate::database::{PgTransaction, models};
use crate::models::collections::{
    CollectionFlags, CollectionStatus, is_public_featured,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

const COLLECTIONS_NAMESPACE: &str = "collections:v7";

#[derive(Clone)]
pub struct CollectionBuilder {
    pub collection_id: DBCollectionId,
    pub user_id: DBUserId,
    pub name: String,
    pub description: Option<String>,
    pub status: CollectionStatus,
    pub flags: CollectionFlags,
    pub projects: Vec<DBProjectId>,
}

impl CollectionBuilder {
    pub async fn insert(
        self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<DBCollectionId> {
        let collection_struct = DBCollection {
            id: self.collection_id,
            name: self.name,
            user_id: self.user_id,
            description: self.description,
            created: Utc::now(),
            updated: Utc::now(),
            icon_url: None,
            raw_icon_url: None,
            color: None,
            status: self.status,
            flags: self.flags,
            follows: 0,
            projects: self.projects,
        };
        collection_struct
            .insert(transaction)
            .await
            .context("inserting built collection")?;

        Ok(self.collection_id)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DBCollection {
    pub id: DBCollectionId,
    pub user_id: DBUserId,
    pub name: String,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub icon_url: Option<String>,
    pub raw_icon_url: Option<String>,
    pub color: Option<u32>,
    pub status: CollectionStatus,
    #[serde(default)]
    pub flags: CollectionFlags,
    pub follows: i32,
    pub projects: Vec<DBProjectId>,
}

impl DBCollection {
    pub async fn insert(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<()> {
        sqlx::query!(
            "
            INSERT INTO collections (
                id, user_id, name, description,
                created, icon_url, raw_icon_url, status, flags
            )
            VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8, $9
            )
            ",
            self.id as DBCollectionId,
            self.user_id as DBUserId,
            &self.name,
            self.description.as_ref(),
            self.created,
            self.icon_url.as_ref(),
            self.raw_icon_url.as_ref(),
            self.status.to_string(),
            self.flags.to_db(),
        )
        .execute(&mut *transaction)
        .await
        .context("inserting collection")?;

        let (collection_ids, project_ids): (Vec<_>, Vec<_>) =
            self.projects.iter().map(|p| (self.id.0, p.0)).unzip();
        sqlx::query!(
            "
                INSERT INTO collections_mods (collection_id, mod_id)
                SELECT * FROM UNNEST($1::bigint[], $2::bigint[])
                ON CONFLICT DO NOTHING
            ",
            &collection_ids[..],
            &project_ids[..],
        )
        .execute(&mut *transaction)
        .await
        .context("inserting collection projects")?;

        Ok(())
    }

    pub async fn remove(
        id: DBCollectionId,
        transaction: &mut PgTransaction<'_>,
        redis: &RedisPool,
    ) -> Result<Option<Vec<DBProjectId>>> {
        let collection = Self::get(id, &mut *transaction, redis)
            .await
            .context("fetching collection to remove")?;

        if let Some(collection) = collection {
            let locked = sqlx::query_scalar!(
                "
                SELECT id FROM collections
                WHERE id = $1
                FOR UPDATE
                ",
                id as DBCollectionId,
            )
            .fetch_optional(&mut *transaction)
            .await
            .context("locking collection to remove")?;
            if locked.is_none() {
                return Ok(None);
            }

            let refresh_projects =
                if is_public_featured(collection.status, collection.flags) {
                    collection.projects.clone()
                } else {
                    Vec::new()
                };

            sqlx::query!(
                "
                DELETE FROM collections_mods
                WHERE collection_id = $1
                ",
                id as DBCollectionId,
            )
            .execute(&mut *transaction)
            .await
            .context("deleting collection projects")?;

            sqlx::query!(
                "
                DELETE FROM collection_follows
                WHERE collection_id = $1
                ",
                id as DBCollectionId,
            )
            .execute(&mut *transaction)
            .await
            .context("deleting collection follows")?;

            sqlx::query!(
                "
                DELETE FROM collections
                WHERE id = $1
                ",
                id as DBCollectionId,
            )
            .execute(&mut *transaction)
            .await
            .context("deleting collection")?;

            models::DBCollection::clear_cache(collection.id, redis)
                .await
                .context("clearing removed collection cache")?;

            Ok(Some(refresh_projects))
        } else {
            Ok(None)
        }
    }

    pub async fn get<'a, 'b, E>(
        id: DBCollectionId,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<DBCollection>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        DBCollection::get_many(&[id], executor, redis)
            .await
            .context("fetching collection")
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(
        collection_ids: &[DBCollectionId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBCollection>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let val = redis
            .get_cached_keys(
                COLLECTIONS_NAMESPACE,
                &collection_ids.iter().map(|x| x.0).collect::<Vec<_>>(),
                |collection_ids| async move {
                    sqlx::query!(
                        "
                    SELECT c.id id, c.name name, c.description description,
                    c.icon_url icon_url, c.raw_icon_url raw_icon_url, c.color color, c.created created, c.user_id user_id,
                    c.updated updated, c.status status, c.flags flags, c.follows follows,
                    ARRAY_AGG(DISTINCT cm.mod_id) filter (where cm.mod_id is not null) mods
                    FROM collections c
                    LEFT JOIN collections_mods cm ON cm.collection_id = c.id
                    WHERE c.id = ANY($1)
                    GROUP BY c.id;
                    ",
                        &collection_ids,
                    )
                    .fetch(exec)
                    .try_fold(DashMap::new(), |acc, m| {
                        let collection = DBCollection {
                            id: DBCollectionId(m.id),
                            user_id: DBUserId(m.user_id),
                            name: m.name.clone(),
                            description: m.description.clone(),
                            icon_url: m.icon_url.clone(),
                            raw_icon_url: m.raw_icon_url.clone(),
                            color: m.color.map(|x| x as u32),
                            created: m.created,
                            updated: m.updated,
                            status: CollectionStatus::from_string(&m.status),
                            flags: CollectionFlags::from_db(m.flags),
                            follows: m.follows,
                            projects: m
                                .mods
                                .unwrap_or_default()
                                .into_iter()
                                .map(DBProjectId)
                                .collect(),
                        };

                        acc.insert(m.id, collection);
                        async move { Ok::<_, sqlx::Error>(acc) }
                    })
                    .await
                },
            )
            .await
            .context("fetching cached collections")?;

        Ok(val)
    }

    pub async fn clear_cache(
        id: DBCollectionId,
        redis: &RedisPool,
    ) -> Result<()> {
        let mut redis = redis
            .connect()
            .await
            .context("connecting to redis to clear collection cache")?;
        let key = redis.key().entity(COLLECTIONS_NAMESPACE, id.0);

        redis
            .delete(&key)
            .await
            .context("clearing collection cache")?;
        Ok(())
    }

    pub async fn clear_many_caches(
        ids: &[DBCollectionId],
        redis: &RedisPool,
    ) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let mut redis = redis
            .connect()
            .await
            .context("connecting to redis to clear collection caches")?;
        let keys = ids
            .iter()
            .map(|id| redis.key().entity(COLLECTIONS_NAMESPACE, id.0))
            .collect::<Vec<_>>();
        redis
            .delete_many(&keys)
            .await
            .context("clearing collection caches")?;
        Ok(())
    }
}

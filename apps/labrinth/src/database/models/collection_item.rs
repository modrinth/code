use super::ids::*;
use crate::database::models;
use crate::database::models::DatabaseError;
use crate::database::redis::RedisPool;
use crate::models::collections::CollectionStatus;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};

const COLLECTIONS_NAMESPACE: &str = "collections";

#[derive(Clone)]
pub struct CollectionBuilder {
    pub collection_id: DBCollectionId,
    pub user_id: DBUserId,
    pub name: String,
    pub description: Option<String>,
    pub status: CollectionStatus,
    pub projects: Vec<DBProjectId>,
}

impl CollectionBuilder {
    pub async fn insert(
        self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<DBCollectionId, DatabaseError> {
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
            projects: self.projects,
        };
        collection_struct.insert(transaction).await?;

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
    pub projects: Vec<DBProjectId>,
}

impl DBCollection {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO collections (
                id, user_id, name, description,
                created, icon_url, raw_icon_url, status
            )
            VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8
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
        )
        .execute(&mut **transaction)
        .await?;

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
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn remove(
        id: DBCollectionId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        let collection = Self::get(id, &mut **transaction, redis).await?;

        if let Some(collection) = collection {
            sqlx::query!(
                "
                DELETE FROM collections_mods
                WHERE collection_id = $1
                ",
                id as DBCollectionId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM collections
                WHERE id = $1
                ",
                id as DBCollectionId,
            )
            .execute(&mut **transaction)
            .await?;

            models::DBCollection::clear_cache(collection.id, redis).await?;

            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    pub async fn get<'a, 'b, E>(
        id: DBCollectionId,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<DBCollection>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        DBCollection::get_many(&[id], executor, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(
        collection_ids: &[DBCollectionId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBCollection>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let val = redis
            .get_cached_keys(
                COLLECTIONS_NAMESPACE,
                &collection_ids.iter().map(|x| x.0).collect::<Vec<_>>(),
                |collection_ids| async move {
                    let collections = sqlx::query!(
                        "
                    SELECT c.id id, c.name name, c.description description,
                    c.icon_url icon_url, c.raw_icon_url raw_icon_url, c.color color, c.created created, c.user_id user_id,
                    c.updated updated, c.status status,
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
                            projects: m
                                .mods
                                .unwrap_or_default()
                                .into_iter()
                                .map(DBProjectId)
                                .collect(),
                        };

                        acc.insert(m.id, collection);
                        async move { Ok(acc) }
                    })
                    .await?;

                    Ok(collections)
                },
            )
            .await?;

        Ok(val)
    }

    pub async fn clear_cache(
        id: DBCollectionId,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;

        redis.delete(COLLECTIONS_NAMESPACE, id.0).await?;
        Ok(())
    }
}

use super::ids::*;
use crate::database::redis::RedisPool;
use crate::{database::models::DatabaseError, models::images::ImageContext};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

const IMAGES_NAMESPACE: &str = "images";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DBImage {
    pub id: DBImageId,
    pub url: String,
    pub raw_url: String,
    pub size: u64,
    pub created: DateTime<Utc>,
    pub owner_id: DBUserId,

    // context it is associated with
    pub context: String,

    pub project_id: Option<DBProjectId>,
    pub version_id: Option<DBVersionId>,
    pub thread_message_id: Option<DBThreadMessageId>,
    pub report_id: Option<DBReportId>,
}

impl DBImage {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO uploaded_images (
                id, url, raw_url, size, created, owner_id, context, mod_id, version_id, thread_message_id, report_id
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11
            );
            ",
            self.id as DBImageId,
            self.url,
            self.raw_url,
            self.size as i64,
            self.created,
            self.owner_id as DBUserId,
            self.context,
            self.project_id.map(|x| x.0),
            self.version_id.map(|x| x.0),
            self.thread_message_id.map(|x| x.0),
            self.report_id.map(|x| x.0),
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn remove(
        id: DBImageId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        let image = Self::get(id, &mut **transaction, redis).await?;

        if let Some(image) = image {
            sqlx::query!(
                "
                DELETE FROM uploaded_images
                WHERE id = $1
                ",
                id as DBImageId,
            )
            .execute(&mut **transaction)
            .await?;

            DBImage::clear_cache(image.id, redis).await?;

            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many_contexted(
        context: ImageContext,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Vec<DBImage>, sqlx::Error> {
        // Set all of project_id, version_id, thread_message_id, report_id to None
        // Then set the one that is relevant to Some

        let mut project_id = None;
        let mut version_id = None;
        let mut thread_message_id = None;
        let mut report_id = None;
        match context {
            ImageContext::Project {
                project_id: Some(id),
            } => {
                project_id = Some(DBProjectId::from(id));
            }
            ImageContext::Version {
                version_id: Some(id),
            } => {
                version_id = Some(DBVersionId::from(id));
            }
            ImageContext::ThreadMessage {
                thread_message_id: Some(id),
            } => {
                thread_message_id = Some(DBThreadMessageId::from(id));
            }
            ImageContext::Report {
                report_id: Some(id),
            } => {
                report_id = Some(DBReportId::from(id));
            }
            _ => {}
        }

        use futures::stream::TryStreamExt;
        sqlx::query!(
            "
            SELECT id, url, raw_url, size, created, owner_id, context, mod_id, version_id, thread_message_id, report_id
            FROM uploaded_images
            WHERE context = $1
            AND (mod_id = $2 OR ($2 IS NULL AND mod_id IS NULL))
            AND (version_id = $3 OR ($3 IS NULL AND version_id IS NULL))
            AND (thread_message_id = $4 OR ($4 IS NULL AND thread_message_id IS NULL))
            AND (report_id = $5 OR ($5 IS NULL AND report_id IS NULL))
            GROUP BY id
            ",
            context.context_as_str(),
            project_id.map(|x| x.0),
            version_id.map(|x| x.0),
            thread_message_id.map(|x| x.0),
            report_id.map(|x| x.0),

        )
        .fetch(&mut **transaction)
        .map_ok(|row| {
            let id = DBImageId(row.id);

            DBImage {
                id,
                url: row.url,
                raw_url: row.raw_url,
                size: row.size as u64,
                created: row.created,
                owner_id: DBUserId(row.owner_id),
                context: row.context,
                project_id: row.mod_id.map(DBProjectId),
                version_id: row.version_id.map(DBVersionId),
                thread_message_id: row.thread_message_id.map(DBThreadMessageId),
                report_id: row.report_id.map(DBReportId),
            }
        })
        .try_collect::<Vec<DBImage>>()
        .await
    }

    pub async fn get<'a, 'b, E>(
        id: DBImageId,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<DBImage>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        DBImage::get_many(&[id], executor, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(
        image_ids: &[DBImageId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBImage>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::TryStreamExt;

        let val = redis.get_cached_keys(
            IMAGES_NAMESPACE,
            &image_ids.iter().map(|x| x.0).collect::<Vec<_>>(),
            |image_ids| async move {
                let images = sqlx::query!(
                    "
                    SELECT id, url, raw_url, size, created, owner_id, context, mod_id, version_id, thread_message_id, report_id
                    FROM uploaded_images
                    WHERE id = ANY($1)
                    GROUP BY id;
                    ",
                    &image_ids,
                )
                    .fetch(exec)
                    .try_fold(DashMap::new(), |acc, i| {
                        let img = DBImage {
                            id: DBImageId(i.id),
                            url: i.url,
                            raw_url: i.raw_url,
                            size: i.size as u64,
                            created: i.created,
                            owner_id: DBUserId(i.owner_id),
                            context: i.context,
                            project_id: i.mod_id.map(DBProjectId),
                            version_id: i.version_id.map(DBVersionId),
                            thread_message_id: i.thread_message_id.map(DBThreadMessageId),
                            report_id: i.report_id.map(DBReportId),
                        };

                        acc.insert(i.id, img);
                        async move { Ok(acc) }
                    })
                    .await?;

                Ok(images)
            },
        ).await?;

        Ok(val)
    }

    pub async fn clear_cache(
        id: DBImageId,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;

        redis.delete(IMAGES_NAMESPACE, id.0).await?;
        Ok(())
    }
}

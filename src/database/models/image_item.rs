use super::ids::*;
use crate::database::redis::RedisPool;
use crate::{database::models::DatabaseError, models::images::ImageContext};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const IMAGES_NAMESPACE: &str = "images";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Image {
    pub id: ImageId,
    pub url: String,
    pub size: u64,
    pub created: DateTime<Utc>,
    pub owner_id: UserId,

    // context it is associated with
    pub context: String,

    pub project_id: Option<ProjectId>,
    pub version_id: Option<VersionId>,
    pub thread_message_id: Option<ThreadMessageId>,
    pub report_id: Option<ReportId>,
}

impl Image {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO uploaded_images (
                id, url, size, created, owner_id, context, mod_id, version_id, thread_message_id, report_id
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
            );
            ",
            self.id as ImageId,
            self.url,
            self.size as i64,
            self.created,
            self.owner_id as UserId,
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
        id: ImageId,
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
                id as ImageId,
            )
            .execute(&mut **transaction)
            .await?;

            Image::clear_cache(image.id, redis).await?;

            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many_contexted(
        context: ImageContext,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Vec<Image>, sqlx::Error> {
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
                project_id = Some(ProjectId::from(id));
            }
            ImageContext::Version {
                version_id: Some(id),
            } => {
                version_id = Some(VersionId::from(id));
            }
            ImageContext::ThreadMessage {
                thread_message_id: Some(id),
            } => {
                thread_message_id = Some(ThreadMessageId::from(id));
            }
            ImageContext::Report {
                report_id: Some(id),
            } => {
                report_id = Some(ReportId::from(id));
            }
            _ => {}
        }

        use futures::stream::TryStreamExt;
        sqlx::query!(
            "
            SELECT id, url, size, created, owner_id, context, mod_id, version_id, thread_message_id, report_id
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
        .fetch_many(&mut **transaction)
        .try_filter_map(|e| async {
            Ok(e.right().map(|row| {
                let id = ImageId(row.id);

                Image {
                    id,
                    url: row.url,
                    size: row.size as u64,
                    created: row.created,
                    owner_id: UserId(row.owner_id),
                    context: row.context,
                    project_id: row.mod_id.map(ProjectId),
                    version_id: row.version_id.map(VersionId),
                    thread_message_id: row.thread_message_id.map(ThreadMessageId),
                    report_id: row.report_id.map(ReportId),
                }
            }))
        })
        .try_collect::<Vec<Image>>()
        .await
    }

    pub async fn get<'a, 'b, E>(
        id: ImageId,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<Image>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        Image::get_many(&[id], executor, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(
        image_ids: &[ImageId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Image>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::TryStreamExt;

        if image_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut found_images = Vec::new();
        let mut remaining_ids = image_ids.to_vec();

        let image_ids = image_ids.iter().map(|x| x.0).collect::<Vec<_>>();

        if !image_ids.is_empty() {
            let images = redis
                .multi_get::<String, _>(IMAGES_NAMESPACE, image_ids)
                .await?;
            for image in images {
                if let Some(image) = image.and_then(|x| serde_json::from_str::<Image>(&x).ok()) {
                    remaining_ids.retain(|x| image.id.0 != x.0);
                    found_images.push(image);
                    continue;
                }
            }
        }

        if !remaining_ids.is_empty() {
            let db_images: Vec<Image> = sqlx::query!(
                "
                SELECT id, url, size, created, owner_id, context, mod_id, version_id, thread_message_id, report_id
                FROM uploaded_images
                WHERE id = ANY($1)
                GROUP BY id;
                ",
                &remaining_ids.iter().map(|x| x.0).collect::<Vec<_>>(),
            )
            .fetch_many(exec)
            .try_filter_map(|e| async {
                Ok(e.right().map(|i| {
                    let id = i.id;

                    Image {
                        id: ImageId(id),
                        url: i.url,
                        size: i.size as u64,
                        created: i.created,
                        owner_id: UserId(i.owner_id),
                        context: i.context,
                        project_id: i.mod_id.map(ProjectId),
                        version_id: i.version_id.map(VersionId),
                        thread_message_id: i.thread_message_id.map(ThreadMessageId),
                        report_id: i.report_id.map(ReportId),
                    }
                }))
            })
            .try_collect::<Vec<Image>>()
            .await?;

            for image in db_images {
                redis
                    .set_serialized_to_json(IMAGES_NAMESPACE, image.id.0, &image, None)
                    .await?;
                found_images.push(image);
            }
        }

        Ok(found_images)
    }

    pub async fn clear_cache(id: ImageId, redis: &RedisPool) -> Result<(), DatabaseError> {
        redis.delete(IMAGES_NAMESPACE, id.0).await?;
        Ok(())
    }
}

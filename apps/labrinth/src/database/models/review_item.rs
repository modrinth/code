use crate::database::PgTransaction;

use super::ids::*;
use chrono::{DateTime, Utc};
use futures::stream::TryStreamExt;

pub struct DBReview {
    pub id: DBReviewId,
    pub project_id: DBProjectId,
    pub user_id: DBUserId,
    pub rating: i16,
    pub body: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl DBReview {
    pub async fn insert(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO project_reviews (id, project_id, user_id, rating, body, created, updated)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ",
            self.id as DBReviewId,
            self.project_id as DBProjectId,
            self.user_id as DBUserId,
            self.rating,
            self.body,
            self.created,
            self.updated,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn get<'a, E>(
        id: DBReviewId,
        exec: E,
    ) -> Result<Option<DBReview>, sqlx::Error>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            "
            SELECT id, project_id, user_id, rating, body, created, updated
            FROM project_reviews
            WHERE id = $1
            ",
            id as DBReviewId
        )
        .fetch_optional(exec)
        .await
        .map(|r| {
            r.map(|x| DBReview {
                id: DBReviewId(x.id),
                project_id: DBProjectId(x.project_id),
                user_id: DBUserId(x.user_id),
                rating: x.rating,
                body: x.body,
                created: x.created,
                updated: x.updated,
            })
        })
    }

    pub async fn get_for_project<'a, E>(
        project_id: DBProjectId,
        exec: E,
        count: i64,
        offset: i64,
    ) -> Result<Vec<DBReview>, sqlx::Error>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            "
            SELECT id, project_id, user_id, rating, body, created, updated
            FROM project_reviews
            WHERE project_id = $1
            ORDER BY created DESC
            LIMIT $2 OFFSET $3
            ",
            project_id as DBProjectId,
            count,
            offset,
        )
        .fetch(exec)
        .map_ok(|x| DBReview {
            id: DBReviewId(x.id),
            project_id: DBProjectId(x.project_id),
            user_id: DBUserId(x.user_id),
            rating: x.rating,
            body: x.body,
            created: x.created,
            updated: x.updated,
        })
        .try_collect::<Vec<DBReview>>()
        .await
    }

    pub async fn get_by_user_and_project<'a, E>(
        user_id: DBUserId,
        project_id: DBProjectId,
        exec: E,
    ) -> Result<Option<DBReview>, sqlx::Error>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            "
            SELECT id, project_id, user_id, rating, body, created, updated
            FROM project_reviews
            WHERE user_id = $1 AND project_id = $2
            ",
            user_id as DBUserId,
            project_id as DBProjectId,
        )
        .fetch_optional(exec)
        .await
        .map(|r| {
            r.map(|x| DBReview {
                id: DBReviewId(x.id),
                project_id: DBProjectId(x.project_id),
                user_id: DBUserId(x.user_id),
                rating: x.rating,
                body: x.body,
                created: x.created,
                updated: x.updated,
            })
        })
    }

    pub async fn count_for_project<'a, E>(
        project_id: DBProjectId,
        exec: E,
    ) -> Result<i64, sqlx::Error>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM project_reviews WHERE project_id = $1",
            project_id as DBProjectId,
        )
        .fetch_one(exec)
        .await
        .map(|x| x.unwrap_or(0))
    }

    pub async fn remove(
        id: DBReviewId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<Option<()>, sqlx::error::Error> {
        let result = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM project_reviews WHERE id = $1)",
            id as DBReviewId
        )
        .fetch_one(&mut *transaction)
        .await?;

        if !result.exists.unwrap_or(false) {
            return Ok(None);
        }

        sqlx::query!(
            "DELETE FROM project_reviews WHERE id = $1",
            id as DBReviewId,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(Some(()))
    }
}

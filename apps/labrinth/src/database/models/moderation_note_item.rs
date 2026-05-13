use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::redis::RedisPool;

use super::{DBOrganizationId, DBUserId, DatabaseError};

const MODERATION_NOTES_USERS_NAMESPACE: &str = "moderation_notes_users";
const MODERATION_NOTES_ORGANIZATIONS_NAMESPACE: &str =
    "moderation_notes_organizations";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBModerationNote {
    pub user_id: Option<DBUserId>,
    pub organization_id: Option<DBOrganizationId>,
    pub last_modified: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_author: DBUserId,
    pub version: i32,
    pub notes: String,
    pub user_rating: i32,
}

impl DBModerationNote {
    pub async fn get_many_users<'a, E>(
        user_ids: &[DBUserId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<HashMap<DBUserId, Self>, DatabaseError>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let ids = user_ids
            .iter()
            .map(|id| id.0.to_string())
            .collect::<Vec<_>>();

        let cached = {
            let mut redis = redis.connect().await?;
            redis
                .get_many_deserialized_from_json::<Self>(
                    MODERATION_NOTES_USERS_NAMESPACE,
                    &ids,
                )
                .await?
        };

        let mut notes = HashMap::new();
        let mut missing_ids = Vec::new();
        for (id, cached_note) in user_ids.iter().copied().zip(cached) {
            if let Some(note) = cached_note {
                notes.insert(id, note);
            } else {
                missing_ids.push(id.0);
            }
        }

        if missing_ids.is_empty() {
            return Ok(notes);
        }

        let rows = sqlx::query!(
            r#"
			SELECT user_id, organization_id, last_modified, created_at, last_author, version, notes, user_rating
			FROM moderation_notes
			WHERE user_id = ANY($1)
			"#,
            &missing_ids,
        )
        .fetch_all(exec)
        .await?;

        let mut redis = redis.connect().await?;
        for row in rows {
            let note = Self {
                user_id: row.user_id.map(DBUserId),
                organization_id: row.organization_id.map(DBOrganizationId),
                last_modified: row.last_modified,
                created_at: row.created_at,
                last_author: DBUserId(row.last_author),
                version: row.version,
                notes: row.notes,
                user_rating: row.user_rating,
            };

            if let Some(user_id) = note.user_id {
                redis
                    .set_serialized_to_json(
                        MODERATION_NOTES_USERS_NAMESPACE,
                        user_id.0,
                        &note,
                        None,
                    )
                    .await?;
                notes.insert(user_id, note);
            }
        }

        Ok(notes)
    }

    pub async fn get_user<'a, E>(
        user_id: DBUserId,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Option<Self>, DatabaseError>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(Self::get_many_users(&[user_id], exec, redis)
            .await?
            .remove(&user_id))
    }

    pub async fn get_many_organizations<'a, E>(
        organization_ids: &[DBOrganizationId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<HashMap<DBOrganizationId, Self>, DatabaseError>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let ids = organization_ids
            .iter()
            .map(|id| id.0.to_string())
            .collect::<Vec<_>>();

        let cached = {
            let mut redis = redis.connect().await?;
            redis
                .get_many_deserialized_from_json::<Self>(
                    MODERATION_NOTES_ORGANIZATIONS_NAMESPACE,
                    &ids,
                )
                .await?
        };

        let mut notes = HashMap::new();
        let mut missing_ids = Vec::new();
        for (id, cached_note) in organization_ids.iter().copied().zip(cached) {
            if let Some(note) = cached_note {
                notes.insert(id, note);
            } else {
                missing_ids.push(id.0);
            }
        }

        if missing_ids.is_empty() {
            return Ok(notes);
        }

        let rows = sqlx::query!(
            r#"
			SELECT user_id, organization_id, last_modified, created_at, last_author, version, notes, user_rating
			FROM moderation_notes
			WHERE organization_id = ANY($1)
			"#,
            &missing_ids,
        )
        .fetch_all(exec)
        .await?;

        let mut redis = redis.connect().await?;
        for row in rows {
            let note = Self {
                user_id: row.user_id.map(DBUserId),
                organization_id: row.organization_id.map(DBOrganizationId),
                last_modified: row.last_modified,
                created_at: row.created_at,
                last_author: DBUserId(row.last_author),
                version: row.version,
                notes: row.notes,
                user_rating: row.user_rating,
            };

            if let Some(organization_id) = note.organization_id {
                redis
                    .set_serialized_to_json(
                        MODERATION_NOTES_ORGANIZATIONS_NAMESPACE,
                        organization_id.0,
                        &note,
                        None,
                    )
                    .await?;
                notes.insert(organization_id, note);
            }
        }

        Ok(notes)
    }

    pub async fn get_organization<'a, E>(
        organization_id: DBOrganizationId,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Option<Self>, DatabaseError>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(
            Self::get_many_organizations(&[organization_id], exec, redis)
                .await?
                .remove(&organization_id),
        )
    }

    pub async fn insert<'a, E>(
        user_id: Option<DBUserId>,
        organization_id: Option<DBOrganizationId>,
        last_author: DBUserId,
        notes: Option<&str>,
        user_rating: Option<i32>,
        exec: E,
    ) -> Result<Option<i32>, DatabaseError>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query_scalar!(
            r#"
            INSERT INTO moderation_notes (user_id, organization_id, last_author, version, notes, user_rating)
            SELECT
              $1, $2, $3, 1, COALESCE($4::text, ''), COALESCE($5::integer, 0)
            WHERE NOT EXISTS (
              SELECT 1 FROM moderation_notes
              WHERE
                ($1::bigint IS NOT NULL AND user_id = $1)
                OR ($2::bigint IS NOT NULL AND organization_id = $2)
            )
            ON CONFLICT DO NOTHING
            RETURNING version
            "#,
            user_id.map(|x| x.0),
            organization_id.map(|x| x.0),
            last_author.0,
            notes,
            user_rating,
        )
        .fetch_optional(exec)
        .await?;

        Ok(result)
    }

    pub async fn update<'a, E>(
        user_id: Option<DBUserId>,
        organization_id: Option<DBOrganizationId>,
        last_author: DBUserId,
        expected_current_version: i32,
        notes: Option<&str>,
        user_rating: Option<i32>,
        exec: E,
    ) -> Result<Option<i32>, DatabaseError>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query_scalar!(
            r#"
            UPDATE moderation_notes
            SET
              last_modified = NOW(),
              last_author = $1,
              version = version + 1,
              notes = COALESCE($2::text, notes),
              user_rating = COALESCE($3::integer, user_rating)
            WHERE (
                ($4::bigint IS NOT NULL AND user_id = $4) OR
                ($5::bigint IS NOT NULL AND organization_id = $5)
              )
              AND version = $6
            RETURNING version
            "#,
            last_author.0,
            notes,
            user_rating,
            user_id.map(|x| x.0),
            organization_id.map(|x| x.0),
            expected_current_version
        )
        .fetch_optional(exec)
        .await?;

        Ok(result)
    }

    pub async fn clear_user_cache(
        user_id: DBUserId,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;
        redis
            .delete(MODERATION_NOTES_USERS_NAMESPACE, user_id.0)
            .await
    }

    pub async fn clear_organization_cache(
        organization_id: DBOrganizationId,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;
        redis
            .delete(MODERATION_NOTES_ORGANIZATIONS_NAMESPACE, organization_id.0)
            .await
    }
}

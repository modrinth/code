use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, postgres::PgRow};

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
    fn from_row(row: PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            user_id: row.try_get::<Option<i64>, _>("user_id")?.map(DBUserId),
            organization_id: row
                .try_get::<Option<i64>, _>("organization_id")?
                .map(DBOrganizationId),
            last_modified: row.try_get("last_modified")?,
            created_at: row.try_get("created_at")?,
            last_author: DBUserId(row.try_get("last_author")?),
            version: row.try_get("version")?,
            notes: row.try_get("notes")?,
            user_rating: row.try_get("user_rating")?,
        })
    }

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

        let rows = sqlx::query(
			r#"
			SELECT user_id, organization_id, last_modified, created_at, last_author, version, notes, user_rating
			FROM moderation_notes
			WHERE user_id = ANY($1)
			"#,
		)
		.bind(&missing_ids)
		.fetch_all(exec)
		.await?;

        let mut redis = redis.connect().await?;
        for row in rows {
            let note = Self::from_row(row)?;

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

        let rows = sqlx::query(
			r#"
			SELECT user_id, organization_id, last_modified, created_at, last_author, version, notes, user_rating
			FROM moderation_notes
			WHERE organization_id = ANY($1)
			"#,
		)
		.bind(&missing_ids)
		.fetch_all(exec)
		.await?;

        let mut redis = redis.connect().await?;
        for row in rows {
            let note = Self::from_row(row)?;

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

    pub async fn patch_user<'a, E>(
        user_id: DBUserId,
        last_author: DBUserId,
        expected_version: i32,
        notes: Option<&str>,
        user_rating: Option<i32>,
        exec: E,
    ) -> Result<Option<Self>, DatabaseError>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let row = sqlx::query(
			r#"
			WITH updated AS (
				UPDATE moderation_notes
				SET
					last_modified = NOW(),
					last_author = $2,
					version = version + 1,
					notes = COALESCE($4::text, notes),
					user_rating = COALESCE($5::integer, user_rating)
				WHERE user_id = $1 AND version = $3::integer
				RETURNING user_id, organization_id, last_modified, created_at, last_author, version, notes, user_rating
			),
			inserted AS (
				INSERT INTO moderation_notes (user_id, organization_id, last_author, version, notes, user_rating)
				SELECT $1, NULL::bigint, $2, 1, COALESCE($4::text, ''), COALESCE($5::integer, 0)
				WHERE $3::integer = 0 AND NOT EXISTS (
					SELECT 1 FROM moderation_notes WHERE user_id = $1
				)
				ON CONFLICT DO NOTHING
				RETURNING user_id, organization_id, last_modified, created_at, last_author, version, notes, user_rating
			)
			SELECT * FROM updated
			UNION ALL
			SELECT * FROM inserted
			"#,
		)
		.bind(user_id.0)
		.bind(last_author.0)
		.bind(expected_version)
		.bind(notes)
		.bind(user_rating)
		.fetch_optional(exec)
		.await?;

        Ok(row.map(Self::from_row).transpose()?)
    }

    pub async fn patch_organization<'a, E>(
        organization_id: DBOrganizationId,
        last_author: DBUserId,
        expected_version: i32,
        notes: Option<&str>,
        user_rating: Option<i32>,
        exec: E,
    ) -> Result<Option<Self>, DatabaseError>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let row = sqlx::query(
			r#"
			WITH updated AS (
				UPDATE moderation_notes
				SET
					last_modified = NOW(),
					last_author = $2,
					version = version + 1,
					notes = COALESCE($4::text, notes),
					user_rating = COALESCE($5::integer, user_rating)
				WHERE organization_id = $1 AND version = $3::integer
				RETURNING user_id, organization_id, last_modified, created_at, last_author, version, notes, user_rating
			),
			inserted AS (
				INSERT INTO moderation_notes (user_id, organization_id, last_author, version, notes, user_rating)
				SELECT NULL::bigint, $1, $2, 1, COALESCE($4::text, ''), COALESCE($5::integer, 0)
				WHERE $3::integer = 0 AND NOT EXISTS (
					SELECT 1 FROM moderation_notes WHERE organization_id = $1
				)
				ON CONFLICT DO NOTHING
				RETURNING user_id, organization_id, last_modified, created_at, last_author, version, notes, user_rating
			)
			SELECT * FROM updated
			UNION ALL
			SELECT * FROM inserted
			"#,
		)
		.bind(organization_id.0)
		.bind(last_author.0)
		.bind(expected_version)
		.bind(notes)
		.bind(user_rating)
		.fetch_optional(exec)
		.await?;

        Ok(row.map(Self::from_row).transpose()?)
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

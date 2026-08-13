use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    database::models::{DBProjectId, DBUserId, DatabaseError},
    models::v3::disclosures::{
        DisclosureLockStatus, ProjectDisclosure, ProjectDisclosureType,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBProjectDisclosure {
    pub project_id: DBProjectId,
    pub disclosure: ProjectDisclosure,
    pub updated_at: DateTime<Utc>,
    pub updated_by: DBUserId,
    pub set_by_moderator: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub lock_status: DisclosureLockStatus,
}

impl DBProjectDisclosure {
    pub async fn upsert(
        &self,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let (disclosure_type, metadata) =
            self.disclosure.to_parts().map_err(|e| {
                DatabaseError::Internal(eyre::Report::new(e).wrap_err(
                    "failed to serialize project disclosure metadata",
                ))
            })?;

        sqlx::query!(
            r#"
			INSERT INTO project_disclosures (project_id, type, metadata, updated_by, set_by_moderator, lock_status)
			VALUES ($1, $2, $3, $4, $5, $6)
			ON CONFLICT (project_id, type) DO UPDATE SET
				metadata = $3,
				updated_at = now(),
				updated_by = $4,
				set_by_moderator = $5,
				deleted_at = NULL,
				lock_status = $6
			"#,
            self.project_id as DBProjectId,
            disclosure_type,
            metadata,
            self.updated_by as DBUserId,
            self.set_by_moderator,
            <&'static str>::from(self.lock_status),
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn get_many_for_project(
        project_id: DBProjectId,
        include_deleted: bool,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBProjectDisclosure>, DatabaseError> {
        let rows = sqlx::query!(
            r#"
			SELECT project_id, type AS "disclosure_type!", metadata, updated_at, updated_by, set_by_moderator, deleted_at, lock_status
			FROM project_disclosures
			WHERE project_id = $1 AND ($2 OR deleted_at IS NULL)
			ORDER BY updated_at DESC
			"#,
            project_id as DBProjectId,
            include_deleted,
        )
        .fetch_all(exec)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(DBProjectDisclosure {
                    project_id: DBProjectId(row.project_id),
                    disclosure: ProjectDisclosure::from_parts(
                        &row.disclosure_type,
                        row.metadata,
                    )
                    .map_err(|e| {
                        DatabaseError::Internal(eyre::Report::new(e).wrap_err(
                            "failed to deserialize project disclosure metadata",
                        ))
                    })?,
                    updated_at: row.updated_at,
                    updated_by: DBUserId(row.updated_by),
                    set_by_moderator: row.set_by_moderator,
                    deleted_at: row.deleted_at,
                    lock_status: DisclosureLockStatus::from_str(
                        &row.lock_status,
                    )
                    .map_err(|e| {
                        DatabaseError::Internal(eyre::Report::new(e).wrap_err(
                            "failed to parse project disclosure lock status",
                        ))
                    })?,
                })
            })
            .collect()
    }

    /// Returns the subset of `project_ids` that carry a disclosure of the given type.
    pub async fn projects_with_type(
        disclosure_type: ProjectDisclosureType,
        project_ids: &[DBProjectId],
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<HashSet<DBProjectId>, DatabaseError> {
        let ids = project_ids.iter().map(|id| id.0).collect::<Vec<_>>();
        let rows = sqlx::query_scalar!(
            r#"
			SELECT project_id
			FROM project_disclosures
			WHERE type = $1 AND project_id = ANY($2) AND deleted_at IS NULL
			"#,
            <&'static str>::from(disclosure_type),
            &ids,
        )
        .fetch_all(exec)
        .await?;

        Ok(rows.into_iter().map(DBProjectId).collect())
    }

    pub async fn get_lock_statuses_for_project(
        project_id: DBProjectId,
        types: &[String],
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<HashMap<String, DisclosureLockStatus>, DatabaseError> {
        let rows = sqlx::query!(
            r#"
			SELECT type AS "disclosure_type!", lock_status
			FROM project_disclosures
			WHERE project_id = $1 AND type = ANY($2)
			"#,
            project_id as DBProjectId,
            types,
        )
        .fetch_all(exec)
        .await?;

        rows.into_iter()
            .map(|row| {
                let lock_status = DisclosureLockStatus::from_str(
                    &row.lock_status,
                )
                .map_err(|e| {
                    DatabaseError::Internal(eyre::Report::new(e).wrap_err(
                        "failed to parse project disclosure lock status",
                    ))
                })?;

                Ok((row.disclosure_type, lock_status))
            })
            .collect()
    }

    pub async fn remove(
        project_id: DBProjectId,
        disclosure_type: ProjectDisclosureType,
        updated_by: DBUserId,
        set_by_moderator: bool,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<bool, DatabaseError> {
        let result = sqlx::query!(
            r#"
			UPDATE project_disclosures
			SET deleted_at = now(), updated_at = now(), updated_by = $3, set_by_moderator = $4
			WHERE project_id = $1 AND type = $2 AND deleted_at IS NULL
			"#,
            project_id as DBProjectId,
            <&'static str>::from(disclosure_type),
            updated_by as DBUserId,
            set_by_moderator,
        )
        .execute(exec)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    database::models::{DBProjectId, DBUserId, DatabaseError},
    models::v3::disclosures::ProjectDisclosure,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBProjectDisclosure {
    pub project_id: DBProjectId,
    pub disclosure: ProjectDisclosure,
    pub updated_at: DateTime<Utc>,
    pub updated_by: DBUserId,
    pub set_by_moderator: bool,
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
			INSERT INTO project_disclosures (project_id, type, metadata, updated_by, set_by_moderator)
			VALUES ($1, $2, $3, $4, $5)
			ON CONFLICT (project_id, type) DO UPDATE SET
				metadata = $3,
				updated_at = now(),
				updated_by = $4,
				set_by_moderator = $5
			"#,
            self.project_id as DBProjectId,
            disclosure_type,
            metadata,
            self.updated_by as DBUserId,
            self.set_by_moderator,
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn get_many_for_project(
        project_id: DBProjectId,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBProjectDisclosure>, DatabaseError> {
        let rows = sqlx::query!(
            r#"
			SELECT project_id, type AS "disclosure_type!", metadata, updated_at, updated_by, set_by_moderator
			FROM project_disclosures
			WHERE project_id = $1
			ORDER BY updated_at DESC
			"#,
            project_id as DBProjectId,
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
                })
            })
            .collect()
    }

    pub async fn remove(
        project_id: DBProjectId,
        disclosure_type: &str,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<bool, DatabaseError> {
        let result = sqlx::query!(
            r#"
			DELETE FROM project_disclosures
			WHERE project_id = $1 AND type = $2
			"#,
            project_id as DBProjectId,
            disclosure_type,
        )
        .execute(exec)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

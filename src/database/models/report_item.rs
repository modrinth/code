use super::ids::*;
use chrono::{DateTime, Utc};

pub struct Report {
    pub id: ReportId,
    pub report_type_id: ReportTypeId,
    pub project_id: Option<ProjectId>,
    pub version_id: Option<VersionId>,
    pub user_id: Option<UserId>,
    pub body: String,
    pub reporter: UserId,
    pub created: DateTime<Utc>,
}

pub struct QueryReport {
    pub id: ReportId,
    pub report_type: String,
    pub project_id: Option<ProjectId>,
    pub version_id: Option<VersionId>,
    pub user_id: Option<UserId>,
    pub body: String,
    pub reporter: UserId,
    pub created: DateTime<Utc>,
}

impl Report {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO reports (
                id, report_type_id, mod_id, version_id, user_id,
                body, reporter
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7
            )
            ",
            self.id as ReportId,
            self.report_type_id as ReportTypeId,
            self.project_id.map(|x| x.0 as i64),
            self.version_id.map(|x| x.0 as i64),
            self.user_id.map(|x| x.0 as i64),
            self.body,
            self.reporter as UserId
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn get<'a, E>(
        id: ReportId,
        exec: E,
    ) -> Result<Option<QueryReport>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let result = sqlx::query!(
            "
            SELECT rt.name, r.mod_id, r.version_id, r.user_id, r.body, r.reporter, r.created
            FROM reports r
            INNER JOIN report_types rt ON rt.id = r.report_type_id
            WHERE r.id = $1
            ",
            id as ReportId,
        )
        .fetch_optional(exec)
        .await?;

        if let Some(row) = result {
            Ok(Some(QueryReport {
                id,
                report_type: row.name,
                project_id: row.mod_id.map(ProjectId),
                version_id: row.version_id.map(VersionId),
                user_id: row.user_id.map(UserId),
                body: row.body,
                reporter: UserId(row.reporter),
                created: row.created,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many<'a, E>(
        report_ids: Vec<ReportId>,
        exec: E,
    ) -> Result<Vec<QueryReport>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let report_ids_parsed: Vec<i64> =
            report_ids.into_iter().map(|x| x.0).collect();
        let reports = sqlx::query!(
            "
            SELECT r.id, rt.name, r.mod_id, r.version_id, r.user_id, r.body, r.reporter, r.created
            FROM reports r
            INNER JOIN report_types rt ON rt.id = r.report_type_id
            WHERE r.id = ANY($1)
            ORDER BY r.created DESC
            ",
            &report_ids_parsed
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|x| QueryReport {
                id: ReportId(x.id),
                report_type: x.name,
                project_id: x.mod_id.map(ProjectId),
                version_id: x.version_id.map(VersionId),
                user_id: x.user_id.map(UserId),
                body: x.body,
                reporter: UserId(x.reporter),
                created: x.created,
            }))
        })
        .try_collect::<Vec<QueryReport>>()
        .await?;

        Ok(reports)
    }

    pub async fn remove_full<'a, E>(
        id: ReportId,
        exec: E,
    ) -> Result<Option<()>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let result = sqlx::query!(
            "
            SELECT EXISTS(SELECT 1 FROM reports WHERE id = $1)
            ",
            id as ReportId
        )
        .fetch_one(exec)
        .await?;

        if !result.exists.unwrap_or(false) {
            return Ok(None);
        }

        sqlx::query!(
            "
            DELETE FROM reports WHERE id = $1
            ",
            id as ReportId,
        )
        .execute(exec)
        .await?;

        Ok(Some(()))
    }
}

use super::ids::*;
use chrono::{DateTime, Utc};

pub struct DBReport {
    pub id: DBReportId,
    pub report_type_id: ReportTypeId,
    pub project_id: Option<DBProjectId>,
    pub version_id: Option<DBVersionId>,
    pub user_id: Option<DBUserId>,
    pub body: String,
    pub reporter: DBUserId,
    pub created: DateTime<Utc>,
    pub closed: bool,
}

pub struct ReportQueryResult {
    pub id: DBReportId,
    pub report_type: String,
    pub project_id: Option<DBProjectId>,
    pub version_id: Option<DBVersionId>,
    pub user_id: Option<DBUserId>,
    pub body: String,
    pub reporter: DBUserId,
    pub created: DateTime<Utc>,
    pub closed: bool,
    pub thread_id: DBThreadId,
}

impl DBReport {
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
            self.id as DBReportId,
            self.report_type_id as ReportTypeId,
            self.project_id.map(|x| x.0 as i64),
            self.version_id.map(|x| x.0 as i64),
            self.user_id.map(|x| x.0 as i64),
            self.body,
            self.reporter as DBUserId
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn get<'a, E>(
        id: DBReportId,
        exec: E,
    ) -> Result<Option<ReportQueryResult>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        Self::get_many(&[id], exec)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(
        report_ids: &[DBReportId],
        exec: E,
    ) -> Result<Vec<ReportQueryResult>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let report_ids_parsed: Vec<i64> =
            report_ids.iter().map(|x| x.0).collect();
        let reports = sqlx::query!(
            "
            SELECT r.id, rt.name, r.mod_id, r.version_id, r.user_id, r.body, r.reporter, r.created, t.id thread_id, r.closed
            FROM reports r
            INNER JOIN report_types rt ON rt.id = r.report_type_id
            INNER JOIN threads t ON t.report_id = r.id
            WHERE r.id = ANY($1)
            ORDER BY r.created DESC
            ",
            &report_ids_parsed
        )
        .fetch(exec)
        .map_ok(|x| ReportQueryResult {
            id: DBReportId(x.id),
            report_type: x.name,
            project_id: x.mod_id.map(DBProjectId),
            version_id: x.version_id.map(DBVersionId),
            user_id: x.user_id.map(DBUserId),
            body: x.body,
            reporter: DBUserId(x.reporter),
            created: x.created,
            closed: x.closed,
            thread_id: DBThreadId(x.thread_id)
        })
        .try_collect::<Vec<ReportQueryResult>>()
        .await?;

        Ok(reports)
    }

    pub async fn remove_full(
        id: DBReportId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, sqlx::error::Error> {
        let result = sqlx::query!(
            "
            SELECT EXISTS(SELECT 1 FROM reports WHERE id = $1)
            ",
            id as DBReportId
        )
        .fetch_one(&mut **transaction)
        .await?;

        if !result.exists.unwrap_or(false) {
            return Ok(None);
        }

        let thread_id = sqlx::query!(
            "
            SELECT id FROM threads
            WHERE report_id = $1
            ",
            id as DBReportId
        )
        .fetch_optional(&mut **transaction)
        .await?;

        if let Some(thread_id) = thread_id {
            crate::database::models::DBThread::remove_full(
                DBThreadId(thread_id.id),
                transaction,
            )
            .await?;
        }

        sqlx::query!(
            "
            DELETE FROM reports WHERE id = $1
            ",
            id as DBReportId,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(Some(()))
    }
}

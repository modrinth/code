use super::ids::{
    DBThreadId, DBThreadIssueFacetId, DBThreadIssueId, DBUserId,
    generate_many_thread_issue_facet_ids, generate_thread_issue_id,
};
use crate::database::PgTransaction;
use crate::models::thread_issues::{
    ThreadIssueContext, ThreadIssueFacet, ThreadIssueTarget, ThreadIssueVerdict,
};
use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr, eyre};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

pub struct ThreadIssueBuilder {
    pub facets: Vec<ThreadIssueTarget>,
    pub why: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DBThreadIssue {
    pub id: DBThreadIssueId,
    pub thread_id: DBThreadId,
    pub created_by: DBUserId,
    pub why: serde_json::Value,
    pub user_addressed: bool,
    pub moderator_verified: bool,
    pub facets: Vec<ThreadIssueFacet>,
    pub verdict: ThreadIssueVerdict,
    pub created_at: DateTime<Utc>,
}

impl DBThreadIssue {
    pub async fn get_many_for_threads<'a, E>(
        thread_ids: &[DBThreadId],
        exec: E,
    ) -> Result<Vec<Self>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        if thread_ids.is_empty() {
            return Ok(Vec::new());
        }

        let thread_ids = thread_ids.iter().map(|id| id.0).collect::<Vec<_>>();
        let rows = sqlx::query!(
            r#"
            SELECT
                issue.id,
                issue.thread_id,
                issue.created_by,
                issue.why,
                issue.user_addressed,
                issue.moderator_verified,
                issue.created_at,
                facet.id AS "facet_id!",
                facet.what AS "what: Json<ThreadIssueTarget>",
                facet.verdict
            FROM threads_issues issue
            INNER JOIN threads_issue_facets facet
                ON facet.issue_id = issue.id
            WHERE issue.thread_id = ANY($1)
            ORDER BY issue.created_at, issue.id, facet.id
            "#,
            &thread_ids,
        )
        .fetch_all(exec)
        .await
        .wrap_err("fetching thread issues")?;

        let mut issues = Vec::<Self>::new();
        for row in rows {
            let facet_verdict =
                parse_verdict(row.id, row.facet_id, &row.verdict)?;
            let what = row.what.0;

            if issues.last().is_none_or(|issue| issue.id.0 != row.id) {
                issues.push(Self {
                    id: DBThreadIssueId(row.id),
                    thread_id: DBThreadId(row.thread_id),
                    created_by: DBUserId(row.created_by),
                    why: row.why,
                    user_addressed: row.user_addressed,
                    moderator_verified: row.moderator_verified,
                    facets: Vec::new(),
                    verdict: ThreadIssueVerdict::Open,
                    created_at: row.created_at,
                });
            }

            issues
                .last_mut()
                .expect("thread issue was inserted above")
                .facets
                .push(ThreadIssueFacet {
                    id: DBThreadIssueFacetId(row.facet_id).into(),
                    what,
                    verdict: facet_verdict,
                });
        }

        for issue in &mut issues {
            issue.verdict = ThreadIssueVerdict::from_facets(&issue.facets);
        }

        Ok(issues)
    }

    pub async fn insert_many(
        thread_id: DBThreadId,
        created_by: DBUserId,
        issues: Vec<ThreadIssueBuilder>,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<Vec<DBThreadIssueId>> {
        if issues.iter().any(|issue| issue.facets.is_empty()) {
            return Err(eyre!("thread issues must contain at least one facet"));
        }

        let mut ids = Vec::with_capacity(issues.len());
        for issue in issues {
            let id = generate_thread_issue_id(&mut *transaction)
                .await
                .wrap_err("generating thread issue ID")?;

            sqlx::query!(
                r#"
                INSERT INTO threads_issues (
                    id,
                    thread_id,
                    created_by,
                    why,
                    user_addressed,
                    moderator_verified
                )
                VALUES ($1, $2, $3, $4, FALSE, FALSE)
                "#,
                id as DBThreadIssueId,
                thread_id as DBThreadId,
                created_by as DBUserId,
                issue.why,
            )
            .execute(&mut *transaction)
            .await
            .wrap_err("inserting thread issue")?;

            let facet_ids = generate_many_thread_issue_facet_ids(
                issue.facets.len(),
                &mut *transaction,
            )
            .await
            .wrap_err("generating thread issue facet IDs")?;
            let facet_ids = facet_ids.iter().map(|id| id.0).collect::<Vec<_>>();
            let whats = issue
                .facets
                .into_iter()
                .map(serde_json::to_value)
                .collect::<std::result::Result<Vec<_>, _>>()
                .wrap_err("serializing thread issue facets")?;
            let verdicts = facet_ids
                .iter()
                .map(|_| ThreadIssueVerdict::Open.as_str().to_string())
                .collect::<Vec<_>>();

            sqlx::query!(
                r#"
                INSERT INTO threads_issue_facets (
                    id,
                    issue_id,
                    what,
                    verdict
                )
                SELECT facet_id, $1, what, verdict
                FROM UNNEST(
                    $2::int8[],
                    $3::jsonb[],
                    $4::varchar[]
                ) AS facet(facet_id, what, verdict)
                "#,
                id as DBThreadIssueId,
                &facet_ids,
                &whats,
                &verdicts,
            )
            .execute(&mut *transaction)
            .await
            .wrap_err("inserting thread issue facets")?;

            ids.push(id);
        }

        Ok(ids)
    }

    pub async fn update_flags(
        id: DBThreadIssueId,
        user_addressed: Option<bool>,
        moderator_verified: Option<bool>,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE threads_issues
            SET
                user_addressed = COALESCE($2, user_addressed),
                moderator_verified = COALESCE($3, moderator_verified)
            WHERE id = $1
            "#,
            id as DBThreadIssueId,
            user_addressed,
            moderator_verified,
        )
        .execute(&mut *transaction)
        .await
        .wrap_err("updating thread issue")?;

        if result.rows_affected() > 0 && moderator_verified == Some(true) {
            sqlx::query!(
                r#"
                UPDATE threads_issue_facets
                SET verdict = $2
                WHERE issue_id = $1
                "#,
                id as DBThreadIssueId,
                ThreadIssueVerdict::Resolved.as_str(),
            )
            .execute(&mut *transaction)
            .await
            .wrap_err("resolving verified thread issue facets")?;
        }

        Ok(result.rows_affected() > 0)
    }

    pub async fn remove(
        id: DBThreadIssueId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM threads_issues WHERE id = $1",
            id as DBThreadIssueId,
        )
        .execute(&mut *transaction)
        .await
        .wrap_err("deleting thread issue")?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn sync_project_verdicts(
        context: &ThreadIssueContext<'_>,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<Vec<Self>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                issue.id,
                issue.thread_id,
                issue.created_by,
                issue.why,
                issue.user_addressed,
                issue.moderator_verified,
                issue.created_at,
                facet.id AS "facet_id!",
                facet.what AS "what: Json<ThreadIssueTarget>",
                facet.verdict
            FROM threads_issues issue
            INNER JOIN threads_issue_facets facet
                ON facet.issue_id = issue.id
            WHERE issue.thread_id = $1
            ORDER BY issue.id, facet.id
            FOR UPDATE OF issue, facet
            "#,
            DBThreadId::from(context.project.thread_id) as DBThreadId,
        )
        .fetch_all(&mut *transaction)
        .await
        .wrap_err(
            "locking project thread issues for verdict synchronization",
        )?;

        let mut issues = Vec::<Self>::new();
        for row in rows {
            let stored_verdict =
                parse_verdict(row.id, row.facet_id, &row.verdict)?;
            let what = row.what.0;
            let verdict = what.verdict(
                context,
                row.user_addressed,
                row.moderator_verified,
            );

            if verdict != stored_verdict {
                sqlx::query!(
                    r#"
                    UPDATE threads_issue_facets
                    SET verdict = $2
                    WHERE id = $1
                    "#,
                    row.facet_id,
                    verdict.as_str(),
                )
                .execute(&mut *transaction)
                .await
                .wrap_err("updating project thread issue facet verdict")?;
            }

            if issues.last().is_none_or(|issue| issue.id.0 != row.id) {
                issues.push(Self {
                    id: DBThreadIssueId(row.id),
                    thread_id: DBThreadId(row.thread_id),
                    created_by: DBUserId(row.created_by),
                    why: row.why,
                    user_addressed: row.user_addressed,
                    moderator_verified: row.moderator_verified,
                    facets: Vec::new(),
                    verdict: ThreadIssueVerdict::Open,
                    created_at: row.created_at,
                });
            }

            issues
                .last_mut()
                .expect("thread issue was inserted above")
                .facets
                .push(ThreadIssueFacet {
                    id: DBThreadIssueFacetId(row.facet_id).into(),
                    what,
                    verdict,
                });
        }

        for issue in &mut issues {
            issue.verdict = ThreadIssueVerdict::from_facets(&issue.facets);
        }

        Ok(issues)
    }
}

fn parse_verdict(
    issue_id: i64,
    facet_id: i64,
    verdict: &str,
) -> Result<ThreadIssueVerdict> {
    match verdict {
        "open" => Ok(ThreadIssueVerdict::Open),
        "addressed" => Ok(ThreadIssueVerdict::Addressed),
        "resolved" => Ok(ThreadIssueVerdict::Resolved),
        verdict => Err(eyre!(
            "thread issue `{issue_id}` facet `{facet_id}` has invalid verdict `{verdict}`"
        )),
    }
}

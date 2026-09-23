use super::ids::{
    DBThreadId, DBThreadIssueId, DBUserId, generate_thread_issue_id,
};
use crate::database::PgTransaction;
use crate::models::thread_issues::{
    ThreadIssueContext, ThreadIssueTarget, ThreadIssueVerdict,
};
use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr, eyre};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

pub struct ThreadIssueBuilder {
    pub what: ThreadIssueTarget,
    pub why: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DBThreadIssue {
    pub id: DBThreadIssueId,
    pub thread_id: DBThreadId,
    pub created_by: DBUserId,
    pub what: ThreadIssueTarget,
    pub why: serde_json::Value,
    pub user_addressed: bool,
    pub moderator_verified: bool,
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
                id,
                thread_id,
                created_by,
                what AS "what: Json<ThreadIssueTarget>",
                why,
                user_addressed,
                moderator_verified,
                verdict,
                created_at
            FROM threads_issues
            WHERE thread_id = ANY($1)
            ORDER BY created_at, id
            "#,
            &thread_ids,
        )
        .fetch_all(exec)
        .await
        .wrap_err("fetching thread issues")?;

        rows.into_iter()
            .map(|row| {
                Ok(Self {
                    id: DBThreadIssueId(row.id),
                    thread_id: DBThreadId(row.thread_id),
                    created_by: DBUserId(row.created_by),
                    what: row.what.0,
                    why: row.why,
                    user_addressed: row.user_addressed,
                    moderator_verified: row.moderator_verified,
                    verdict: parse_verdict(row.id, &row.verdict)?,
                    created_at: row.created_at,
                })
            })
            .collect()
    }

    pub async fn insert_many(
        thread_id: DBThreadId,
        created_by: DBUserId,
        issues: Vec<ThreadIssueBuilder>,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<Vec<DBThreadIssueId>> {
        let mut ids = Vec::with_capacity(issues.len());
        for issue in issues {
            let id = generate_thread_issue_id(&mut *transaction)
                .await
                .wrap_err("generating thread issue ID")?;
            let what = serde_json::to_value(issue.what)
                .wrap_err("serializing thread issue target")?;

            sqlx::query!(
                r#"
                INSERT INTO threads_issues (
                    id,
                    thread_id,
                    created_by,
                    what,
                    why,
                    user_addressed,
                    moderator_verified,
                    verdict
                )
                VALUES ($1, $2, $3, $4, $5, FALSE, FALSE, $6)
                "#,
                id as DBThreadIssueId,
                thread_id as DBThreadId,
                created_by as DBUserId,
                what,
                issue.why,
                ThreadIssueVerdict::Open.as_str(),
            )
            .execute(&mut *transaction)
            .await
            .wrap_err("inserting thread issue")?;

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
                moderator_verified = COALESCE($3, moderator_verified),
                verdict = CASE
                    WHEN $3 IS TRUE THEN $4
                    ELSE verdict
                END
            WHERE id = $1
            "#,
            id as DBThreadIssueId,
            user_addressed,
            moderator_verified,
            ThreadIssueVerdict::Resolved.as_str(),
        )
        .execute(&mut *transaction)
        .await
        .wrap_err("updating thread issue")?;

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
				id,
				thread_id,
				created_by,
				what AS "what: Json<ThreadIssueTarget>",
				why,
				user_addressed,
				moderator_verified,
				verdict,
				created_at
			FROM threads_issues
			WHERE thread_id = $1
			ORDER BY id
			FOR UPDATE
			"#,
            DBThreadId::from(context.project.thread_id) as DBThreadId,
        )
        .fetch_all(&mut *transaction)
        .await
        .wrap_err(
            "locking project thread issues for verdict synchronization",
        )?;

        let mut issues = Vec::with_capacity(rows.len());
        for row in rows {
            let stored_verdict = parse_verdict(row.id, &row.verdict)?;
            let what = row.what.0;
            let verdict = what.verdict(
                context,
                row.user_addressed,
                row.moderator_verified,
            );

            if verdict != stored_verdict {
                sqlx::query!(
                    r#"
					UPDATE threads_issues
					SET verdict = $2
					WHERE id = $1
					"#,
                    row.id,
                    verdict.as_str(),
                )
                .execute(&mut *transaction)
                .await
                .wrap_err("updating project thread issue verdict")?;
            }

            issues.push(Self {
                id: DBThreadIssueId(row.id),
                thread_id: DBThreadId(row.thread_id),
                created_by: DBUserId(row.created_by),
                what,
                why: row.why,
                user_addressed: row.user_addressed,
                moderator_verified: row.moderator_verified,
                verdict,
                created_at: row.created_at,
            });
        }

        Ok(issues)
    }
}

fn parse_verdict(id: i64, verdict: &str) -> Result<ThreadIssueVerdict> {
    match verdict {
        "open" => Ok(ThreadIssueVerdict::Open),
        "addressed" => Ok(ThreadIssueVerdict::Addressed),
        "resolved" => Ok(ThreadIssueVerdict::Resolved),
        verdict => {
            Err(eyre!("thread issue `{id}` has invalid verdict `{verdict}`"))
        }
    }
}

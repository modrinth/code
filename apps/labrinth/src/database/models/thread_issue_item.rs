use super::ids::{DBThreadId, DBThreadIssueId};
use crate::database::PgTransaction;
use crate::models::thread_issues::{
    ThreadIssueContext, ThreadIssueTarget, ThreadIssueVerdict,
};
use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr, eyre};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DBThreadIssue {
    pub id: DBThreadIssueId,
    pub thread_id: DBThreadId,
    pub what: ThreadIssueTarget,
    pub why: serde_json::Value,
    pub user_addressed: bool,
    pub moderator_verified: bool,
    pub verdict: ThreadIssueVerdict,
    pub created_at: DateTime<Utc>,
}

impl DBThreadIssue {
    pub async fn sync_project_verdicts(
        context: &ThreadIssueContext<'_>,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<Vec<Self>> {
        let rows = sqlx::query!(
            r#"
			SELECT
				id,
				thread_id,
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
            let stored_verdict = match row.verdict.as_str() {
                "open" => ThreadIssueVerdict::Open,
                "addressed" => ThreadIssueVerdict::Addressed,
                "resolved" => ThreadIssueVerdict::Resolved,
                verdict => {
                    return Err(eyre!(
                        "thread issue `{}` has invalid verdict `{verdict}`",
                        row.id
                    ));
                }
            };
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

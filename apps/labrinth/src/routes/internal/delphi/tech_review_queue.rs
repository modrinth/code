//! Maintains explicit project membership in the technical review queue.
//!
//! Queue membership is represented by a row in `delphi_tech_review_queue`.
//! Enter and exit thread messages are emitted only when an insert or delete
//! actually changes that membership, in the same transaction.

use itertools::Itertools;

use crate::{
    database::{
        PgTransaction,
        models::{DBProjectId, DBThreadId, thread_item::ThreadMessageBuilder},
    },
    models::threads::MessageBody,
    routes::ApiError,
    util::error::Context,
};

#[derive(Debug, Clone, Copy)]
pub enum TechReviewRemovalReason {
    RulesChanged,
    ScanCompleted,
    FileDeleted,
}

pub async fn add_projects(
    project_ids: &[DBProjectId],
    txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
    let project_ids = project_ids.iter().copied().unique().collect::<Vec<_>>();
    if project_ids.is_empty() {
        return Ok(());
    }

    let rows = sqlx::query!(
        r#"
        WITH inserted AS (
            INSERT INTO delphi_tech_review_queue (project_id)
            SELECT unnest($1::bigint[])
            ON CONFLICT (project_id) DO NOTHING
            RETURNING project_id
        )
        SELECT
            inserted.project_id AS "project_id!: DBProjectId",
            (
                SELECT thread.id
                FROM threads thread
                WHERE thread.mod_id = inserted.project_id
                ORDER BY thread.id
                LIMIT 1
            ) AS "thread_id!: DBThreadId"
        FROM inserted
        "#,
        &project_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
    )
    .fetch_all(&mut *txn)
    .await
    .wrap_internal_err("failed to add projects to technical review queue")?;

    for row in rows {
        ThreadMessageBuilder {
            author_id: None,
            body: MessageBody::TechReviewEntered,
            thread_id: row.thread_id,
            hide_identity: false,
        }
        .insert(txn)
        .await
        .wrap_internal_err("failed to add entering technical review message")?;
    }

    Ok(())
}

pub async fn remove_projects(
    project_ids: &[DBProjectId],
    reason: TechReviewRemovalReason,
    txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
    let project_ids = project_ids.iter().copied().unique().collect::<Vec<_>>();
    if project_ids.is_empty() {
        return Ok(());
    }

    let rows = sqlx::query!(
        r#"
        WITH removed AS (
            DELETE FROM delphi_tech_review_queue
            WHERE project_id = ANY($1::bigint[])
            RETURNING project_id
        )
        SELECT
            removed.project_id AS "project_id!: DBProjectId",
            (
                SELECT thread.id
                FROM threads thread
                WHERE thread.mod_id = removed.project_id
                ORDER BY thread.id
                LIMIT 1
            ) AS "thread_id!: DBThreadId"
        FROM removed
        "#,
        &project_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
    )
    .fetch_all(&mut *txn)
    .await
    .wrap_internal_err(
        "failed to remove projects from technical review queue",
    )?;

    let body = match reason {
        TechReviewRemovalReason::RulesChanged
        | TechReviewRemovalReason::ScanCompleted => {
            MessageBody::TechReviewExited
        }
        TechReviewRemovalReason::FileDeleted => {
            MessageBody::TechReviewExitFileDeleted
        }
    };

    for row in rows {
        ThreadMessageBuilder {
            author_id: None,
            body: body.clone(),
            thread_id: row.thread_id,
            hide_identity: false,
        }
        .insert(txn)
        .await
        .wrap_internal_err("failed to add exiting technical review message")?;
    }

    Ok(())
}

pub async fn sync_projects(
    project_ids: &[DBProjectId],
    removal_reason: TechReviewRemovalReason,
    txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
    let project_ids = project_ids.iter().copied().unique().collect::<Vec<_>>();
    if project_ids.is_empty() {
        return Ok(());
    }

    let rows = sqlx::query!(
        r#"
        WITH current_details AS (
            SELECT
                detail.project_id,
                detail.status,
                detail.severity
            FROM delphi_issue_details_with_statuses detail
            INNER JOIN delphi_report_issues issue ON issue.id = detail.issue_id
            INNER JOIN delphi_reports report
                ON report.id = issue.report_id
                AND NOT EXISTS (
                    SELECT 1
                    FROM delphi_reports newer_report
                    WHERE
                        newer_report.file_id = report.file_id
                        AND newer_report.delphi_version > report.delphi_version
                )
        )
        SELECT
            requested.project_id AS "project_id!: DBProjectId",
            BOOL_OR(current_details.project_id IS NOT NULL) AS "has_details!",
            COALESCE(
                BOOL_OR(
                    current_details.status IN ('pending', 'unsafe')
                    AND current_details.severity != 'hidden'
                ),
                FALSE
            ) AS "needs_review!"
        FROM unnest($1::bigint[]) AS requested(project_id)
        LEFT JOIN current_details
            ON current_details.project_id = requested.project_id
        GROUP BY requested.project_id
        "#,
        &project_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
    )
    .fetch_all(&mut *txn)
    .await
    .wrap_internal_err(
        "failed to determine technical review queue membership",
    )?;

    add_projects(
        &rows
            .iter()
            .filter(|row| row.needs_review)
            .map(|row| row.project_id)
            .collect::<Vec<_>>(),
        txn,
    )
    .await?;

    remove_projects(
        &rows
            .into_iter()
            .filter(|row| !row.has_details)
            .map(|row| row.project_id)
            .collect::<Vec<_>>(),
        removal_reason,
        txn,
    )
    .await
}

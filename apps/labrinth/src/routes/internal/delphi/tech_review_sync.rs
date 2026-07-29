//! Synchronizes moderation thread messages and dummy queue blockers with the
//! current computed tech review state of affected projects.
//!
//! When a project has a Delphi report submitted for it, or when a moderator
//! updates one of its issue details' rows (like flagging a detail as *globally*
//! safe or unsafe), we will need to recheck if the project still belongs in the
//! tech review queue or if it needs to exit now.
//!
//! Side-note: "entering the queue" or "exiting the queue" right now just means
//! adding a new message to the project's moderation thread which indicates if
//! it entered/exited. In the future this should be replaced with a more proper
//! audit log table, or "is project currently in tech review" table.
//!
//! A project is considered to need tech review when it has at least one
//! non-dummy issue detail whose effective status is pending or unsafe, or when
//! it already has a dummy pending detail blocking the final review submission.
//! Effective status is just the local detail's verdict (from
//! `delphi_issue_detail_verdicts`), or if it's null then the global verdict for
//! the same `drid.key` (from `delphi_global_detail_verdicts`).
//!
//! Some examples of how this behavior manifests: let's assume you have projects
//! _A_ and _B_ currently in tech review. They each have one (unresolved) issue
//! detail with key _K_.
//! - If you mark _K_ on _A_ as locally safe/unsafe, then _A_ is fully resolved,
//!   but we still have the `__dummy` detail, which means it's still in the
//!   queue until the moderator submits the actual report. _B_ is entirely
//!   unaffected.
//! - If you mark _K_ on _A_ as globally safe, then _A_ and _B_ both get fully
//!   resolved, but both still have the `__dummy` detail, so they also still
//!   need the final report to be submitted by the moderator.
//!
//!   In practice, this means that some projects may have e.g. "100/100 traces
//!   are safe" reported, but they will just be waiting for final moderator
//!   approval.
//!
//! The logic for checking whether a project is now in tech review or not, and
//! sending the appropriate message, is complex! That's why this module exists:
//! to act as a single chokepoint which (correctly) syncs all the state, instead
//! of having each mutation run its own ad-hoc update logic.

use itertools::Itertools;

use crate::{
    database::{
        PgTransaction,
        models::{
            DBProjectId, DBThreadId, DelphiReportId,
            delphi_report_item::DelphiVerdict,
            thread_item::ThreadMessageBuilder,
        },
    },
    models::threads::MessageBody,
    routes::ApiError,
    util::error::Context,
};

const DUMMY_ISSUE_TYPE: &str = "__dummy";

#[derive(Debug, Clone, Copy)]
pub enum TechReviewExitReason {
    Resolved,
    FileDeleted,
}

struct ProjectTechReviewState {
    has_pending_detail: bool,
    has_unsafe_detail: bool,
    has_dummy: bool,
    thread_id: Option<DBThreadId>,
    report_id: Option<DelphiReportId>,
    last_tech_review_message_type: Option<String>,
}

pub async fn sync_project_tech_review_state(
    project_ids: &[DBProjectId],
    exit_reason: TechReviewExitReason,
    txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
    let project_ids = project_ids.iter().copied().unique().collect::<Vec<_>>();
    if project_ids.is_empty() {
        return Ok(());
    }

    let project_ids_raw = project_ids.iter().map(|id| id.0).collect::<Vec<_>>();
    let tech_review_message_types = tech_review_message_types();

    let rows = sqlx::query!(
        r#"
        WITH project_ids AS (
            SELECT unnest($1::bigint[]) AS project_id
        )
        SELECT
            p.project_id AS "project_id!: DBProjectId",
            EXISTS(
                SELECT 1
                FROM delphi_issue_details_with_statuses didws
                INNER JOIN delphi_report_issues dri ON dri.id = didws.issue_id
                WHERE
                    didws.project_id = p.project_id
                    AND didws.status = 'pending'
                    AND dri.issue_type != $3
            ) AS "has_pending_detail!",
            EXISTS(
                SELECT 1
                FROM delphi_issue_details_with_statuses didws
                INNER JOIN delphi_report_issues dri ON dri.id = didws.issue_id
                WHERE
                    didws.project_id = p.project_id
                    AND didws.status = 'unsafe'
                    AND dri.issue_type != $3
            ) AS "has_unsafe_detail!",
            EXISTS(
                SELECT 1
                FROM delphi_issue_details_with_statuses didws
                INNER JOIN delphi_report_issues dri ON dri.id = didws.issue_id
                WHERE
                    didws.project_id = p.project_id
                    AND didws.status = 'pending'
                    AND dri.issue_type = $3
            ) AS "has_dummy!",
            (
                SELECT t.id
                FROM threads t
                WHERE t.mod_id = p.project_id
                ORDER BY t.id
                LIMIT 1
            ) AS "thread_id: DBThreadId",
            (
                SELECT tm.body->>'type'
                FROM threads t
                INNER JOIN threads_messages tm ON tm.thread_id = t.id
                WHERE
                    t.mod_id = p.project_id
                    AND tm.body->>'type' = ANY($2::text[])
                ORDER BY tm.created DESC, tm.id DESC
                LIMIT 1
            ) AS "last_tech_review_message_type",
            (
                SELECT dr.id
                FROM versions v
                INNER JOIN files f ON f.version_id = v.id
                INNER JOIN delphi_reports dr ON dr.file_id = f.id
                WHERE v.mod_id = p.project_id
                ORDER BY dr.created DESC, dr.id DESC
                LIMIT 1
            ) AS "report_id: DelphiReportId"
        FROM project_ids p
        "#,
        &project_ids_raw,
        &tech_review_message_types,
        DUMMY_ISSUE_TYPE,
    )
    .fetch_all(&mut *txn)
    .await
    .wrap_internal_err("failed to fetch project tech review state")?;

    for row in rows {
        let state = ProjectTechReviewState {
            has_pending_detail: row.has_pending_detail,
            has_unsafe_detail: row.has_unsafe_detail,
            has_dummy: row.has_dummy,
            thread_id: row.thread_id,
            report_id: row.report_id,
            last_tech_review_message_type: row.last_tech_review_message_type,
        };

        sync_one_project_tech_review_state(state, exit_reason, txn).await?;
    }

    Ok(())
}

pub async fn sync_detail_key_tech_review_state(
    detail_keys: &[String],
    exit_reason: TechReviewExitReason,
    txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
    let detail_keys = detail_keys.iter().cloned().unique().collect::<Vec<_>>();

    if detail_keys.is_empty() {
        return Ok(());
    }

    let rows = sqlx::query!(
        r#"
        SELECT DISTINCT didws.project_id AS "project_id!: DBProjectId"
        FROM delphi_issue_details_with_statuses didws
        WHERE didws.key = ANY($1::text[])
        "#,
        &detail_keys,
    )
    .fetch_all(&mut *txn)
    .await
    .wrap_internal_err("failed to fetch projects affected by detail keys")?;

    let project_ids = rows
        .into_iter()
        .map(|row| row.project_id)
        .collect::<Vec<_>>();

    sync_project_tech_review_state(&project_ids, exit_reason, txn).await
}

pub async fn sync_deleted_project_tech_review_exit(
    project_id: DBProjectId,
    txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
    let tech_review_message_types = tech_review_message_types();

    let row = sqlx::query!(
        r#"
        SELECT
            (
                SELECT t.id
                FROM threads t
                WHERE t.mod_id = $1
                ORDER BY t.id
                LIMIT 1
            ) AS "thread_id: DBThreadId",
            (
                SELECT tm.body->>'type'
                FROM threads t
                INNER JOIN threads_messages tm ON tm.thread_id = t.id
                WHERE
                    t.mod_id = $1
                    AND tm.body->>'type' = ANY($2::text[])
                ORDER BY tm.created DESC, tm.id DESC
                LIMIT 1
            ) AS "last_tech_review_message_type"
        "#,
        project_id as DBProjectId,
        &tech_review_message_types,
    )
    .fetch_one(&mut *txn)
    .await
    .wrap_internal_err("failed to fetch deleted project tech review state")?;

    if let Some(thread_id) = row.thread_id
        && should_send_exit(row.last_tech_review_message_type.as_deref())
    {
        insert_exit_message(thread_id, TechReviewExitReason::FileDeleted, txn)
            .await?;
    }

    Ok(())
}

async fn sync_one_project_tech_review_state(
    state: ProjectTechReviewState,
    exit_reason: TechReviewExitReason,
    txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
    let needs_tech_review =
        state.has_pending_detail || state.has_unsafe_detail || state.has_dummy;

    if needs_tech_review {
        if (state.has_pending_detail || state.has_unsafe_detail)
            && !state.has_dummy
            && let Some(report_id) = state.report_id
        {
            // TODO: Currently, the queue query determines whether a project is
            // in tech review by checking whether it has any pending issue
            // details. If all visible issue details are marked safe or unsafe
            // before the final report is submitted, the project would otherwise
            // leave the tech review queue without a final tech review verdict
            // message.
            //
            // This should be replaced with explicit tech review state, such as
            // an append-only project tech review event table where the latest
            // enter/exit event is the current state. Until then, this dummy
            // issue detail acts as the pending queue blocker.
            ensure_dummy_issue_detail(report_id, txn).await?;
        }

        if let Some(thread_id) = state.thread_id
            && state.last_tech_review_message_type.as_deref()
                != Some(MessageBody::TechReviewEntered.as_ref())
        {
            ThreadMessageBuilder {
                author_id: None,
                body: MessageBody::TechReviewEntered,
                thread_id,
                hide_identity: false,
            }
            .insert(txn)
            .await
            .wrap_internal_err("failed to add entering tech review message")?;
        }

        return Ok(());
    }

    if matches!(exit_reason, TechReviewExitReason::Resolved)
        && state.last_tech_review_message_type.as_deref()
            == Some(MessageBody::TechReviewEntered.as_ref())
    {
        if let Some(report_id) = state.report_id {
            ensure_dummy_issue_detail(report_id, txn).await?;
        }

        return Ok(());
    }

    if let Some(thread_id) = state.thread_id
        && should_send_exit(state.last_tech_review_message_type.as_deref())
    {
        insert_exit_message(thread_id, exit_reason, txn).await?;
    }

    Ok(())
}

fn should_send_exit(last_tech_review_message_type: Option<&str>) -> bool {
    matches!(last_tech_review_message_type, Some(message_type) if !matches!(
        message_type,
        message_type if message_type == MessageBody::TechReviewExited.as_ref()
            || message_type == MessageBody::TechReviewExitFileDeleted.as_ref()
            || message_type == tech_review_completed_message_type()
    ))
}

async fn insert_exit_message(
    thread_id: DBThreadId,
    exit_reason: TechReviewExitReason,
    txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
    let body = match exit_reason {
        TechReviewExitReason::Resolved => MessageBody::TechReviewExited,
        TechReviewExitReason::FileDeleted => {
            MessageBody::TechReviewExitFileDeleted
        }
    };

    ThreadMessageBuilder {
        author_id: None,
        body,
        thread_id,
        hide_identity: false,
    }
    .insert(txn)
    .await
    .wrap_internal_err("failed to add exiting tech review message")?;

    Ok(())
}

async fn ensure_dummy_issue_detail(
    report_id: DelphiReportId,
    txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
    sqlx::query!(
        r#"
        WITH dummy_issue AS (
            INSERT INTO delphi_report_issues (report_id, issue_type)
            VALUES ($1, $2)
            ON CONFLICT (report_id, issue_type)
            DO UPDATE SET issue_type = EXCLUDED.issue_type
            RETURNING id
        )
        INSERT INTO delphi_report_issue_details (
            issue_id,
            key,
            jar,
            file_path,
            decompiled_source,
            data,
            severity
        )
        SELECT
            id,
            '',
            NULL,
            '',
            NULL,
            '{}'::jsonb,
            'low'::delphi_severity
        FROM dummy_issue
        WHERE NOT EXISTS (
            SELECT 1
            FROM delphi_report_issue_details drid
            WHERE drid.issue_id = dummy_issue.id
        )
        "#,
        report_id as DelphiReportId,
        DUMMY_ISSUE_TYPE,
    )
    .execute(&mut *txn)
    .await
    .wrap_internal_err("failed to ensure dummy Delphi report issue detail")?;

    Ok(())
}

fn tech_review_message_types() -> Vec<String> {
    [
        MessageBody::TechReviewEntered.as_ref(),
        MessageBody::TechReviewExited.as_ref(),
        MessageBody::TechReviewExitFileDeleted.as_ref(),
        tech_review_completed_message_type(),
    ]
    .into_iter()
    .map(|message_type| message_type.to_string())
    .collect()
}

fn tech_review_completed_message_type() -> &'static str {
    MessageBody::TechReview {
        verdict: DelphiVerdict::Safe,
    }
    .as_ref()
}

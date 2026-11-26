use std::collections::BTreeMap;

use actix_web::{HttpRequest, query, web};
use chrono::{DateTime, Utc};
use xredis::RedisPool;

use super::{
    MAX_PROJECTS_PER_PAGE, ModerationQueueProject, default_count,
    listed_version_statuses, row_to_queue_project,
};
use crate::{
    auth::get_user_from_headers,
    database::ReadOnlyPgPool,
    models::pats::Scopes,
    queue::session::AuthQueue,
    routes::ApiError,
    util::error::Context,
    workflow::project_flag::approve::{self, ApprovalQueueEntry},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(query_queue);
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(default)]
pub struct QueryQueue {
    pub queue_kind: QueueKind,
    pub project_types: Vec<ProjectType>,
    pub submission_kind: Option<SubmissionKind>,
    pub project_flags: Vec<String>,
    pub order: QueueOrder,
    /// Number of projects to return; defaults to 100 and is capped at 200. Zero returns only the total.
    #[serde(default = "default_count")]
    pub limit: u16,
    pub offset: u32,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(transparent)]
pub struct ProjectType(pub String);

#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueueKind {
    #[default]
    InReview,
    ScheduledForApproval,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionKind {
    New,
    Resubmission,
}

#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueueOrder {
    #[default]
    OldestFirst,
    NewestFirst,
}

impl Default for QueryQueue {
    fn default() -> Self {
        Self {
            queue_kind: QueueKind::default(),
            project_types: Vec::new(),
            submission_kind: None,
            project_flags: Vec::new(),
            order: QueueOrder::default(),
            limit: default_count(),
            offset: 0,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QueueProject {
    #[serde(flatten)]
    pub project: ModerationQueueProject,
    pub scheduled_for: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QueueResponse {
    pub total: i64,
    pub projects: Vec<QueueProject>,
}

/// Query moderation cards. Empty filter lists select all; entries within each list are ORed.
/// Scheduled membership and deadlines use eventually consistent active Temporal approval metadata.
#[query("/queue")]
async fn query_queue(
    req: HttpRequest,
    query: web::Json<QueryQueue>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    temporal: web::Data<crate::util::temporal::TemporalClient>,
) -> Result<web::Json<QueueResponse>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .auth_context("authenticating moderator")?;
    let query = query.into_inner();
    let scheduled = matches!(query.queue_kind, QueueKind::ScheduledForApproval);
    let schedules = if scheduled {
        scheduled_membership(
            approve::queue(&temporal.client)
                .await
                .internal_context("listing scheduled approval workflows")?,
        )
    } else {
        BTreeMap::new()
    };
    let (project_ids, deadlines): (Vec<_>, Vec<_>) =
        schedules.into_iter().unzip();
    let types: Vec<String> =
        query.project_types.into_iter().map(|t| t.0).collect();
    let submission = query.submission_kind.map(|kind| match kind {
        SubmissionKind::New => "new",
        SubmissionKind::Resubmission => "resubmission",
    });
    let newest = matches!(query.order, QueueOrder::NewestFirst);
    let statuses = listed_version_statuses();
    let rows = sqlx::query!(
        r#"
        WITH schedules AS (
            SELECT id, scheduled_for
            FROM UNNEST($9::bigint[], $10::timestamptz[]) AS s(id, scheduled_for)
        ), candidates AS (
            SELECT m.*, s.scheduled_for,
                CASE WHEN jsonb_typeof(m.components -> 'minecraft_server') = 'object'
                    THEN array_append(array_remove(COALESCE(t.types, ARRAY[]::text[]), 'modpack'), 'minecraft_java_server')
                    ELSE COALESCE(t.types, ARRAY[]::text[]) END AS project_types
            FROM mods m
            LEFT JOIN schedules s ON s.id = m.id
            LEFT JOIN LATERAL (
                SELECT array_agg(DISTINCT pt.name::text) AS types
                FROM versions v
                JOIN loaders_versions lv ON lv.version_id = v.id
                JOIN loaders_project_types lpt ON lpt.joining_loader_id = lv.loader_id
                JOIN project_types pt ON pt.id = lpt.joining_project_type_id
                WHERE v.mod_id = m.id AND v.status = ANY($2)
            ) t ON true
            WHERE m.in_review AND (NOT $1::boolean OR s.id IS NOT NULL)
                AND ($3::text IS NULL
                    OR ($3 = 'new' AND m.queued = m.initial_queued)
                    OR ($3 = 'resubmission' AND m.queued > m.initial_queued))
                AND (cardinality($4::text[]) = 0 OR EXISTS (
                    SELECT 1 FROM project_flags f WHERE f.project_id = m.id
                        AND (f.active OR (f.what->>'retain_when_resolved')::boolean = true)
                        AND f.what->>'key' = ANY($4)
                ))
        ), filtered AS (
            SELECT *, CASE WHEN $1 THEN scheduled_for ELSE queued END AS sort_time
            FROM candidates
            WHERE cardinality($5::text[]) = 0 OR project_types && $5
        ), total AS (
            SELECT count(*) AS total_count FROM filtered
        ), page AS (
            SELECT * FROM filtered
            ORDER BY CASE WHEN $6 THEN sort_time END DESC NULLS LAST,
                CASE WHEN NOT $6 THEN sort_time END ASC NULLS LAST, id ASC
            LIMIT $7 OFFSET $8
        ), cards AS (
            SELECT p.*, o.name AS organization_name, o.icon_url AS organization_icon_url,
                owner.user_id AS owner_id, owner.username AS owner_name, owner.avatar_url AS owner_icon_url,
                (SELECT count(*) FROM versions v JOIN dependencies d ON d.dependent_id = v.id
                    WHERE v.mod_id = p.id AND d.dependency_file_name IS NOT NULL) AS external_dependencies_count
            FROM page p
            LEFT JOIN organizations o ON o.id = p.organization_id
            LEFT JOIN LATERAL (
                SELECT tm.user_id, u.username, u.avatar_url
                FROM team_members tm JOIN users u ON u.id = tm.user_id
                WHERE tm.team_id = p.team_id AND tm.is_owner
                ORDER BY tm.ordering ASC LIMIT 1
            ) owner ON p.organization_id IS NULL
        )
        SELECT total.total_count AS "total_count!",
                cards.id AS "id?",
                cards.slug AS "slug?",
                cards.name AS "name?",
                cards.summary AS "summary?",
                cards.icon_url AS "icon_url?",
                cards.visibility AS "visibility?",
                cards.in_review AS "in_review?",
                cards.review_outcome AS "review_outcome?",
                cards.queued AS "queued?",
                cards.initial_queued AS "initial_queued?",
                cards.published AS "published?",
                cards.updated AS "updated?",
                cards.organization_id AS "organization_id?",
                cards.organization_name AS "organization_name?",
                cards.organization_icon_url AS "organization_icon_url?",
                cards.owner_id AS "owner_id?",
                cards.owner_name AS "owner_name?",
                cards.owner_icon_url AS "owner_icon_url?",
                cards.project_types AS "project_types?: Vec<String>",
                cards.external_dependencies_count AS "external_dependencies_count?",
            cards.scheduled_for AS "scheduled_for?"
        FROM total LEFT JOIN cards ON true
        ORDER BY CASE WHEN $6 THEN cards.sort_time END DESC NULLS LAST,
            CASE WHEN NOT $6 THEN cards.sort_time END ASC NULLS LAST, cards.id ASC
        "#,
        scheduled, &statuses, submission,
        &query.project_flags, &types, newest,
        i64::from(query.limit.min(MAX_PROJECTS_PER_PAGE)), i64::from(query.offset),
        &project_ids, &deadlines,
    ).fetch_all(&***ro_pool).await.internal_context("querying moderation queue")?;
    let total = rows.first().map_or(0, |row| row.total_count);
    let mut projects = Vec::new();
    for row in rows {
        if let Some(project) = row_to_queue_project(
            row.id,
            row.slug,
            row.name,
            row.summary,
            row.icon_url,
            row.visibility,
            row.in_review,
            row.review_outcome,
            row.queued,
            row.initial_queued,
            row.published,
            row.updated,
            row.organization_id,
            row.organization_name,
            row.organization_icon_url,
            row.owner_id,
            row.owner_name,
            row.owner_icon_url,
            row.project_types,
            row.external_dependencies_count,
        )? {
            projects.push(QueueProject {
                project,
                scheduled_for: row.scheduled_for,
            });
        }
    }
    Ok(web::Json(QueueResponse { total, projects }))
}

fn scheduled_membership(
    entries: Vec<ApprovalQueueEntry>,
) -> BTreeMap<i64, DateTime<Utc>> {
    let mut schedules = BTreeMap::new();
    for entry in entries {
        let Some(state) = entry.state else { continue };
        let Some(project_id) = state.project_id else {
            tracing::warn!(
                workflow_id = entry.workflow_id,
                "approval metadata is missing project id"
            );
            continue;
        };
        if state.finished || state.auto_approve_blocked || !state.eligible {
            continue;
        }
        let Some(deadline) = state.deadline else {
            continue;
        };
        schedules
            .entry(project_id.0 as i64)
            .and_modify(|existing: &mut DateTime<Utc>| {
                *existing = (*existing).min(deadline)
            })
            .or_insert(deadline);
    }
    schedules
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::v3::ids::ProjectId,
        workflow::project_flag::approve::ApprovalStatus,
    };

    fn entry(id: u64, deadline: i64) -> ApprovalQueueEntry {
        ApprovalQueueEntry {
            workflow_id: "opaque-sdk-workflow-id".into(),
            state: Some(ApprovalStatus {
                project_id: Some(ProjectId(id)),
                auto_approve_blocked: false,
                eligible: true,
                finished: false,
                deadline: Some(DateTime::from_timestamp(deadline, 0).unwrap()),
                phase: "waiting".into(),
            }),
        }
    }

    #[test]
    fn membership_uses_explicit_metadata_and_deduplicates_deterministically() {
        let mut rechecking = entry(2, 1);
        rechecking.state.as_mut().unwrap().phase = "scanning".into();
        let entries = vec![entry(1, 20), rechecking, entry(1, 10)];
        let schedules = scheduled_membership(entries);
        assert_eq!(schedules.keys().copied().collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(schedules[&1].timestamp(), 10);
        assert_eq!(schedules[&2].timestamp(), 1);
        assert_eq!(
            scheduled_membership(vec![entry(1, 10), entry(1, 20)])[&1]
                .timestamp(),
            10
        );
    }

    #[test]
    fn membership_excludes_blocked_ineligible_finished_and_missing_metadata() {
        let mut entries = Vec::new();
        for id in 1..=6 {
            let mut item = entry(id, 10);
            let state = item.state.as_mut().unwrap();
            match id {
                1 => state.auto_approve_blocked = true,
                2 => state.eligible = false,
                3 => state.finished = true,
                4 => state.deadline = None,
                5 => state.project_id = None,
                _ => item.state = None,
            }
            entries.push(item);
        }
        assert!(scheduled_membership(entries).is_empty());
    }

    #[actix_rt::test]
    async fn sql_filters_counts_and_sorts_before_pagination() {
        use crate::test::{
            api_v3::ApiV3,
            environment::{TestEnvironment, with_test_environment},
        };
        with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
            for newest in [false, true] {
                for (limit, offset, expected) in [(1_i64, 0_i64, Some(if newest { 2 } else { 1 })), (1, 1, Some(if newest { 1 } else { 2 })), (0, 0, None), (1, 2, None)] {
                    let rows = sqlx::query!(
                        r#"
                        WITH schedules AS (
                            SELECT * FROM UNNEST(ARRAY[1,2,3,4,5,6]::bigint[],
                                ARRAY['2026-01-01','2026-02-01','2025-01-01','2025-01-01','2025-01-01','2025-01-01']::timestamptz[])
                                AS s(id, scheduled_for)
                        ), projects(id, in_review, project_types, submission, flag) AS (
                            VALUES (1::bigint, true, ARRAY['mod'], 'new', 'test'),
                                (2, true, ARRAY['mod'], 'new', 'test'),
                                (3, false, ARRAY['mod'], 'new', 'test'),
                                (4, true, ARRAY['modpack'], 'new', 'test'),
                                (5, true, ARRAY['mod'], 'resubmission', 'test'),
                                (6, true, ARRAY['mod'], 'new', 'other')
                        ), filtered AS (
                            SELECT p.id, s.scheduled_for FROM projects p JOIN schedules s USING (id)
                            WHERE p.in_review AND p.project_types && ARRAY['mod']
                                AND p.submission = 'new' AND p.flag = 'test'
                        ), total AS (
                            SELECT count(*) AS count FROM filtered
                        ), page AS (
                            SELECT * FROM filtered
                            ORDER BY CASE WHEN $1 THEN scheduled_for END DESC,
                                CASE WHEN NOT $1 THEN scheduled_for END ASC, id ASC
                            LIMIT $2 OFFSET $3
                        )
                        SELECT total.count AS "total!", page.id AS "id?"
                        FROM total LEFT JOIN page ON true
                        "#,
                        newest, limit, offset,
                    ).fetch_all(&*env.db.ro_pool).await.unwrap();
                    assert_eq!(rows.len(), 1);
                    assert_eq!(rows[0].total, 2);
                    assert_eq!(rows[0].id, expected);
                }
            }
        }).await;
    }

    #[test]
    fn approval_metadata_round_trips_typed_project_id() {
        let state = entry(123, 10).state.unwrap();
        let json = serde_json::to_value(&state).unwrap();
        assert_eq!(
            json["project_id"],
            serde_json::to_value(ProjectId(123)).unwrap()
        );
        let decoded: ApprovalStatus = serde_json::from_value(json).unwrap();
        assert_eq!(decoded.project_id, Some(ProjectId(123)));
    }
}

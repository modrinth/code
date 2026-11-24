use actix_web::{HttpRequest, query, web};
use chrono::{DateTime, Utc};
use xredis::RedisPool;

use super::{
    MAX_PROJECTS_PER_PAGE, ModerationQueueProject, default_count,
    listed_version_statuses, row_to_queue_project,
};
use crate::{
    auth::get_user_from_headers, database::ReadOnlyPgPool,
    models::pats::Scopes, queue::session::AuthQueue, routes::ApiError,
    util::error::Context,
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
/// Scheduled membership and deadlines use the latest scheduling or cancellation event in project history.
#[query("/queue")]
async fn query_queue(
    req: HttpRequest,
    query: web::Json<QueryQueue>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
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
            SELECT DISTINCT ON (h.project_id) h.project_id AS id,
                h.delta->>'kind' AS kind,
                (h.delta->>'scheduled_for')::timestamptz AS scheduled_for
            FROM project_history h
            JOIN mods m ON m.id = h.project_id AND m.in_review
            WHERE $1::boolean AND h.delta->>'kind' IN ('auto_approval_scheduled', 'auto_approval_cancelled')
            ORDER BY h.project_id, h.occurred_at DESC, h.id DESC
        ), candidates AS (
            SELECT m.*, s.scheduled_for,
                CASE WHEN jsonb_typeof(m.components -> 'minecraft_server') = 'object'
                    THEN array_append(array_remove(COALESCE(t.types, ARRAY[]::text[]), 'modpack'), 'minecraft_java_server')
                    ELSE COALESCE(t.types, ARRAY[]::text[]) END AS project_types
            FROM mods m
            LEFT JOIN schedules s ON s.id = m.id AND s.kind = 'auto_approval_scheduled'
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

#[cfg(test)]
mod tests {
    use crate::test::{
        api_v3::ApiV3,
        environment::{TestEnvironment, with_test_environment},
    };

    #[actix_rt::test]
    async fn latest_approval_event_controls_scheduled_membership() {
        with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
            let ids = sqlx::query_scalar!(
                r#"
                WITH history (project_id, id, occurred_at, kind) AS (
                    VALUES
                        (1, 1, 1, 'auto_approval_scheduled'),
                        (1, 2, 2, 'auto_approval_cancelled'),
                        (2, 3, 1, 'auto_approval_scheduled'),
                        (2, 4, 2, 'auto_approval_cancelled'),
                        (2, 5, 3, 'auto_approval_scheduled'),
                        (3, 6, 1, 'auto_approval_scheduled'),
                        (3, 7, 1, 'auto_approval_cancelled'),
                        (4, 8, 1, 'auto_approval_cancelled'),
                        (4, 9, 1, 'auto_approval_scheduled'),
                        (5, 10, 1, 'auto_approval_cancelled')
                ), latest AS (
                    SELECT DISTINCT ON (project_id) project_id, kind
                    FROM history
                    WHERE kind IN ('auto_approval_scheduled', 'auto_approval_cancelled')
                    ORDER BY project_id, occurred_at DESC, id DESC
                )
                SELECT project_id AS "project_id!"
                FROM latest WHERE kind = 'auto_approval_scheduled'
                ORDER BY project_id
                "#
            )
            .fetch_all(&*env.db.ro_pool)
            .await
            .unwrap();
            assert_eq!(ids, vec![2, 4]);
        })
        .await;
    }
}

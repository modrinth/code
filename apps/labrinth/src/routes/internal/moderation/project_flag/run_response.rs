use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::models::ProjectFlagRunId;
use crate::database::models::project_flag_item::{
    DBProjectFlagRun, DBProjectFlagRunResult, ProjectFlagRunCounts,
    ProjectFlagRunOutput, ProjectFlagRunStatus, ProjectFlagRunUser,
};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunResponseStatus {
    Done,
    Running { num_queued: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectFlagRunResponse {
    pub id: ProjectFlagRunId,
    pub ruleset_id: Option<uuid::Uuid>,
    pub status: RunResponseStatus,
    /// Projects originally queued, including projects not yet evaluated.
    pub total_projects: i64,
    /// Projects with a persisted evaluation result, including failed evaluations.
    pub num_scanned: i64,
    /// Successfully evaluated projects with at least one flag.
    pub num_flagged: i64,
    /// Total emitted flags, including partial results from failed evaluations.
    pub num_flags: i64,
    /// Successfully evaluated projects with no flags (would be approved).
    pub num_no_flags: i64,
    /// Projects with a project-level or rule-level error, even if also flagged.
    pub num_errors: i64,
    pub started_by: Option<ProjectFlagRunUser>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
    pub applied_by: Option<ProjectFlagRunUser>,
    pub output: Option<ProjectFlagRunOutput>,
    pub error: Option<String>,
}

impl ProjectFlagRunResponse {
    pub async fn from_runs(
        runs: Vec<DBProjectFlagRun>,
        pool: &crate::database::PgPool,
    ) -> eyre::Result<Vec<Self>> {
        let ids = runs.iter().map(|run| run.id).collect::<Vec<_>>();
        let mut counts = DBProjectFlagRunResult::counts(&ids, pool).await?;
        Ok(runs
            .into_iter()
            .map(|run| {
                let counts = counts.remove(&run.id).unwrap_or_default();
                let num_queued =
                    (run.total_projects - counts.num_scanned).max(0);
                Self::from_run(run, num_queued, counts)
            })
            .collect())
    }

    fn from_run(
        run: DBProjectFlagRun,
        num_queued: i64,
        counts: ProjectFlagRunCounts,
    ) -> Self {
        Self {
            id: run.id,
            ruleset_id: run.ruleset_id,
            status: match run.status {
                ProjectFlagRunStatus::Done => RunResponseStatus::Done,
                ProjectFlagRunStatus::Running => {
                    RunResponseStatus::Running { num_queued }
                }
            },
            total_projects: run.total_projects,
            num_scanned: counts.num_scanned,
            num_flagged: counts.num_flagged,
            num_flags: counts.num_flags,
            num_no_flags: counts.num_no_flags,
            num_errors: counts.num_errors,
            started_by: run.started_by,
            started_at: run.started_at,
            completed_at: run.completed_at,
            applied_at: run.applied_at,
            applied_by: run.applied_by,
            output: run.output,
            error: run.error,
        }
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::models::ProjectFlagRunId;
use crate::database::models::project_flag_item::{
    DBProjectFlagRun, ProjectFlagRunOutput, ProjectFlagRunStatus,
    ProjectFlagRunUser,
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
    pub status: RunResponseStatus,
    pub total_projects: i64,
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
    pub fn from_run(run: DBProjectFlagRun, num_queued: i64) -> Self {
        Self {
            id: run.id,
            status: match run.status {
                ProjectFlagRunStatus::Done => RunResponseStatus::Done,
                ProjectFlagRunStatus::Running => {
                    RunResponseStatus::Running { num_queued }
                }
            },
            total_projects: run.total_projects,
            num_errors: run.num_errors,
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

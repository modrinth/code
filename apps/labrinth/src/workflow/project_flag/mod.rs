mod model_scan;

use std::{sync::Arc, time::Duration};

use eyre::{Result, WrapErr};
use temporalio_client::{
    WorkflowIdConflictPolicy, WorkflowIdReusePolicy, WorkflowStartOptions,
};
use temporalio_macros::{activities, workflow, workflow_methods};
use temporalio_sdk::{
    ActivityOptions, WorkerOptions, WorkflowContext, WorkflowResult,
    activities::{ActivityContext, ActivityError},
};

use crate::env::ProjectFlags;
use crate::models::v3::ids::ProjectId;
use crate::util::temporal::TemporalClient;

pub const PROJECT_FLAG_TASK_QUEUE: &str = "project-flags";

pub fn project_flag_worker_options(
    config: ProjectFlags,
) -> Result<WorkerOptions> {
    Ok(WorkerOptions::new(PROJECT_FLAG_TASK_QUEUE)
        .register_workflow::<ReconcileProjectFlagsWorkflow>()
        .wrap_err("registering project flag reconciliation workflow")?
        .register_activities(ProjectFlagActivities { config })
        .build())
}

/// Starts a reconcile unless one is already running for this project.
pub async fn reconcile_project_flags(
    temporal: &TemporalClient,
    project_id: ProjectId,
) -> Result<()> {
    temporal
        .client
        .start_workflow(
            ReconcileProjectFlagsWorkflow::run,
            project_id,
            WorkflowStartOptions::new(
                PROJECT_FLAG_TASK_QUEUE,
                format!("reconcile-project-flags:{project_id}"),
            )
            .id_conflict_policy(WorkflowIdConflictPolicy::UseExisting)
            .id_reuse_policy(WorkflowIdReusePolicy::AllowDuplicate)
            .build(),
        )
        .await
        .wrap_err("starting project flag reconciliation workflow")?;

    Ok(())
}

#[workflow]
#[derive(Default)]
pub struct ReconcileProjectFlagsWorkflow;

#[workflow_methods]
impl ReconcileProjectFlagsWorkflow {
    #[run]
    pub async fn run(
        ctx: &mut WorkflowContext<Self>,
        project_id: ProjectId,
    ) -> WorkflowResult<()> {
        ctx.execute_activity(
            ProjectFlagActivities::reconcile,
            project_id,
            ActivityOptions::start_to_close_timeout(Duration::from_secs(60)),
        )
        .await?;

        Ok(())
    }
}

pub struct ProjectFlagActivities {
    pub config: ProjectFlags,
}

#[activities]
impl ProjectFlagActivities {
    #[activity]
    pub async fn reconcile(
        self: Arc<Self>,
        _ctx: ActivityContext,
        _project_id: ProjectId,
    ) -> Result<(), ActivityError> {
        // TODO: load the current project state and idempotently reconcile its flags
        Ok(())
    }
}

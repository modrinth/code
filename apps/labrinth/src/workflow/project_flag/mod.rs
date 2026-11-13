mod content_scan;
mod model_scan;

use content_scan::{
    ImageScanInput, ImageScanResult, image_scan_activity_error, scan_image,
};
use model_scan::ModelScanner;

use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use temporalio_client::{
    WorkflowIdConflictPolicy, WorkflowIdReusePolicy, WorkflowStartOptions,
};
use temporalio_macros::{activities, workflow, workflow_methods};
use temporalio_sdk::{
    ActivityOptions, WorkerOptions, WorkflowContext, WorkflowResult,
    activities::{ActivityContext, ActivityError},
};

use crate::database::PgPool;
use crate::env::ProjectFlags;
use crate::models::v3::ids::ProjectId;
use crate::util::temporal::TemporalClient;
use xredis::RedisPool;

pub const PROJECT_FLAG_TASK_QUEUE: &str = "project-flags";

pub fn project_flag_worker_options(
    config: ProjectFlags,
    pool: PgPool,
    redis: RedisPool,
) -> Result<WorkerOptions> {
    Ok(WorkerOptions::new(PROJECT_FLAG_TASK_QUEUE)
        .register_workflow::<ReconcileProjectFlagsWorkflow>()
        .context("registering project flag reconciliation workflow")?
        .register_activities(ProjectFlagActivities {
            scanner: ModelScanner::new(config, redis),
            pool,
        })
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
        .context("starting project flag reconciliation workflow")?;

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
    pub scanner: ModelScanner,
    pub pool: PgPool,
}

#[activities]
impl ProjectFlagActivities {
    #[activity]
    pub async fn scan_image(
        self: Arc<Self>,
        _ctx: ActivityContext,
        input: ImageScanInput,
    ) -> Result<ImageScanResult, ActivityError> {
        scan_image(&self.scanner, &self.pool, input)
            .await
            .map_err(image_scan_activity_error)
    }

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

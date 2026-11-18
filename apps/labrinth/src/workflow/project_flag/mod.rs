pub(crate) mod image_scan;
mod model_scan;
pub mod run;
mod run_rules;
pub(crate) mod scan_cache;
pub(crate) mod text_scan;

use image_scan::{
    ImageScanInput, ImageScanResult, image_scan_activity_error, scan_image,
};
use model_scan::ModelScanner;
use text_scan::{ProjectTextScan, TextScanInput, TextScanResult};

use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use temporalio_client::{
    WorkflowIdConflictPolicy, WorkflowIdReusePolicy, WorkflowStartOptions,
};
use temporalio_macros::{activities, workflow, workflow_methods};
use temporalio_sdk::{
    ActivityOptions, SyncWorkflowContext, WorkerOptions, WorkflowContext,
    WorkflowResult,
    activities::{ActivityContext, ActivityError},
};

use crate::database::PgPool;
use crate::env::ProjectFlags;
use crate::file_hosting::FileHost;
use crate::models::v3::ids::ProjectId;
use crate::util::temporal::TemporalClient;
use xredis::RedisPool;

pub const PROJECT_FLAG_TASK_QUEUE: &str = "project-flags";

pub fn project_flag_worker_options(
    config: ProjectFlags,
    pool: PgPool,
    redis: RedisPool,
    file_host: Arc<dyn FileHost>,
) -> Result<WorkerOptions> {
    let engine =
        crate::queue::moderation::project_flag::ProjectFlagEngine::new(
            pool.clone().into(),
            redis.clone(),
            file_host.clone(),
            config.clone(),
        )
        .map_err(|error| anyhow::anyhow!("{error:#}"))?;
    Ok(WorkerOptions::new(PROJECT_FLAG_TASK_QUEUE)
        .register_workflow::<run::ProjectFlagRunWorkflow>()
        .context("registering project flag run workflow")?
        .register_activities(run::RunActivities {
            pool: pool.clone(),
            engine: engine.clone(),
        })
        .register_workflow::<ReconcileProjectFlagsWorkflow>()
        .context("registering project flag reconciliation workflow")?
        .register_activities(ProjectFlagActivities {
            scanner: ModelScanner::new(config, redis, file_host),
            pool,
            engine,
        })
        .build())
}

/// Starts reconciliation or notifies the active workflow that the project changed.
pub async fn reconcile_project_flags(
    temporal: &TemporalClient,
    project_id: ProjectId,
) -> Result<()> {
    temporal
        .client
        .signal_with_start_workflow(
            ReconcileProjectFlagsWorkflow::run,
            project_id,
            ReconcileProjectFlagsWorkflow::project_changed,
            (),
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
pub struct ReconcileProjectFlagsWorkflow {
    needs_reconcile: bool,
}

#[workflow_methods]
impl ReconcileProjectFlagsWorkflow {
    #[run]
    pub async fn run(
        ctx: &mut WorkflowContext<Self>,
        project_id: ProjectId,
    ) -> WorkflowResult<()> {
        loop {
            ctx.state_mut(|state| state.needs_reconcile = false);
            futures::try_join!(
                image_scan::scan_project_images(ctx, project_id),
                text_scan::scan_project_text(ctx, project_id),
            )?;

            ctx.execute_activity(
                ProjectFlagActivities::evaluate_rules,
                project_id,
                ActivityOptions::start_to_close_timeout(Duration::from_secs(
                    600,
                )),
            )
            .await?;

            if !ctx.state(|state| state.needs_reconcile) {
                return Ok(());
            }
        }
    }

    #[signal]
    pub fn project_changed(&mut self, _ctx: &mut SyncWorkflowContext<Self>) {
        self.needs_reconcile = true;
    }
}

pub struct ProjectFlagActivities {
    pub scanner: ModelScanner,
    pub pool: PgPool,
    pub engine: crate::queue::moderation::project_flag::ProjectFlagEngine,
}

#[activities]
impl ProjectFlagActivities {
    #[activity]
    pub async fn evaluate_rules(
        self: Arc<Self>,
        _ctx: ActivityContext,
        project_id: ProjectId,
    ) -> Result<(), ActivityError> {
        run_rules::reconcile_rules(&self.engine, &self.pool, project_id.into())
            .await
            .map_err(|error| ActivityError::from(anyhow::anyhow!("{error:#}")))
    }

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
    pub async fn project_text(
        self: Arc<Self>,
        _ctx: ActivityContext,
        project_id: ProjectId,
    ) -> Result<Option<ProjectTextScan>, ActivityError> {
        if self.scanner.decision_model_version()?.is_none() {
            return Ok(None);
        }
        Ok(text_scan::project_text(&self.pool, project_id).await?)
    }

    #[activity]
    pub async fn scan_text(
        self: Arc<Self>,
        _ctx: ActivityContext,
        input: TextScanInput,
    ) -> Result<
        Option<std::collections::BTreeMap<String, TextScanResult>>,
        ActivityError,
    > {
        text_scan::scan_text(&self.scanner, &self.pool, input)
            .await
            .map_err(image_scan_activity_error)
    }

    #[activity]
    pub async fn project_images(
        self: Arc<Self>,
        _ctx: ActivityContext,
        project_id: ProjectId,
    ) -> Result<Vec<ImageScanInput>, ActivityError> {
        Ok(image_scan::project_images(&self.pool, project_id).await?)
    }
}

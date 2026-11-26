pub(crate) mod approve;
pub(crate) mod image_scan;
mod model_scan;
pub mod run;
pub(crate) mod run_rules;
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
    temporal: temporalio_client::Client,
    kafka: actix_web::web::Data<crate::util::kafka::KafkaClientState>,
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
            temporal: temporal.clone(),
            pool: pool.clone(),
            engine: engine.clone(),
        })
        .register_workflow::<ReconcileProjectFlagsWorkflow>()
        .context("registering project flag reconciliation workflow")?
        .register_workflow::<approve::ApproveProjectWorkflow>()
        .context("registering delayed project approval workflow")?
        .register_activities(approve::ApprovalActivities {
            config: config.clone(),
            kafka: kafka.clone(),
            pool: pool.clone(),
            redis: redis.clone(),
            temporal: temporal.clone(),
        })
        .register_activities(ProjectFlagActivities {
            scanner: ModelScanner::new(config, redis.clone(), file_host),
            redis,
            pool,
            engine,
            temporal,
            kafka,
        })
        .build())
}

/// Starts reconciliation or notifies the active workflow that the project changed.
pub async fn reconcile_project_flags(
    temporal: &TemporalClient,
    project_id: ProjectId,
) -> Result<()> {
    reconcile_project_review_state(temporal, project_id, true).await
}

pub(crate) async fn reconcile_project_review_state(
    temporal: &TemporalClient,
    project_id: ProjectId,
    in_review: bool,
) -> Result<()> {
    if !in_review {
        return Ok(());
    }
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
            let in_review = ctx
                .execute_activity(
                    ProjectFlagActivities::project_in_review,
                    project_id,
                    ActivityOptions::start_to_close_timeout(
                        Duration::from_secs(60),
                    ),
                )
                .await?;
            if !in_review {
                return Ok(());
            }
            let ruleset_id = ctx
                .execute_activity(
                    ProjectFlagActivities::deployed_ruleset,
                    (),
                    ActivityOptions::start_to_close_timeout(
                        Duration::from_secs(60),
                    ),
                )
                .await?;
            let input = ReconcileRulesInput {
                project_id,
                ruleset_id,
                approval_due: false,
            };
            let (images, text) = futures::join!(
                image_scan::scan_project_images(ctx, project_id),
                text_scan::scan_project_text(ctx, input.clone()),
            );
            if ctx.state(|state| state.needs_reconcile) {
                continue;
            }
            images?;
            text?;

            let applied = ctx
                .execute_activity(
                    ProjectFlagActivities::evaluate_and_apply_rules,
                    input.clone(),
                    ActivityOptions::start_to_close_timeout(
                        Duration::from_secs(600),
                    ),
                )
                .await?;
            if !applied {
                continue;
            }

            if ctx.state(|state| state.needs_reconcile) {
                continue;
            }
            ctx.execute_activity(
                approve::ApprovalActivities::schedule_approval,
                input,
                ActivityOptions::start_to_close_timeout(Duration::from_secs(
                    60,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReconcileRulesInput {
    project_id: ProjectId,
    ruleset_id: Option<uuid::Uuid>,
    #[serde(default)]
    approval_due: bool,
}

impl ReconcileRulesInput {
    fn project_id(&self) -> ProjectId {
        self.project_id
    }

    fn ruleset_id(&self) -> Option<uuid::Uuid> {
        self.ruleset_id
    }
}

/// Reads the primary database so queued automatic work never trusts cached review state.
async fn project_in_review(
    pool: &PgPool,
    project_id: ProjectId,
) -> Result<bool> {
    let id = crate::database::models::DBProjectId::from(project_id);
    let in_review = sqlx::query_scalar!(
        r#"SELECT in_review FROM mods WHERE id = $1"#,
        id.0,
    )
    .fetch_optional(pool)
    .await
    .context("checking project review state for automatic reconciliation")?;
    Ok(in_review == Some(true))
}

#[cfg(test)]
mod review_state_tests {
    use super::*;
    use crate::{
        database::models::DBProjectId,
        test::{
            api_v3::ApiV3,
            environment::{TestEnvironment, with_test_environment},
        },
    };

    #[actix_rt::test]
    async fn automatic_reconciliation_requires_current_review_membership() {
        with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
            let project_id = env.dummy.project_alpha.project_id_parsed;
            let id = DBProjectId::from(project_id);
            for in_review in [false, true, false, true] {
                sqlx::query!(
                    r#"UPDATE mods SET in_review = $2 WHERE id = $1"#,
                    id.0,
                    in_review,
                )
                .execute(&env.db.pool)
                .await
                .unwrap();
                assert_eq!(
                    project_in_review(&env.db.pool, project_id).await.unwrap(),
                    in_review,
                );
            }
            assert!(
                !project_in_review(&env.db.pool, DBProjectId(0).into())
                    .await
                    .unwrap()
            );
        })
        .await;
    }
}

pub struct ProjectFlagActivities {
    pub redis: RedisPool,
    pub scanner: ModelScanner,
    pub pool: PgPool,
    pub engine: crate::queue::moderation::project_flag::ProjectFlagEngine,
    pub temporal: temporalio_client::Client,
    pub kafka: actix_web::web::Data<crate::util::kafka::KafkaClientState>,
}

#[activities]
impl ProjectFlagActivities {
    #[activity]
    pub async fn project_in_review(
        self: Arc<Self>,
        _ctx: ActivityContext,
        project_id: ProjectId,
    ) -> Result<bool, ActivityError> {
        Ok(project_in_review(&self.pool, project_id).await?)
    }

    #[activity]
    pub async fn deployed_ruleset(
        self: Arc<Self>,
        _ctx: ActivityContext,
        _input: (),
    ) -> Result<Option<uuid::Uuid>, ActivityError> {
        sqlx::query_scalar!(
            r#"SELECT ruleset_id FROM project_flags_deployed WHERE singleton"#
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| ActivityError::from(anyhow::anyhow!(error)))
    }

    #[activity]
    pub async fn evaluate_and_apply_rules(
        self: Arc<Self>,
        _ctx: ActivityContext,
        input: ReconcileRulesInput,
    ) -> Result<bool, ActivityError> {
        match run_rules::reconcile_rules(
            &self.engine,
            &self.pool,
            input.project_id().into(),
            input.ruleset_id(),
            input.approval_due,
            input.approval_due.then(|| self.temporal.clone()),
        )
        .await
        {
            Ok(()) => {
                crate::routes::v3::projects::mutation::publish_automatic_review_search_update(
                    input.project_id().into(),
                    &self.pool,
                    &self.redis,
                    self.kafka.clone(),
                )
                .await
                .map_err(|error| ActivityError::from(anyhow::anyhow!("{error}")))?;
                Ok(true)
            }
            Err(error)
                if error.is::<run_rules::ReconciliationInputsChanged>() =>
            {
                Ok(false)
            }
            Err(error) => {
                Err(ActivityError::from(anyhow::anyhow!("{error:#}")))
            }
        }
    }

    #[activity]
    pub async fn scan_image(
        self: Arc<Self>,
        _ctx: ActivityContext,
        input: ImageScanInput,
    ) -> Result<ImageScanResult, ActivityError> {
        if !project_in_review(&self.pool, input.project_id).await?
            || !image_scan::project_images(&self.pool, input.project_id)
                .await?
                .iter()
                .any(|image| image.url == input.url)
        {
            return Ok(ImageScanResult::default());
        }
        scan_image(&self.scanner, &self.pool, input)
            .await
            .map_err(image_scan_activity_error)
    }

    #[activity]
    pub async fn project_text(
        self: Arc<Self>,
        _ctx: ActivityContext,
        input: ReconcileRulesInput,
    ) -> Result<Option<ProjectTextScan>, ActivityError> {
        if !project_in_review(&self.pool, input.project_id()).await?
            || self.scanner.decision_model_version()?.is_none()
        {
            return Ok(None);
        }
        Ok(text_scan::project_text(&self.pool, &self.redis, input).await?)
    }

    #[activity]
    pub async fn scan_text(
        self: Arc<Self>,
        _ctx: ActivityContext,
        input: TextScanInput,
    ) -> Result<Option<Vec<TextScanResult>>, ActivityError> {
        if !project_in_review(&self.pool, input.project_id).await?
            || !text_scan::text_input_is_current(
                &self.pool,
                &self.redis,
                &input,
            )
            .await?
        {
            return Ok(None);
        }
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
        if !project_in_review(&self.pool, project_id).await? {
            return Ok(Vec::new());
        }
        Ok(image_scan::project_images(&self.pool, project_id).await?)
    }
}

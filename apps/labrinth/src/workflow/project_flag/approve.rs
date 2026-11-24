use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use temporalio_client::{
    WorkflowIdConflictPolicy, WorkflowIdReusePolicy, WorkflowStartOptions,
};
use temporalio_macros::{activities, workflow, workflow_methods};
use temporalio_sdk::{
    ActivityOptions, SyncWorkflowContext, WorkflowContext, WorkflowResult,
    activities::{ActivityContext, ActivityError},
};
use xredis::RedisPool;

use super::{
    PROJECT_FLAG_TASK_QUEUE, ProjectFlagActivities, ReconcileRulesInput,
    image_scan, project_in_review, text_scan,
};
use crate::{database::PgPool, env::ProjectFlags, models::v3::ids::ProjectId};

pub(super) async fn update_approval(
    temporal: &temporalio_client::Client,
    project_id: ProjectId,
    eligible: bool,
) -> Result<()> {
    temporal
        .signal_with_start_workflow(
            ApproveProjectWorkflow::run,
            project_id,
            ApproveProjectWorkflow::eligibility_changed,
            eligible,
            WorkflowStartOptions::new(
                PROJECT_FLAG_TASK_QUEUE,
                format!("approve-project:{project_id}"),
            )
            .id_conflict_policy(WorkflowIdConflictPolicy::UseExisting)
            .id_reuse_policy(WorkflowIdReusePolicy::AllowDuplicate)
            .build(),
        )
        .await
        .context("updating delayed project approval workflow")?;
    Ok(())
}

#[workflow]
#[derive(Default)]
pub struct ApproveProjectWorkflow {
    eligible: bool,
    generation: u64,
}

#[workflow_methods]
impl ApproveProjectWorkflow {
    #[run]
    pub async fn run(
        ctx: &mut WorkflowContext<Self>,
        project_id: ProjectId,
    ) -> WorkflowResult<()> {
        loop {
            if !ctx.state(|state| state.eligible) {
                return Ok(());
            }
            let generation = ctx.state(|state| state.generation);
            let delay = ctx
                .execute_activity(
                    ApprovalActivities::auto_approval_delay,
                    (),
                    ActivityOptions::start_to_close_timeout(
                        Duration::from_secs(60),
                    ),
                )
                .await?;
            tokio::select! {
                _ = ctx.timer(delay) => {}
                result = ctx.wait_condition(|state| state.generation != generation) => {
                    result?;
                    continue;
                }
            }
            loop {
                if ctx.state(|state| state.generation != generation) {
                    break;
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
                    approval_due: true,
                };
                let (images, text) = futures::join!(
                    image_scan::scan_project_images(ctx, project_id),
                    text_scan::scan_project_text(ctx, input.clone()),
                );
                images?;
                text?;
                if ctx.state(|state| state.generation != generation) {
                    break;
                }
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
                ctx.execute_activity(
                    ApprovalActivities::auto_approval_eligible,
                    input,
                    ActivityOptions::start_to_close_timeout(
                        Duration::from_secs(60),
                    ),
                )
                .await?;
                if ctx.state(|state| state.generation != generation) {
                    break;
                }
                return Ok(());
            }
        }
    }

    #[signal]
    pub fn eligibility_changed(
        &mut self,
        _ctx: &mut SyncWorkflowContext<Self>,
        eligible: bool,
    ) {
        self.eligible = eligible;
        if !eligible {
            self.generation += 1;
        }
    }
}

pub struct ApprovalActivities {
    pub config: ProjectFlags,
    pub pool: PgPool,
    pub redis: RedisPool,
    pub temporal: temporalio_client::Client,
}

#[activities]
impl ApprovalActivities {
    #[activity]
    pub async fn auto_approval_delay(
        self: Arc<Self>,
        _ctx: ActivityContext,
        _input: (),
    ) -> Result<Duration, ActivityError> {
        Ok(sample_auto_approval_delay(&self.config)?)
    }

    #[activity]
    pub async fn schedule_approval(
        self: Arc<Self>,
        _ctx: ActivityContext,
        input: ReconcileRulesInput,
    ) -> Result<(), ActivityError> {
        let eligible = self.eligible(&input).await?;
        update_approval(&self.temporal, input.project_id, eligible).await?;
        Ok(())
    }

    #[activity]
    pub async fn auto_approval_eligible(
        self: Arc<Self>,
        _ctx: ActivityContext,
        input: ReconcileRulesInput,
    ) -> Result<bool, ActivityError> {
        self.eligible(&input).await
    }
}

impl ApprovalActivities {
    async fn eligible(
        &self,
        input: &ReconcileRulesInput,
    ) -> Result<bool, ActivityError> {
        if !project_in_review(&self.pool, input.project_id).await? {
            return Ok(false);
        }
        crate::routes::v3::projects::mutation::delayed_auto_approval_eligible(
            input.project_id.into(),
            &self.pool,
            &self.redis,
            input.approval_due && input.ruleset_id.is_none(),
        )
        .await
        .map_err(|error| ActivityError::from(anyhow::anyhow!("{error}")))
    }
}

fn sample_auto_approval_delay(config: &ProjectFlags) -> Result<Duration> {
    use rand::RngExt;
    let min = config.auto_approve_min_delay.as_secs();
    let max = config.auto_approve_max_delay.as_secs();
    anyhow::ensure!(
        min <= max,
        "automatic approval minimum delay exceeds maximum delay"
    );
    Ok(Duration::from_secs(rand::rng().random_range(min..=max)))
}

#[cfg(test)]
mod approval_delay_tests {
    use super::*;

    #[test]
    fn sampled_delay_respects_inclusive_bounds() {
        for (min, max) in [(0, 0), (7, 7), (0, 1), (7, 19)] {
            let config = ProjectFlags {
                auto_approve_min_delay: Duration::from_secs(min),
                auto_approve_max_delay: Duration::from_secs(max),
                ..ProjectFlags::default()
            };
            for _ in 0..256 {
                let delay = sample_auto_approval_delay(&config).unwrap();
                assert!(delay >= Duration::from_secs(min));
                assert!(delay <= Duration::from_secs(max));
            }
        }
    }
}

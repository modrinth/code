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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApprovalToken {
    pub run_id: String,
    pub generation: u64,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
pub struct ApprovalStatus {
    pub auto_approve_blocked: bool,
    pub eligible: bool,
    pub generation: u64,
    pub finished: bool,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub phase: String,
}

impl ApprovalStatus {
    fn permits_generation(&self, generation: u64) -> bool {
        !self.finished
            && !self.auto_approve_blocked
            && self.eligible
            && self.generation == generation
    }
}

fn publish_state(
    ctx: &WorkflowContext<ApproveProjectWorkflow>,
) -> WorkflowResult<()> {
    let status = ctx.state(ApproveProjectWorkflow::snapshot);
    ctx.upsert_memo([(
        "auto_approval",
        Some(temporalio_sdk::MemoValue::new(status)),
    )])?;
    Ok(())
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ApprovalQueueEntry {
    pub workflow_id: String,
    pub run_id: String,
    pub state: Option<ApprovalStatus>,
}

pub async fn queue(
    temporal: &temporalio_client::Client,
) -> Result<Vec<ApprovalQueueEntry>> {
    use futures::TryStreamExt;
    let mut executions = temporal.list_workflows(
        "WorkflowType = 'ApproveProjectWorkflow' AND ExecutionStatus = 'Running'",
        temporalio_client::WorkflowListOptions::default(),
    );
    let mut entries = Vec::new();
    while let Some(execution) = executions.try_next().await? {
        entries.push(ApprovalQueueEntry {
            workflow_id: execution.id().to_owned(),
            run_id: execution.run_id().to_owned(),
            state: execution.memo().get("auto_approval")?,
        });
    }
    Ok(entries)
}

pub type ApprovalHandle = temporalio_client::WorkflowHandle<
    temporalio_client::Client,
    ApproveProjectWorkflow,
>;

pub async fn current_execution(
    temporal: &temporalio_client::Client,
    project_id: ProjectId,
) -> Result<Option<ApprovalHandle>> {
    let handle = temporal.get_workflow_handle::<ApproveProjectWorkflow>(
        format!("approve-project:{project_id}"),
    );
    let description = match handle
        .describe(temporalio_client::WorkflowDescribeOptions::default())
        .await
    {
        Ok(description) => description,
        Err(temporalio_client::errors::WorkflowInteractionError::NotFound(
            _,
        )) => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if description.status()
        != temporalio_client::WorkflowExecutionStatus::Running
    {
        return Ok(None);
    }
    let mut info = handle.info().clone();
    info.run_id = Some(description.run_id().to_owned());
    Ok(Some(temporalio_client::WorkflowHandle::new(
        temporal.clone(),
        info,
    )))
}

/// Completes only the captured run, never a later review using the same workflow ID.
/// The workflow's finish branch does not await activities, so a requeue barrier can
/// safely hold the project lock even if an old approval activity is waiting for it.
pub async fn finish_execution(handle: Option<ApprovalHandle>) -> Result<()> {
    if let Some(handle) = handle {
        match handle
            .signal(
                ApproveProjectWorkflow::finish,
                (),
                temporalio_client::WorkflowSignalOptions::default(),
            )
            .await
        {
            Ok(()) => {}
            Err(
                temporalio_client::errors::WorkflowInteractionError::NotFound(
                    _,
                ),
            ) => {}
            Err(error) => return Err(error.into()),
        }
        tokio::time::timeout(
            Duration::from_secs(60),
            handle.get_result(
                temporalio_client::WorkflowGetResultOptions::builder()
                    .follow_runs(false)
                    .build(),
            ),
        )
        .await
        .context("waiting for project approval workflow completion")??;
    }
    Ok(())
}

pub async fn status(
    temporal: &temporalio_client::Client,
    project_id: ProjectId,
) -> Result<Option<ApprovalStatus>> {
    match current_execution(temporal, project_id).await? {
        Some(handle) => Ok(Some(
            handle
                .query(
                    ApproveProjectWorkflow::status,
                    (),
                    temporalio_client::WorkflowQueryOptions::default(),
                )
                .await?,
        )),
        None => Ok(None),
    }
}

pub struct ApprovalGuard {
    pub temporal: temporalio_client::Client,
    pub project_id: ProjectId,
    pub token: ApprovalToken,
}

impl ApprovalGuard {
    /// Must be checked while holding the project mutation lock, shared with the block endpoint.
    pub async fn permits_approval(&self) -> Result<bool> {
        let Some(handle) =
            current_execution(&self.temporal, self.project_id).await?
        else {
            return Ok(false);
        };
        if handle.run_id() != Some(self.token.run_id.as_str()) {
            return Ok(false);
        }
        let state = handle
            .query(
                ApproveProjectWorkflow::status,
                (),
                temporalio_client::WorkflowQueryOptions::default(),
            )
            .await?;
        Ok(state.permits_generation(self.token.generation))
    }
}

#[workflow]
#[derive(Default, Clone)]
pub struct ApproveProjectWorkflow {
    eligible: bool,
    generation: u64,
    auto_approve_blocked: bool,
    finished: bool,
    deadline: Option<chrono::DateTime<chrono::Utc>>,
    phase: String,
    recheck_requested: bool,
}

#[workflow_methods]
impl ApproveProjectWorkflow {
    #[run]
    pub async fn run(
        ctx: &mut WorkflowContext<Self>,
        project_id: ProjectId,
    ) -> WorkflowResult<()> {
        loop {
            let generation = ctx.state(|state| state.generation);
            tokio::select! {
                biased;
                result = ctx.wait_condition(|state| state.finished) => { result?; return Ok(()); }
                result = ctx.wait_condition(|state| state.generation != generation) => { result?; }
                result = Self::approval_loop(ctx, project_id) => return result,
            }
        }
    }

    async fn approval_loop(
        ctx: &WorkflowContext<Self>,
        project_id: ProjectId,
    ) -> WorkflowResult<()> {
        loop {
            if ctx.state(|state| {
                state.recheck_requested && !state.auto_approve_blocked
            }) {
                ctx.state_mut(|state| state.recheck_requested = false);
                let generation = ctx.state(|state| state.generation);
                let eligible = ctx
                    .execute_activity(
                        ApprovalActivities::auto_approval_eligible,
                        ReconcileRulesInput {
                            project_id,
                            ruleset_id: None,
                            approval_due: false,
                            approval_token: None,
                        },
                        ActivityOptions::start_to_close_timeout(
                            Duration::from_secs(60),
                        ),
                    )
                    .await?;
                if ctx.state(|state| state.generation == generation) {
                    ctx.state_mut(|state| state.eligible = eligible);
                }
            }
            if !ctx.state(|state| state.eligible && !state.auto_approve_blocked)
            {
                ctx.state_mut(|state| {
                    state.phase = if state.auto_approve_blocked {
                        "blocked"
                    } else {
                        "ineligible"
                    }
                    .into();
                });
                publish_state(ctx)?;
                let generation = ctx.state(|state| state.generation);
                tokio::select! {
                    _ = ctx.timer(Duration::from_secs(300)) => {}
                    result = ctx.wait_condition(|state| state.generation != generation) => { result?; continue; }
                }
                if !ctx
                    .execute_activity(
                        ProjectFlagActivities::project_in_review,
                        project_id,
                        ActivityOptions::start_to_close_timeout(
                            Duration::from_secs(60),
                        ),
                    )
                    .await?
                {
                    return Ok(());
                }
                continue;
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
            if ctx.state(|state| state.generation != generation) {
                continue;
            }
            let deadline =
                ctx.workflow_time().expect("workflow time is available")
                    + delay;
            ctx.state_mut(|state| {
                state.deadline = Some(deadline.into());
                state.phase = "waiting".into();
            });
            publish_state(ctx)?;
            tokio::select! {
                _ = ctx.timer(delay) => {}
                result = ctx.wait_condition(|state| state.generation != generation) => {
                    result?;
                    continue;
                }
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
            ctx.state_mut(|state| state.phase = "scanning".into());
            publish_state(ctx)?;
            let input = ReconcileRulesInput {
                project_id,
                ruleset_id,
                approval_due: true,
                approval_token: Some(ApprovalToken {
                    run_id: ctx.info().run_id().to_owned(),
                    generation,
                }),
            };
            let (images, text) = futures::join!(
                image_scan::scan_project_images(ctx, project_id),
                text_scan::scan_project_text(ctx, input.clone()),
            );
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
            if applied {
                ctx.execute_activity(
                    ApprovalActivities::auto_approval_eligible,
                    input,
                    ActivityOptions::start_to_close_timeout(
                        Duration::from_secs(60),
                    ),
                )
                .await?;
            }
            if !ctx
                .execute_activity(
                    ProjectFlagActivities::project_in_review,
                    project_id,
                    ActivityOptions::start_to_close_timeout(
                        Duration::from_secs(60),
                    ),
                )
                .await?
            {
                return Ok(());
            }
            ctx.state_mut(|state| state.eligible = false);
        }
    }

    #[query]
    pub fn status(
        &self,
        _ctx: &temporalio_sdk::WorkflowContextView,
    ) -> ApprovalStatus {
        self.snapshot()
    }

    #[signal]
    pub fn set_blocked(
        &mut self,
        ctx: &mut SyncWorkflowContext<Self>,
        blocked: bool,
    ) {
        self.change_block(blocked);
        ctx.upsert_memo([(
            "auto_approval",
            Some(temporalio_sdk::MemoValue::new(self.snapshot())),
        )])
        .expect("approval state is serializable");
    }

    #[signal]
    pub fn finish(&mut self, _ctx: &mut SyncWorkflowContext<Self>) {
        self.finish_review();
    }

    #[signal]
    pub fn eligibility_changed(
        &mut self,
        _ctx: &mut SyncWorkflowContext<Self>,
        eligible: bool,
    ) {
        self.change_eligibility(eligible);
    }
}

impl ApproveProjectWorkflow {
    fn finish_review(&mut self) {
        self.finished = true;
        self.deadline = None;
        self.phase = "completed".into();
        self.generation += 1;
    }

    fn change_block(&mut self, blocked: bool) {
        self.auto_approve_blocked = blocked;
        self.recheck_requested = !blocked;
        self.eligible = false;
        self.deadline = None;
        self.phase = if blocked { "blocked" } else { "checking" }.into();
        self.generation += 1;
    }

    fn change_eligibility(&mut self, eligible: bool) {
        if self.auto_approve_blocked || self.finished {
            return;
        }
        if eligible && self.phase == "scanning" {
            self.recheck_requested = true;
        }
        if self.eligible != eligible {
            self.eligible = eligible;
            if !eligible {
                self.deadline = None;
            }
            self.generation += 1;
        }
    }

    fn snapshot(&self) -> ApprovalStatus {
        ApprovalStatus {
            auto_approve_blocked: self.auto_approve_blocked,
            eligible: self.eligible,
            generation: self.generation,
            finished: self.finished,
            deadline: self.deadline,
            phase: self.phase.clone(),
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
            input
                .approval_token
                .clone()
                .map(|token| ApprovalGuard {
                    temporal: self.temporal.clone(),
                    project_id: input.project_id,
                    token,
                })
                .as_ref(),
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
    fn block_survives_eligibility_changes_in_the_same_review() {
        let mut state = ApproveProjectWorkflow::default();
        state.change_eligibility(true);
        state.change_block(true);
        let generation = state.generation;
        for eligible in [true, false, true] {
            state.change_eligibility(eligible);
        }
        assert!(state.auto_approve_blocked);
        assert!(!state.finished);
        assert!(!state.eligible);
        assert_eq!(state.generation, generation);
        assert!(state.deadline.is_none());
    }

    #[test]
    fn temporary_ineligibility_does_not_finish_review() {
        let mut state = ApproveProjectWorkflow::default();
        state.change_eligibility(true);
        let generation = state.generation;
        state.change_eligibility(false);
        assert!(!state.finished);
        assert!(!state.eligible);
        assert!(state.generation > generation);
        state.change_eligibility(true);
        assert!(state.eligible);
    }

    #[test]
    fn unblock_requires_a_fresh_check_and_timer() {
        let mut state = ApproveProjectWorkflow::default();
        state.change_block(true);
        state.change_block(false);
        assert!(!state.auto_approve_blocked);
        assert!(!state.eligible);
        assert!(state.recheck_requested);
        assert!(state.deadline.is_none());
    }

    #[test]
    fn unchanged_eligibility_keeps_existing_timer() {
        let mut state = ApproveProjectWorkflow::default();
        state.change_eligibility(true);
        state.deadline = Some(chrono::Utc::now());
        let generation = state.generation;
        let deadline = state.deadline;
        state.change_eligibility(true);
        assert_eq!(state.generation, generation);
        assert_eq!(state.deadline, deadline);
    }

    #[test]
    fn block_and_unblock_invalidate_in_flight_approval() {
        let mut state = ApproveProjectWorkflow::default();
        state.change_eligibility(true);
        let old_generation = state.generation;
        assert!(state.snapshot().permits_generation(old_generation));
        state.change_block(true);
        assert!(!state.snapshot().permits_generation(old_generation));
        state.change_block(false);
        state.change_eligibility(true);
        assert!(!state.snapshot().permits_generation(old_generation));
        assert!(state.snapshot().permits_generation(state.generation));
    }

    #[test]
    fn completed_review_cannot_be_rearmed_and_next_review_is_unblocked() {
        let mut state = ApproveProjectWorkflow::default();
        state.change_block(true);
        state.finish_review();
        state.change_eligibility(true);
        assert!(!state.snapshot().permits_generation(state.generation));
        let next_review = ApproveProjectWorkflow::default();
        assert!(!next_review.auto_approve_blocked);
        assert!(!next_review.finished);
    }

    #[test]
    fn reconciliation_during_due_attempt_requests_fresh_check() {
        let mut state = ApproveProjectWorkflow::default();
        state.change_eligibility(true);
        state.phase = "scanning".into();
        state.change_eligibility(true);
        assert!(state.recheck_requested);
    }

    #[test]
    fn invalid_delay_range_is_rejected() {
        let config = ProjectFlags {
            auto_approve_min_delay: Duration::from_secs(2),
            auto_approve_max_delay: Duration::from_secs(1),
            ..ProjectFlags::default()
        };
        assert!(sample_auto_approval_delay(&config).is_err());
    }

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

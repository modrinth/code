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
    image_scan, text_scan,
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

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
pub struct ApprovalStatus {
    /// Explicit project identity shared by workflow queries and visibility memos.
    /// Absent before workflow initialization or in older visibility metadata.
    pub project_id: Option<ProjectId>,
    pub auto_approve_blocked: bool,
    pub eligible: bool,
    pub finished: bool,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub phase: String,
}

impl ApprovalStatus {
    fn permits_approval(&self) -> bool {
        !self.finished && !self.auto_approve_blocked && self.eligible
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApprovalSchedule {
    pub project_id: ProjectId,
    pub scheduled_for: chrono::DateTime<chrono::Utc>,
}

/// Checked under the project lock shared with cancellation.
pub async fn approval_permitted(
    temporal: &temporalio_client::Client,
    project_id: ProjectId,
) -> Result<bool> {
    Ok(status(temporal, project_id)
        .await?
        .is_some_and(|state| state.permits_approval()))
}

#[workflow]
#[derive(Default, Clone)]
pub struct ApproveProjectWorkflow {
    project_id: Option<ProjectId>,
    eligible: bool,
    auto_approve_blocked: bool,
    finished: bool,
    deadline: Option<chrono::DateTime<chrono::Utc>>,
    phase: String,
}

#[workflow_methods]
impl ApproveProjectWorkflow {
    #[run]
    pub async fn run(
        ctx: &mut WorkflowContext<Self>,
        project_id: ProjectId,
    ) -> WorkflowResult<()> {
        ctx.state_mut(|state| state.project_id = Some(project_id));
        loop {
            let cancelled = ctx.state(|state| state.auto_approve_blocked);
            tokio::select! {
                biased;
                result = ctx.wait_condition(|state| state.finished) => { result?; return Ok(()); }
                result = ctx.wait_condition(|state| state.auto_approve_blocked), if !cancelled => { result?; }
                result = Self::approval_loop(ctx, project_id) => return result,
            }
        }
    }

    async fn approval_loop(
        ctx: &WorkflowContext<Self>,
        project_id: ProjectId,
    ) -> WorkflowResult<()> {
        loop {
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
                tokio::select! {
                    _ = ctx.timer(Duration::from_secs(300)) => {}
                    result = ctx.wait_condition(|state| state.eligible && !state.auto_approve_blocked) => {
                        result?;
                        continue;
                    }
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
            if ctx.state(|state| state.deadline.is_none()) {
                let delay = ctx
                    .execute_activity(
                        ApprovalActivities::auto_approval_delay,
                        (),
                        ActivityOptions::start_to_close_timeout(
                            Duration::from_secs(60),
                        ),
                    )
                    .await?;
                ctx.state_mut(|state| {
                    if !state.auto_approve_blocked {
                        state.deadline = Some(
                            (ctx.workflow_time()
                                .expect("workflow time is available")
                                + delay)
                                .into(),
                        );
                    }
                });
            }
            if ctx.state(|state| state.auto_approve_blocked || !state.eligible)
            {
                continue;
            }
            let deadline = ctx.state(|state| {
                state.deadline.expect("eligible review has a deadline")
            });
            ctx.state_mut(|state| state.phase = "waiting".into());
            publish_state(ctx)?;
            let recorded = ctx
                .execute_activity(
                    ApprovalActivities::record_approval_schedule,
                    ApprovalSchedule {
                        project_id,
                        scheduled_for: deadline,
                    },
                    ActivityOptions::start_to_close_timeout(
                        Duration::from_secs(60),
                    ),
                )
                .await?;
            if !recorded {
                ctx.state_mut(|state| state.eligible = false);
                continue;
            }
            let remaining = (deadline
                - chrono::DateTime::<chrono::Utc>::from(
                    ctx.workflow_time().expect("workflow time is available"),
                ))
            .to_std()
            .unwrap_or_default();
            tokio::select! {
                _ = ctx.timer(remaining) => {}
                result = ctx.wait_condition(|state| state.auto_approve_blocked || !state.eligible) => {
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
            if !applied {
                continue;
            }
            ctx.execute_activity(
                ApprovalActivities::auto_approval_eligible,
                input,
                ActivityOptions::start_to_close_timeout(Duration::from_secs(
                    60,
                )),
            )
            .await?;
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
    pub fn cancel(&mut self, ctx: &mut SyncWorkflowContext<Self>, _input: ()) {
        self.cancel_approval();
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
    }

    fn cancel_approval(&mut self) {
        if self.finished {
            return;
        }
        self.auto_approve_blocked = true;
        self.eligible = false;
        self.deadline = None;
        self.phase = "blocked".into();
    }

    fn change_eligibility(&mut self, eligible: bool) {
        if !self.auto_approve_blocked && !self.finished {
            self.eligible = eligible;
        }
    }

    fn snapshot(&self) -> ApprovalStatus {
        ApprovalStatus {
            project_id: self.project_id,
            auto_approve_blocked: self.auto_approve_blocked,
            eligible: self.eligible,
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
    pub kafka: actix_web::web::Data<crate::util::kafka::KafkaClientState>,
}

#[activities]
impl ApprovalActivities {
    #[activity]
    pub async fn record_approval_schedule(
        self: Arc<Self>,
        _ctx: ActivityContext,
        input: ApprovalSchedule,
    ) -> Result<bool, ActivityError> {
        if !self.config.auto_approve_enabled {
            return Ok(false);
        }
        crate::routes::v3::projects::mutation::delayed_auto_approval_eligible(
            input.project_id.into(),
            &self.pool,
            &self.redis,
            false,
            Some(&self.temporal),
            Some(&input),
        )
        .await
        .map_err(|error| ActivityError::from(anyhow::anyhow!("{error}")))
    }

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
        if !self.config.auto_approve_enabled {
            return Ok(false);
        }
        let eligible = crate::routes::v3::projects::mutation::delayed_auto_approval_eligible(
            input.project_id.into(),
            &self.pool,
            &self.redis,
            input.approval_due && input.ruleset_id.is_none(),
            input.approval_due.then_some(&self.temporal),
            None,
        )
        .await
        .map_err(|error| ActivityError::from(anyhow::anyhow!("{error}")))?;
        crate::routes::v3::projects::mutation::publish_automatic_review_search_update(
            input.project_id.into(),
            &self.pool,
            &self.redis,
            self.kafka.clone(),
        )
        .await
        .map_err(|error| ActivityError::from(anyhow::anyhow!("{error}")))?;
        Ok(eligible)
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
    fn cancellation_holds_current_review_even_before_eligibility() {
        let mut state = ApproveProjectWorkflow::default();
        state.cancel_approval();
        for eligible in [true, false, true] {
            state.change_eligibility(eligible);
        }
        assert!(state.auto_approve_blocked);
        assert!(!state.finished);
        assert!(!state.snapshot().permits_approval());
        assert!(state.deadline.is_none());
    }

    #[test]
    fn temporary_ineligibility_preserves_original_deadline() {
        let mut state = ApproveProjectWorkflow::default();
        state.change_eligibility(true);
        state.deadline = Some(chrono::Utc::now());
        let deadline = state.deadline;
        state.change_eligibility(false);
        assert!(!state.finished);
        assert!(!state.snapshot().permits_approval());
        assert_eq!(state.deadline, deadline);
        state.change_eligibility(true);
        assert!(state.snapshot().permits_approval());
        assert_eq!(state.deadline, deadline);
    }

    #[test]
    fn cancellation_stops_existing_timer_and_completion_ends_veto() {
        let mut state = ApproveProjectWorkflow::default();
        state.change_eligibility(true);
        state.deadline = Some(chrono::Utc::now());
        state.cancel_approval();
        assert!(state.deadline.is_none());
        assert!(!state.finished);
        state.finish_review();
        state.change_eligibility(true);
        assert!(!state.snapshot().permits_approval());
        let next_review = ApproveProjectWorkflow::default();
        assert!(!next_review.auto_approve_blocked);
        assert!(!next_review.finished);
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

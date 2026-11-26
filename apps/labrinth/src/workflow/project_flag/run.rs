use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use temporalio_client::{
    WorkflowIdConflictPolicy, WorkflowIdReusePolicy, WorkflowStartOptions,
};
use temporalio_macros::{activities, workflow, workflow_methods};
use temporalio_sdk::{
    ActivityOptions, WorkflowContext, WorkflowResult,
    activities::{ActivityContext, ActivityError},
};
use temporalio_workflow::ContinueAsNewOptions;

use super::{PROJECT_FLAG_TASK_QUEUE, run_rules};
use crate::{
    database::{
        PgPool,
        models::{
            DBProjectId, ProjectFlagRunId, project_flag_item::DBProjectFlagRule,
        },
    },
    queue::moderation::project_flag::ProjectFlagEngine,
    util::temporal::TemporalClient,
};

const BATCH_SIZE: usize = 64;

const BATCHES_PER_EXECUTION: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSnapshot {
    pub rules: Vec<DBProjectFlagRule>,
    pub modules: HashMap<String, String>,
}

impl RunSnapshot {
    pub(crate) fn rule_definition(
        rule: &DBProjectFlagRule,
    ) -> serde_json::Value {
        serde_json::json!({ "id": rule.id, "name": rule.name, "params": rule.params })
    }

    pub(crate) fn content_hash(&self) -> Result<blake3::Hash> {
        let mut rules = self.rules.iter().collect::<Vec<_>>();
        rules.sort_by_key(|rule| rule.id.0);
        let rules = rules
            .into_iter()
            .map(Self::rule_definition)
            .collect::<Vec<_>>();
        super::scan_cache::rules_input_hash(&serde_json::json!({
            "rules": rules, "modules": self.modules,
        }))
    }
}

#[cfg(test)]
mod ruleset_tests {
    use super::*;
    use crate::database::models::DBProjectFlagRuleId;

    fn snapshot() -> RunSnapshot {
        RunSnapshot {
            rules: vec![DBProjectFlagRule {
                id: DBProjectFlagRuleId(1),
                name: "test".into(),
                created_by: None,
                created_at: chrono::Utc::now(),
                updated_by: None,
                updated_at: chrono::Utc::now(),
                params: serde_json::from_value(serde_json::json!({
                    "expression": "[]", "decision_questions": {"q": {"question": "Question", "include_context": ["title"]}},
                }))
                .unwrap(),
            }],
            modules: HashMap::from([(
                "shared".into(),
                "fn test() { [] }".into(),
            )]),
        }
    }

    #[test]
    fn ignores_editor_metadata_and_rule_order() {
        let mut snapshot = snapshot();
        let mut second = snapshot.rules[0].clone();
        second.id = DBProjectFlagRuleId(2);
        snapshot.rules.push(second);
        let hash = snapshot.content_hash().unwrap();
        snapshot.rules.reverse();
        snapshot.rules[0].updated_at += chrono::Duration::seconds(1);
        snapshot.rules[0].created_at += chrono::Duration::seconds(1);
        snapshot.rules[0].updated_by =
            Some(crate::database::models::DBUserId(42));
        assert_eq!(hash, snapshot.content_hash().unwrap());
    }

    #[test]
    fn tracks_rule_identity_name_parameters_and_modules() {
        let original = snapshot();
        let hash = original.content_hash().unwrap();
        let mut changed = original.clone();
        changed.rules[0].id = DBProjectFlagRuleId(2);
        assert_ne!(hash, changed.content_hash().unwrap());
        let mut changed = original.clone();
        changed.rules[0].name.push('!');
        assert_ne!(hash, changed.content_hash().unwrap());
        let mut changed = original.clone();
        changed.rules[0].params.expression.push(' ');
        assert_ne!(hash, changed.content_hash().unwrap());
        let mut changed = original.clone();
        changed.rules[0]
            .params
            .decision_questions
            .get_mut("q")
            .unwrap()
            .question = "Different".into();
        assert_ne!(hash, changed.content_hash().unwrap());
        let mut changed = original;
        changed
            .modules
            .insert("shared".into(), "fn test() { [1] }".into());
        assert_ne!(hash, changed.content_hash().unwrap());
    }
}

pub fn workflow_id(id: ProjectFlagRunId) -> String {
    format!("project-flag-run:{}", id.0)
}

const RUN_MEMO: &str = "project_flag_run";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetadata {
    pub id: ProjectFlagRunId,
    pub started_by: ProjectFlagRunUser,
    pub total_projects: i64,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ruleset_id: uuid::Uuid,
}

pub use crate::database::models::project_flag_item::ProjectFlagRunUser;

pub async fn start(temporal: &TemporalClient, input: RunInput) -> Result<()> {
    let mut memo = temporalio_common::MemoValues::new();
    memo.insert(RUN_MEMO, input.metadata.clone());
    let id = input.metadata.id;
    temporal
        .client
        .start_workflow(
            ProjectFlagRunWorkflow::run,
            input,
            WorkflowStartOptions::new(PROJECT_FLAG_TASK_QUEUE, workflow_id(id))
                .id_conflict_policy(WorkflowIdConflictPolicy::UseExisting)
                .id_reuse_policy(WorkflowIdReusePolicy::RejectDuplicate)
                .memo(memo)
                .build(),
        )
        .await
        .context("starting project flag run workflow")?;
    Ok(())
}

#[workflow]
#[derive(Default)]
pub struct ProjectFlagRunWorkflow;

#[workflow_methods]
impl ProjectFlagRunWorkflow {
    #[run]
    pub async fn run(
        ctx: &mut WorkflowContext<Self>,
        mut input: RunInput,
    ) -> WorkflowResult<()> {
        for _ in 0..BATCHES_PER_EXECUTION {
            let end = (input.offset + BATCH_SIZE).min(input.projects.len());
            let projects = input.projects[input.offset..end].to_vec();
            if projects.is_empty() {
                return Ok(());
            }
            ctx.execute_activity(
                RunActivities::evaluate_batch,
                BatchInput {
                    id: input.metadata.id,
                    ruleset_id: input.metadata.ruleset_id,
                    projects,
                },
                ActivityOptions::start_to_close_timeout(Duration::from_secs(
                    600,
                )),
            )
            .await?;
            input.offset = end;
        }
        input.projects.drain(..input.offset);
        input.offset = 0;
        ctx.continue_as_new(input, ContinueAsNewOptions::default())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunInput {
    pub metadata: RunMetadata,
    pub projects: Vec<DBProjectId>,
    pub offset: usize,
}

impl RunInput {
    pub fn new(
        id: ProjectFlagRunId,
        mut projects: Vec<DBProjectId>,
        started_by: ProjectFlagRunUser,
        ruleset_id: uuid::Uuid,
    ) -> Self {
        projects.sort_unstable_by_key(|id| id.0);
        projects.dedup_by_key(|id| id.0);
        Self {
            metadata: RunMetadata {
                id,
                started_by,
                total_projects: projects.len() as i64,
                started_at: chrono::Utc::now(),
                ruleset_id,
            },
            projects,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInput {
    pub id: ProjectFlagRunId,
    pub ruleset_id: uuid::Uuid,
    pub projects: Vec<DBProjectId>,
}

pub struct RunActivities {
    pub pool: PgPool,
    pub engine: ProjectFlagEngine,
    pub temporal: temporalio_client::Client,
}

#[activities]
impl RunActivities {
    #[activity]
    pub async fn evaluate_batch(
        self: Arc<Self>,
        ctx: ActivityContext,
        input: BatchInput,
    ) -> Result<(), ActivityError> {
        if ctx.is_cancelled() {
            return Err(ActivityError::from(anyhow!(
                "project flag activity cancelled"
            )));
        }
        let snapshot = crate::database::models::project_flag_item::project_flag_ruleset_snapshot(&self.pool, input.ruleset_id)
            .await?.context("project flag run snapshot missing")?;
        tokio::select! {
            result = run_rules::scan_batch(&self.engine, &self.pool, Some(input.id), input.projects, snapshot, &self.temporal, &ctx) => result.map_err(ActivityError::from),
            _ = ctx.cancelled() => Err(ActivityError::from(anyhow!("project flag activity cancelled"))),
        }
    }
}

/// Recently polling activity-worker identities, not a guarantee of available capacity.
pub async fn num_workers(temporal: &TemporalClient) -> Result<usize> {
    use temporalio_client::{NamespacedClient, grpc::WorkflowService};
    use temporalio_common::protos::temporal::api::{
        enums::v1::TaskQueueType, taskqueue::v1::TaskQueue,
        workflowservice::v1::DescribeTaskQueueRequest,
    };
    let mut client = temporal.client.clone();
    let response = client
        .describe_task_queue(temporalio_client::tonic::Request::new(
            DescribeTaskQueueRequest {
                namespace: temporal.client.namespace(),
                task_queue: Some(TaskQueue {
                    name: PROJECT_FLAG_TASK_QUEUE.into(),
                    ..Default::default()
                }),
                task_queue_type: TaskQueueType::Activity as i32,
                ..Default::default()
            },
        ))
        .await
        .context("describing project flag activity workers")?
        .into_inner();
    Ok(response
        .pollers
        .into_iter()
        .map(|poller| poller.identity)
        .collect::<HashSet<_>>()
        .len())
}

use temporalio_client::{NamespacedClient, grpc::WorkflowService};
use temporalio_common::protos::temporal::api::{
    common::v1::WorkflowExecution,
    enums::v1::{HistoryEventFilterType, WorkflowExecutionStatus},
    workflow::v1::WorkflowExecutionInfo,
    workflowservice::v1::{
        DeleteWorkflowExecutionRequest, GetWorkflowExecutionHistoryRequest,
        TerminateWorkflowExecutionRequest,
    },
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Running,
    Completed,
    Failed,
    Canceled,
    Terminated,
    TimedOut,
    ContinuedAsNew,
    Unknown,
}

impl From<i32> for RunStatus {
    fn from(value: i32) -> Self {
        match WorkflowExecutionStatus::try_from(value).ok() {
            Some(WorkflowExecutionStatus::Running) => Self::Running,
            Some(WorkflowExecutionStatus::Completed) => Self::Completed,
            Some(WorkflowExecutionStatus::Failed) => Self::Failed,
            Some(WorkflowExecutionStatus::Canceled) => Self::Canceled,
            Some(WorkflowExecutionStatus::Terminated) => Self::Terminated,
            Some(WorkflowExecutionStatus::TimedOut) => Self::TimedOut,
            Some(WorkflowExecutionStatus::ContinuedAsNew) => {
                Self::ContinuedAsNew
            }
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TemporalRun {
    pub id: ProjectFlagRunId,
    pub ruleset_id: Option<uuid::Uuid>,
    pub status: RunStatus,
    pub total_projects: i64,
    pub started_by: Option<ProjectFlagRunUser>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
    execution_id: String,
}

fn decode_run(
    info: WorkflowExecutionInfo,
    metadata: RunMetadata,
) -> Result<TemporalRun> {
    let execution = info.execution.context("missing temporal execution")?;
    anyhow::ensure!(
        execution.workflow_id == workflow_id(metadata.id),
        "project flag run memo does not match workflow id"
    );
    Ok(TemporalRun {
        id: metadata.id,
        ruleset_id: Some(metadata.ruleset_id),
        status: info.status.into(),
        total_projects: metadata.total_projects,
        started_by: Some(metadata.started_by),
        started_at: metadata.started_at,
        completed_at: info.close_time.and_then(|t| {
            chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32)
        }),
        error: None,
        execution_id: execution.run_id,
    })
}

pub async fn describe(
    client: &temporalio_client::Client,
    id: ProjectFlagRunId,
) -> Result<Option<TemporalRun>> {
    let description = match client
        .get_workflow_handle::<ProjectFlagRunWorkflow>(workflow_id(id))
        .describe(temporalio_client::WorkflowDescribeOptions::default())
        .await
    {
        Ok(description) => description,
        Err(temporalio_client::errors::WorkflowInteractionError::NotFound(
            _,
        )) => {
            return Ok(None);
        }
        Err(error) => return Err(error).context("describing project flag run"),
    };
    let metadata = description
        .memo()
        .get::<RunMetadata>(RUN_MEMO)
        .context("decoding project flag run memo")?
        .context("missing project flag run metadata")?;
    let info = description
        .raw_description
        .workflow_execution_info
        .context("missing temporal workflow information")?;
    let mut run = decode_run(info, metadata)?;
    let activity_errors = description
        .raw_description
        .pending_activities
        .iter()
        .filter_map(|activity| activity.last_failure.as_ref())
        .map(|failure| format!("{failure:?}"))
        .collect::<Vec<_>>();
    if !activity_errors.is_empty() {
        run.error = Some(activity_errors.join("\n"));
    }
    load_error(client, &mut run).await?;
    Ok(Some(run))
}

async fn load_error(
    client: &temporalio_client::Client,
    run: &mut TemporalRun,
) -> Result<()> {
    let mut rpc = client.clone();
    if matches!(
        run.status,
        RunStatus::Failed
            | RunStatus::TimedOut
            | RunStatus::Canceled
            | RunStatus::Terminated
    ) {
        let history = rpc
            .get_workflow_execution_history(
                temporalio_client::tonic::Request::new(
                    GetWorkflowExecutionHistoryRequest {
                        namespace: client.namespace(),
                        execution: Some(WorkflowExecution {
                            workflow_id: workflow_id(run.id),
                            run_id: run.execution_id.clone(),
                        }),
                        history_event_filter_type:
                            HistoryEventFilterType::CloseEvent as i32,
                        ..Default::default()
                    },
                ),
            )
            .await
            .context("reading project flag run failure")?
            .into_inner();
        use temporalio_common::protos::temporal::api::history::v1::history_event::Attributes;
        run.error =
            history.history.into_iter().flat_map(|h| h.events).find_map(
                |event| match event.attributes? {
                    Attributes::WorkflowExecutionFailedEventAttributes(a) => {
                        a.failure.map(|f| format!("{f:?}"))
                    }
                    Attributes::WorkflowExecutionTimedOutEventAttributes(_) => {
                        Some("workflow execution timed out".into())
                    }
                    Attributes::WorkflowExecutionCanceledEventAttributes(a) => {
                        Some(format!(
                            "workflow execution canceled: {:?}",
                            a.details
                        ))
                    }
                    Attributes::WorkflowExecutionTerminatedEventAttributes(
                        a,
                    ) => Some(a.reason),
                    _ => None,
                },
            );
    }
    Ok(())
}

pub async fn list(
    temporal: &TemporalClient,
    limit: usize,
    offset: usize,
    running_only: bool,
) -> Result<Vec<TemporalRun>> {
    use futures::{StreamExt, TryStreamExt};
    let query = format!(
        "WorkflowType = 'ProjectFlagRunWorkflow' AND WorkflowId STARTS_WITH 'project-flag-run:'{}",
        if running_only {
            " AND ExecutionStatus = 'Running'"
        } else {
            " AND ExecutionStatus != 'ContinuedAsNew'"
        },
    );
    let executions = temporal.client.list_workflows(
        query,
        temporalio_client::WorkflowListOptions::default(),
    );
    executions
        .try_filter(|execution| {
            std::future::ready(execution.memo().contains_key(RUN_MEMO))
        })
        .skip(offset)
        .take(limit)
        .map(|execution| {
            let execution =
                execution.context("listing temporal project flag runs")?;
            let metadata = execution
                .memo()
                .get::<RunMetadata>(RUN_MEMO)
                .context("decoding project flag run memo")?
                .context("missing project flag run metadata")?;
            decode_run(execution.raw().clone(), metadata)
        })
        .try_collect()
        .await
}

/// Serialize deletion and result writes without persisting execution state in Postgres.
pub async fn lock_run(
    transaction: &mut sqlx_tracing::Transaction<'_, sqlx::Postgres>,
    id: ProjectFlagRunId,
) -> Result<()> {
    sqlx::query!(
        r#"SELECT pg_advisory_xact_lock(hashtextextended($1, 0))"#,
        workflow_id(id)
    )
    .execute(&mut *transaction)
    .await
    .context("locking project flag run results")?;
    Ok(())
}

pub async fn delete(
    temporal: &TemporalClient,
    pool: &PgPool,
    id: ProjectFlagRunId,
) -> Result<()> {
    let mut transaction = pool
        .begin()
        .await
        .context("starting project flag run deletion")?;
    lock_run(&mut transaction, id).await?;
    let mut rpc = temporal.client.clone();
    let execution = WorkflowExecution {
        workflow_id: workflow_id(id),
        run_id: String::new(),
    };
    let terminated = rpc
        .terminate_workflow_execution(temporalio_client::tonic::Request::new(
            TerminateWorkflowExecutionRequest {
                namespace: temporal.client.namespace(),
                workflow_execution: Some(execution.clone()),
                reason: "deleted by moderator".into(),
                ..Default::default()
            },
        ))
        .await;
    if let Err(error) = terminated
        && !matches!(
            error.code(),
            temporalio_client::tonic::Code::NotFound
                | temporalio_client::tonic::Code::FailedPrecondition
        )
    {
        return Err(error).context("terminating project flag run");
    }
    let mut executions = Vec::new();
    let mut execution = execution;
    loop {
        let history = rpc
            .get_workflow_execution_history(
                temporalio_client::tonic::Request::new(
                    GetWorkflowExecutionHistoryRequest {
                        namespace: temporal.client.namespace(),
                        execution: Some(execution.clone()),
                        maximum_page_size: 1,
                        ..Default::default()
                    },
                ),
            )
            .await;
        let history = match history {
            Ok(response) => response.into_inner(),
            Err(error)
                if error.code() == temporalio_client::tonic::Code::NotFound =>
            {
                break;
            }
            Err(error) => {
                return Err(error)
                    .context("reading project flag continuation chain");
            }
        };
        use temporalio_common::protos::temporal::api::history::v1::history_event::Attributes;
        let started = history
            .history
            .and_then(|h| h.events.into_iter().next())
            .and_then(|event| event.attributes)
            .context("missing project flag workflow start event")?;
        let Attributes::WorkflowExecutionStartedEventAttributes(started) =
            started
        else {
            return Err(anyhow!(
                "unexpected project flag workflow start event"
            ));
        };
        executions.push(execution.clone());
        if started.continued_execution_run_id.is_empty() {
            break;
        }
        execution.run_id = started.continued_execution_run_id;
    }
    for execution in executions.into_iter().rev() {
        let deleted = rpc
            .delete_workflow_execution(temporalio_client::tonic::Request::new(
                DeleteWorkflowExecutionRequest {
                    namespace: temporal.client.namespace(),
                    workflow_execution: Some(execution),
                },
            ))
            .await;
        if let Err(error) = deleted
            && error.code() != temporalio_client::tonic::Code::NotFound
        {
            return Err(error).context("deleting temporal project flag run");
        }
    }
    sqlx::query!(r#"UPDATE project_flags SET run_id = NULL WHERE run_id = $1 AND (active OR (what->>'retain_when_resolved')::bool = TRUE)"#, id.0)
        .execute(&mut transaction).await.context("preserving active and retained project flags")?;
    sqlx::query!(r#"DELETE FROM project_flags WHERE run_id = $1"#, id.0)
        .execute(&mut transaction)
        .await
        .context("deleting inactive run flags")?;
    sqlx::query!(
        r#"DELETE FROM project_flag_run_results WHERE run_id = $1"#,
        id.0
    )
    .execute(&mut transaction)
    .await
    .context("deleting project flag run results")?;
    transaction
        .commit()
        .await
        .context("committing project flag run deletion")?;
    Ok(())
}

pub async fn num_scanning(
    temporal: &TemporalClient,
    pool: &PgPool,
) -> Result<i64> {
    let runs = list(temporal, usize::MAX, 0, true).await?;
    let ids = runs.iter().map(|r| r.id).collect::<Vec<_>>();
    let counts = crate::database::models::project_flag_item::DBProjectFlagRunResult::counts(&ids, pool).await?;
    Ok(runs
        .iter()
        .map(|r| {
            (r.total_projects - counts.get(&r.id).map_or(0, |c| c.num_scanned))
                .max(0)
        })
        .sum())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::DBUserId;
    #[test]
    fn selection_is_sorted_deduplicated_and_round_trips() {
        let input = RunInput::new(
            ProjectFlagRunId(uuid::Uuid::now_v7()),
            vec![DBProjectId(3), DBProjectId(1), DBProjectId(3)],
            ProjectFlagRunUser {
                id: DBUserId(42).into(),
                username: "moderator".into(),
            },
            uuid::Uuid::now_v7(),
        );
        let restored: RunInput =
            serde_json::from_slice(&serde_json::to_vec(&input).unwrap())
                .unwrap();
        assert_eq!(restored.projects, vec![DBProjectId(1), DBProjectId(3)]);
        assert_eq!(restored.metadata.total_projects, 2);
        assert_eq!(restored.metadata.id, input.metadata.id);
        assert_eq!(restored.metadata.ruleset_id, input.metadata.ruleset_id);
        let mut missing_ruleset = serde_json::to_value(&input).unwrap();
        missing_ruleset["metadata"]
            .as_object_mut()
            .unwrap()
            .remove("ruleset_id");
        assert!(serde_json::from_value::<RunInput>(missing_ruleset).is_err());
    }
    #[test]
    fn continuation_preserves_original_metadata() {
        let mut input = RunInput::new(
            ProjectFlagRunId(uuid::Uuid::now_v7()),
            vec![DBProjectId(1), DBProjectId(2)],
            ProjectFlagRunUser {
                id: DBUserId(42).into(),
                username: "moderator".into(),
            },
            uuid::Uuid::now_v7(),
        );
        let metadata = serde_json::to_value(&input.metadata).unwrap();
        input.projects.drain(..1);
        let restored: RunInput =
            serde_json::from_value(serde_json::to_value(input).unwrap())
                .unwrap();
        assert_eq!(restored.projects, vec![DBProjectId(2)]);
        assert_eq!(serde_json::to_value(restored.metadata).unwrap(), metadata);
    }
    #[test]
    fn queued_run_memo_exposes_submitted_ruleset() {
        let id = ProjectFlagRunId(uuid::Uuid::now_v7());
        let ruleset_id = uuid::Uuid::now_v7();
        let input = RunInput::new(
            id,
            vec![DBProjectId(1)],
            ProjectFlagRunUser {
                id: DBUserId(42).into(),
                username: "moderator".into(),
            },
            ruleset_id,
        );
        let metadata = serde_json::from_value(
            serde_json::to_value(&input.metadata).unwrap(),
        )
        .unwrap();
        let run = decode_run(
            WorkflowExecutionInfo {
                execution: Some(
                    temporalio_common::protos::temporal::api::common::v1::WorkflowExecution {
                        workflow_id: workflow_id(id),
                        run_id: uuid::Uuid::now_v7().to_string(),
                    },
                ),
                status: 1,
                ..Default::default()
            },
            metadata,
        )
        .unwrap();
        assert_eq!(run.ruleset_id, Some(ruleset_id));
        assert_eq!(run.status, RunStatus::Running);
    }

    #[test]
    fn statuses_come_from_temporal_not_progress_counts() {
        for (raw, expected) in [
            (1, RunStatus::Running),
            (2, RunStatus::Completed),
            (3, RunStatus::Failed),
            (4, RunStatus::Canceled),
            (5, RunStatus::Terminated),
            (6, RunStatus::ContinuedAsNew),
            (7, RunStatus::TimedOut),
            (0, RunStatus::Unknown),
            (99, RunStatus::Unknown),
        ] {
            assert_eq!(RunStatus::from(raw), expected);
        }
    }
}

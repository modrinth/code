use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use eyre::{Result, WrapErr, eyre};
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

const BATCH_SIZE: i64 = 64;
const BATCHES_PER_EXECUTION: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSnapshot {
    pub rules: Vec<DBProjectFlagRule>,
    pub modules: HashMap<String, String>,
}

pub fn workflow_id(id: ProjectFlagRunId) -> String {
    format!("project-flag-run:{}", id.0)
}

pub async fn start(
    temporal: &TemporalClient,
    id: ProjectFlagRunId,
) -> Result<()> {
    temporal
        .client
        .start_workflow(
            ProjectFlagRunWorkflow::run,
            RunInput { id, offset: 0 },
            WorkflowStartOptions::new(PROJECT_FLAG_TASK_QUEUE, workflow_id(id))
                .id_conflict_policy(WorkflowIdConflictPolicy::UseExisting)
                .id_reuse_policy(WorkflowIdReusePolicy::RejectDuplicate)
                .build(),
        )
        .await
        .wrap_err("starting project flag run workflow")?;
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
            let projects = ctx
                .execute_activity(
                    RunActivities::next_batch,
                    input.clone(),
                    ActivityOptions::start_to_close_timeout(
                        Duration::from_secs(60),
                    ),
                )
                .await?;
            if projects.is_empty() {
                ctx.execute_activity(
                    RunActivities::finalize,
                    input.id,
                    ActivityOptions::start_to_close_timeout(
                        Duration::from_secs(60),
                    ),
                )
                .await?;
                return Ok(());
            }
            let count = projects.len() as i64;
            ctx.execute_activity(
                RunActivities::evaluate_batch,
                BatchInput {
                    id: input.id,
                    projects,
                },
                ActivityOptions::start_to_close_timeout(Duration::from_secs(
                    600,
                )),
            )
            .await?;
            input.offset += count;
        }
        ctx.continue_as_new(input, ContinueAsNewOptions::default())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunInput {
    pub id: ProjectFlagRunId,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInput {
    pub id: ProjectFlagRunId,
    pub projects: Vec<DBProjectId>,
}

pub struct RunActivities {
    pub pool: PgPool,
    pub engine: ProjectFlagEngine,
}

#[activities]
impl RunActivities {
    #[activity]
    pub async fn next_batch(
        self: Arc<Self>,
        _ctx: ActivityContext,
        input: RunInput,
    ) -> Result<Vec<DBProjectId>, ActivityError> {
        self.next(input)
            .await
            .map_err(|error| ActivityError::from(anyhow::anyhow!("{error:#}")))
    }

    #[activity]
    pub async fn finalize(
        self: Arc<Self>,
        _ctx: ActivityContext,
        id: ProjectFlagRunId,
    ) -> Result<(), ActivityError> {
        sqlx::query!(r#"
            UPDATE project_flag_runs SET status = 'done', completed_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND status = 'running'
        "#, id.0).execute(&self.pool).await
            .map_err(|error| ActivityError::from(anyhow::anyhow!(error)))?;
        Ok(())
    }

    #[activity]
    pub async fn evaluate_batch(
        self: Arc<Self>,
        _ctx: ActivityContext,
        input: BatchInput,
    ) -> Result<(), ActivityError> {
        self.evaluate(input)
            .await
            .map_err(|error| ActivityError::from(anyhow::anyhow!("{error:#}")))
    }
}

impl RunActivities {
    async fn next(&self, input: RunInput) -> Result<Vec<DBProjectId>> {
        let row = sqlx::query!(
            r#"
            SELECT snapshots.project_ids[($2 + 1):($2 + $3)] AS project_ids
            FROM project_flag_run_snapshots AS snapshots
            JOIN project_flag_runs AS runs ON runs.id = snapshots.run_id
            WHERE snapshots.run_id = $1 AND runs.status = 'running'
        "#,
            input.id.0,
            input.offset as i32,
            BATCH_SIZE as i32
        )
        .fetch_optional(&self.pool)
        .await
        .wrap_err("loading project flag run batch")?;
        if row.is_none() {
            let active = sqlx::query_scalar!(r#"SELECT id FROM project_flag_runs WHERE id = $1 AND status = 'running'"#, input.id.0)
                .fetch_optional(&self.pool).await.wrap_err("checking project flag run")?;
            if active.is_some() {
                return Err(eyre!("project flag run snapshot missing"));
            }
        }
        Ok(row
            .and_then(|row| row.project_ids)
            .unwrap_or_default()
            .into_iter()
            .map(DBProjectId)
            .collect())
    }

    async fn evaluate(&self, input: BatchInput) -> Result<()> {
        let snapshot = sqlx::query!(r#"
            SELECT snapshots.rules, snapshots.modules FROM project_flag_run_snapshots AS snapshots
            JOIN project_flag_runs AS runs ON runs.id = snapshots.run_id
            WHERE snapshots.run_id = $1 AND runs.status = 'running'
        "#, input.id.0).fetch_optional(&self.pool).await.wrap_err("loading project flag run snapshot")?;
        let Some(snapshot) = snapshot else {
            let active = sqlx::query_scalar!(r#"SELECT id FROM project_flag_runs WHERE id = $1 AND status = 'running'"#, input.id.0)
                .fetch_optional(&self.pool).await.wrap_err("checking project flag run")?;
            if active.is_some() {
                return Err(eyre!("project flag run snapshot missing"));
            }
            return Ok(());
        };
        run_rules::scan_batch(
            &self.engine,
            &self.pool,
            Some(input.id),
            input.projects,
            RunSnapshot {
                rules: serde_json::from_value(snapshot.rules)
                    .wrap_err("decoding run rules")?,
                modules: serde_json::from_value(snapshot.modules)
                    .wrap_err("decoding run modules")?,
            },
        )
        .await
    }
}

/// Resumes durable snapshots after an interrupted API start request.
pub async fn recover(temporal: &TemporalClient, pool: &PgPool) -> Result<()> {
    let mut after = uuid::Uuid::nil();
    loop {
        let ids = sqlx::query_scalar!(r#"
            SELECT runs.id FROM project_flag_runs AS runs
            JOIN project_flag_run_snapshots AS snapshots ON snapshots.run_id = runs.id
            WHERE runs.status = 'running' AND runs.id > $1 ORDER BY runs.id LIMIT 100
        "#, after).fetch_all(pool).await.wrap_err("loading unfinished project flag runs")?;
        if ids.is_empty() {
            return Ok(());
        }
        for id in ids {
            after = id;
            if let Err(error) = start(temporal, ProjectFlagRunId(id)).await {
                if matches!(error.downcast_ref::<temporalio_client::errors::WorkflowStartError>(),
                    Some(temporalio_client::errors::WorkflowStartError::AlreadyStarted { .. })) {
                    sqlx::query!(r#"
                        UPDATE project_flag_runs SET status = 'done', completed_at = CURRENT_TIMESTAMP,
                            error = 'temporal workflow already closed; remaining projects were not evaluated'
                        WHERE id = $1 AND status = 'running'
                    "#, id).execute(pool).await.wrap_err("recording closed project flag workflow")?;
                } else {
                    return Err(error);
                }
            }
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
        .wrap_err("describing project flag activity workers")?
        .into_inner();
    Ok(response
        .pollers
        .into_iter()
        .map(|poller| poller.identity)
        .collect::<HashSet<_>>()
        .len())
}

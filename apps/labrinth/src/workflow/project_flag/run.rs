use std::{
    collections::{BTreeMap, HashMap, HashSet},
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
use xredis::RedisPool;

use super::PROJECT_FLAG_TASK_QUEUE;
use crate::{
    database::{
        PgPool,
        models::{
            DBProjectId, ProjectFlagRuleId, ProjectFlagRunId,
            project_flag_item::{
                DBProjectFlagRule, DBProjectFlagRunResult, PatternName,
                ProjectFlagRuleResult,
            },
        },
    },
    queue::moderation::project_flag::{ProjectFlagEngine, ProjectRuleCall},
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
            id,
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
        id: ProjectFlagRunId,
    ) -> WorkflowResult<()> {
        for _ in 0..BATCHES_PER_EXECUTION {
            let projects = ctx
                .execute_activity(
                    RunActivities::next_batch,
                    id,
                    ActivityOptions::start_to_close_timeout(
                        Duration::from_secs(60),
                    ),
                )
                .await?;
            if projects.is_empty() {
                return Ok(());
            }
            ctx.execute_activity(
                RunActivities::evaluate_batch,
                BatchInput { id, projects },
                ActivityOptions::start_to_close_timeout(Duration::from_secs(
                    600,
                )),
            )
            .await?;
        }
        ctx.continue_as_new(id, ContinueAsNewOptions::default())?;
        Ok(())
    }
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
        id: ProjectFlagRunId,
    ) -> Result<Vec<DBProjectId>, ActivityError> {
        self.next(id)
            .await
            .map_err(|error| ActivityError::from(anyhow::anyhow!("{error:#}")))
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
    async fn next(&self, id: ProjectFlagRunId) -> Result<Vec<DBProjectId>> {
        let rows = sqlx::query_scalar!(r#"
            SELECT pending.project_id FROM project_flag_run_pending_jobs AS pending
            JOIN project_flag_runs AS runs ON runs.id = pending.run_id
            WHERE pending.run_id = $1 AND runs.status = 'running'
            ORDER BY pending.project_id LIMIT $2
        "#, id.0, BATCH_SIZE).fetch_all(&self.pool).await.wrap_err("loading project flag run batch")?;
        if rows.is_empty() {
            sqlx::query!(r#"
                UPDATE project_flag_runs SET status = 'done', completed_at = CURRENT_TIMESTAMP
                WHERE id = $1 AND status = 'running'
                    AND NOT EXISTS (SELECT 1 FROM project_flag_run_pending_jobs WHERE run_id = $1)
            "#, id.0).execute(&self.pool).await.wrap_err("completing empty project flag run")?;
        }
        Ok(rows.into_iter().map(DBProjectId).collect())
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
        scan_batch(
            &self.engine,
            &self.pool,
            input.id,
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

struct PreparedRule {
    id: ProjectFlagRuleId,
    call: Result<ProjectRuleCall, String>,
    pattern_names: BTreeMap<PatternName, PatternName>,
}

async fn scan_batch(
    engine: &ProjectFlagEngine,
    pool: &PgPool,
    run_id: ProjectFlagRunId,
    project_ids: Vec<DBProjectId>,
    snapshot: RunSnapshot,
) -> Result<()> {
    let engine = engine.with_modules(snapshot.modules);
    let compile_engine = engine.clone();
    let (rules, patterns) = tokio::task::spawn_blocking(move || {
        let mut rules = Vec::new();
        let mut patterns = BTreeMap::new();
        for rule in snapshot.rules {
            let call = compile_engine
                .make_call(&rule.params)
                .map_err(|error| format!("{error:#}"));
            let mut pattern_names = BTreeMap::new();
            for (name, regex) in rule.params.scan_files_for {
                let internal = PatternName(format!("{}:{}", rule.id.0, name.0));
                patterns.insert(internal.clone(), regex);
                pattern_names.insert(internal, name);
            }
            rules.push(PreparedRule {
                id: rule.id,
                call,
                pattern_names,
            });
        }
        (rules, patterns)
    })
    .await
    .wrap_err("joining project flag rule compilation")?;
    let rules = Arc::new(rules);
    let (contexts, files) = tokio::try_join!(
        engine.load_contexts(&project_ids),
        engine.load_files(&project_ids),
    )?;
    let consumed_jobs = project_ids
        .iter()
        .map(|id| (run_id, *id))
        .collect::<Vec<_>>();
    let prepared = futures::future::join_all(
        project_ids.into_iter().filter_map(|project_id| {
            let context = contexts.get(&project_id)?.clone();
            let engine = &engine;
            let patterns = &patterns;
            let files = files
                .get(&project_id)
                .map(Vec::as_slice)
                .unwrap_or_default();
            Some(async move {
                let (context, scan_error) = match engine
                    .prepare_context(context.clone(), files, patterns)
                    .await
                {
                    Ok(context) => (context, None),
                    Err(error) => (context, Some(format!("{error:#}"))),
                };
                (project_id, context, scan_error)
            })
        }),
    )
    .await;
    let tasks =
        prepared
            .into_iter()
            .map(|(project_id, mut context, scan_error)| {
                let engine = engine.clone();
                let rules = rules.clone();
                tokio::task::spawn_blocking(move || {
                    let original_patterns = context
                        .scanned_files
                        .iter()
                        .map(|file| file.found_patterns.clone())
                        .collect::<Vec<_>>();
                    let mut results = BTreeMap::new();
                    for rule in rules.iter() {
                        for (file, names) in context
                            .scanned_files
                            .iter_mut()
                            .zip(&original_patterns)
                        {
                            file.found_patterns = names
                                .iter()
                                .filter_map(|name| {
                                    rule.pattern_names.get(name).cloned()
                                })
                                .collect();
                        }
                        let (evaluation, log) = match &rule.call {
                            Ok(call) => engine.evaluate_with_scan_error(
                                call,
                                &context,
                                scan_error.as_deref(),
                            ),
                            Err(error) => {
                                (Err(eyre!(error.clone())), Vec::new())
                            }
                        };
                        let (flags, error) = match evaluation {
                            Ok(flags) => (flags, None),
                            Err(error) => {
                                (Vec::new(), Some(format!("{error:#}")))
                            }
                        };
                        results.insert(
                            rule.id.0.to_string(),
                            ProjectFlagRuleResult { flags, log, error },
                        );
                    }
                    DBProjectFlagRunResult {
                        run_id: Some(run_id),
                        project_id,
                        rules: results,
                        error: None,
                    }
                })
            });
    let results = futures::future::try_join_all(tasks)
        .await
        .wrap_err("joining project flag batch evaluation")?;
    let result_count = results.len();
    DBProjectFlagRunResult::upsert_many(results, &consumed_jobs, pool).await?;
    tracing::info!(run_id = %run_id.0, result_count, "Persisted project flag run batch results");
    Ok(())
}

/// Resumes persisted work after deployment or an interrupted API start request.
/// Legacy Kafka runs retain their original rules; modules were not snapshotted by Kafka.
pub async fn recover(
    temporal: &TemporalClient,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<()> {
    let mut after = uuid::Uuid::nil();
    loop {
        let ids = sqlx::query_scalar!(r#"
            SELECT id FROM project_flag_runs WHERE status = 'running' AND id > $1 ORDER BY id LIMIT 100
        "#, after).fetch_all(pool).await.wrap_err("loading unfinished project flag runs")?;
        if ids.is_empty() {
            return Ok(());
        }
        for id in ids {
            after = id;
            let snapshot = sqlx::query_scalar!(r#"SELECT run_id FROM project_flag_run_snapshots WHERE run_id = $1"#, id)
                .fetch_optional(pool).await.wrap_err("checking legacy run snapshot")?;
            if snapshot.is_none() {
                let mut connection = redis
                    .connect()
                    .await
                    .wrap_err("connecting to legacy rule snapshots")?;
                let key =
                    connection.key().entity("project_flag_run_rules:v1", id);
                let rules: Option<Vec<u8>> = redis::cmd("GET")
                    .arg(key)
                    .query_async(&mut connection)
                    .await
                    .wrap_err("loading legacy rule snapshot")?;
                let rules = rules.and_then(|rules| {
                    serde_json::from_slice::<Vec<DBProjectFlagRule>>(&rules)
                        .ok()
                });
                let Some(rules) = rules else {
                    sqlx::query!(r#"
                        UPDATE project_flag_runs SET status = 'done', completed_at = CURRENT_TIMESTAMP,
                            error = 'legacy rule snapshot missing or invalid; remaining projects were not evaluated'
                        WHERE id = $1 AND status = 'running'
                    "#, id).execute(pool).await.wrap_err("recording unrecoverable legacy run")?;
                    continue;
                };
                let rules = serde_json::to_value(rules)
                    .wrap_err("encoding legacy rules")?;
                sqlx::query!(r#"
                    INSERT INTO project_flag_run_snapshots (run_id, rules, modules)
                    SELECT id, $2, COALESCE((SELECT jsonb_object_agg(name, script) FROM project_flag_modules), '{}'::jsonb)
                    FROM project_flag_runs WHERE id = $1 AND status = 'running'
                    ON CONFLICT (run_id) DO NOTHING
                "#, id, rules).execute(pool).await.wrap_err("persisting legacy run snapshot")?;
                tracing::warn!(run_id = %id, "Recovered legacy run with original rules and current module snapshot");
            }
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

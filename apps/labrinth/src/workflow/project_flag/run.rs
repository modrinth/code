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
            DBProjectId, DBUserId, ProjectFlagRunId,
            project_flag_item::DBProjectFlagRule,
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
    pub(crate) fn content_hash(&self) -> Result<blake3::Hash> {
        let mut rules = self.rules.iter().collect::<Vec<_>>();
        rules.sort_by_key(|rule| rule.id.0);
        let rules = rules
            .into_iter()
            .map(|rule| {
                serde_json::json!({
                    "id": rule.id, "name": rule.name, "params": rule.params,
                })
            })
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
                    "expression": "[]", "decision_questions": {"q": "Question"},
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
            .insert("q".into(), "Different".into());
        assert_ne!(hash, changed.content_hash().unwrap());
        let mut changed = original.clone();
        changed
            .modules
            .insert("shared".into(), "fn test() { [1] }".into());
        assert_ne!(hash, changed.content_hash().unwrap());
    }
}

pub fn workflow_id(id: ProjectFlagRunId) -> String {
    format!("project-flag-run:{}", id.0)
}

pub async fn start(temporal: &TemporalClient, input: RunInput) -> Result<()> {
    let id = input.id;
    temporal
        .client
        .start_workflow(
            ProjectFlagRunWorkflow::run,
            input,
            WorkflowStartOptions::new(PROJECT_FLAG_TASK_QUEUE, workflow_id(id))
                .id_conflict_policy(WorkflowIdConflictPolicy::UseExisting)
                .id_reuse_policy(WorkflowIdReusePolicy::RejectDuplicate)
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
        if !input.initialized {
            ctx.execute_activity(
                RunActivities::initialize,
                InitializationInput {
                    id: input.id,
                    started_by: input.started_by,
                    total_projects: input.projects.len(),
                },
                ActivityOptions::start_to_close_timeout(Duration::from_secs(
                    60,
                )),
            )
            .await?;
            input.initialized = true;
        }
        for _ in 0..BATCHES_PER_EXECUTION {
            let end = (input.offset + BATCH_SIZE).min(input.projects.len());
            let projects = input.projects[input.offset..end].to_vec();
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
            input.offset = end;
        }
        input.discard_completed_projects();
        ctx.continue_as_new(input, ContinueAsNewOptions::default())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunInput {
    pub id: ProjectFlagRunId,
    pub projects: Vec<DBProjectId>,
    pub started_by: DBUserId,
    pub initialized: bool,
    pub offset: usize,
}

impl RunInput {
    pub fn new(
        id: ProjectFlagRunId,
        mut projects: Vec<DBProjectId>,
        started_by: DBUserId,
    ) -> Self {
        projects.sort_unstable_by_key(|id| id.0);
        projects.dedup_by_key(|id| id.0);
        Self {
            id,
            projects,
            started_by,
            initialized: false,
            offset: 0,
        }
    }

    fn discard_completed_projects(&mut self) {
        self.projects.drain(..self.offset);
        self.offset = 0;
    }
}

#[cfg(test)]
mod run_input_tests {
    use super::*;

    #[test]
    fn selection_is_sorted_deduplicated_and_round_trips() {
        let input = RunInput::new(
            ProjectFlagRunId(uuid::Uuid::now_v7()),
            vec![DBProjectId(3), DBProjectId(1), DBProjectId(3)],
            DBUserId(42),
        );
        let restored: RunInput =
            serde_json::from_slice(&serde_json::to_vec(&input).unwrap())
                .unwrap();
        assert_eq!(restored.id, input.id);
        assert_eq!(restored.projects, vec![DBProjectId(1), DBProjectId(3)]);
        assert_eq!(restored.offset, 0);
        assert_eq!(restored.started_by, DBUserId(42));
        assert!(!restored.initialized);
    }

    #[actix_rt::test]
    async fn concurrent_initialization_and_retries_preserve_frozen_run() {
        use crate::test::{
            api_v3::ApiV3,
            environment::{TestEnvironment, with_test_environment},
        };
        with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
            let user = sqlx::query_scalar!(r#"SELECT id FROM users ORDER BY id LIMIT 1"#)
                .fetch_one(&env.db.pool).await.unwrap();
            let input = InitializationInput {
                id: ProjectFlagRunId(uuid::Uuid::now_v7()),
                started_by: DBUserId(user),
                total_projects: 3,
            };
            let (first, retry) = tokio::join!(
                initialize_run(&env.db.pool, input.clone()),
                initialize_run(&env.db.pool, input.clone()),
            );
            first.unwrap();
            retry.unwrap();
            let ruleset = sqlx::query_scalar!(r#"SELECT ruleset_id FROM project_flag_runs WHERE id = $1"#, input.id.0)
                .fetch_one(&env.db.pool).await.unwrap().unwrap();
            sqlx::query!(r#"INSERT INTO project_flag_modules (name, script) VALUES ($1, $2)"#,
                format!("retry-test-{}", input.id.0), "fn changed() { [] }")
                .execute(&env.db.pool).await.unwrap();
            let frozen_count = sqlx::query_scalar!(r#"SELECT COUNT(*) FROM project_flag_rulesets"#)
                .fetch_one(&env.db.pool).await.unwrap();
            initialize_run(&env.db.pool, input.clone()).await.unwrap();
            assert_eq!(sqlx::query_scalar!(r#"SELECT COUNT(*) FROM project_flag_rulesets"#)
                .fetch_one(&env.db.pool).await.unwrap(), frozen_count);
            let retained = sqlx::query_scalar!(r#"SELECT ruleset_id FROM project_flag_runs WHERE id = $1"#, input.id.0)
                .fetch_one(&env.db.pool).await.unwrap();
            assert_eq!(retained, Some(ruleset));
            let mut mismatched = input;
            mismatched.total_projects += 1;
            assert!(initialize_run(&env.db.pool, mismatched).await.is_err());
        }).await;
    }

    #[test]
    fn initialization_retries_require_same_user_total_and_frozen_ruleset() {
        let input = InitializationInput {
            id: ProjectFlagRunId(uuid::Uuid::now_v7()),
            started_by: DBUserId(42),
            total_projects: 3,
        };
        let ruleset = Some(uuid::Uuid::now_v7());
        assert!(verify_initialization(&input, Some(42), 3, ruleset).is_ok());
        assert!(verify_initialization(&input, Some(43), 3, ruleset).is_err());
        assert!(verify_initialization(&input, None, 3, ruleset).is_err());
        assert!(verify_initialization(&input, Some(42), 4, ruleset).is_err());
        assert!(verify_initialization(&input, Some(42), 3, None).is_err());
    }

    #[test]
    fn continuation_keeps_only_unprocessed_selection() {
        let mut input = RunInput::new(
            ProjectFlagRunId(uuid::Uuid::now_v7()),
            (0..(BATCH_SIZE * BATCHES_PER_EXECUTION + 1))
                .map(|id| DBProjectId(id as i64))
                .collect(),
            DBUserId(42),
        );
        input.initialized = true;
        input.offset = BATCH_SIZE * BATCHES_PER_EXECUTION;
        input.discard_completed_projects();
        assert_eq!(
            input.projects,
            vec![DBProjectId((BATCH_SIZE * BATCHES_PER_EXECUTION) as i64,)]
        );
        assert_eq!(input.offset, 0);
        assert!(input.initialized);
        assert_eq!(input.started_by, DBUserId(42));
        input.offset = 1;
        input.discard_completed_projects();
        assert!(input.projects.is_empty());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInput {
    pub id: ProjectFlagRunId,
    pub projects: Vec<DBProjectId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializationInput {
    pub id: ProjectFlagRunId,
    pub started_by: DBUserId,
    pub total_projects: usize,
}

pub struct RunActivities {
    pub pool: PgPool,
    pub engine: ProjectFlagEngine,
}

#[activities]
impl RunActivities {
    #[activity]
    pub async fn initialize(
        self: Arc<Self>,
        _ctx: ActivityContext,
        input: InitializationInput,
    ) -> Result<(), ActivityError> {
        initialize_run(&self.pool, input)
            .await
            .map_err(|error| ActivityError::from(anyhow!("{error:#}")))
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

async fn initialize_run(
    pool: &PgPool,
    input: InitializationInput,
) -> Result<()> {
    let total = i64::try_from(input.total_projects)
        .context("counting project flag run jobs")?;
    let mut transaction = pool
        .begin()
        .await
        .context("starting project flag run initialization")?;
    // A missing row cannot be row-locked; serialize retries before reading staging content.
    sqlx::query!(
        r#"SELECT pg_advisory_xact_lock(hashtextextended($1, 0))"#,
        workflow_id(input.id)
    )
    .execute(&mut transaction)
    .await
    .context("locking project flag run initialization")?;
    let existing = sqlx::query!(
        r#"SELECT started_by, total_projects, ruleset_id FROM project_flag_runs WHERE id = $1"#,
        input.id.0,
    ).fetch_optional(&mut transaction).await.context("checking initialized project flag run")?;
    if let Some(existing) = existing {
        verify_initialization(
            &input,
            existing.started_by,
            existing.total_projects,
            existing.ruleset_id,
        )?;
        transaction
            .commit()
            .await
            .context("finishing existing project flag run initialization")?;
        return Ok(());
    }
    let row = sqlx::query!(r#"
        SELECT
            COALESCE((SELECT jsonb_agg(to_jsonb(rules) ORDER BY rules.id) FROM project_flag_rules AS rules), '[]'::jsonb) AS "rules!",
            COALESCE((SELECT jsonb_object_agg(name, script) FROM project_flag_modules), '{}'::jsonb) AS "modules!"
    "#).fetch_one(&mut transaction).await.context("freezing staging project flag content")?;
    let snapshot = RunSnapshot {
        rules: serde_json::from_value(row.rules.clone())
            .context("decoding staging rules")?,
        modules: serde_json::from_value(row.modules.clone())
            .context("decoding staging modules")?,
    };
    let hash = snapshot.content_hash()?.to_hex().to_string();
    let inserted = sqlx::query_scalar!(r#"
        INSERT INTO project_flag_rulesets (id, content_hash, rules, modules, created_by)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (content_hash) DO NOTHING
        RETURNING id
    "#, uuid::Uuid::now_v7(), hash, row.rules, row.modules, input.started_by.0)
        .fetch_optional(&mut transaction).await.context("persisting immutable project flag ruleset")?;
    let ruleset_id = match inserted {
        Some(id) => id,
        None => sqlx::query_scalar!(
            r#"SELECT id FROM project_flag_rulesets WHERE content_hash = $1"#,
            hash
        )
        .fetch_one(&mut transaction)
        .await
        .context("loading existing immutable project flag ruleset")?,
    };
    sqlx::query!(r#"
        INSERT INTO project_flag_runs (id, status, started_by, started_at, total_projects, ruleset_id)
        VALUES ($1, 'running', $2, CURRENT_TIMESTAMP, $3, $4)
    "#, input.id.0, input.started_by.0, total, ruleset_id)
        .execute(&mut transaction).await.context("creating initialized project flag run")?;
    transaction
        .commit()
        .await
        .context("committing project flag run initialization")?;
    Ok(())
}

fn verify_initialization(
    input: &InitializationInput,
    started_by: Option<i64>,
    total_projects: i64,
    ruleset_id: Option<uuid::Uuid>,
) -> Result<()> {
    if started_by != Some(input.started_by.0)
        || i64::try_from(input.total_projects).ok() != Some(total_projects)
        || ruleset_id.is_none()
    {
        return Err(anyhow!(
            "existing project flag run does not match initialization input"
        ));
    }
    Ok(())
}

impl RunActivities {
    async fn evaluate(&self, input: BatchInput) -> Result<()> {
        let snapshot = sqlx::query!(r#"
            SELECT rulesets.rules, rulesets.modules FROM project_flag_runs AS runs
            JOIN project_flag_rulesets AS rulesets ON rulesets.id = runs.ruleset_id
            WHERE runs.id = $1 AND runs.status = 'running'
        "#, input.id.0).fetch_optional(&self.pool).await.context("loading project flag run snapshot")?;
        let Some(snapshot) = snapshot else {
            let active = sqlx::query_scalar!(r#"SELECT id FROM project_flag_runs WHERE id = $1 AND status = 'running'"#, input.id.0)
                .fetch_optional(&self.pool).await.context("checking project flag run")?;
            if active.is_some() {
                return Err(anyhow!("project flag run snapshot missing"));
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
                    .context("decoding run rules")?,
                modules: serde_json::from_value(snapshot.modules)
                    .context("decoding run modules")?,
            },
        )
        .await
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

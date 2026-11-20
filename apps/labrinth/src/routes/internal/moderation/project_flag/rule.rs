use actix_web::{HttpRequest, delete, get, post, put, web};
use anyhow::anyhow;
use futures::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use validator::Validate;
use xredis::RedisPool;

use crate::auth::get_user_from_headers;
use crate::database::models::project_flag_item::{
    DBProjectFlagRunResult, ProjectFlagRunResults, ProjectRuleParams,
};
use crate::database::models::{
    DBProject, DBProjectFlagRule, DBProjectFlagRun, ProjectFlagRunId,
};
use crate::database::{PgPool, ReadOnlyPgPool};
use crate::models::ids::{ProjectFlagRuleId, ProjectId};
use crate::models::pats::Scopes;

use crate::queue::moderation::project_flag::{
    FlagContext, ProjectFlagEngine, ProjectFlagEvaluation,
};
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context as _;
use crate::util::temporal::TemporalClient;
use crate::util::validate::{validate_name, validation_errors_to_string};
use crate::workflow::project_flag::run;

use super::run_response::ProjectFlagRunResponse;
use super::schema_example;

/// Get an example project flag rule context.
#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    security(("bearer_auth" = [])),
    responses((status = OK, body = FlagContext))
)]
#[get("/rules/schema")]
pub async fn get_rule_schema(
    req: HttpRequest,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<serde_json::Value>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .auth_context("authenticating moderator")?;

    Ok(web::Json(
        schema_example::generate::<FlagContext>()
            .internal_context("generating project flag context example")?,
    ))
}

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListRules {
    #[serde(default = "default_limit")]
    pub limit: u16,
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u16 {
    100
}

#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    params(ListRules),
    responses((status = OK, body = [ProjectFlagRuleResponse]))
)]
#[get("/rules")]
pub async fn list_rules(
    req: HttpRequest,
    query: web::Query<ListRules>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<ProjectFlagRuleResponse>>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .auth_context("authenticating moderator")?;

    if query.limit == 0 || query.limit > 200 {
        return Err(ApiError::Request(anyhow!(
            "`limit` must be between 1 and 200"
        )));
    }
    let rules = DBProjectFlagRule::list(
        i64::from(query.limit),
        i64::from(query.offset),
        &ro_pool,
    )
    .await
    .internal_context("listing project flag rules")?;
    Ok(web::Json(rules.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    params(("id" = ProjectFlagRuleId, Path)),
    responses((status = OK, body = ProjectFlagRuleResponse), (status = NOT_FOUND))
)]
#[get("/rules/{id}")]
pub async fn get_rule(
    req: HttpRequest,
    path: web::Path<ProjectFlagRuleId>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ProjectFlagRuleResponse>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .auth_context("authenticating moderator")?;

    let rule = DBProjectFlagRule::get(path.into_inner().into(), &ro_pool)
        .await
        .internal_context("fetching project flag rule")?
        .not_found_context("project flag rule not found")?;
    Ok(web::Json(rule.into()))
}

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateRule {
    #[validate(
        length(min = 1, max = 1023),
        custom(function = "validate_name")
    )]
    pub name: String,
    #[validate(nested)]
    pub params: ProjectRuleParams,
}

#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    request_body = CreateRule,
    responses((status = OK, body = ProjectFlagRuleResponse))
)]
#[post("/rules")]
pub async fn create_rule(
    req: HttpRequest,
    body: web::Json<CreateRule>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ProjectFlagRuleResponse>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_WRITE,
    )
    .await
    .auth_context("authenticating moderator")?;
    body.validate().map_err(|error| {
        ApiError::Request(anyhow!(validation_errors_to_string(error, None)))
    })?;

    let rule = DBProjectFlagRule::create(
        &body.name,
        &body.params,
        user.user.id.into(),
        &pool,
    )
    .await
    .internal_context("creating project flag rule")?;
    Ok(web::Json(rule.into()))
}

#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    params(("id" = ProjectFlagRuleId, Path)),
    request_body = CreateRule,
    responses((status = OK, body = ProjectFlagRuleResponse), (status = NOT_FOUND))
)]
#[put("/rules/{id}")]
pub async fn update_rule(
    req: HttpRequest,
    path: web::Path<ProjectFlagRuleId>,
    body: web::Json<CreateRule>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ProjectFlagRuleResponse>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_WRITE,
    )
    .await
    .auth_context("authenticating moderator")?;
    body.validate().map_err(|error| {
        ApiError::Request(anyhow!(validation_errors_to_string(error, None)))
    })?;

    let rule = DBProjectFlagRule::update(
        path.into_inner().into(),
        &body.name,
        &body.params,
        user.user.id.into(),
        &pool,
    )
    .await
    .internal_context("updating project flag rule")?
    .not_found_context("project flag rule not found")?;
    Ok(web::Json(rule.into()))
}

#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    params(("id" = ProjectFlagRuleId, Path)),
    responses((status = NO_CONTENT), (status = NOT_FOUND))
)]
#[delete("/rules/{id}")]
pub async fn delete_rule(
    req: HttpRequest,
    path: web::Path<ProjectFlagRuleId>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_WRITE,
    )
    .await
    .auth_context("authenticating moderator")?;

    if !DBProjectFlagRule::delete(path.into_inner().into(), &pool)
        .await
        .internal_context("deleting project flag rule")?
    {
        return Err(ApiError::NotFound(anyhow!("project flag rule not found")));
    }
    Ok(())
}

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListRunResults {
    #[serde(default = "default_results_limit")]
    #[param(default = 25, minimum = 1)]
    pub limit: u32,
    #[serde(default)]
    #[param(default = 0)]
    pub offset: u32,
}

fn default_results_limit() -> u32 {
    25
}

/// List completed project evaluations for a project flag run.
#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    security(("bearer_auth" = [])),
    params(("id" = ProjectFlagRunId, Path), ListRunResults),
    responses(
        (status = OK, body = ProjectFlagRunResults),
        (status = BAD_REQUEST),
        (status = UNAUTHORIZED),
        (status = FORBIDDEN),
        (status = NOT_FOUND)
    )
)]
#[get("/rules/run/{id}/results")]
pub async fn list_run_results(
    req: HttpRequest,
    path: web::Path<ProjectFlagRunId>,
    query: web::Query<ListRunResults>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ProjectFlagRunResults>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .auth_context("authenticating moderator")?;

    if query.limit == 0 || query.limit > 200 {
        return Err(ApiError::Request(anyhow!(
            "`limit` must be between 1 and 200"
        )));
    }
    let mut results = DBProjectFlagRunResult::list(
        path.into_inner(),
        i64::from(query.limit),
        i64::from(query.offset),
        &ro_pool,
    )
    .await
    .internal_context("listing project flag run results")?
    .not_found_context("project flag run not found")?;
    for result in &mut results.results {
        result.error = result.error.take().map(|message| {
            message
                .lines()
                .map(|line| {
                    if let Some((id, rest)) = line.split_once(": ")
                        && result.rules.contains_key(id)
                        && let Ok(id) = id.parse::<u64>()
                    {
                        return format!("{}: {rest}", ProjectFlagRuleId(id));
                    }
                    line.to_owned()
                })
                .collect::<Vec<_>>()
                .join("\n")
        });
        result.rules = std::mem::take(&mut result.rules)
            .into_iter()
            .map(|(id, rule)| {
                let public_id = id
                    .parse::<u64>()
                    .map(|id| ProjectFlagRuleId(id).to_string())
                    .unwrap_or(id);
                (public_id, rule)
            })
            .collect();
    }
    Ok(web::Json(results))
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ProjectFlagRunActivity {
    pub num_scanning: i64,
    pub num_workers: usize,
}

/// Count unfinished projects and recently polling Temporal activity worker identities.
#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    security(("bearer_auth" = [])),
    responses((status = OK, body = ProjectFlagRunActivity))
)]
#[get("/rules/run/activity")]
pub async fn get_run_activity(
    req: HttpRequest,
    temporal: web::Data<TemporalClient>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ProjectFlagRunActivity>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .auth_context("authenticating moderator")?;
    let (num_scanning, num_workers) = tokio::try_join!(
        DBProjectFlagRun::num_scanning(&ro_pool),
        run::num_workers(&temporal),
    )
    .internal_context("fetching project flag scan activity")?;
    Ok(web::Json(ProjectFlagRunActivity {
        num_scanning,
        num_workers,
    }))
}

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListRuns {
    #[serde(default = "default_limit")]
    pub limit: u16,
    #[serde(default)]
    pub offset: u32,
}

#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    params(ListRuns),
    responses((status = OK, body = [ProjectFlagRunResponse]))
)]
#[get("/rules/run")]
pub async fn list_runs(
    req: HttpRequest,
    query: web::Query<ListRuns>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<ProjectFlagRunResponse>>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .auth_context("authenticating moderator")?;

    if query.limit == 0 || query.limit > 200 {
        return Err(ApiError::Request(anyhow!(
            "`limit` must be between 1 and 200"
        )));
    }
    let runs = DBProjectFlagRun::list(
        i64::from(query.limit),
        i64::from(query.offset),
        &ro_pool,
    )
    .await
    .internal_context("listing project flag runs")?;
    Ok(web::Json(
        ProjectFlagRunResponse::from_runs(runs, &ro_pool)
            .await
            .internal_context("aggregating project flag run counts")?,
    ))
}

#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    security(("bearer_auth" = [])),
    responses((status = OK, body = ProjectFlagRunResponse))
)]
#[post("/rules/run")]
pub async fn start_run(
    req: HttpRequest,
    temporal: web::Data<TemporalClient>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ProjectFlagRunResponse>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_WRITE,
    )
    .await
    .auth_context("authenticating moderator")?;

    let project_ids = DBProjectFlagRun::queued_project_ids(&pool)
        .await
        .internal_context("fetching queued project IDs")?;
    let input =
        run::RunInput::new(ProjectFlagRunId(uuid::Uuid::now_v7()), project_ids);

    let response_pool = pool.clone();
    let run = tokio::spawn(async move {
        let run = DBProjectFlagRun::create(
            input.id, user.user.id.into(), input.projects.len(), &pool,
        ).await.internal_context("creating project flag run")?;
        tokio::time::timeout(std::time::Duration::from_secs(30), run::start(&temporal, input))
            .await
            .internal_context(format!("timed out confirming project flag run {}; check Temporal workflow {} before starting another run", run.id.0, run::workflow_id(run.id)))?
            .internal_context(format!("starting project flag run {}; check Temporal workflow {} before starting another run", run.id.0, run::workflow_id(run.id)))?;
        Ok::<_, ApiError>(run)
    }).await.internal_context("joining project flag run start")??;
    let mut responses =
        ProjectFlagRunResponse::from_runs(vec![run], &response_pool)
            .await
            .internal_context("aggregating project flag run counts")?;
    Ok(web::Json(responses.remove(0)))
}

/// Delete a run, its results, and its inactive flags, preserving active flags.
#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    security(("bearer_auth" = [])),
    params(("id" = ProjectFlagRunId, Path)),
    responses((status = OK), (status = NOT_FOUND))
)]
#[delete("/rules/run/{id}")]
pub async fn delete_run(
    req: HttpRequest,
    path: web::Path<ProjectFlagRunId>,
    temporal: web::Data<TemporalClient>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_WRITE,
    )
    .await
    .auth_context("authenticating moderator")?;
    let id = path.into_inner();
    if !DBProjectFlagRun::delete(id, &pool)
        .await
        .internal_context("deleting project flag run")?
    {
        return Err(ApiError::NotFound(anyhow!("project flag run not found")));
    }
    if let Err(error) = temporal
        .client
        .get_workflow_handle::<run::ProjectFlagRunWorkflow>(run::workflow_id(
            id,
        ))
        .cancel(temporalio_client::WorkflowCancelOptions::default())
        .await
    {
        tracing::warn!(run_id = %id.0, ?error, "Run deleted; workflow cancellation was not confirmed");
    }
    Ok(())
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ProjectFlagDeployment {
    pub ruleset_id: uuid::Uuid,
    pub deployed_by: Option<ariadne::ids::UserId>,
    pub deployed_at: chrono::DateTime<chrono::Utc>,
}

/// The active production revision, or null when automatic rules are disabled.
#[utoipa::path(
    context_path = "/moderation/project-flag", tag = "moderation",
    security(("bearer_auth" = [])),
    responses((status = OK, body = Option<ProjectFlagDeployment>))
)]
#[get("/deployment")]
pub async fn get_deployment(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Option<ProjectFlagDeployment>>, ApiError> {
    get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .auth_context("authenticating moderator")?;
    let row = sqlx::query!(r#"
        SELECT ruleset_id, deployed_by, deployed_at FROM project_flags_deployed WHERE singleton
    "#).fetch_optional(&**pool).await.internal_context("loading project flag deployment")?;
    Ok(web::Json(row.map(|row| {
        ProjectFlagDeployment {
            ruleset_id: row.ruleset_id,
            deployed_by: row
                .deployed_by
                .map(|id| crate::database::models::DBUserId(id).into()),
            deployed_at: row.deployed_at,
        }
    })))
}

/// Deploys the run's frozen ruleset; does not apply its proposed project review outcomes.
#[utoipa::path(
    context_path = "/moderation/project-flag", tag = "moderation",
    security(("bearer_auth" = [])), params(("id" = ProjectFlagRunId, Path)),
    responses((status = OK, body = ProjectFlagDeployment), (status = BAD_REQUEST), (status = NOT_FOUND))
)]
#[post("/rules/run/{id}/deploy")]
pub async fn deploy_run(
    req: HttpRequest,
    path: web::Path<ProjectFlagRunId>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ProjectFlagDeployment>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_WRITE,
    )
    .await
    .auth_context("authenticating moderator")?;
    let mut transaction = pool
        .begin()
        .await
        .internal_context("starting project flag deployment")?;
    let run = sqlx::query!(r#"
        SELECT ruleset_id, status, completed_at FROM project_flag_runs WHERE id = $1 FOR UPDATE
    "#, path.into_inner().0).fetch_optional(&mut transaction).await
        .internal_context("locking project flag run for deployment")?
        .not_found_context("project flag run not found")?;
    if run.status != "done" || run.completed_at.is_none() {
        return Err(ApiError::Request(anyhow!(
            "only completed runs can be deployed"
        )));
    }
    let ruleset_id = run
        .ruleset_id
        .request_context("run has no frozen ruleset")?;
    let user_id = crate::database::models::DBUserId::from(user.user.id);
    let row = sqlx::query!(r#"
        INSERT INTO project_flags_deployed (singleton, ruleset_id, deployed_by, deployed_at)
        VALUES (TRUE, $1, $2, clock_timestamp())
        ON CONFLICT (singleton) DO UPDATE SET ruleset_id = EXCLUDED.ruleset_id,
            deployed_by = EXCLUDED.deployed_by, deployed_at = EXCLUDED.deployed_at
        RETURNING deployed_at
    "#, ruleset_id, user_id.0).fetch_one(&mut transaction).await
        .internal_context("deploying project flag ruleset")?;
    transaction
        .commit()
        .await
        .internal_context("committing project flag deployment")?;
    Ok(web::Json(ProjectFlagDeployment {
        ruleset_id,
        deployed_by: Some(user_id.into()),
        deployed_at: row.deployed_at,
    }))
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ProjectFlagEvaluationResult {
    pub project_id: ProjectId,
    #[serde(flatten)]
    pub evaluation: ProjectFlagEvaluation,
    pub error: Option<String>,
}

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct EvalRule {
    #[validate(length(min = 1, max = 100))]
    pub project_ids: Vec<ProjectId>,
    #[validate(nested)]
    pub params: ProjectRuleParams,
}

#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    request_body = EvalRule,
    responses(
        (status = OK, body = Vec<ProjectFlagEvaluationResult>),
        (status = BAD_REQUEST)
    )
)]
#[post("/rule/eval")]
pub async fn eval_rule(
    req: HttpRequest,
    body: web::Json<EvalRule>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    engine: web::Data<ProjectFlagEngine>,
) -> Result<web::Json<Vec<ProjectFlagEvaluationResult>>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .auth_context("authenticating moderator")?;
    body.validate().map_err(|error| {
        ApiError::Request(anyhow!(validation_errors_to_string(error, None)))
    })?;
    let body = body.into_inner();
    let engine = engine
        .with_module_snapshot()
        .await
        .internal_context("loading project flag module snapshot")?;
    let compile_engine = engine.clone();
    let call = tokio::task::spawn_blocking(move || {
        compile_engine.make_call(&body.params)
    })
    .await
    .internal_context("joining project flag compilation")?
    .request_context("compiling project flag expression")?;
    let mut seen = HashSet::new();
    let project_ids =
        body.project_ids.into_iter().filter(|id| seen.insert(*id));
    let evaluations = stream::iter(project_ids)
        .map(|project_id| {
            let call = &call;
            let engine = &engine;
            let ro_pool = &***ro_pool;
            let redis = &redis;
            async move {
                let evaluation = async {
                    DBProject::get_id(project_id.into(), ro_pool, redis)
                        .await
                        .internal_context(
                            "fetching project for flag evaluation",
                        )?
                        .not_found_context("project not found")?;
                    engine
                        .scan_flags_with_log(call, project_id.into())
                        .await
                        .request_context("evaluating project flag expression")
                }
                .await;
                match evaluation {
                    Ok(evaluation) => ProjectFlagEvaluationResult {
                        project_id,
                        evaluation,
                        error: None,
                    },
                    Err(error) => ProjectFlagEvaluationResult {
                        project_id,
                        evaluation: ProjectFlagEvaluation {
                            flags: Vec::new(),
                            log: Vec::new(),
                        },
                        error: Some(error.to_string()),
                    },
                }
            }
        })
        .buffered(4)
        .collect::<Vec<_>>()
        .await;
    Ok(web::Json(evaluations))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_batch_project_ids() {
        let body: EvalRule = serde_json::from_value(json!({
            "project_ids": ["Abcd", "Efgh"],
            "params": { "expression": "[]" },
        }))
        .unwrap();
        assert!(body.validate().is_ok());
        assert_eq!(body.project_ids.len(), 2);
    }

    #[test]
    fn rejects_empty_and_oversized_batches() {
        for ids in [Vec::<String>::new(), vec!["Abcd".to_owned(); 101]] {
            let body: EvalRule = serde_json::from_value(json!({
                "project_ids": ids,
                "params": { "expression": "[]" },
            }))
            .unwrap();
            assert!(body.validate().is_err());
        }
    }

    #[test]
    fn serializes_per_project_evaluation() {
        let project_id = serde_json::from_value(json!("Abcd")).unwrap();
        let result = ProjectFlagEvaluationResult {
            project_id,
            evaluation: ProjectFlagEvaluation {
                flags: Vec::new(),
                log: vec!["tested".to_owned()],
            },
            error: None,
        };
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            json!({
                "project_id": "Abcd", "flags": [], "log": ["tested"], "error": null,
            })
        );
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ProjectFlagRuleResponse {
    pub id: ProjectFlagRuleId,
    pub name: String,
    pub created_by: Option<crate::database::models::DBUserId>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_by: Option<crate::database::models::DBUserId>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub params: ProjectRuleParams,
}

impl From<DBProjectFlagRule> for ProjectFlagRuleResponse {
    fn from(rule: DBProjectFlagRule) -> Self {
        Self {
            id: rule.id.into(),
            name: rule.name,
            created_by: rule.created_by,
            created_at: rule.created_at,
            updated_by: rule.updated_by,
            updated_at: rule.updated_at,
            params: rule.params,
        }
    }
}

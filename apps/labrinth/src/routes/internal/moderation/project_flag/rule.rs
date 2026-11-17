use actix_web::{HttpRequest, delete, get, post, put, web};
use eyre::eyre;
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
    DBProject, DBProjectFlagRule, DBProjectFlagRun, ProjectFlagRuleId,
    ProjectFlagRunId,
};
use crate::database::{PgPool, ReadOnlyPgPool};
use crate::models::ids::ProjectId;
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
    .wrap_auth_err("authenticating moderator")?;

    Ok(web::Json(
        schema_example::generate::<FlagContext>()
            .wrap_internal_err("generating project flag context example")?,
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
    responses((status = OK, body = [DBProjectFlagRule]))
)]
#[get("/rules")]
pub async fn list_rules(
    req: HttpRequest,
    query: web::Query<ListRules>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<DBProjectFlagRule>>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .wrap_auth_err("authenticating moderator")?;

    if query.limit == 0 || query.limit > 200 {
        return Err(ApiError::Request(eyre!(
            "`limit` must be between 1 and 200"
        )));
    }
    let rules = DBProjectFlagRule::list(
        i64::from(query.limit),
        i64::from(query.offset),
        &ro_pool,
    )
    .await
    .wrap_internal_err("listing project flag rules")?;
    Ok(web::Json(rules))
}

#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    params(("id" = ProjectFlagRuleId, Path)),
    responses((status = OK, body = DBProjectFlagRule), (status = NOT_FOUND))
)]
#[get("/rules/{id}")]
pub async fn get_rule(
    req: HttpRequest,
    path: web::Path<ProjectFlagRuleId>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<DBProjectFlagRule>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .wrap_auth_err("authenticating moderator")?;

    let rule = DBProjectFlagRule::get(path.into_inner(), &ro_pool)
        .await
        .wrap_internal_err("fetching project flag rule")?
        .wrap_not_found_err("project flag rule not found")?;
    Ok(web::Json(rule))
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
    responses((status = OK, body = DBProjectFlagRule))
)]
#[post("/rules")]
pub async fn create_rule(
    req: HttpRequest,
    body: web::Json<CreateRule>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<DBProjectFlagRule>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_WRITE,
    )
    .await
    .wrap_auth_err("authenticating moderator")?;
    body.validate().map_err(|error| {
        ApiError::Request(eyre!(validation_errors_to_string(error, None)))
    })?;

    let rule = DBProjectFlagRule::create(
        &body.name,
        &body.params,
        user.user.id.into(),
        &pool,
    )
    .await
    .wrap_internal_err("creating project flag rule")?;
    Ok(web::Json(rule))
}

#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    params(("id" = ProjectFlagRuleId, Path)),
    request_body = CreateRule,
    responses((status = OK, body = DBProjectFlagRule), (status = NOT_FOUND))
)]
#[put("/rules/{id}")]
pub async fn update_rule(
    req: HttpRequest,
    path: web::Path<ProjectFlagRuleId>,
    body: web::Json<CreateRule>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<DBProjectFlagRule>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_WRITE,
    )
    .await
    .wrap_auth_err("authenticating moderator")?;
    body.validate().map_err(|error| {
        ApiError::Request(eyre!(validation_errors_to_string(error, None)))
    })?;

    let rule = DBProjectFlagRule::update(
        path.into_inner(),
        &body.name,
        &body.params,
        user.user.id.into(),
        &pool,
    )
    .await
    .wrap_internal_err("updating project flag rule")?
    .wrap_not_found_err("project flag rule not found")?;
    Ok(web::Json(rule))
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
    .wrap_auth_err("authenticating moderator")?;

    if !DBProjectFlagRule::delete(path.into_inner(), &pool)
        .await
        .wrap_internal_err("deleting project flag rule")?
    {
        return Err(ApiError::NotFound(eyre!("project flag rule not found")));
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
    .wrap_auth_err("authenticating moderator")?;

    if query.limit == 0 {
        return Err(ApiError::Request(eyre!("`limit` must be positive")));
    }
    let results = DBProjectFlagRunResult::list(
        path.into_inner(),
        i64::from(query.limit),
        i64::from(query.offset),
        &ro_pool,
    )
    .await
    .wrap_internal_err("listing project flag run results")?
    .wrap_not_found_err("project flag run not found")?;
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
    .wrap_auth_err("authenticating moderator")?;
    let (num_scanning, num_workers) = tokio::try_join!(
        DBProjectFlagRun::num_scanning(&ro_pool),
        run::num_workers(&temporal),
    )
    .wrap_internal_err("fetching project flag scan activity")?;
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
    .wrap_auth_err("authenticating moderator")?;

    if query.limit == 0 || query.limit > 200 {
        return Err(ApiError::Request(eyre!(
            "`limit` must be between 1 and 200"
        )));
    }
    let runs = DBProjectFlagRun::list(
        i64::from(query.limit),
        i64::from(query.offset),
        &ro_pool,
    )
    .await
    .wrap_internal_err("listing project flag runs")?;
    Ok(web::Json(
        ProjectFlagRunResponse::from_runs(runs, &ro_pool)
            .await
            .wrap_internal_err("aggregating project flag run counts")?,
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
    .wrap_auth_err("authenticating moderator")?;

    let project_ids = DBProjectFlagRun::queued_project_ids(&pool)
        .await
        .wrap_internal_err("fetching queued project IDs")?;
    let run =
        DBProjectFlagRun::create(user.user.id.into(), &project_ids, &pool)
            .await
            .wrap_internal_err("creating project flag run")?;

    let response_pool = pool.clone();
    let run = tokio::spawn(async move {
        tokio::time::timeout(std::time::Duration::from_secs(30), run::start(&temporal, run.id))
            .await
            .wrap_internal_err(format!("timed out confirming project flag run {}; restart the workflow worker to recover persisted work", run.id.0))?
            .wrap_internal_err(format!("starting project flag run {}; restart the workflow worker to recover persisted work", run.id.0))?;
        Ok::<_, ApiError>(run)
    }).await.wrap_internal_err("joining project flag run start")??;
    let mut responses =
        ProjectFlagRunResponse::from_runs(vec![run], &response_pool)
            .await
            .wrap_internal_err("aggregating project flag run counts")?;
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
    .wrap_auth_err("authenticating moderator")?;
    let id = path.into_inner();
    if !DBProjectFlagRun::delete(id, &pool)
        .await
        .wrap_internal_err("deleting project flag run")?
    {
        return Err(ApiError::NotFound(eyre!("project flag run not found")));
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
    .wrap_auth_err("authenticating moderator")?;
    body.validate().map_err(|error| {
        ApiError::Request(eyre!(validation_errors_to_string(error, None)))
    })?;
    let body = body.into_inner();
    let engine = engine
        .with_module_snapshot()
        .await
        .wrap_internal_err("loading project flag module snapshot")?;
    let compile_engine = engine.clone();
    let call = tokio::task::spawn_blocking(move || {
        compile_engine.make_call(&body.params)
    })
    .await
    .wrap_internal_err("joining project flag compilation")?
    .wrap_request_err("compiling project flag expression")?;
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
                        .wrap_internal_err(
                            "fetching project for flag evaluation",
                        )?
                        .wrap_not_found_err("project not found")?;
                    engine
                        .scan_flags_with_log(call, project_id.into())
                        .await
                        .wrap_request_err("evaluating project flag expression")
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

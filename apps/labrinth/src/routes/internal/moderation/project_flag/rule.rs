use actix_web::{HttpRequest, delete, get, post, put, web};
use eyre::eyre;
use serde::Deserialize;
use validator::Validate;
use xredis::RedisPool;

use crate::auth::get_user_from_headers;
use crate::database::models::project_flag_item::{
    DBProjectFlagRunResult, ProjectFlagRunResults, ProjectFlagRunStatus,
    ProjectRuleParams,
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
use crate::queue::moderation::project_flag_run::ProjectFlagDispatchUncertain;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context as _;
use crate::util::kafka::KafkaClientState;
use crate::util::validate::{validate_name, validation_errors_to_string};

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
    #[param(default = 25, minimum = 1, maximum = 200)]
    pub limit: u16,
    #[serde(default)]
    #[param(default = 0)]
    pub offset: u32,
}

fn default_results_limit() -> u16 {
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

    if query.limit == 0 || query.limit > 200 {
        return Err(ApiError::Request(eyre!(
            "`limit` must be between 1 and 200"
        )));
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
    pub num_consumers: usize,
}

/// Count unfinished projects and consumers assigned to the project scan topic.
#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    security(("bearer_auth" = [])),
    responses((status = OK, body = ProjectFlagRunActivity))
)]
#[get("/rules/run/activity")]
pub async fn get_run_activity(
    req: HttpRequest,
    kafka: web::Data<KafkaClientState>,
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
    let (num_scanning, num_consumers) = tokio::try_join!(
        DBProjectFlagRun::num_scanning(&ro_pool),
        kafka.project_flag_scan_consumers(),
    )
    .wrap_internal_err("fetching project flag scan activity")?;
    Ok(web::Json(ProjectFlagRunActivity {
        num_scanning,
        num_consumers,
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
    kafka: web::Data<KafkaClientState>,
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
    let num_queued = if runs
        .iter()
        .any(|run| run.status == ProjectFlagRunStatus::Running)
    {
        kafka
            .project_flag_scan_lag()
            .await
            .wrap_internal_err("fetching project flag scan lag")?
    } else {
        0
    };
    Ok(web::Json(
        runs.into_iter()
            .map(|run| ProjectFlagRunResponse::from_run(run, num_queued))
            .collect(),
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
    kafka: web::Data<KafkaClientState>,
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

    let rules = DBProjectFlagRule::all(&pool)
        .await
        .wrap_internal_err("snapshotting project flag rules")?;
    let project_ids = DBProjectFlagRun::queued_project_ids(&pool)
        .await
        .wrap_internal_err("fetching queued project IDs")?;
    let mut run =
        DBProjectFlagRun::create(user.user.id.into(), &project_ids, &pool)
            .await
            .wrap_internal_err("creating project flag run")?;

    let response_kafka = kafka.clone();
    // Keep dispatch alive if the caller disconnects while Kafka is publishing.
    let run = tokio::spawn(async move {
        let empty = project_ids.is_empty();
        let result = crate::queue::moderation::project_flag_run::dispatch_project_flag_run(
            run.id, rules, project_ids, &redis, &kafka,
        ).await;
        if let Err(error) = result {
            if error.downcast_ref::<ProjectFlagDispatchUncertain>().is_some() {
                tracing::error!(run_id = %run.id.0, error = %error, "Project flag run dispatch outcome unknown; leaving run active");
                return Err(ApiError::Internal(error));
            }
            run.finish(Some(format!("{error:#}")), &pool)
                .await
                .wrap_internal_err("recording project flag run dispatch failure")?;
            return Err(ApiError::Internal(error));
        }
        if empty {
            run.finish(None, &pool)
                .await
                .wrap_internal_err("completing empty project flag run")?;
        }
        Ok(run)
    })
    .await
    .wrap_internal_err("joining project flag run dispatch")??;
    let num_queued = if run.status == ProjectFlagRunStatus::Running {
        response_kafka
            .project_flag_scan_lag()
            .await
            .wrap_internal_err("fetching project flag scan lag")?
    } else {
        0
    };
    Ok(web::Json(ProjectFlagRunResponse::from_run(run, num_queued)))
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
    let cleanup = async {
        let mut connection = redis.connect().await?;
        let key = connection.key().entity("project_flag_run_rules:v1", id.0);
        redis::cmd("DEL")
            .arg(key)
            .query_async::<i64>(&mut connection)
            .await?;
        eyre::Ok(())
    }
    .await;
    if let Err(error) = cleanup {
        tracing::warn!(run_id = %id.0, ?error, "Failed to remove deleted run rule snapshot");
    }
    Ok(())
}

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct EvalRule {
    pub project_id: ProjectId,
    #[validate(nested)]
    pub params: ProjectRuleParams,
}

#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    request_body = EvalRule,
    responses(
        (status = OK, body = ProjectFlagEvaluation),
        (status = BAD_REQUEST),
        (status = NOT_FOUND)
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
) -> Result<web::Json<ProjectFlagEvaluation>, ApiError> {
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
    let project_id = body.project_id.into();
    DBProject::get_id(project_id, &***ro_pool, &redis)
        .await
        .wrap_internal_err("fetching project for flag evaluation")?
        .wrap_not_found_err("project not found")?;
    let compile_engine = engine.clone();
    let call = tokio::task::spawn_blocking(move || {
        compile_engine.make_call(&body.params)
    })
    .await
    .wrap_internal_err("joining project flag compilation")?
    .wrap_request_err("compiling project flag expression")?;
    let evaluation = engine
        .scan_flags_with_log(&call, project_id)
        .await
        .wrap_request_err("evaluating project flag expression")?;
    Ok(web::Json(evaluation))
}

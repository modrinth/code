use actix_web::{HttpRequest, delete, get, post, put, web};
use eyre::eyre;
use serde::Deserialize;
use validator::Validate;
use xredis::RedisPool;

use crate::auth::check_is_moderator_from_headers;
use crate::database::models::project_flag_item::ProjectRuleParams;
use crate::database::models::{
    DBProject, DBProjectFlagRule, DBProjectFlagRun, ProjectFlagRuleId,
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
use crate::util::validate::{validate_name, validation_errors_to_string};

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
    check_is_moderator_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
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
    check_is_moderator_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
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
    check_is_moderator_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
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
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating moderator")?;
    body.validate().map_err(|error| {
        ApiError::Request(eyre!(validation_errors_to_string(error, None)))
    })?;

    let rule = DBProjectFlagRule::create(
        &body.name,
        &body.params,
        user.id.into(),
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
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
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
        user.id.into(),
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
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
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
    responses((status = OK, body = [DBProjectFlagRun]))
)]
#[get("/rules/run")]
pub async fn list_runs(
    req: HttpRequest,
    query: web::Query<ListRuns>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<DBProjectFlagRun>>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
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
    Ok(web::Json(runs))
}

#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    security(("bearer_auth" = [])),
    responses((status = OK, body = DBProjectFlagRun))
)]
#[post("/rules/run")]
pub async fn start_run(
    req: HttpRequest,
    kafka: web::Data<crate::util::kafka::KafkaClientState>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<DBProjectFlagRun>, ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating moderator")?;

    let rules = DBProjectFlagRule::all(&pool)
        .await
        .wrap_internal_err("snapshotting project flag rules")?;
    let project_ids = DBProjectFlagRun::queued_project_ids(&pool)
        .await
        .wrap_internal_err("fetching queued project IDs")?;
    let total_projects = i64::try_from(project_ids.len())
        .wrap_internal_err("counting queued project IDs")?;
    let mut run =
        DBProjectFlagRun::create(user.id.into(), total_projects, &pool)
            .await
            .wrap_internal_err("creating project flag run")?;

    // Keep dispatch alive if the caller disconnects while Kafka is publishing.
    let run = tokio::spawn(async move {
        let empty = project_ids.is_empty();
        let result = crate::queue::moderation::project_flag_run::dispatch_project_flag_run(
            run.id, rules, project_ids, &redis, &kafka,
        ).await;
        if let Err(error) = result {
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
        Ok(web::Json(run))
    })
    .await
    .wrap_internal_err("joining project flag run dispatch")??;
    Ok(run)
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
    check_is_moderator_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
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

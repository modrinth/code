use actix_web::{HttpRequest, delete, get, post, put, web};
use anyhow::anyhow;
use serde::Deserialize;
use xredis::RedisPool;

use crate::auth::get_user_from_headers;
use crate::database::models::project_flag_item::DBProjectFlagModule;
use crate::database::{PgPool, ReadOnlyPgPool};
use crate::models::pats::Scopes;
use crate::queue::moderation::project_flag::ProjectFlagEngine;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context as _;

use super::rule::ListRules;

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ModuleName {
    pub name: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct WriteModule {
    pub name: String,
    pub script: String,
}

fn validate_module_name(name: &str) -> Result<(), ApiError> {
    if name.chars().count() > 1024
        || name.contains(['\\', '\0'])
        || name
            .split('/')
            .any(|segment| matches!(segment, "" | "." | ".."))
    {
        return Err(ApiError::Request(anyhow!(
            "module name must be a canonical slash-delimited name of at most 1024 characters without empty, `.` or `..` segments, backslashes or null characters"
        )));
    }
    Ok(())
}

fn validate_module(
    body: &WriteModule,
    engine: &ProjectFlagEngine,
) -> Result<(), ApiError> {
    validate_module_name(&body.name)?;
    if body.script.chars().count() > 65535 || body.script.contains('\0') {
        return Err(ApiError::Request(anyhow!(
            "module script must be at most 65535 characters without null characters"
        )));
    }
    engine
        .validate_module_script(&body.script)
        .request_context("invalid project flag module script")
}

fn write_error(error: anyhow::Error) -> ApiError {
    if error
        .downcast_ref::<sqlx::Error>()
        .and_then(sqlx::Error::as_database_error)
        .is_some_and(|error| error.is_unique_violation())
    {
        ApiError::Conflict(anyhow!("project flag module name already exists"))
    } else {
        ApiError::Internal(error)
    }
}

#[utoipa::path(
    context_path = "/moderation/project-flag", tag = "moderation",
    params(ListRules), responses((status = OK, body = [DBProjectFlagModule]))
)]
#[get("/modules")]
pub async fn list_modules(
    req: HttpRequest,
    query: web::Query<ListRules>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<DBProjectFlagModule>>, ApiError> {
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
    let modules = DBProjectFlagModule::list(
        i64::from(query.limit),
        i64::from(query.offset),
        &ro_pool,
    )
    .await
    .internal_context("listing project flag modules")?;
    Ok(web::Json(modules))
}

#[utoipa::path(
    context_path = "/moderation/project-flag", tag = "moderation",
    params(ModuleName), responses((status = OK, body = DBProjectFlagModule), (status = NOT_FOUND))
)]
#[get("/module")]
pub async fn get_module(
    req: HttpRequest,
    query: web::Query<ModuleName>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<DBProjectFlagModule>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .auth_context("authenticating moderator")?;
    validate_module_name(&query.name)?;
    let module = DBProjectFlagModule::get(&query.name, &ro_pool)
        .await
        .internal_context("fetching project flag module")?
        .not_found_context("project flag module not found")?;
    Ok(web::Json(module))
}

#[utoipa::path(
    context_path = "/moderation/project-flag", tag = "moderation",
    request_body = WriteModule,
    responses((status = OK, body = DBProjectFlagModule), (status = CONFLICT))
)]
#[post("/modules")]
pub async fn create_module(
    req: HttpRequest,
    body: web::Json<WriteModule>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    engine: web::Data<ProjectFlagEngine>,
) -> Result<web::Json<DBProjectFlagModule>, ApiError> {
    get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_WRITE,
    )
    .await
    .auth_context("authenticating moderator")?;
    validate_module(&body, &engine)?;
    let module = DBProjectFlagModule::create(&body.name, &body.script, &pool)
        .await
        .map_err(write_error)?;
    Ok(web::Json(module))
}

#[utoipa::path(
    context_path = "/moderation/project-flag", tag = "moderation",
    params(ModuleName), request_body = WriteModule,
    responses((status = OK, body = DBProjectFlagModule), (status = NOT_FOUND), (status = CONFLICT))
)]
#[put("/module")]
pub async fn update_module(
    req: HttpRequest,
    query: web::Query<ModuleName>,
    body: web::Json<WriteModule>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    engine: web::Data<ProjectFlagEngine>,
) -> Result<web::Json<DBProjectFlagModule>, ApiError> {
    get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_WRITE,
    )
    .await
    .auth_context("authenticating moderator")?;
    validate_module_name(&query.name)?;
    validate_module(&body, &engine)?;
    let module = DBProjectFlagModule::update(
        &query.name,
        &body.name,
        &body.script,
        &pool,
    )
    .await
    .map_err(write_error)?
    .not_found_context("project flag module not found")?;
    Ok(web::Json(module))
}

#[utoipa::path(
    context_path = "/moderation/project-flag", tag = "moderation",
    params(ModuleName), responses((status = NO_CONTENT), (status = NOT_FOUND))
)]
#[delete("/module")]
pub async fn delete_module(
    req: HttpRequest,
    query: web::Query<ModuleName>,
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
    validate_module_name(&query.name)?;
    if !DBProjectFlagModule::delete(&query.name, &pool)
        .await
        .internal_context("deleting project flag module")?
    {
        return Err(ApiError::NotFound(anyhow!(
            "project flag module not found"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_module_name;

    #[test]
    fn canonical_names() {
        for name in [
            "",
            "/foo",
            "foo/",
            "foo//bar",
            ".",
            "..",
            "foo/./bar",
            "foo/../bar",
            "foo\\bar",
            "foo\0bar",
        ] {
            assert!(validate_module_name(name).is_err(), "{name:?}");
        }
        for name in ["foo", "foo/bar", "foo/bar.rhai", "...", "模块/规则"] {
            assert!(validate_module_name(name).is_ok(), "{name:?}");
        }
        assert!(validate_module_name(&"é".repeat(1024)).is_ok());
        assert!(validate_module_name(&"é".repeat(1025)).is_err());
    }
}

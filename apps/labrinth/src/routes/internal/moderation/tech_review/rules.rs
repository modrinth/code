use actix_web::{HttpRequest, delete, get, post, put, web};
use chrono::{DateTime, Utc};
use eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::{
    auth::check_is_moderator_from_headers,
    database::{PgPool, ReadOnlyPgPool, redis::RedisPool},
    models::pats::Scopes,
    queue::session::AuthQueue,
    routes::ApiError,
    util::error::Context,
};

const MAX_RULE_NAME_LENGTH: usize = 128;
const MAX_RULE_EXPRESSION_LENGTH: usize = 65_536;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_rules)
        .service(create_rule)
        .service(update_rule)
        .service(delete_rule);
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DelphiRule {
    pub id: i64,
    pub name: String,
    pub priority: i32,
    pub expression: String,
    pub revision_id: i64,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub revision_created: DateTime<Utc>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub revision_created_by: Option<i64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct WriteDelphiRule {
    pub name: String,
    pub priority: i32,
    pub expression: String,
}

struct ValidatedRule {
    name: String,
    priority: i32,
    expression: String,
}

impl WriteDelphiRule {
    fn validate(self) -> Result<ValidatedRule, ApiError> {
        let name = self.name.trim().to_string();
        if name.is_empty() {
            return Err(ApiError::Request(eyre!("rule name cannot be empty")));
        }
        if name.chars().count() > MAX_RULE_NAME_LENGTH {
            return Err(ApiError::Request(eyre!(
                "rule name cannot exceed {MAX_RULE_NAME_LENGTH} characters"
            )));
        }

        let expression = self.expression.trim().to_string();
        if expression.is_empty() {
            return Err(ApiError::Request(eyre!(
                "rule expression cannot be empty"
            )));
        }
        if expression.len() > MAX_RULE_EXPRESSION_LENGTH {
            return Err(ApiError::Request(eyre!(
                "rule expression cannot exceed {MAX_RULE_EXPRESSION_LENGTH} bytes"
            )));
        }

        cel::Program::compile(&expression).map_err(|error| {
            ApiError::Request(eyre!("invalid cel expression: {error}"))
        })?;

        Ok(ValidatedRule {
            name,
            priority: self.priority,
            expression,
        })
    }
}

/// List the current revision of every Delphi rule.
#[utoipa::path(
	context_path = "/moderation/tech-review",
	tag = "moderation",
	security(("bearer_auth" = [])),
	responses((status = OK, body = Vec<DelphiRule>))
)]
#[get("/rules")]
pub async fn get_rules(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<DelphiRule>>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let rows = sqlx::query!(
        r#"
		SELECT
			r.id,
			r.name,
			r.priority,
			r.created,
			r.updated,
			r.created_by,
			r.updated_by,
			rr.id AS revision_id,
			rr.expression,
			rr.created AS revision_created,
			rr.created_by AS revision_created_by
		FROM delphi_rules r
		INNER JOIN delphi_rule_revisions rr
			ON rr.rule_id = r.id
			AND rr.active
		ORDER BY r.priority DESC, r.id
		"#,
    )
    .fetch_all(&***ro_pool)
    .await
    .wrap_internal_err("failed to fetch delphi rules")?;

    Ok(web::Json(
        rows.into_iter()
            .map(|row| DelphiRule {
                id: row.id,
                name: row.name,
                priority: row.priority,
                expression: row.expression,
                revision_id: row.revision_id,
                created: row.created,
                updated: row.updated,
                revision_created: row.revision_created,
                created_by: row.created_by,
                updated_by: row.updated_by,
                revision_created_by: row.revision_created_by,
            })
            .collect(),
    ))
}

/// Create a Delphi rule and its first revision.
#[utoipa::path(
	context_path = "/moderation/tech-review",
	tag = "moderation",
	security(("bearer_auth" = [])),
	request_body = WriteDelphiRule,
	responses((status = OK, body = DelphiRule))
)]
#[post("/rules")]
pub async fn create_rule(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<WriteDelphiRule>,
) -> Result<web::Json<DelphiRule>, ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?;
    let rule = body.into_inner().validate()?;
    let user_id = user.id.0 as i64;
    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("failed to begin delphi rule transaction")?;

    let row = sqlx::query!(
        r#"
		INSERT INTO delphi_rules (
			name,
			priority,
			created_by,
			updated_by
		)
		VALUES ($1, $2, $3, $3)
		RETURNING id, name, priority, created, updated, created_by, updated_by
		"#,
        rule.name,
        rule.priority,
        user_id,
    )
    .fetch_one(&mut transaction)
    .await
    .wrap_internal_err("failed to create delphi rule")?;

    let revision = sqlx::query!(
        r#"
		INSERT INTO delphi_rule_revisions (rule_id, expression, created_by)
		VALUES ($1, $2, $3)
		RETURNING id, expression, created, created_by
		"#,
        row.id,
        rule.expression,
        user_id,
    )
    .fetch_one(&mut transaction)
    .await
    .wrap_internal_err("failed to create delphi rule revision")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit delphi rule transaction")?;

    Ok(web::Json(DelphiRule {
        id: row.id,
        name: row.name,
        priority: row.priority,
        expression: revision.expression,
        revision_id: revision.id,
        created: row.created,
        updated: row.updated,
        revision_created: revision.created,
        created_by: row.created_by,
        updated_by: row.updated_by,
        revision_created_by: revision.created_by,
    }))
}

/// Replace a Delphi rule and create a new current revision.
#[utoipa::path(
	context_path = "/moderation/tech-review",
	tag = "moderation",
	security(("bearer_auth" = [])),
	request_body = WriteDelphiRule,
	responses((status = OK, body = DelphiRule), (status = NOT_FOUND))
)]
#[put("/rules/{id}")]
pub async fn update_rule(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(i64,)>,
    body: web::Json<WriteDelphiRule>,
) -> Result<web::Json<DelphiRule>, ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?;
    let (id,) = path.into_inner();
    let rule = body.into_inner().validate()?;
    let user_id = user.id.0 as i64;
    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("failed to begin delphi rule transaction")?;

    let row = sqlx::query!(
        r#"
		UPDATE delphi_rules
		SET
			name = $2,
			priority = $3,
			updated = CURRENT_TIMESTAMP,
			updated_by = $4
		WHERE id = $1
		RETURNING id, name, priority, created, updated, created_by, updated_by
		"#,
        id,
        rule.name,
        rule.priority,
        user_id,
    )
    .fetch_optional(&mut transaction)
    .await
    .wrap_internal_err("failed to update delphi rule")?
    .ok_or(ApiError::NotFound)?;

    sqlx::query!(
        r#"
		UPDATE delphi_rule_revisions
		SET active = FALSE
		WHERE rule_id = $1 AND active
		"#,
        id,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("failed to deactivate delphi rule revision")?;

    let revision = sqlx::query!(
        r#"
		INSERT INTO delphi_rule_revisions (rule_id, expression, created_by)
		VALUES ($1, $2, $3)
		RETURNING id, expression, created, created_by
		"#,
        id,
        rule.expression,
        user_id,
    )
    .fetch_one(&mut transaction)
    .await
    .wrap_internal_err("failed to create delphi rule revision")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit delphi rule transaction")?;

    Ok(web::Json(DelphiRule {
        id: row.id,
        name: row.name,
        priority: row.priority,
        expression: revision.expression,
        revision_id: revision.id,
        created: row.created,
        updated: row.updated,
        revision_created: revision.created,
        created_by: row.created_by,
        updated_by: row.updated_by,
        revision_created_by: revision.created_by,
    }))
}

/// Delete a Delphi rule and all its revisions and materialized effects.
#[utoipa::path(
	context_path = "/moderation/tech-review",
	tag = "moderation",
	security(("bearer_auth" = [])),
	responses((status = OK), (status = NOT_FOUND))
)]
#[delete("/rules/{id}")]
pub async fn delete_rule(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(i64,)>,
) -> Result<(), ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?;
    let (id,) = path.into_inner();

    let deleted = sqlx::query!(
        "DELETE FROM delphi_rules WHERE id = $1 RETURNING id",
        id,
    )
    .fetch_optional(&**pool)
    .await
    .wrap_internal_err("failed to delete delphi rule")?;

    if deleted.is_none() {
        return Err(ApiError::NotFound);
    }

    Ok(())
}

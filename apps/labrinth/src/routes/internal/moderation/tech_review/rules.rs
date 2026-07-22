use std::collections::{BTreeMap, HashMap};

use actix_web::{HttpRequest, delete, get, post, put, web};
use chrono::{DateTime, Utc};
use eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::{
    auth::check_is_moderator_from_headers,
    database::{
        PgPool, ReadOnlyPgPool, models::delphi_report_item::DelphiSeverity,
        redis::RedisPool,
    },
    models::pats::Scopes,
    queue::session::AuthQueue,
    routes::ApiError,
    util::error::Context,
};

const MAX_RULE_NAME_LENGTH: usize = 256;
const MAX_RULE_EXPRESSION_LENGTH: usize = 65_536;
const MAX_RULE_TEST_TRACES: usize = 10;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_rules)
        .service(test_rule)
        .service(create_rule)
        .service(update_rule)
        .service(delete_rule);
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DelphiRule {
    pub id: i64,
    pub name: String,
    pub rule: String,
    pub revision: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct WriteDelphiRule {
    pub name: String,
    pub rule: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TestDelphiRule {
    pub rule: String,
    pub traces: Vec<TestDelphiRuleTrace>,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct TestDelphiRuleTrace {
    pub key: String,
    pub issue_type: String,
    pub severity: DelphiSeverity,
    pub jar: Option<String>,
    pub file_path: String,
    pub data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TestDelphiRuleResponse {
    pub effects: Vec<Option<DelphiRuleEffect>>,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DelphiRuleEffect {
    #[serde(default)]
    pub severity: Option<DelphiSeverity>,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Serialize)]
struct TestRuleInput<'a> {
    schema_version: u32,
    trace: &'a TestDelphiRuleTrace,
    scan: TestRuleScan,
    artifact: TestRuleArtifact,
    scope: TestRuleScope,
}

#[derive(Serialize)]
struct TestRuleScan {
    delphi_version: i32,
}

#[derive(Serialize)]
struct TestRuleArtifact {
    size: u32,
    hashes: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct TestRuleScope {
    project_id: String,
    version_id: String,
    file_id: String,
}

struct ValidatedRule {
    name: String,
    rule: String,
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

        let rule = self.rule.trim().to_string();
        if rule.is_empty() {
            return Err(ApiError::Request(eyre!(
                "rule expression cannot be empty"
            )));
        }
        if rule.len() > MAX_RULE_EXPRESSION_LENGTH {
            return Err(ApiError::Request(eyre!(
                "rule expression cannot exceed {MAX_RULE_EXPRESSION_LENGTH} bytes"
            )));
        }

        cel::Program::compile(&rule).map_err(|error| {
            ApiError::Request(eyre!("invalid cel expression: {error}"))
        })?;

        Ok(ValidatedRule { name, rule })
    }
}

/// Evaluate a CEL rule against caller-provided issue traces without saving it.
#[utoipa::path(
	context_path = "/moderation/tech-review",
	tag = "moderation",
	security(("bearer_auth" = [])),
	request_body = TestDelphiRule,
	responses((status = OK, body = TestDelphiRuleResponse))
)]
#[post("/rules/test")]
pub async fn test_rule(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<TestDelphiRule>,
) -> Result<web::Json<TestDelphiRuleResponse>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let request = body.into_inner();
    let rule = request.rule.trim();
    if rule.is_empty() {
        return Err(ApiError::Request(eyre!(
            "rule expression cannot be empty"
        )));
    }
    if rule.len() > MAX_RULE_EXPRESSION_LENGTH {
        return Err(ApiError::Request(eyre!(
            "rule expression cannot exceed {MAX_RULE_EXPRESSION_LENGTH} bytes"
        )));
    }
    if request.traces.len() > MAX_RULE_TEST_TRACES {
        return Err(ApiError::Request(eyre!(
            "cannot test more than {MAX_RULE_TEST_TRACES} traces at once"
        )));
    }

    let program = cel::Program::compile(rule).map_err(|error| {
        ApiError::Request(eyre!("invalid cel expression: {error}"))
    })?;
    let mut effects = Vec::with_capacity(request.traces.len());

    for (index, trace) in request.traces.iter().enumerate() {
        let input = test_rule_input(trace);
        let effect = super::rules_scan::evaluate_rule(&program, input)
            .map_err(|error| {
                ApiError::Request(eyre!(
                    "failed to evaluate test trace {index}: {error}"
                ))
            })?;
        effects.push(effect);
    }

    Ok(web::Json(TestDelphiRuleResponse { effects }))
}

fn test_rule_input(trace: &TestDelphiRuleTrace) -> TestRuleInput<'_> {
    TestRuleInput {
        schema_version: 1,
        trace,
        scan: TestRuleScan { delphi_version: 17 },
        artifact: TestRuleArtifact {
            size: 412_892,
            hashes: BTreeMap::from([
                ("sha1".to_string(), "0123456789abcdef".to_string()),
                ("sha512".to_string(), "fedcba9876543210".to_string()),
            ]),
        },
        scope: TestRuleScope {
            project_id: "example-project".to_string(),
            version_id: "example-version".to_string(),
            file_id: "example-file".to_string(),
        },
    }
}

/// List all Delphi rules that are not pending deletion.
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

    let rules = sqlx::query!(
        r#"
		SELECT
			id,
			name,
			rule,
			revision,
			created_at,
			updated_at,
			created_by,
			updated_by
		FROM delphi_rules
		WHERE NOT delete_on_next_revision
		ORDER BY id
		"#,
    )
    .fetch_all(&***ro_pool)
    .await
    .wrap_internal_err("failed to fetch delphi rules")?;

    Ok(web::Json(
        rules
            .into_iter()
            .map(|rule| DelphiRule {
                id: rule.id,
                name: rule.name,
                rule: rule.rule,
                revision: rule.revision,
                created_at: rule.created_at,
                updated_at: rule.updated_at,
                created_by: rule.created_by,
                updated_by: rule.updated_by,
            })
            .collect(),
    ))
}

/// Create a Delphi rule. It will be applied by the next manual rule scan.
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

    let rule = sqlx::query!(
        r#"
		INSERT INTO delphi_rules (
			name,
			rule,
			revision,
			created_by,
			updated_by
		)
		VALUES (
			$1,
			$2,
			(SELECT revision + 1 FROM delphi_rule_revisions LIMIT 1),
			$3,
			$3
		)
		RETURNING
			id,
			name,
			rule,
			revision,
			created_at,
			updated_at,
			created_by,
			updated_by
		"#,
        rule.name,
        rule.rule,
        user_id,
    )
    .fetch_one(&**pool)
    .await
    .wrap_internal_err("failed to create delphi rule")?;

    Ok(web::Json(DelphiRule {
        id: rule.id,
        name: rule.name,
        rule: rule.rule,
        revision: rule.revision,
        created_at: rule.created_at,
        updated_at: rule.updated_at,
        created_by: rule.created_by,
        updated_by: rule.updated_by,
    }))
}

/// Update a Delphi rule. Its materialized effects remain unchanged until the next scan.
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

    let rule = sqlx::query!(
        r#"
		UPDATE delphi_rules
		SET
			name = $2,
			rule = $3,
			revision = (
				SELECT revision + 1 FROM delphi_rule_revisions LIMIT 1
			),
			updated_at = CURRENT_TIMESTAMP,
			updated_by = $4
		WHERE id = $1 AND NOT delete_on_next_revision
		RETURNING
			id,
			name,
			rule,
			revision,
			created_at,
			updated_at,
			created_by,
			updated_by
		"#,
        id,
        rule.name,
        rule.rule,
        user_id,
    )
    .fetch_optional(&**pool)
    .await
    .wrap_internal_err("failed to update delphi rule")?
    .ok_or(ApiError::NotFound)?;

    Ok(web::Json(DelphiRule {
        id: rule.id,
        name: rule.name,
        rule: rule.rule,
        revision: rule.revision,
        created_at: rule.created_at,
        updated_at: rule.updated_at,
        created_by: rule.created_by,
        updated_by: rule.updated_by,
    }))
}

/// Mark a Delphi rule for deletion when the next rule scan is published.
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
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?;
    let (id,) = path.into_inner();

    let deleted = sqlx::query!(
        r#"
		UPDATE delphi_rules
		SET
			delete_on_next_revision = TRUE,
			revision = (
				SELECT revision + 1 FROM delphi_rule_revisions LIMIT 1
			),
			updated_at = CURRENT_TIMESTAMP,
			updated_by = $2
		WHERE id = $1 AND NOT delete_on_next_revision
		RETURNING id
		"#,
        id,
        user.id.0 as i64,
    )
    .fetch_optional(&**pool)
    .await
    .wrap_internal_err("failed to mark delphi rule for deletion")?;

    if deleted.is_none() {
        return Err(ApiError::NotFound);
    }

    Ok(())
}

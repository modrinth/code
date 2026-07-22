use std::collections::{BTreeMap, HashMap};

use actix_web::{HttpRequest, delete, get, post, put, web};
use chrono::{DateTime, Utc};
use eyre::eyre;
use serde::{Deserialize, Serialize};

use super::rules_scan::{
    RuleArtifact, RuleInput, RuleScan, RuleScope, RuleTrace,
};
use crate::{
    auth::check_is_moderator_from_headers,
    database::{
        PgPool, ReadOnlyPgPool,
        models::{
            DBProjectId, DBVersionId, DelphiReportIssueDetailsId,
            DelphiReportIssueId, delphi_report_item::DelphiSeverity,
        },
        redis::RedisPool,
    },
    models::{
        ids::{ProjectId, VersionId},
        pats::Scopes,
    },
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
        .service(get_rule_affected_details)
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
    pub affected_details_count: i64,
    pub affected_details: Vec<DelphiRuleAffectedDetail>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DelphiRuleAffectedDetail {
    pub detail_id: DelphiReportIssueDetailsId,
    pub issue_id: DelphiReportIssueId,
    pub project_id: Option<ProjectId>,
    pub project_name: Option<String>,
    pub project_icon_url: Option<String>,
    pub version_id: Option<VersionId>,
    pub version_name: Option<String>,
    pub version_number: Option<String>,
    pub issue_type: String,
    pub key: String,
    pub jar: Option<String>,
    pub file_path: String,
    pub original_severity: DelphiSeverity,
    pub severity: Option<DelphiSeverity>,
    pub hidden: bool,
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

fn test_rule_input(trace: &TestDelphiRuleTrace) -> RuleInput {
    RuleInput {
        schema_version: 1,
        trace: RuleTrace {
            key: trace.key.clone(),
            issue_type: trace.issue_type.clone(),
            severity: trace.severity,
            jar: trace.jar.clone(),
            file_path: trace.file_path.clone(),
            data: trace.data.clone(),
        },
        scan: RuleScan { delphi_version: 17 },
        artifact: RuleArtifact {
            size: Some(412_892),
            hashes: BTreeMap::from([
                ("sha1".to_string(), "0123456789abcdef".to_string()),
                ("sha512".to_string(), "fedcba9876543210".to_string()),
            ]),
        },
        scope: RuleScope {
            project_id: Some("example-project".to_string()),
            version_id: Some("example-version".to_string()),
            file_id: Some("example-file".to_string()),
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
			delphi_rule.id,
			delphi_rule.name,
			delphi_rule.rule,
			delphi_rule.revision,
			delphi_rule.created_at,
			delphi_rule.updated_at,
			delphi_rule.created_by,
			delphi_rule.updated_by,
			COALESCE(preview.affected_details_count, 0)
				AS "affected_details_count!",
			preview.detail_id AS "detail_id?: DelphiReportIssueDetailsId",
			preview.issue_id AS "issue_id?: DelphiReportIssueId",
			preview.project_id AS "project_id?: DBProjectId",
			preview.project_name AS "project_name?",
			preview.project_icon_url AS "project_icon_url?",
			preview.version_id AS "version_id?: DBVersionId",
			preview.version_name AS "version_name?",
			preview.version_number AS "version_number?",
			preview.issue_type AS "issue_type?",
			preview.key AS "key?",
			preview.jar AS "jar?",
			preview.file_path AS "file_path?",
			preview.original_severity AS "original_severity?: DelphiSeverity",
			preview.severity AS "effect_severity?: DelphiSeverity",
			preview.hidden AS "hidden?"
		FROM delphi_rules delphi_rule
		LEFT JOIN LATERAL (
			SELECT
				effect.detail_id,
				detail.issue_id,
				version.mod_id AS project_id,
				project.name AS project_name,
				project.icon_url AS project_icon_url,
				version.id AS version_id,
				version.name AS version_name,
				version.version_number,
				issue.issue_type,
				detail.key,
				detail.jar,
				detail.file_path,
				detail.severity AS original_severity,
				effect.severity,
				effect.hidden,
				COUNT(*) OVER () AS affected_details_count
			FROM delphi_rule_effects effect
			INNER JOIN delphi_rule_revisions published
				ON published.revision = effect.revision
			INNER JOIN delphi_report_issue_details detail
				ON detail.id = effect.detail_id
			INNER JOIN delphi_report_issues issue
				ON issue.id = detail.issue_id
			INNER JOIN delphi_reports report
				ON report.id = issue.report_id
			LEFT JOIN files file ON file.id = report.file_id
			LEFT JOIN versions version ON version.id = file.version_id
			LEFT JOIN mods project ON project.id = version.mod_id
			WHERE effect.rule_id = delphi_rule.id
			ORDER BY effect.detail_id DESC
			LIMIT 3
		) preview ON TRUE
		WHERE NOT delphi_rule.delete_on_next_revision
		ORDER BY delphi_rule.id, preview.detail_id DESC
		"#,
    )
    .fetch_all(&***ro_pool)
    .await
    .wrap_internal_err("failed to fetch delphi rules")?;

    let mut response = Vec::<DelphiRule>::new();
    for rule in rules {
        if response
            .last()
            .is_none_or(|existing| existing.id != rule.id)
        {
            response.push(DelphiRule {
                id: rule.id,
                name: rule.name,
                rule: rule.rule,
                revision: rule.revision,
                created_at: rule.created_at,
                updated_at: rule.updated_at,
                created_by: rule.created_by,
                updated_by: rule.updated_by,
                affected_details_count: rule.affected_details_count,
                affected_details: Vec::new(),
            });
        }

        if let (
            Some(detail_id),
            Some(issue_id),
            Some(issue_type),
            Some(key),
            Some(file_path),
            Some(original_severity),
            Some(hidden),
        ) = (
            rule.detail_id,
            rule.issue_id,
            rule.issue_type,
            rule.key,
            rule.file_path,
            rule.original_severity,
            rule.hidden,
        ) {
            response
                .last_mut()
                .expect("a delphi rule was inserted above")
                .affected_details
                .push(DelphiRuleAffectedDetail {
                    detail_id,
                    issue_id,
                    project_id: rule.project_id.map(ProjectId::from),
                    project_name: rule.project_name,
                    project_icon_url: rule.project_icon_url,
                    version_id: rule.version_id.map(VersionId::from),
                    version_name: rule.version_name,
                    version_number: rule.version_number,
                    issue_type,
                    key,
                    jar: rule.jar,
                    file_path,
                    original_severity,
                    severity: rule.effect_severity,
                    hidden,
                });
        }
    }

    Ok(web::Json(response))
}

/// List all details affected by a Delphi rule in the published revision.
#[utoipa::path(
    context_path = "/moderation/tech-review",
    tag = "moderation",
    security(("bearer_auth" = [])),
    responses((status = OK, body = Vec<DelphiRuleAffectedDetail>))
)]
#[get("/rules/{id}/effects")]
pub async fn get_rule_affected_details(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(i64,)>,
) -> Result<web::Json<Vec<DelphiRuleAffectedDetail>>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;
    let (rule_id,) = path.into_inner();

    let details = sqlx::query!(
        r#"
		SELECT
			effect.detail_id AS "detail_id!: DelphiReportIssueDetailsId",
			detail.issue_id AS "issue_id!: DelphiReportIssueId",
			version.mod_id AS "project_id?: DBProjectId",
			project.name AS "project_name?",
			project.icon_url AS "project_icon_url?",
			version.id AS "version_id?: DBVersionId",
			version.name AS "version_name?",
			version.version_number AS "version_number?",
			issue.issue_type,
			detail.key,
			detail.jar,
			detail.file_path,
			detail.severity AS "original_severity!: DelphiSeverity",
			effect.severity AS "effect_severity: DelphiSeverity",
			effect.hidden
		FROM delphi_rule_effects effect
		INNER JOIN delphi_rule_revisions published
			ON published.revision = effect.revision
		INNER JOIN delphi_report_issue_details detail
			ON detail.id = effect.detail_id
		INNER JOIN delphi_report_issues issue ON issue.id = detail.issue_id
		INNER JOIN delphi_reports report ON report.id = issue.report_id
		LEFT JOIN files file ON file.id = report.file_id
		LEFT JOIN versions version ON version.id = file.version_id
		LEFT JOIN mods project ON project.id = version.mod_id
		WHERE effect.rule_id = $1
		ORDER BY effect.detail_id DESC
		"#,
        rule_id,
    )
    .fetch_all(&***ro_pool)
    .await
    .wrap_internal_err("failed to fetch details affected by delphi rule")?;

    Ok(web::Json(
        details
            .into_iter()
            .map(|detail| DelphiRuleAffectedDetail {
                detail_id: detail.detail_id,
                issue_id: detail.issue_id,
                project_id: detail.project_id.map(ProjectId::from),
                project_name: detail.project_name,
                project_icon_url: detail.project_icon_url,
                version_id: detail.version_id.map(VersionId::from),
                version_name: detail.version_name,
                version_number: detail.version_number,
                issue_type: detail.issue_type,
                key: detail.key,
                jar: detail.jar,
                file_path: detail.file_path,
                original_severity: detail.original_severity,
                severity: detail.effect_severity,
                hidden: detail.hidden,
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
        affected_details_count: 0,
        affected_details: Vec::new(),
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
        affected_details_count: 0,
        affected_details: Vec::new(),
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

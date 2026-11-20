use crate::util::temporal::TemporalClient;
use eyre::eyre;
use xredis::RedisPool;

use crate::auth::UserWithScopes;
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::project_item::ProjectQueryResult;
use crate::database::models::thread_item::ThreadMessageBuilder;
use crate::database::models::{
    DBProjectId, DBTeamId, DBThreadIssue, DBThreadIssueFacetId, DBUserId,
    DBVersionId,
};
use crate::database::{PgPool, PgTransaction, models as db_models};
use crate::models::ids::ProjectId;
use crate::models::notifications::NotificationBody;
use crate::models::pats::Scopes;
use crate::models::projects::{
    ApprovalProvenance, ApprovalReason, AutoApprovalCause, Project,
    ProjectStatus, Version,
};
use crate::models::thread_issues::{
    ThreadIssueContext, ThreadIssueTarget, ThreadIssueTeamMember,
    ThreadIssueValueState, ThreadIssueVerdict,
};
use crate::models::threads::MessageBody;
use crate::routes::ApiError;
use crate::routes::internal::delphi;
use crate::util::error::{ApiContext as _, Context as _};
use crate::validate::project::{
    ProjectNagSeverity, validate_with_context as validate_project,
};

pub(crate) struct SyncedProjectState {
    data: ProjectQueryResult,
    project: Project,
    versions: Vec<Version>,
    disclosures: Vec<crate::models::disclosures::ProjectDisclosure>,
    team_members: Vec<ThreadIssueTeamMember>,
    thread_issues: Vec<DBThreadIssue>,
    has_required_nags: bool,
    has_outstanding_technical_review: bool,
}

impl SyncedProjectState {
    pub(crate) fn status(&self) -> ProjectStatus {
        self.project.status
    }

    fn auto_approval_status(&self) -> Option<ProjectStatus> {
        if self.has_required_nags
            || self.has_outstanding_technical_review
            || self.thread_issues.is_empty()
            || self
                .thread_issues
                .iter()
                .any(|issue| issue.verdict != ThreadIssueVerdict::Resolved)
        {
            return None;
        }

        Some(
            self.project
                .requested_status
                .filter(|status| {
                    status.is_approved() && status.can_be_requested()
                })
                .unwrap_or(ProjectStatus::Approved),
        )
    }

    async fn verify_facets(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), ApiError> {
        for facet in self.thread_issues.iter().flat_map(|issue| &issue.facets) {
            DBThreadIssue::update_facet_flags(
                facet.id.into(),
                Some(true),
                Some(true),
                transaction,
            )
            .await
            .wrap_internal_err("verifying project moderation issues")?;
        }
        Ok(())
    }

    pub(crate) async fn try_auto_approve(
        &mut self,
        cause: AutoApprovalCause,
        transaction: &mut PgTransaction<'_>,
        redis: &RedisPool,
    ) -> Result<bool, ApiError> {
        let Some(new_status) = self.auto_approval_status() else {
            return Ok(false);
        };
        let old_status = self.project.status;
        let provenance = ApprovalProvenance {
            trigger: cause,
            previous_status: old_status,
            resolved_issue_ids: self
                .thread_issues
                .iter()
                .map(|issue| issue.id.into())
                .collect(),
        };
        let reason =
            serde_json::to_value(ApprovalReason::Auto(provenance.clone()))
                .wrap_internal_err(
                    "serializing project auto-approval reason",
                )?;

        sqlx::query!(
            r#"
			UPDATE mods
			SET status = $1,
			    approved_because = CASE
			        WHEN approved IS NULL THEN $3::jsonb
			        ELSE approved_because
			    END,
			    approved = COALESCE(approved, NOW())
			WHERE id = $2
			"#,
            new_status.as_str(),
            self.data.inner.id as DBProjectId,
            reason,
        )
        .execute(&mut *transaction)
        .await
        .wrap_internal_err("auto-approving project")?;

        self.verify_facets(transaction).await?;

        ThreadMessageBuilder {
            author_id: None,
            body: MessageBody::AutoApproval {
                old_status,
                new_status,
                provenance: Some(provenance),
            },
            thread_id: self.data.thread_id,
            hide_identity: false,
        }
        .insert(transaction)
        .await
        .wrap_internal_err("recording project auto-approval")?;

        let notified_members = sqlx::query!(
            r#"
			SELECT user_id
			FROM team_members
			WHERE team_id = $1 AND accepted
			"#,
            self.data.inner.team_id as DBTeamId,
        )
        .fetch_all(&mut *transaction)
        .await
        .wrap_internal_err("fetching project approval notification recipients")?
        .into_iter()
        .map(|row| DBUserId(row.user_id))
        .collect::<Vec<_>>();

        for body in [
            NotificationBody::StatusChange {
                project_id: self.data.inner.id.into(),
                old_status,
                new_status,
            },
            NotificationBody::ProjectStatusApproved {
                project_id: self.data.inner.id.into(),
            },
        ] {
            NotificationBuilder { body }
                .insert_many(notified_members.clone(), transaction, redis)
                .await
                .wrap_internal_err("notifying project team of auto-approval")?;
        }

        self.data.inner.status = new_status;
        self.project.status = new_status;
        Ok(true)
    }

    pub(crate) fn can_address_facet(
        &self,
        facet_id: DBThreadIssueFacetId,
    ) -> Option<bool> {
        let context = ThreadIssueContext {
            project: &self.project,
            versions: &self.versions,
            disclosures: &self.disclosures,
            team_members: &self.team_members,
        };
        self.thread_issues
            .iter()
            .flat_map(|issue| &issue.facets)
            .find(|facet| facet.id == facet_id.into())
            .map(|facet| {
                matches!(
                    &facet.what,
                    ThreadIssueTarget::Acknowledge { .. }
                        | ThreadIssueTarget::MarkAddressed
                ) || facet.what.value_state(&context, false)
                    != ThreadIssueValueState::SameAsOriginal
            })
    }
}

pub(crate) async fn sync_project_state(
    project_id: DBProjectId,
    transaction: &mut PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<SyncedProjectState, ApiError> {
    let (exited_projects, queued_projects) =
        sync_technical_review(&[project_id], transaction).await?;
    let mut state = load_project_state_with_membership(
        project_id,
        queued_projects.contains(&project_id),
        transaction,
        pool,
        redis,
    )
    .await?;
    if exited_projects.contains(&project_id)
        && state.status() == ProjectStatus::Processing
    {
        state
            .try_auto_approve(
                AutoApprovalCause::TechnicalReviewNoDetails,
                transaction,
                redis,
            )
            .await?;
    }
    Ok(state)
}

async fn sync_technical_review(
    project_ids: &[DBProjectId],
    transaction: &mut PgTransaction<'_>,
) -> Result<(Vec<DBProjectId>, Vec<DBProjectId>), ApiError> {
    delphi::tech_review_queue::lock_projects(project_ids, transaction).await?;
    for project_id in project_ids {
        sqlx::query!("SELECT pg_advisory_xact_lock($1)", project_id.0)
            .fetch_one(&mut *transaction)
            .await
            .wrap_internal_err("locking project state synchronization")?;
    }

    let exited_projects = delphi::tech_review_queue::sync_projects(
        project_ids,
        delphi::tech_review_queue::TechReviewRemovalReason::FileDeleted,
        transaction,
    )
    .await
    .wrap_api_err("synchronizing project technical review state")?;
    let queued_projects = sqlx::query_scalar!(
        r#"
		SELECT project_id AS "project_id!: DBProjectId"
		FROM delphi_tech_review_queue
		WHERE project_id = ANY($1::bigint[])
		"#,
        &project_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
    )
    .fetch_all(&mut *transaction)
    .await
    .wrap_internal_err("fetching synchronized technical review membership")?;
    Ok((exited_projects, queued_projects))
}

/// Loads project state without reconciling technical review membership.
/// Explicit review exits must not re-add a project based on its existing scan details.
pub(crate) async fn load_project_state(
    project_id: DBProjectId,
    transaction: &mut PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<SyncedProjectState, ApiError> {
    delphi::tech_review_queue::lock_projects(&[project_id], transaction)
        .await?;
    sqlx::query!("SELECT pg_advisory_xact_lock($1)", project_id.0)
        .fetch_one(&mut *transaction)
        .await
        .wrap_internal_err("locking project state synchronization")?;

    let queued_projects = sqlx::query_scalar!(
        r#"
		SELECT project_id AS "project_id!: DBProjectId"
		FROM delphi_tech_review_queue
		WHERE project_id = ANY($1::bigint[])
		"#,
        &[project_id.0],
    )
    .fetch_all(&mut *transaction)
    .await
    .wrap_internal_err("fetching synchronized technical review membership")?;

    load_project_state_with_membership(
        project_id,
        queued_projects.contains(&project_id),
        transaction,
        pool,
        redis,
    )
    .await
}

async fn load_project_state_with_membership(
    project_id: DBProjectId,
    has_outstanding_technical_review: bool,
    transaction: &mut PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<SyncedProjectState, ApiError> {
    let mut projects = db_models::DBProject::get_many_uncached(
        &[ProjectId::from(project_id)],
        &mut *transaction,
        redis,
    )
    .await
    .wrap_internal_err("reloading project for state synchronization")?;
    let data = projects.pop().wrap_not_found_err("resource not found")?;
    let version_ids = sqlx::query_scalar!(
		r#"SELECT id AS "id!: DBVersionId" FROM versions WHERE mod_id = $1 ORDER BY id"#,
		project_id.0,
	)
	.fetch_all(&mut *transaction)
	.await
	.wrap_internal_err("fetching all project version IDs")?;
    let versions = db_models::DBVersion::get_many_uncached(
        &version_ids,
        &mut *transaction,
        redis,
    )
    .await
    .wrap_internal_err("reloading project versions for state synchronization")?
    .into_iter()
    .map(Version::from)
    .collect::<Vec<_>>();
    let project = Project::from(data.clone());
    let available_categories =
        db_models::categories::Category::list(&mut *transaction, redis)
            .await
            .wrap_internal_err("fetching project categories")?;
    let disclosures = db_models::DBProjectDisclosure::get_many_for_project(
        data.inner.id,
        false,
        &mut *transaction,
    )
    .await
    .wrap_internal_err("fetching project disclosures")?
    .into_iter()
    .map(|disclosure| disclosure.disclosure)
    .collect::<Vec<_>>();
    let team_members = sqlx::query!(
        r#"
        SELECT team_id, user_id, role
        FROM team_members
        WHERE
            team_id = $1
            OR team_id = (
                SELECT team_id
                FROM organizations
                WHERE id = $2
            )
        "#,
        data.inner.team_id as DBTeamId,
        data.inner.organization_id.map(|id| id.0),
    )
    .fetch_all(&mut *transaction)
    .await
    .wrap_internal_err("fetching project team members")?
    .into_iter()
    .map(|member| ThreadIssueTeamMember {
        team_id: DBTeamId(member.team_id).into(),
        user_id: DBUserId(member.user_id).into(),
        role: member.role,
    })
    .collect::<Vec<_>>();
    let has_required_nags = validate_project(
        project.clone(),
        versions.clone(),
        available_categories,
        disclosures.clone(),
        pool,
        redis,
    )
    .await?
    .iter()
    .any(|nag| nag.severity == ProjectNagSeverity::Required);

    let context = ThreadIssueContext {
        project: &project,
        versions: &versions,
        disclosures: &disclosures,
        team_members: &team_members,
    };
    let thread_issues =
        db_models::DBThreadIssue::sync_project_verdicts(&context, transaction)
            .await
            .wrap_internal_err("synchronizing project thread issue verdicts")?;

    Ok(SyncedProjectState {
        data,
        project,
        versions,
        disclosures,
        team_members,
        thread_issues,
        has_required_nags,
        has_outstanding_technical_review,
    })
}

fn ensure_reconciliation_sources_match(
    current: &impl serde::Serialize,
    evaluated: &impl serde::Serialize,
) -> Result<(), ApiError> {
    if serde_json::to_value(current)
        .wrap_internal_err("serializing current project flag sources")?
        != serde_json::to_value(evaluated)
            .wrap_internal_err("serializing evaluated project flag sources")?
    {
        return Err(ApiError::Internal(eyre!(
            crate::workflow::project_flag::run_rules::ReconciliationInputsChanged
        )));
    }
    Ok(())
}

/// Revalidates the evaluated project, versions and file manifest under locks.
/// Other rule context (teams, users, organizations, disclosures, attributions and
/// moderation history) is included in the cache key, not transactionally reloaded.
pub(crate) async fn apply_project_flag_reconciliation(
    context: &crate::queue::moderation::project_flag::FlagContext,
    result: &db_models::project_flag_item::DBProjectFlagRunResult,
    ruleset_id: uuid::Uuid,
    file_manifest_hash: blake3::Hash,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<(), ApiError> {
    use db_models::project_flag_item::FlagOutcome;
    use std::collections::BTreeMap;

    let project_id = DBProjectId::from(context.project.id);
    if result.project_id != project_id
        || result.run_id.is_some()
        || result.error.is_some()
        || result.rules.values().any(|rule| rule.error.is_some())
    {
        return Err(ApiError::Internal(eyre!(
            "invalid project flag reconciliation result"
        )));
    }
    let mut flags = BTreeMap::new();
    for flag in result.rules.values().flat_map(|rule| &rule.flags) {
        let what = serde_json::to_value(flag)
            .wrap_internal_err("serializing reconciled project flag")?;
        if let Some(previous) = flags.insert(flag.key.clone(), what.clone())
            && previous != what
        {
            return Err(ApiError::Internal(eyre!(
                "conflicting project flag key `{}`",
                flag.key
            )));
        }
    }
    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting project flag application")?;
    let deployed = sqlx::query_scalar!(
        r#"SELECT ruleset_id FROM project_flags_deployed WHERE singleton FOR SHARE"#
    ).fetch_optional(&mut transaction).await
        .wrap_internal_err("locking deployed project flag revision")?;
    if deployed != Some(ruleset_id) {
        return Err(ApiError::Internal(eyre!(
            crate::workflow::project_flag::run_rules::ReconciliationInputsChanged
        )));
    }
    let status = sqlx::query_scalar!(
        r#"SELECT status FROM mods WHERE id = $1 FOR UPDATE"#,
        project_id.0,
    )
    .fetch_optional(&mut transaction)
    .await
    .wrap_internal_err("locking reconciled project")?;
    if status.as_deref() != Some(ProjectStatus::Processing.as_str()) {
        return Ok(());
    }
    // Lock versions before loading them so file insertions cannot race the comparison.
    sqlx::query!(
        r#"SELECT id FROM versions WHERE mod_id = $1 ORDER BY id FOR UPDATE"#,
        project_id.0,
    )
    .fetch_all(&mut transaction)
    .await
    .wrap_internal_err("locking reconciled project versions")?;
    let files = sqlx::query!(
        r#"SELECT f.id, f.version_id, f.filename, f.url, f.size FROM files f INNER JOIN versions v ON v.id = f.version_id WHERE v.mod_id = $1 ORDER BY f.id FOR SHARE OF f"#,
        project_id.0,
    ).fetch_all(&mut transaction).await
        .wrap_internal_err("locking reconciled project files")?;
    let manifest = files
        .into_iter()
        .map(|file| {
            (
                file.id,
                file.version_id,
                file.filename,
                file.url,
                i64::from(file.size),
            )
        })
        .collect::<Vec<_>>();
    if crate::workflow::project_flag::scan_cache::rules_input_hash(&manifest)
        .map_err(ApiError::Internal)?
        != file_manifest_hash
    {
        return Err(ApiError::Internal(eyre!(
            crate::workflow::project_flag::run_rules::ReconciliationInputsChanged
        )));
    }
    let mut state =
        load_project_state(project_id, &mut transaction, pool, redis).await?;
    let mut versions = state.versions.clone();
    crate::queue::moderation::project_flag::sort_versions_newest_first(
        &mut versions,
    );
    ensure_reconciliation_sources_match(
        &(&state.project, &versions),
        &(&context.project, &context.versions),
    )?;
    let desired =
        crate::workflow::project_flag::run_rules::determine_project_status(
            state.status(),
            result.rules.values(),
            false,
        )
        .ok_or_else(|| {
            ApiError::Internal(eyre!(
                "project flag reconciliation has no decision"
            ))
        })?;
    let mut changed = sqlx::query!(
        r#"UPDATE project_flags SET active = FALSE WHERE project_id = $1 AND active AND NOT ((what->>'key') = ANY($2::text[]))"#,
        project_id.0, &flags.keys().cloned().collect::<Vec<_>>(),
    ).execute(&mut transaction).await
        .wrap_internal_err("deactivating superseded project flags")?.rows_affected() > 0;
    for what in flags.values() {
        let id = db_models::ids::generate_project_flag_id(&mut transaction)
            .await
            .wrap_internal_err("generating reconciled project flag ID")?;
        changed |= sqlx::query!(
            r#"INSERT INTO project_flags (id, run_id, project_id, what, active)
            VALUES ($1, NULL, $2, $3, TRUE)
            ON CONFLICT (project_id, (what->>'key')) DO UPDATE
            SET run_id = NULL, what = EXCLUDED.what, active = TRUE
            WHERE project_flags.what IS DISTINCT FROM EXCLUDED.what OR NOT project_flags.active OR project_flags.run_id IS NOT NULL"#,
            id.0, project_id.0, what,
        ).execute(&mut transaction).await
            .wrap_internal_err("upserting reconciled project flag")?.rows_affected() > 0;
    }
    match desired {
        ProjectStatus::Approved => {
            changed |= state
                .try_auto_approve(
                    AutoApprovalCause::ProjectFlags { ruleset_id },
                    &mut transaction,
                    redis,
                )
                .await?;
        }
        ProjectStatus::Withheld => {
            sqlx::query!(
                r#"UPDATE mods SET status = $1 WHERE id = $2"#,
                ProjectStatus::Withheld.as_str(),
                project_id.0,
            )
            .execute(&mut transaction)
            .await
            .wrap_internal_err("withholding reconciled project")?;
            let reasons = result
                .rules
                .values()
                .flat_map(|rule| &rule.flags)
                .filter(|flag| flag.flag_outcome == FlagOutcome::Withheld)
                .filter_map(|flag| flag.reason.as_deref())
                .filter(|reason| !reason.trim().is_empty())
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join("\n\n");
            if !reasons.is_empty() {
                ThreadMessageBuilder {
                    author_id: None,
                    body: MessageBody::Text {
                        body: reasons,
                        private: false,
                        replying_to: None,
                        associated_images: Vec::new(),
                    },
                    thread_id: state.data.thread_id,
                    hide_identity: false,
                }
                .insert(&mut transaction)
                .await
                .wrap_internal_err(
                    "recording project flag withholding reasons",
                )?;
            }
            ThreadMessageBuilder {
                author_id: None,
                body: MessageBody::StatusChange {
                    old_status: ProjectStatus::Processing,
                    new_status: ProjectStatus::Withheld,
                },
                thread_id: state.data.thread_id,
                hide_identity: false,
            }
            .insert(&mut transaction)
            .await
            .wrap_internal_err("recording project flag status change")?;
            let members = sqlx::query!(
                r#"SELECT user_id FROM team_members WHERE team_id = $1 AND accepted"#,
                state.data.inner.team_id as DBTeamId,
            ).fetch_all(&mut transaction).await
                .wrap_internal_err("loading project flag notification recipients")?
                .into_iter().map(|row| DBUserId(row.user_id)).collect::<Vec<_>>();
            for body in [
                NotificationBody::StatusChange {
                    project_id: context.project.id,
                    old_status: ProjectStatus::Processing,
                    new_status: ProjectStatus::Withheld,
                },
                NotificationBody::ProjectStatusNeutral {
                    project_id: context.project.id,
                    old_status: ProjectStatus::Processing,
                    new_status: ProjectStatus::Withheld,
                },
            ] {
                NotificationBuilder { body }
                    .insert_many(members.clone(), &mut transaction, redis)
                    .await
                    .wrap_internal_err(
                        "notifying project team of project flag withholding",
                    )?;
            }
            state.project.status = ProjectStatus::Withheld;
            state.data.inner.status = ProjectStatus::Withheld;
            changed = true;
        }
        ProjectStatus::Processing => {}
        _ => {
            return Err(ApiError::Internal(eyre!(
                "invalid project flag status decision"
            )));
        }
    }
    if !changed {
        return Ok(());
    }
    PreparedMutations {
        states: vec![state],
    }
    .commit_without_reconciliation(transaction, redis)
    .await
}

pub(crate) async fn finalize_mutation(
    project_id: DBProjectId,
    transaction: PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
    project_editor: Option<&UserWithScopes>,
    temporal: &TemporalClient,
) -> Result<(), ApiError> {
    finalize_mutations(
        &[project_id],
        transaction,
        pool,
        redis,
        project_editor,
        temporal,
    )
    .await
}

pub(crate) async fn finalize_mutations(
    project_ids: &[DBProjectId],
    mut transaction: PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
    project_editor: Option<&UserWithScopes>,
    temporal: &TemporalClient,
) -> Result<(), ApiError> {
    let prepared = prepare_mutations(
        project_ids,
        &mut transaction,
        pool,
        redis,
        project_editor,
    )
    .await?;
    prepared.commit(transaction, redis, temporal).await
}

pub(crate) struct PreparedMutations {
    states: Vec<SyncedProjectState>,
}

pub(crate) async fn prepare_technical_review_exits(
    project_ids: &[DBProjectId],
    cause: AutoApprovalCause,
    transaction: &mut PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<PreparedMutations, ApiError> {
    let mut project_ids = project_ids.to_vec();
    project_ids.sort_by_key(|project_id| project_id.0);
    project_ids.dedup();

    let mut states = Vec::with_capacity(project_ids.len());
    for project_id in project_ids {
        let mut state =
            load_project_state(project_id, transaction, pool, redis).await?;
        if state.status() == ProjectStatus::Processing {
            state
                .try_auto_approve(cause.clone(), transaction, redis)
                .await?;
        }
        states.push(state);
    }

    Ok(PreparedMutations { states })
}

pub(crate) async fn prepare_mutations(
    project_ids: &[DBProjectId],
    transaction: &mut PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
    project_editor: Option<&UserWithScopes>,
) -> Result<PreparedMutations, ApiError> {
    let mut project_ids = project_ids.to_vec();
    project_ids.sort_by_key(|project_id| project_id.0);
    project_ids.dedup();

    let (exited_projects, queued_projects) =
        sync_technical_review(&project_ids, transaction).await?;
    let queued_projects = queued_projects
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    let mut states = Vec::with_capacity(project_ids.len());
    for project_id in project_ids {
        let mut state = load_project_state_with_membership(
            project_id,
            queued_projects.contains(&project_id),
            transaction,
            pool,
            redis,
        )
        .await?;
        if exited_projects.contains(&project_id)
            && state.status() == ProjectStatus::Processing
        {
            state
                .try_auto_approve(
                    AutoApprovalCause::TechnicalReviewNoDetails,
                    transaction,
                    redis,
                )
                .await?;
        }
        states.push(state);
    }

    if project_editor
        .is_some_and(|user| !user.scopes.contains(Scopes::MODERATION_WRITE))
    {
        for state in &states {
            if state.project.status == ProjectStatus::Processing
                && state.has_required_nags
            {
                return Err(ApiError::Request(eyre!(
                    "project cannot have required validation nags while in review"
                )));
            }
        }
    }

    Ok(PreparedMutations { states })
}

impl PreparedMutations {
    async fn commit_without_reconciliation(
        &self,
        transaction: PgTransaction<'_>,
        redis: &RedisPool,
    ) -> Result<(), ApiError> {
        transaction
            .commit()
            .await
            .wrap_internal_err("committing project mutation")?;

        for state in &self.states {
            db_models::DBProject::clear_cache(
                state.data.inner.id,
                state.data.inner.slug.clone(),
                Some(true),
                redis,
            )
            .await
            .wrap_internal_err("clearing cached project data")?;
        }

        Ok(())
    }

    pub(crate) async fn commit(
        self,
        transaction: PgTransaction<'_>,
        redis: &RedisPool,
        temporal: &TemporalClient,
    ) -> Result<(), ApiError> {
        self.commit_without_reconciliation(transaction, redis)
            .await?;
        for state in &self.states {
            crate::workflow::project_flag::reconcile_project_flags(
                temporal,
                state.data.inner.id.into(),
            )
            .await
            .map_err(|error| eyre!(error))
            .wrap_internal_err("dispatching project flag reconciliation")?;
        }

        Ok(())
    }
}

pub(crate) async fn finalize_project_edit(
    project_id: DBProjectId,
    original_status: ProjectStatus,
    original_slug: Option<String>,
    submit_for_review: bool,
    mut transaction: PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
    project_editor: Option<&UserWithScopes>,
    temporal: &TemporalClient,
) -> Result<(), ApiError> {
    let state =
        sync_project_state(project_id, &mut transaction, pool, redis).await?;

    let entered_review = state.project.status == ProjectStatus::Processing
        && original_status != ProjectStatus::Processing;
    let entered_approved_state =
        state.project.status.is_approved() && !original_status.is_approved();

    let validate_review_state = entered_review || submit_for_review;

    if (state.project.status == ProjectStatus::Processing
        || validate_review_state
        || entered_approved_state)
        && !project_editor
            .is_some_and(|user| user.scopes.contains(Scopes::MODERATION_WRITE))
        && state.has_required_nags
    {
        return Err(ApiError::Request(eyre!(
            "project must have no required validation nags before or during review or approval"
        )));
    }

    if validate_review_state
        && !project_editor
            .is_some_and(|user| user.scopes.contains(Scopes::MODERATION_WRITE))
        && state
            .thread_issues
            .iter()
            .any(|issue| issue.verdict == ThreadIssueVerdict::Open)
    {
        return Err(ApiError::Request(eyre!(
            "project must have no open moderation issues before review"
        )));
    }

    if entered_approved_state {
        if state
            .thread_issues
            .iter()
            .any(|issue| issue.verdict != ThreadIssueVerdict::Resolved)
        {
            return Err(ApiError::Request(eyre!(
                "all moderation issues must be resolved before approval"
            )));
        }

        state.verify_facets(&mut transaction).await?;
    }

    transaction
        .commit()
        .await
        .wrap_internal_err("committing project mutation")?;

    db_models::DBProject::clear_cache(
        project_id,
        state.data.inner.slug.clone(),
        Some(true),
        redis,
    )
    .await
    .wrap_internal_err("clearing cached project data")?;

    if original_slug != state.data.inner.slug {
        db_models::DBProject::clear_cache(
            project_id,
            original_slug,
            Some(true),
            redis,
        )
        .await
        .wrap_internal_err("clearing previous project slug cache")?;
    }

    crate::workflow::project_flag::reconcile_project_flags(
        temporal,
        project_id.into(),
    )
    .await
    .map_err(|error| eyre!(error))
    .wrap_internal_err("dispatching project flag reconciliation")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ids::ThreadIssueId;
    use ariadne::ids::UserId;
    use serde_json::json;

    #[test]
    fn reconciliation_source_comparison_rejects_stale_project_and_versions() {
        let evaluated = json!({"project": {"status": "processing", "title": "old"}, "versions": [{"id": "1", "files": ["old.jar"]}]});
        assert!(
            ensure_reconciliation_sources_match(&evaluated, &evaluated).is_ok()
        );
        for current in [
            json!({"project": {"status": "approved", "title": "old"}, "versions": [{"id": "1", "files": ["old.jar"]}]}),
            json!({"project": {"status": "processing", "title": "new"}, "versions": [{"id": "1", "files": ["old.jar"]}]}),
            json!({"project": {"status": "processing", "title": "old"}, "versions": [{"id": "1", "files": ["new.jar"]}]}),
        ] {
            assert!(
                matches!(ensure_reconciliation_sources_match(&current, &evaluated), Err(ApiError::Internal(error)) if error.is::<crate::workflow::project_flag::run_rules::ReconciliationInputsChanged>())
            );
        }
    }

    #[test]
    fn project_flag_approval_provenance_round_trips() {
        let ruleset_id = uuid::Uuid::nil();
        let cause = AutoApprovalCause::ProjectFlags { ruleset_id };
        let value = serde_json::to_value(cause).unwrap();
        assert_eq!(
            value,
            json!({"type": "project_flags", "ruleset_id": ruleset_id})
        );
        assert!(
            matches!(serde_json::from_value::<AutoApprovalCause>(value).unwrap(), AutoApprovalCause::ProjectFlags { ruleset_id: id } if id == ruleset_id)
        );
    }

    #[test]
    fn historical_auto_approval_message_has_no_provenance() {
        let value = json!({
            "type": "auto_approval",
            "old_status": "processing",
            "new_status": "approved"
        });
        let message: MessageBody =
            serde_json::from_value(value.clone()).unwrap();
        assert!(matches!(
            &message,
            MessageBody::AutoApproval {
                provenance: None,
                ..
            }
        ));
        assert_eq!(serde_json::to_value(message).unwrap(), value);
    }

    #[test]
    fn thread_and_database_share_approval_provenance() {
        let provenance = ApprovalProvenance {
            trigger: AutoApprovalCause::Resubmission { user_id: UserId(1) },
            previous_status: ProjectStatus::Rejected,
            resolved_issue_ids: vec![ThreadIssueId(2)],
        };
        let reason =
            serde_json::to_value(ApprovalReason::Auto(provenance.clone()))
                .unwrap();
        let message = MessageBody::AutoApproval {
            old_status: ProjectStatus::Rejected,
            new_status: ProjectStatus::Approved,
            provenance: Some(provenance),
        };
        let value = serde_json::to_value(message).unwrap();
        assert_eq!(reason["Auto"], value["provenance"]);
        let round_trip: MessageBody =
            serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(round_trip).unwrap(), value);
    }

    #[test]
    fn manual_approval_reason_serializes_user_id() {
        assert_eq!(
            serde_json::to_value(ApprovalReason::ByUser(UserId(1))).unwrap(),
            json!({ "ByUser": "1" }),
        );
    }

    #[test]
    fn auto_approval_reason_serializes_evidence() {
        let reason = ApprovalReason::Auto(ApprovalProvenance {
            trigger: AutoApprovalCause::ModeratorTechnicalReviewPass {
                user_id: UserId(1),
            },
            previous_status: ProjectStatus::Processing,
            resolved_issue_ids: vec![ThreadIssueId(2)],
        });
        assert_eq!(
            serde_json::to_value(reason).unwrap(),
            json!({
                "Auto": {
                    "trigger": {
                        "type": "moderator_technical_review_pass",
                        "user_id": "1"
                    },
                    "previous_status": "processing",
                    "resolved_issue_ids": ["2"]
                }
            }),
        );
    }
}

use eyre::eyre;
use xredis::RedisPool;

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
use crate::models::projects::{Project, ProjectStatus, Version};
use crate::models::thread_issues::{
    ThreadIssueContext, ThreadIssueTarget, ThreadIssueTeamMember,
    ThreadIssueValueState, ThreadIssueVerdict,
};
use crate::models::threads::MessageBody;
use crate::models::users::User;
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
        transaction: &mut PgTransaction<'_>,
        redis: &RedisPool,
    ) -> Result<bool, ApiError> {
        let Some(new_status) = self.auto_approval_status() else {
            return Ok(false);
        };
        let old_status = self.project.status;

        sqlx::query!(
            r#"
			UPDATE mods
			SET status = $1, approved = COALESCE(approved, NOW())
			WHERE id = $2
			"#,
            new_status.as_str(),
            self.data.inner.id as DBProjectId,
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
        state.try_auto_approve(transaction, redis).await?;
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

pub(crate) async fn finalize_mutation(
    project_id: DBProjectId,
    transaction: PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
    project_editor: Option<&User>,
) -> Result<(), ApiError> {
    finalize_mutations(&[project_id], transaction, pool, redis, project_editor)
        .await
}

pub(crate) async fn finalize_mutations(
    project_ids: &[DBProjectId],
    mut transaction: PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
    project_editor: Option<&User>,
) -> Result<(), ApiError> {
    let prepared = prepare_mutations(
        project_ids,
        &mut transaction,
        pool,
        redis,
        project_editor,
    )
    .await?;
    prepared.commit(transaction, redis).await
}

pub(crate) struct PreparedMutations {
    states: Vec<SyncedProjectState>,
}

pub(crate) async fn prepare_technical_review_exits(
    project_ids: &[DBProjectId],
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
            state.try_auto_approve(transaction, redis).await?;
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
    project_editor: Option<&User>,
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
            state.try_auto_approve(transaction, redis).await?;
        }
        states.push(state);
    }

    if project_editor.is_some_and(|user| !user.role.is_mod()) {
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
    pub(crate) async fn commit(
        self,
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
}

pub(crate) async fn finalize_project_edit(
    project_id: DBProjectId,
    original_status: ProjectStatus,
    original_slug: Option<String>,
    submit_for_review: bool,
    mut transaction: PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
    project_editor: Option<&User>,
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
        && !project_editor.is_some_and(|user| user.role.is_mod())
        && state.has_required_nags
    {
        return Err(ApiError::Request(eyre!(
            "project must have no required validation nags before or during review or approval"
        )));
    }

    if validate_review_state
        && !project_editor.is_some_and(|user| user.role.is_mod())
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

    Ok(())
}

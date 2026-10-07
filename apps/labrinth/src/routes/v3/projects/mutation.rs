use eyre::eyre;
use xredis::RedisPool;

use crate::database::models::project_item::ProjectQueryResult;
use crate::database::models::{
    DBProjectId, DBTeamId, DBThreadIssue, DBThreadIssueFacetId, DBUserId,
    DBVersionId,
};
use crate::database::{PgTransaction, models as db_models};
use crate::models::ids::ProjectId;
use crate::models::projects::{Project, ProjectStatus, Version};
use crate::models::thread_issues::{
    ThreadIssueContext, ThreadIssueTarget, ThreadIssueTeamMember,
    ThreadIssueValueState, ThreadIssueVerdict,
};
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
}

impl SyncedProjectState {
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
                matches!(&facet.what, ThreadIssueTarget::Acknowledge { .. })
                    || facet.what.value_state(&context, false)
                        != ThreadIssueValueState::SameAsOriginal
            })
    }
}

pub(crate) async fn sync_project_state(
    project_id: DBProjectId,
    transaction: &mut PgTransaction<'_>,
    redis: &RedisPool,
) -> Result<SyncedProjectState, ApiError> {
    delphi::tech_review_queue::sync_projects(
        &[project_id],
        delphi::tech_review_queue::TechReviewRemovalReason::FileDeleted,
        transaction,
    )
    .await
    .wrap_api_err("synchronizing project technical review state")?;

    sqlx::query!("SELECT pg_advisory_xact_lock($1)", project_id.0)
        .fetch_one(&mut *transaction)
        .await
        .wrap_internal_err("locking project state synchronization")?;

    let mut projects = db_models::DBProject::get_many_uncached(
        &[ProjectId::from(project_id)],
        &mut *transaction,
        redis,
    )
    .await
    .wrap_internal_err("reloading project for state synchronization")?;
    let data = projects.pop().wrap_not_found_err("resource not found")?;
    let project_ids = [project_id.0];
    let version_ids = sqlx::query!(
        "SELECT id FROM versions WHERE mod_id = ANY($1::bigint[]) ORDER BY id",
        &project_ids,
    )
    .fetch_all(&mut *transaction)
    .await
    .wrap_internal_err("fetching all project version IDs")?
    .into_iter()
    .map(|row| DBVersionId(row.id))
    .collect::<Vec<_>>();
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
        &project,
        &versions,
        &available_categories,
        &disclosures,
    )
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
    })
}

pub(crate) async fn finalize_mutation(
    project_id: DBProjectId,
    transaction: PgTransaction<'_>,
    redis: &RedisPool,
    project_editor: Option<&User>,
) -> Result<(), ApiError> {
    finalize_mutations(&[project_id], transaction, redis, project_editor).await
}

pub(crate) async fn finalize_mutations(
    project_ids: &[DBProjectId],
    mut transaction: PgTransaction<'_>,
    redis: &RedisPool,
    project_editor: Option<&User>,
) -> Result<(), ApiError> {
    let mut project_ids = project_ids.to_vec();
    project_ids.sort_by_key(|project_id| project_id.0);
    project_ids.dedup();

    let mut states = Vec::with_capacity(project_ids.len());
    for project_id in project_ids {
        states.push(
            sync_project_state(project_id, &mut transaction, redis).await?,
        );
    }

    if project_editor.is_some_and(|user| !user.role.is_mod()) {
        for state in &states {
            if state.project.status != ProjectStatus::Processing {
                continue;
            }
            if state.has_required_nags {
                return Err(ApiError::Request(eyre!(
                    "project cannot have required validation nags while in review"
                )));
            }
            if state
                .thread_issues
                .iter()
                .any(|issue| issue.verdict == ThreadIssueVerdict::Open)
            {
                return Err(ApiError::Request(eyre!(
                    "project cannot have open moderation issues while in review"
                )));
            }
        }
    }

    transaction
        .commit()
        .await
        .wrap_internal_err("committing project mutation")?;

    for state in &states {
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

pub(crate) async fn finalize_project_edit(
    project_id: DBProjectId,
    original_status: ProjectStatus,
    original_slug: Option<String>,
    mut transaction: PgTransaction<'_>,
    redis: &RedisPool,
    project_editor: Option<&User>,
) -> Result<(), ApiError> {
    let state = sync_project_state(project_id, &mut transaction, redis).await?;

    let entered_review = state.project.status == ProjectStatus::Processing
        && original_status != ProjectStatus::Processing;
    let entered_approved_state =
        state.project.status.is_approved() && !original_status.is_approved();

    let validate_review_state = entered_review
        || project_editor.is_some_and(|user| {
            !user.role.is_mod()
                && state.project.status == ProjectStatus::Processing
        });

    if (validate_review_state || entered_approved_state)
        && state.has_required_nags
    {
        return Err(ApiError::Request(eyre!(
            "project must have no required validation nags before or during review or approval"
        )));
    }

    if validate_review_state
        && state
            .thread_issues
            .iter()
            .any(|issue| issue.verdict == ThreadIssueVerdict::Open)
    {
        return Err(ApiError::Request(eyre!(
            "project must have no open moderation issues before or during review"
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

        for facet in state.thread_issues.iter().flat_map(|issue| &issue.facets)
        {
            DBThreadIssue::update_facet_flags(
                facet.id.into(),
                None,
                Some(true),
                &mut transaction,
            )
            .await
            .wrap_internal_err("verifying project moderation issues")?;
        }
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

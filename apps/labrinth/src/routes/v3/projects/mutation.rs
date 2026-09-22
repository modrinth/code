use eyre::eyre;
use xredis::RedisPool;

use crate::database::models::project_item::ProjectQueryResult;
use crate::database::models::{DBProjectId, DBTeamId, DBThreadIssue, DBUserId};
use crate::database::{PgTransaction, models as db_models};
use crate::models::ids::ProjectId;
use crate::models::projects::{Project, ProjectStatus, Version};
use crate::models::thread_issues::{
    ThreadIssueContext, ThreadIssueTeamMember, ThreadIssueVerdict,
};
use crate::routes::ApiError;
use crate::routes::internal::delphi;
use crate::util::error::{ApiContext as _, Context as _};
use crate::validate::project::has_required_nags_with_context;

struct SyncedProjectState {
    data: ProjectQueryResult,
    project: Project,
    thread_issues: Vec<DBThreadIssue>,
    has_required_nags: bool,
}

async fn sync_project_state(
    project_id: DBProjectId,
    transaction: &mut PgTransaction<'_>,
    redis: &RedisPool,
) -> Result<SyncedProjectState, ApiError> {
    delphi::tech_review_queue::remove_projects_without_details(
        &[project_id],
        delphi::tech_review_queue::TechReviewRemovalReason::FileDeleted,
        transaction,
    )
    .await
    .wrap_api_err(
        "removing project from technical review when no details remain",
    )?;

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
    let versions = db_models::DBVersion::get_many_uncached(
        &data.versions,
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
    let has_required_nags = has_required_nags_with_context(
        &project,
        &versions,
        &available_categories,
        &disclosures,
    );

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
        thread_issues,
        has_required_nags,
    })
}

pub(crate) async fn finalize_mutation(
    project_id: DBProjectId,
    transaction: PgTransaction<'_>,
    redis: &RedisPool,
) -> Result<(), ApiError> {
    finalize_mutations(&[project_id], transaction, redis).await
}

pub(crate) async fn finalize_mutations(
    project_ids: &[DBProjectId],
    mut transaction: PgTransaction<'_>,
    redis: &RedisPool,
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
) -> Result<(), ApiError> {
    let state = sync_project_state(project_id, &mut transaction, redis).await?;

    let entered_review = state.project.status == ProjectStatus::Processing
        && original_status != ProjectStatus::Processing;
    let entered_approved_state =
        state.project.status.is_approved() && !original_status.is_approved();

    if (entered_review || entered_approved_state) && state.has_required_nags {
        return Err(ApiError::Request(eyre!(
            "project must have no required validation nags before review or approval"
        )));
    }

    if entered_review
        && state
            .thread_issues
            .iter()
            .any(|issue| issue.verdict == ThreadIssueVerdict::Open)
    {
        return Err(ApiError::Request(eyre!(
            "all open moderation issues must be addressed before review"
        )));
    }

    if entered_approved_state
        && state
            .thread_issues
            .iter()
            .any(|issue| issue.verdict != ThreadIssueVerdict::Resolved)
    {
        return Err(ApiError::Request(eyre!(
            "all moderation issues must be resolved before approval"
        )));
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

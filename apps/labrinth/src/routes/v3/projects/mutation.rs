use anyhow::anyhow;
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
    ApprovalProvenance, ApprovalReason, AutoApprovalCause, LegacyProjectStatus,
    Project, ProjectDelta, ProjectReviewOutcome, ProjectVisibility,
    ReviewOutcomeReason, Version,
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

pub(crate) fn legacy_status(
    visibility: ProjectVisibility,
    in_review: bool,
    review_outcome: ProjectReviewOutcome,
) -> LegacyProjectStatus {
    if in_review {
        return LegacyProjectStatus::Processing;
    }
    match review_outcome {
        ProjectReviewOutcome::NeverReviewed => LegacyProjectStatus::Draft,
        ProjectReviewOutcome::Rejected => LegacyProjectStatus::Rejected,
        ProjectReviewOutcome::Withheld => LegacyProjectStatus::Withheld,
        ProjectReviewOutcome::Approved => match visibility {
            ProjectVisibility::Public => LegacyProjectStatus::Approved,
            ProjectVisibility::Unlisted => LegacyProjectStatus::Unlisted,
            ProjectVisibility::Private => LegacyProjectStatus::Private,
        },
    }
}

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
    pub(crate) fn in_review(&self) -> bool {
        self.project.in_review
    }

    pub(crate) fn review_outcome(&self) -> ProjectReviewOutcome {
        self.project.review_outcome
    }

    fn can_auto_approve(&self) -> bool {
        !self.has_required_nags
            && !self.has_outstanding_technical_review
            && !self.thread_issues.is_empty()
            && self
                .thread_issues
                .iter()
                .all(|issue| issue.verdict == ThreadIssueVerdict::Resolved)
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
            .internal_context("verifying project moderation issues")?;
        }
        Ok(())
    }

    pub(crate) async fn try_auto_approve(
        &mut self,
        cause: AutoApprovalCause,
        transaction: &mut PgTransaction<'_>,
        redis: &RedisPool,
    ) -> Result<bool, ApiError> {
        if !self.can_auto_approve() {
            return Ok(false);
        };
        let old_status = legacy_status(
            self.project.visibility,
            self.project.in_review,
            self.project.review_outcome,
        );
        let new_status = legacy_status(
            self.project.visibility,
            false,
            ProjectReviewOutcome::Approved,
        );
        let provenance = ApprovalProvenance {
            trigger: cause,

            resolved_issue_ids: self
                .thread_issues
                .iter()
                .map(|issue| issue.id.into())
                .collect(),
        };
        let reason =
            serde_json::to_value(ApprovalReason::Auto(provenance.clone()))
                .internal_context("serializing project auto-approval reason")?;

        sqlx::query!(
            r#"
			UPDATE mods
			SET review_outcome = $1,
                in_review = FALSE,
                initial_queued = NULL,
			    approved_because = CASE
			        WHEN approved IS NULL THEN $3::jsonb
			        ELSE approved_because
			    END,
			    approved = COALESCE(approved, NOW())
			WHERE id = $2
			"#,
            ProjectReviewOutcome::Approved.as_str(),
            self.data.inner.id as DBProjectId,
            reason,
        )
        .execute(&mut *transaction)
        .await
        .internal_context("auto-approving project")?;

        if self.project.review_outcome != ProjectReviewOutcome::Approved {
            db_models::DBProject::record_history(
                self.data.inner.id,
                None,
                ProjectDelta::ChangeReviewOutcome {
                    from: self.project.review_outcome,
                    to: ProjectReviewOutcome::Approved,
                    reason: Some(ReviewOutcomeReason::Auto {
                        provenance: provenance.clone(),
                    }),
                },
                transaction,
            )
            .await
            .internal_context("recording project auto-approval history")?;
        }

        if self.project.in_review {
            db_models::DBProject::record_history(
                self.data.inner.id,
                None,
                ProjectDelta::ChangeInReview {
                    from: true,
                    to: false,
                },
                transaction,
            )
            .await
            .internal_context("recording project auto-approval queue change")?;
        }

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
        .internal_context("recording project auto-approval")?;

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
        .internal_context("fetching project approval notification recipients")?
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
                .internal_context("notifying project team of auto-approval")?;
        }

        self.data.inner.review_outcome = ProjectReviewOutcome::Approved;
        self.data.inner.in_review = false;
        self.project.review_outcome = ProjectReviewOutcome::Approved;
        self.project.in_review = false;
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
    if exited_projects.contains(&project_id) && state.in_review() {
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
            .internal_context("locking project state synchronization")?;
    }

    let exited_projects = delphi::tech_review_queue::sync_projects(
        project_ids,
        delphi::tech_review_queue::TechReviewRemovalReason::FileDeleted,
        transaction,
    )
    .await
    .api_context("synchronizing project technical review state")?;
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
    .internal_context("fetching synchronized technical review membership")?;
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
        .internal_context("locking project state synchronization")?;

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
    .internal_context("fetching synchronized technical review membership")?;

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
    .internal_context("reloading project for state synchronization")?;
    let data = projects.pop().not_found_context("resource not found")?;
    let version_ids = sqlx::query_scalar!(
		r#"SELECT id AS "id!: DBVersionId" FROM versions WHERE mod_id = $1 ORDER BY id"#,
		project_id.0,
	)
	.fetch_all(&mut *transaction)
	.await
	.internal_context("fetching all project version IDs")?;
    let versions = db_models::DBVersion::get_many_uncached(
        &version_ids,
        &mut *transaction,
        redis,
    )
    .await
    .internal_context("reloading project versions for state synchronization")?
    .into_iter()
    .map(Version::from)
    .collect::<Vec<_>>();
    let project = Project::from(data.clone());
    let available_categories =
        db_models::categories::Category::list(&mut *transaction, redis)
            .await
            .internal_context("fetching project categories")?;
    let disclosures = db_models::DBProjectDisclosure::get_many_for_project(
        data.inner.id,
        false,
        &mut *transaction,
    )
    .await
    .internal_context("fetching project disclosures")?
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
    .internal_context("fetching project team members")?
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
            .internal_context("synchronizing project thread issue verdicts")?;

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
    project_editor: Option<&UserWithScopes>,
) -> Result<(), ApiError> {
    finalize_mutations(&[project_id], transaction, pool, redis, project_editor)
        .await
}

pub(crate) async fn finalize_mutations(
    project_ids: &[DBProjectId],
    mut transaction: PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
    project_editor: Option<&UserWithScopes>,
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
        if state.in_review() {
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
        if exited_projects.contains(&project_id) && state.in_review() {
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
            if state.project.in_review && state.has_required_nags {
                return Err(ApiError::Request(anyhow!(
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
            .internal_context("committing project mutation")?;

        for state in &self.states {
            db_models::DBProject::clear_cache(
                state.data.inner.id,
                state.data.inner.slug.clone(),
                Some(true),
                redis,
            )
            .await
            .internal_context("clearing cached project data")?;
        }

        Ok(())
    }
}

pub(crate) async fn finalize_project_edit(
    project_id: DBProjectId,
    original_in_review: bool,
    original_review_outcome: ProjectReviewOutcome,
    original_slug: Option<String>,
    submit_for_review: bool,
    mut transaction: PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
    project_editor: Option<&UserWithScopes>,
) -> Result<bool, ApiError> {
    let state =
        sync_project_state(project_id, &mut transaction, pool, redis).await?;

    let entered_review = state.project.in_review && !original_in_review;
    let entered_approved_state = state.project.is_approved()
        && original_review_outcome != ProjectReviewOutcome::Approved;

    let validate_review_state = entered_review || submit_for_review;

    if (state.project.in_review
        || validate_review_state
        || entered_approved_state)
        && !project_editor
            .is_some_and(|user| user.scopes.contains(Scopes::MODERATION_WRITE))
        && state.has_required_nags
    {
        return Err(ApiError::Request(anyhow!(
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
        return Err(ApiError::Request(anyhow!(
            "project must have no open moderation issues before review"
        )));
    }

    if entered_approved_state {
        if state
            .thread_issues
            .iter()
            .any(|issue| issue.verdict != ThreadIssueVerdict::Resolved)
        {
            return Err(ApiError::Request(anyhow!(
                "all moderation issues must be resolved before approval"
            )));
        }

        state.verify_facets(&mut transaction).await?;
    }

    transaction
        .commit()
        .await
        .internal_context("committing project mutation")?;

    db_models::DBProject::clear_cache(
        project_id,
        state.data.inner.slug.clone(),
        Some(true),
        redis,
    )
    .await
    .internal_context("clearing cached project data")?;

    if original_slug != state.data.inner.slug {
        db_models::DBProject::clear_cache(
            project_id,
            original_slug,
            Some(true),
            redis,
        )
        .await
        .internal_context("clearing previous project slug cache")?;
    }

    Ok(state.project.is_searchable())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ids::ThreadIssueId;
    use ariadne::ids::UserId;
    use serde_json::json;

    #[test]
    fn historical_status_preserves_private_and_unlisted_approvals() {
        for (visibility, expected) in [
            (ProjectVisibility::Private, LegacyProjectStatus::Private),
            (ProjectVisibility::Unlisted, LegacyProjectStatus::Unlisted),
            (ProjectVisibility::Public, LegacyProjectStatus::Approved),
        ] {
            assert_eq!(
                legacy_status(
                    visibility,
                    false,
                    ProjectReviewOutcome::Approved
                ),
                expected
            );
        }
    }

    #[test]
    fn historical_review_status_does_not_replace_the_review_outcome() {
        for outcome in [
            ProjectReviewOutcome::NeverReviewed,
            ProjectReviewOutcome::Rejected,
            ProjectReviewOutcome::Withheld,
        ] {
            assert_eq!(
                legacy_status(ProjectVisibility::Public, true, outcome),
                LegacyProjectStatus::Processing
            );
        }
        assert_eq!(
            legacy_status(
                ProjectVisibility::Public,
                false,
                ProjectReviewOutcome::Withheld
            ),
            LegacyProjectStatus::Withheld
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

            resolved_issue_ids: vec![ThreadIssueId(2)],
        };
        let reason =
            serde_json::to_value(ApprovalReason::Auto(provenance.clone()))
                .unwrap();
        let message = MessageBody::AutoApproval {
            old_status: LegacyProjectStatus::Rejected,
            new_status: LegacyProjectStatus::Approved,
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

            resolved_issue_ids: vec![ThreadIssueId(2)],
        });
        assert_eq!(
            serde_json::to_value(reason).unwrap(),
            json!({
                "Auto": {
                    "trigger": {
                        "kind": "moderator_technical_review_pass",
                        "user_id": "1"
                    },

                    "resolved_issue_ids": ["2"]
                }
            }),
        );
    }
}

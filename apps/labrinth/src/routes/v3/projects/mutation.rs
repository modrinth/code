use crate::util::temporal::TemporalClient;
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

/// Replays the persisted search state even when a retried activity made no mutation.
/// Each attempt owns its buffer so a failed removal cannot suppress a later upsert.
pub(crate) async fn publish_automatic_review_search_update(
    project_id: DBProjectId,
    pool: &PgPool,
    redis: &RedisPool,
    kafka: actix_web::web::Data<crate::util::kafka::KafkaClientState>,
) -> Result<(), ApiError> {
    let state = sqlx::query!(
        r#"SELECT visibility, review_outcome, slug FROM mods WHERE id = $1"#,
        project_id.0,
    )
    .fetch_optional(pool)
    .await
    .internal_context("reading persisted automatic review search state")?;
    db_models::DBProject::clear_cache(
        project_id,
        state.as_ref().and_then(|state| state.slug.clone()),
        Some(true),
        redis,
    )
    .await
    .internal_context("clearing automatic review search cache")?;
    let state = state.map(|state| {
        (
            ProjectVisibility::from_string(&state.visibility),
            ProjectReviewOutcome::from_string(&state.review_outcome),
        )
    });
    publish_review_search_action(
        project_id.into(),
        state,
        |action| async move {
            use crate::search::incremental::{
                IncrementalSearchQueue, SearchProjectIndexQueueEventData,
            };
            let queue = IncrementalSearchQueue::new(kafka);
            match action {
                SearchProjectIndexQueueEventData::ChangeWithAllVersions {
                    project_id,
                } => {
                    queue
                        .push_project_with_all_versions_change(project_id)
                        .await;
                }
                SearchProjectIndexQueueEventData::Removal { project_id } => {
                    queue.push_project_removal(project_id).await;
                }
                _ => unreachable!(),
            }
            queue.drain().await
        },
    )
    .await
    .internal_context("publishing automatic review search update")
}

async fn publish_review_search_action<F, Fut>(
    project_id: ProjectId,
    state: Option<(ProjectVisibility, ProjectReviewOutcome)>,
    publish: F,
) -> anyhow::Result<()>
where
    F: FnOnce(
        crate::search::incremental::SearchProjectIndexQueueEventData,
    ) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<()>>,
{
    use crate::search::incremental::SearchProjectIndexQueueEventData;
    let searchable = state.is_some_and(|(visibility, outcome)| {
        db_models::DBProject::calculate_effective_visibility(
            visibility, outcome,
        ) == ProjectVisibility::Public
    });
    let action = if searchable {
        SearchProjectIndexQueueEventData::ChangeWithAllVersions { project_id }
    } else {
        SearchProjectIndexQueueEventData::Removal { project_id }
    };
    publish(action).await
}

#[cfg(test)]
mod automatic_review_search_tests {
    use super::*;
    use crate::search::incremental::SearchProjectIndexQueueEventData;
    use std::cell::RefCell;

    #[actix_rt::test]
    async fn persisted_visibility_selects_search_action_on_every_attempt() {
        let project_id = DBProjectId(1).into();
        for visibility in [
            ProjectVisibility::Public,
            ProjectVisibility::Unlisted,
            ProjectVisibility::Private,
        ] {
            for outcome in [
                ProjectReviewOutcome::NeverReviewed,
                ProjectReviewOutcome::Approved,
                ProjectReviewOutcome::Rejected,
                ProjectReviewOutcome::Withheld,
            ] {
                let attempts = RefCell::new(Vec::new());
                for fail in [true, false, false] {
                    let result = publish_review_search_action(
                        project_id,
                        Some((visibility, outcome)),
                        |action| {
                            attempts.borrow_mut().push(action);
                            std::future::ready(if fail {
                                Err(anyhow!("mock queue delivery failure"))
                            } else {
                                Ok(())
                            })
                        },
                    )
                    .await;
                    assert_eq!(result.is_err(), fail);
                }
                assert_eq!(attempts.borrow().len(), 3);
                for action in attempts.into_inner() {
                    if visibility == ProjectVisibility::Public
                        && outcome == ProjectReviewOutcome::Approved
                    {
                        assert!(
                            matches!(action, SearchProjectIndexQueueEventData::ChangeWithAllVersions { project_id: id } if id == project_id)
                        );
                    } else {
                        assert!(
                            matches!(action, SearchProjectIndexQueueEventData::Removal { project_id: id } if id == project_id)
                        );
                    }
                }
            }
        }
        publish_review_search_action(project_id, None, |action| async move {
            assert!(matches!(action, SearchProjectIndexQueueEventData::Removal { project_id: id } if id == project_id));
            Ok(())
        }).await.unwrap();
    }
}

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

fn auto_approval_checks_pass(
    no_required_nags: bool,
    no_project_flags: bool,
    no_technical_review: bool,
    all_issues_resolved: bool,
) -> bool {
    no_required_nags
        && no_project_flags
        && no_technical_review
        && all_issues_resolved
}

#[cfg(test)]
#[test]
fn auto_approval_requires_every_check_but_not_any_issues() {
    for mask in 0_u8..16 {
        assert_eq!(
            auto_approval_checks_pass(
                mask & 1 == 0,
                mask & 2 == 0,
                mask & 4 == 0,
                mask & 8 == 0,
            ),
            mask == 0,
        );
    }
    assert!(auto_approval_checks_pass(
        true,
        true,
        true,
        std::iter::empty::<ThreadIssueVerdict>()
            .all(|verdict| verdict == ThreadIssueVerdict::Resolved),
    ));
}

pub(crate) struct SyncedProjectState {
    data: ProjectQueryResult,
    project: Project,
    versions: Vec<Version>,
    disclosures: Vec<crate::models::disclosures::ProjectDisclosure>,
    team_members: Vec<ThreadIssueTeamMember>,
    thread_issues: Vec<DBThreadIssue>,
    required_nags: Vec<crate::validate::project::ProjectNag>,
    has_outstanding_technical_review: bool,
    project_flag_count: i64,
}

impl SyncedProjectState {
    pub(crate) fn in_review(&self) -> bool {
        self.project.in_review
    }

    pub(crate) fn review_outcome(&self) -> ProjectReviewOutcome {
        self.project.review_outcome
    }

    fn can_auto_approve(&self) -> bool {
        auto_approval_checks_pass(
            self.required_nags.is_empty(),
            self.project_flag_count == 0,
            !self.has_outstanding_technical_review,
            self.thread_issues
                .iter()
                .all(|issue| issue.verdict == ThreadIssueVerdict::Resolved),
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
            .internal_context("verifying project moderation issues")?;
        }
        Ok(())
    }

    pub(crate) async fn approve_after_delay(
        &mut self,
        cause: AutoApprovalCause,
        transaction: &mut PgTransaction<'_>,
        redis: &RedisPool,
    ) -> Result<bool, ApiError> {
        if !self.in_review() || !self.can_auto_approve() {
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

pub(crate) async fn delayed_auto_approval_eligible(
    project_id: DBProjectId,
    pool: &PgPool,
    redis: &RedisPool,
    approve_without_ruleset: bool,
    approval_workflow: Option<&temporalio_client::Client>,
    schedule: Option<&crate::workflow::project_flag::approve::ApprovalSchedule>,
) -> Result<bool, ApiError> {
    let mut transaction = pool
        .begin()
        .await
        .internal_context("starting delayed approval check")?;
    let deployed = sqlx::query_scalar!(
        r#"SELECT ruleset_id FROM project_flags_deployed WHERE singleton FOR SHARE"#
    ).fetch_optional(&mut transaction).await.internal_context("locking delayed approval ruleset")?;
    let in_review = sqlx::query_scalar!(
        r#"SELECT in_review FROM mods WHERE id = $1 FOR UPDATE"#,
        project_id.0,
    )
    .fetch_optional(&mut transaction)
    .await
    .internal_context("locking delayed approval project")?;
    if in_review != Some(true) {
        return Ok(false);
    }
    let technical_review = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM delphi_tech_review_queue WHERE project_id = $1) AS "exists!""#,
        project_id.0,
    ).fetch_one(&mut transaction).await.internal_context("checking delayed approval technical review")?;
    let mut state = load_project_state_with_options(
        project_id,
        technical_review,
        false,
        &mut transaction,
        pool,
        redis,
    )
    .await?;
    let eligible = state.in_review() && state.can_auto_approve();
    if let Some(schedule) = schedule {
        if !eligible
            || !match approval_workflow {
                Some(temporal) => {
                    crate::workflow::project_flag::approve::approval_permitted(
                        temporal,
                        project_id.into(),
                    )
                    .await
                    .internal_context(
                        "checking workflow scheduling permission",
                    )?
                }
                None => false,
            }
        {
            return Ok(false);
        }
        let provenance = ApprovalProvenance {
            trigger: deployed
                .map(|ruleset_id| AutoApprovalCause::ProjectFlags {
                    ruleset_id,
                })
                .unwrap_or(AutoApprovalCause::TechnicalReviewNoDetails),
            resolved_issue_ids: state
                .thread_issues
                .iter()
                .map(|issue| issue.id.into())
                .collect(),
        };
        let delta = serde_json::to_value(ProjectDelta::AutoApprovalScheduled {
            provenance,
            scheduled_for: schedule.scheduled_for,
        })
        .internal_context("serializing project approval schedule")?;
        record_approval_schedule_history(project_id, delta, &mut transaction)
            .await?;
        transaction
            .commit()
            .await
            .internal_context("committing project approval schedule")?;
        return Ok(true);
    }
    if eligible
        && approve_without_ruleset
        && deployed.is_none()
        && match approval_workflow {
            Some(temporal) => {
                crate::workflow::project_flag::approve::approval_permitted(
                    temporal,
                    project_id.into(),
                )
                .await
                .internal_context("checking workflow approval permission")?
            }
            None => false,
        }
    {
        state
            .approve_after_delay(
                AutoApprovalCause::TechnicalReviewNoDetails,
                &mut transaction,
                redis,
            )
            .await?;
        PreparedMutations {
            states: vec![state],
        }
        .commit_without_reconciliation(transaction, redis)
        .await?;
        return Ok(false);
    }
    Ok(eligible)
}

/// The caller must hold the project's mutation lock until commit.
async fn record_approval_schedule_history(
    project_id: DBProjectId,
    delta: serde_json::Value,
    transaction: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
    sqlx::query!(
        r#"
            INSERT INTO project_history (project_id, done_by, delta)
            VALUES ($1, NULL, $2::jsonb)
            "#,
        project_id.0,
        delta,
    )
    .execute(&mut *transaction)
    .await
    .internal_context("recording project approval schedule")?;
    Ok(())
}

pub(crate) async fn sync_project_state(
    project_id: DBProjectId,
    transaction: &mut PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<SyncedProjectState, ApiError> {
    let (_, queued_projects) =
        sync_technical_review(&[project_id], transaction).await?;
    let state = load_project_state_with_membership(
        project_id,
        queued_projects.contains(&project_id),
        transaction,
        pool,
        redis,
    )
    .await?;
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
    load_project_state_with_options(
        project_id,
        has_outstanding_technical_review,
        true,
        transaction,
        pool,
        redis,
    )
    .await
}

async fn load_project_state_with_options(
    project_id: DBProjectId,
    has_outstanding_technical_review: bool,
    sync_verdicts: bool,
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
    let required_nags = validate_project(
        project.clone(),
        versions.clone(),
        available_categories,
        disclosures.clone(),
        pool,
        redis,
    )
    .await?
    .into_iter()
    .filter(|nag| nag.severity == ProjectNagSeverity::Required)
    .collect();

    let context = ThreadIssueContext {
        project: &project,
        versions: &versions,
        disclosures: &disclosures,
        team_members: &team_members,
    };
    let thread_issues = if sync_verdicts {
        db_models::DBThreadIssue::sync_project_verdicts(&context, transaction)
            .await
            .internal_context("synchronizing project thread issue verdicts")?
    } else {
        db_models::DBThreadIssue::get_many_for_threads(
            &[data.thread_id],
            &mut *transaction,
        )
        .await
        .internal_context("fetching project thread issue verdicts")?
    };

    let project_flag_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM project_flags WHERE project_id = $1 AND (active OR (what->>'retain_when_resolved')::bool = TRUE)"#,
        project_id.0,
    ).fetch_one(&mut *transaction).await
        .internal_context("counting project flag violations")?;
    Ok(SyncedProjectState {
        data,
        project,
        versions,
        disclosures,
        team_members,
        thread_issues,
        required_nags,
        has_outstanding_technical_review,
        project_flag_count,
    })
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct AutoApprovalDiagnostics {
    pub workflow:
        Option<crate::workflow::project_flag::approve::ApprovalStatus>,
    /// Current checks only; not evidence of a successful scan, deployed ruleset, or inference run.
    pub can_auto_approve: bool,
    pub eligible_by_checks: bool,
    pub in_review: bool,
    pub outstanding_technical_review: bool,
    pub blockers: Vec<AutoApprovalBlocker>,
    pub required_nags: Vec<crate::validate::project::ProjectNag>,
    pub project_flags: Vec<AutoApprovalFlag>,
    pub unresolved_thread_issues: Vec<AutoApprovalIssue>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AutoApprovalBlocker {
    NotInReview,
    RequiredNags,
    ProjectFlags,
    OutstandingTechnicalReview,
    UnresolvedThreadIssues,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct AutoApprovalFlag {
    pub key: String,
    pub active: bool,
    pub retain_when_resolved: bool,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct AutoApprovalIssue {
    pub id: crate::models::ids::ThreadIssueId,
    pub status: ThreadIssueVerdict,
}

pub(crate) async fn auto_approval_diagnostics(
    project_id: DBProjectId,
    pool: &crate::database::ReadOnlyPgPool,
    redis: &RedisPool,
) -> Result<AutoApprovalDiagnostics, ApiError> {
    let mut transaction = pool
        .begin()
        .await
        .internal_context("starting read-only eligibility transaction")?;
    sqlx::Executor::execute(
        &mut transaction,
        "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY",
    )
    .await
    .internal_context("enforcing read-only eligibility transaction")?;
    let queued_projects = sqlx::query_scalar!(
        r#"
		SELECT project_id AS "project_id!: DBProjectId"
		FROM delphi_tech_review_queue
		WHERE project_id = ANY($1::bigint[])
		"#,
        &[project_id.0],
    )
    .fetch_all(&mut transaction)
    .await
    .internal_context("fetching technical review membership")?;
    let state = load_project_state_with_options(
        project_id,
        queued_projects.contains(&project_id),
        false,
        &mut transaction,
        pool,
        redis,
    )
    .await?;
    let flags = db_models::project_flag_item::DBProjectFlag::get_project_flags_with_executor(
        project_id, &mut transaction,
    )
    .await
    .internal_context("fetching project flags")?
    .not_found_context("project not found")?;
    let eligible_by_checks = state.can_auto_approve();
    let in_review = state.in_review();
    let mut blockers = Vec::new();
    if !in_review {
        blockers.push(AutoApprovalBlocker::NotInReview);
    }
    if !state.required_nags.is_empty() {
        blockers.push(AutoApprovalBlocker::RequiredNags);
    }
    if state.project_flag_count != 0 {
        blockers.push(AutoApprovalBlocker::ProjectFlags);
    }
    if state.has_outstanding_technical_review {
        blockers.push(AutoApprovalBlocker::OutstandingTechnicalReview);
    }
    let unresolved_thread_issues = state
        .thread_issues
        .iter()
        .filter(|issue| issue.verdict != ThreadIssueVerdict::Resolved)
        .map(|issue| AutoApprovalIssue {
            id: issue.id.into(),
            status: issue.verdict,
        })
        .collect::<Vec<_>>();
    if !unresolved_thread_issues.is_empty() {
        blockers.push(AutoApprovalBlocker::UnresolvedThreadIssues);
    }
    transaction
        .rollback()
        .await
        .internal_context("closing read-only eligibility transaction")?;
    Ok(AutoApprovalDiagnostics {
        workflow: None,
        can_auto_approve: in_review && eligible_by_checks,
        eligible_by_checks,
        in_review,
        outstanding_technical_review: state.has_outstanding_technical_review,
        blockers,
        required_nags: state.required_nags,
        project_flags: flags
            .into_iter()
            .map(|flag| AutoApprovalFlag {
                key: flag.flag.key,
                active: flag.active,
                retain_when_resolved: flag.flag.retain_when_resolved,
            })
            .collect(),
        unresolved_thread_issues,
    })
}

fn ensure_reconciliation_sources_match(
    current: &impl serde::Serialize,
    evaluated: &impl serde::Serialize,
) -> Result<(), ApiError> {
    if serde_json::to_value(current)
        .internal_context("serializing current project flag sources")?
        != serde_json::to_value(evaluated)
            .internal_context("serializing evaluated project flag sources")?
    {
        return Err(ApiError::Internal(anyhow!(
            crate::workflow::project_flag::run_rules::ReconciliationInputsChanged
        )));
    }
    Ok(())
}

/// Revalidates the evaluated project, versions and file manifest under locks.
/// Other rule context (teams, users, organizations, disclosures, attributions and
/// moderation history) is included in the cache key, not transactionally reloaded.
#[cfg(test)]
async fn apply_project_flag_reconciliation(
    context: &crate::queue::moderation::project_flag::FlagContext,
    result: &db_models::project_flag_item::DBProjectFlagRunResult,
    ruleset_id: uuid::Uuid,
    file_manifest_hash: blake3::Hash,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<(), ApiError> {
    apply_project_flag_reconciliation_after_delay(
        context,
        result,
        ruleset_id,
        file_manifest_hash,
        pool,
        redis,
        false,
        None,
        true,
    )
    .await
}

pub(crate) async fn apply_project_flag_reconciliation_after_delay(
    context: &crate::queue::moderation::project_flag::FlagContext,
    result: &db_models::project_flag_item::DBProjectFlagRunResult,
    ruleset_id: uuid::Uuid,
    file_manifest_hash: blake3::Hash,
    pool: &PgPool,
    redis: &RedisPool,
    approval_due: bool,
    approval_workflow: Option<&temporalio_client::Client>,
    auto_withhold_enabled: bool,
) -> Result<(), ApiError> {
    use db_models::project_flag_item::ProjectFlag;
    use std::collections::BTreeMap;

    let project_id = DBProjectId::from(context.project.id);
    if result.project_id != project_id
        || result.run_id.is_some()
        || result.error.is_some()
        || result.rules.values().any(|rule| rule.error.is_some())
    {
        return Err(ApiError::Internal(anyhow!(
            "invalid project flag reconciliation result"
        )));
    }
    let mut flags = BTreeMap::new();
    for flag in result.rules.values().flat_map(|rule| &rule.flags) {
        let what = serde_json::to_value(flag)
            .internal_context("serializing reconciled project flag")?;
        if let Some(previous) = flags.insert(flag.key.clone(), what.clone())
            && previous != what
        {
            return Err(ApiError::Internal(anyhow!(
                "conflicting project flag key `{}`",
                flag.key
            )));
        }
    }
    let mut transaction = pool
        .begin()
        .await
        .internal_context("starting project flag application")?;
    let deployed = sqlx::query_scalar!(
        r#"SELECT ruleset_id FROM project_flags_deployed WHERE singleton FOR SHARE"#
    ).fetch_optional(&mut transaction).await
        .internal_context("locking deployed project flag revision")?;
    if deployed != Some(ruleset_id) {
        return Err(ApiError::Internal(anyhow!(
            crate::workflow::project_flag::run_rules::ReconciliationInputsChanged
        )));
    }
    let in_review = sqlx::query_scalar!(
        r#"SELECT in_review FROM mods WHERE id = $1 FOR UPDATE"#,
        project_id.0,
    )
    .fetch_optional(&mut transaction)
    .await
    .internal_context("locking reconciled project")?;
    if in_review != Some(true) {
        return Ok(());
    }
    // Lock versions before loading them so file insertions cannot race the comparison.
    sqlx::query!(
        r#"SELECT id FROM versions WHERE mod_id = $1 ORDER BY id FOR UPDATE"#,
        project_id.0,
    )
    .fetch_all(&mut transaction)
    .await
    .internal_context("locking reconciled project versions")?;
    let files = sqlx::query!(
        r#"SELECT f.id, f.version_id, f.filename, f.url, f.size FROM files f INNER JOIN versions v ON v.id = f.version_id WHERE v.mod_id = $1 ORDER BY f.id FOR SHARE OF f"#,
        project_id.0,
    ).fetch_all(&mut transaction).await
        .internal_context("locking reconciled project files")?;
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
        return Err(ApiError::Internal(anyhow!(
            crate::workflow::project_flag::run_rules::ReconciliationInputsChanged
        )));
    }
    let has_outstanding_technical_review = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM delphi_tech_review_queue WHERE project_id = $1) AS "exists!""#,
        project_id.0,
    )
    .fetch_one(&mut transaction)
    .await
    .internal_context("fetching reconciled technical review membership")?;
    let mut state = load_project_state_with_options(
        project_id,
        has_outstanding_technical_review,
        false,
        &mut transaction,
        pool,
        redis,
    )
    .await?;
    let mut versions = state.versions.clone();
    crate::queue::moderation::project_flag::sort_versions_newest_first(
        &mut versions,
    );
    ensure_reconciliation_sources_match(
        &(&state.project, &versions),
        &(&context.project, &context.versions),
    )?;

    let persisted = sqlx::query!(
        r#"SELECT what AS "what!: sqlx::types::Json<ProjectFlag>"
        FROM project_flags
        WHERE project_id = $1 AND (active OR (what->>'retain_when_resolved')::bool = TRUE)
        ORDER BY what->>'key', id"#,
        project_id.0,
    )
    .fetch_all(&mut transaction)
    .await
    .internal_context("loading reconciled project violations")?;
    let effective = db_models::project_flag_item::effective_project_flags(
        result.rules.values().flat_map(|rule| &rule.flags),
        persisted.iter().map(|row| &row.what.0),
    );
    let flags = effective
        .iter()
        .filter(|flag| flag.active)
        .map(|flag| {
            serde_json::to_value(&flag.flag)
                .map(|what| (flag.flag.key.clone(), what))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()
        .internal_context("serializing effective project flags")?;
    let mut changed = sqlx::query!(
        r#"UPDATE project_flags SET active = FALSE WHERE project_id = $1 AND active AND NOT ((what->>'key') = ANY($2::text[]))"#,
        project_id.0, &flags.keys().cloned().collect::<Vec<_>>(),
    ).execute(&mut transaction).await
        .internal_context("deactivating superseded project flags")?.rows_affected() > 0;
    for what in flags.values() {
        let id = db_models::ids::generate_project_flag_id(&mut transaction)
            .await
            .internal_context("generating reconciled project flag ID")?;
        changed |= sqlx::query!(
            r#"INSERT INTO project_flags (id, run_id, project_id, what, active)
            VALUES ($1, NULL, $2, $3, TRUE)
            ON CONFLICT (project_id, (what->>'key')) DO UPDATE
            SET run_id = NULL, what = EXCLUDED.what, active = TRUE
            WHERE project_flags.what IS DISTINCT FROM EXCLUDED.what OR NOT project_flags.active OR project_flags.run_id IS NOT NULL"#,
            id.0, project_id.0, what,
        ).execute(&mut transaction).await
            .internal_context("upserting reconciled project flag")?.rows_affected() > 0;
    }
    state.project_flag_count = effective.len() as i64;
    let desired = db_models::project_flag_item::effective_project_flag_outcome(
        state.in_review(),
        &effective,
        false,
    );
    let outcome = desired.filter(|outcome| {
        *outcome != ProjectReviewOutcome::Approved
            && (*outcome != ProjectReviewOutcome::Withheld
                || auto_withhold_enabled)
    });
    if approval_due
        && desired == Some(ProjectReviewOutcome::Approved)
        && match approval_workflow {
            Some(temporal) => {
                crate::workflow::project_flag::approve::approval_permitted(
                    temporal,
                    project_id.into(),
                )
                .await
                .internal_context("checking workflow approval permission")?
            }
            None => false,
        }
    {
        changed |= state
            .approve_after_delay(
                AutoApprovalCause::ProjectFlags { ruleset_id },
                &mut transaction,
                redis,
            )
            .await?;
    }
    if let Some(outcome) =
        outcome.filter(|outcome| *outcome != state.review_outcome())
    {
        sqlx::query!(
            r#"UPDATE mods SET review_outcome = $1, in_review = FALSE WHERE id = $2"#,
            outcome.as_str(),
            project_id.0,
        )
        .execute(&mut transaction)
        .await
        .internal_context("updating reconciled project review outcome")?;
        db_models::DBProject::record_history(
            project_id,
            None,
            ProjectDelta::ChangeReviewOutcome {
                from: state.review_outcome(),
                to: outcome,
                reason: Some(ReviewOutcomeReason::Auto {
                    provenance: ApprovalProvenance {
                        trigger: AutoApprovalCause::ProjectFlags { ruleset_id },
                        resolved_issue_ids: state
                            .thread_issues
                            .iter()
                            .map(|issue| issue.id.into())
                            .collect(),
                    },
                }),
            },
            &mut transaction,
        )
        .await
        .internal_context("recording project flag review outcome history")?;
        db_models::DBProject::record_history(
            project_id,
            None,
            ProjectDelta::ChangeInReview {
                from: true,
                to: false,
            },
            &mut transaction,
        )
        .await
        .internal_context("recording project flag review completion history")?;
        state.project.review_outcome = outcome;
        state.project.in_review = false;
        state.data.inner.review_outcome = outcome;
        state.data.inner.in_review = false;
        changed = true;
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
    _cause: AutoApprovalCause,
    transaction: &mut PgTransaction<'_>,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<PreparedMutations, ApiError> {
    let mut project_ids = project_ids.to_vec();
    project_ids.sort_by_key(|project_id| project_id.0);
    project_ids.dedup();

    let mut states = Vec::with_capacity(project_ids.len());
    for project_id in project_ids {
        let state =
            load_project_state(project_id, transaction, pool, redis).await?;
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

    let (_, queued_projects) =
        sync_technical_review(&project_ids, transaction).await?;
    let queued_projects = queued_projects
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    let mut states = Vec::with_capacity(project_ids.len());
    for project_id in project_ids {
        let state = load_project_state_with_membership(
            project_id,
            queued_projects.contains(&project_id),
            transaction,
            pool,
            redis,
        )
        .await?;
        states.push(state);
    }

    if project_editor
        .is_some_and(|user| !user.scopes.contains(Scopes::MODERATION_WRITE))
    {
        for state in &states {
            if state.project.in_review && !state.required_nags.is_empty() {
                return Err(ApiError::Request(anyhow!(
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

    pub(crate) async fn commit(
        self,
        transaction: PgTransaction<'_>,
        redis: &RedisPool,
        temporal: &TemporalClient,
    ) -> Result<(), ApiError> {
        let mut approval_executions = Vec::new();
        for state in &self.states {
            if !state.in_review() {
                approval_executions.push(
                    crate::workflow::project_flag::approve::current_execution(
                        &temporal.client,
                        state.data.inner.id.into(),
                    )
                    .await
                    .internal_context("capturing approval execution")?,
                );
            }
        }
        self.commit_without_reconciliation(transaction, redis)
            .await?;
        for execution in approval_executions {
            crate::workflow::project_flag::approve::finish_execution(execution)
                .await
                .internal_context("completing approval execution")?;
        }
        self.dispatch_reconciliation(temporal).await
    }

    /// Uploads do not exit review, so they need no approval-run completion lookup.
    /// Publishes their files and search changes before making any Temporal request.
    pub(crate) async fn commit_with_post_commit(
        self,
        transaction: PgTransaction<'_>,
        redis: &RedisPool,
        temporal: &TemporalClient,
        post_commit: impl std::future::Future<Output = ()>,
    ) -> Result<(), ApiError> {
        self.commit_without_reconciliation(transaction, redis)
            .await?;
        post_commit.await;
        self.dispatch_reconciliation(temporal).await
    }

    async fn dispatch_reconciliation(
        &self,
        temporal: &TemporalClient,
    ) -> Result<(), ApiError> {
        for state in &self.states {
            crate::workflow::project_flag::reconcile_project_review_state(
                temporal,
                state.data.inner.id.into(),
                state.in_review(),
            )
            .await
            .internal_context("dispatching project flag reconciliation")?;
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
    temporal: &TemporalClient,
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
        && !state.required_nags.is_empty()
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

    let approval_execution = if original_in_review || entered_review {
        crate::workflow::project_flag::approve::current_execution(
            &temporal.client,
            project_id.into(),
        )
        .await
        .internal_context("capturing approval execution")?
    } else {
        None
    };
    if entered_review {
        crate::workflow::project_flag::approve::finish_execution(
            approval_execution.clone(),
        )
        .await
        .internal_context("completing previous review approval execution")?;
    }
    transaction
        .commit()
        .await
        .internal_context("committing project mutation")?;
    if !state.in_review() {
        crate::workflow::project_flag::approve::finish_execution(
            approval_execution,
        )
        .await
        .internal_context("completing approval execution")?;
    }

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

    if state.in_review() || original_in_review {
        crate::workflow::project_flag::reconcile_project_review_state(
            temporal,
            project_id.into(),
            state.in_review(),
        )
        .await
        .internal_context("dispatching project flag reconciliation")?;
    }

    Ok(state.project.is_searchable())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ids::ThreadIssueId;
    use ariadne::ids::UserId;
    use serde_json::json;

    #[actix_rt::test]
    async fn approval_schedule_history_records_each_event() {
        use crate::test::{
            api_v3::ApiV3,
            environment::{TestEnvironment, with_test_environment},
        };
        with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
            let project_id =
                DBProjectId::from(env.dummy.project_alpha.project_id_parsed);
            let before =
                db_models::DBProject::get_history(project_id, &env.db.ro_pool)
                    .await
                    .unwrap()
                    .len();
            let mut transaction = env.db.pool.begin().await.unwrap();
            sqlx::query_scalar!(
                r#"SELECT in_review FROM mods WHERE id = $1 FOR UPDATE"#,
                project_id.0
            )
            .fetch_one(&mut transaction)
            .await
            .unwrap();
            for deadline in [
                "2026-11-24T12:00:00Z",
                "2026-11-24T12:00:00Z",
                "2026-11-24T13:00:00Z",
            ] {
                let delta =
                    serde_json::to_value(ProjectDelta::AutoApprovalScheduled {
                        provenance: ApprovalProvenance {
                            trigger:
                                AutoApprovalCause::TechnicalReviewNoDetails,
                            resolved_issue_ids: vec![],
                        },
                        scheduled_for: deadline.parse().unwrap(),
                    })
                    .unwrap();
                record_approval_schedule_history(
                    project_id,
                    delta,
                    &mut transaction,
                )
                .await
                .unwrap();
            }
            transaction.commit().await.unwrap();
            let history =
                db_models::DBProject::get_history(project_id, &env.db.ro_pool)
                    .await
                    .unwrap();
            assert_eq!(history.len(), before + 3);
            assert!(history[before..].iter().all(
                |entry| entry.done_by.is_none()
                    && matches!(
                        entry.delta,
                        ProjectDelta::AutoApprovalScheduled { .. }
                    )
            ));
        })
        .await;
    }

    #[actix_rt::test]
    async fn reconciliation_completes_only_changed_review_outcomes() {
        use crate::database::models::project_flag_item::{
            DBProjectFlagRunResult, ProjectFlag, ProjectFlagRuleResult,
        };
        use crate::database::models::thread_issue_item::ThreadIssueBuilder;
        use crate::models::thread_issues::ThreadIssueTarget;
        use crate::queue::moderation::project_flag::{
            FlagContext, FlagModerationThread, FlagTeam,
        };
        use crate::test::{
            api_v3::ApiV3,
            environment::{TestEnvironment, with_test_environment},
        };
        use std::collections::BTreeMap;

        with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
            let project_id = DBProjectId::from(env.dummy.project_alpha.project_id_parsed);
            let ruleset_id = uuid::Uuid::new_v4();
            sqlx::query!(
                r#"INSERT INTO project_flag_rulesets (id, content_hash, rules, modules) VALUES ($1, $2, '[]'::jsonb, '[]'::jsonb)"#,
                ruleset_id, ruleset_id.to_string(),
            ).execute(&*env.db.pool).await.unwrap();
            sqlx::query!(
                r#"INSERT INTO project_flags_deployed (singleton, ruleset_id) VALUES (TRUE, $1)
                ON CONFLICT (singleton) DO UPDATE SET ruleset_id = EXCLUDED.ruleset_id"#,
                ruleset_id,
            ).execute(&*env.db.pool).await.unwrap();
            let files = sqlx::query!(
                r#"SELECT f.id, f.version_id, f.filename, f.url, f.size FROM files f INNER JOIN versions v ON v.id = f.version_id WHERE v.mod_id = $1 ORDER BY f.id FOR SHARE OF f"#,
                project_id.0,
            ).fetch_all(&*env.db.pool).await.unwrap();
            let manifest = files.into_iter().map(|file| (file.id, file.version_id, file.filename, file.url, i64::from(file.size))).collect::<Vec<_>>();
            let manifest_hash = crate::workflow::project_flag::scan_cache::rules_input_hash(&manifest).unwrap();

            for (initial, flag, expected, exits_review) in [
                (ProjectReviewOutcome::NeverReviewed, None, ProjectReviewOutcome::NeverReviewed, false),
                (ProjectReviewOutcome::Approved, Some("withheld"), ProjectReviewOutcome::Withheld, true),
                (ProjectReviewOutcome::Withheld, Some("withheld"), ProjectReviewOutcome::Withheld, false),
                (ProjectReviewOutcome::NeverReviewed, Some("manual_review"), ProjectReviewOutcome::NeverReviewed, false),
            ] {
                sqlx::query!(r#"UPDATE mods SET review_outcome = $1, in_review = TRUE WHERE id = $2"#, initial.as_str(), project_id.0)
                    .execute(&*env.db.pool).await.unwrap();
                let mut transaction = env.db.pool.begin().await.unwrap();
                let mut state = load_project_state_with_options(project_id, false, false, &mut transaction, &env.db.pool, &env.db.redis_pool).await.unwrap();
                if state.thread_issues.is_empty() {
                    DBThreadIssue::insert_many(state.data.thread_id, db_models::DBUserId(crate::test::database::MOD_USER_ID_PARSED), vec![ThreadIssueBuilder {
                        facets: vec![ThreadIssueTarget::RemoveIcon], why: json!({}),
                    }], &mut transaction).await.unwrap();
                }
                let issues = DBThreadIssue::get_many_for_threads(&[state.data.thread_id], &mut transaction).await.unwrap();
                for facet in issues.iter().flat_map(|issue| &issue.facets) {
                    DBThreadIssue::update_facet_flags(facet.id.into(), Some(true), Some(true), &mut transaction).await.unwrap();
                }
                state = load_project_state_with_options(project_id, false, false, &mut transaction, &env.db.pool, &env.db.redis_pool).await.unwrap();
                if flag.is_none() {
                    assert!(state.can_auto_approve(), "fixture must be eligible for approval");
                }
                transaction.commit().await.unwrap();
                db_models::DBProject::clear_cache(project_id, state.data.inner.slug.clone(), Some(true), &env.db.redis_pool).await.unwrap();
                let cached = db_models::DBProject::get_id(project_id, &*env.db.ro_pool, &env.db.redis_pool).await.unwrap().unwrap();
                assert!(cached.inner.in_review);
                crate::queue::moderation::project_flag::sort_versions_newest_first(&mut state.versions);
                let context = FlagContext {
                    ai: crate::queue::moderation::project_flag::FlagAiContext::default(), project: state.project,
                    team: FlagTeam { id: state.data.inner.team_id.into(), members: vec![] },
                    users: vec![], organization: None, scanned_files: vec![],
                    versions: state.versions, disclosures: vec![], attributions: vec![],
                    moderation_thread: FlagModerationThread { id: state.data.thread_id.into(), messages: vec![] },
                };
                let history_before = db_models::DBProject::get_history(project_id, &env.db.ro_pool).await.unwrap().len();
                let mut rules = BTreeMap::new();
                if let Some(outcome) = flag {
                    let flag: ProjectFlag = serde_json::from_value(json!({
                        "key": "retained-review", "flag_outcome": outcome, "retain_when_resolved": true,
                    })).unwrap();
                    rules.insert("test".into(), ProjectFlagRuleResult { flags: vec![flag], log: vec![], error: None });
                }
                let result = DBProjectFlagRunResult { run_id: None, project_id, rules, error: None, proposed_review_outcome: None };
                if flag == Some("withheld") {
                    apply_project_flag_reconciliation_after_delay(&context, &result, ruleset_id, manifest_hash, &env.db.pool, &env.db.redis_pool, false, None, false).await.unwrap();
                    let retained = db_models::DBProject::get_id(project_id, &*env.db.ro_pool, &env.db.redis_pool).await.unwrap().unwrap();
                    assert!(retained.inner.in_review);
                    assert_eq!(retained.inner.review_outcome, initial);
                    assert_eq!(db_models::DBProject::get_history(project_id, &env.db.ro_pool).await.unwrap().len(), history_before);
                }
                apply_project_flag_reconciliation(&context, &result, ruleset_id, manifest_hash, &env.db.pool, &env.db.redis_pool).await.unwrap();
                let updated = db_models::DBProject::get_id(project_id, &*env.db.ro_pool, &env.db.redis_pool).await.unwrap().unwrap();
                assert_eq!(updated.inner.review_outcome, expected);
                assert_eq!(updated.inner.in_review, !exits_review);
                let project = Project::from(updated);
                assert_eq!(project.review_outcome, expected);
                assert_eq!(project.in_review, !exits_review);
                assert_eq!(serde_json::to_value(crate::models::v2::projects::ProjectStatus::from_project(&project)).unwrap(), json!(if !exits_review { "processing" } else if expected == ProjectReviewOutcome::Approved { "approved" } else { "withheld" }));
                let history = db_models::DBProject::get_history(project_id, &env.db.ro_pool).await.unwrap();
                let added = &history[history_before..];
                assert_eq!(added.len(), if exits_review { 2 } else { 0 });
                if exits_review {
                    assert!(added.iter().any(|entry| matches!(entry.delta, ProjectDelta::ChangeReviewOutcome { from, to, .. } if from == initial && to == expected)));
                    assert!(added.iter().any(|entry| matches!(entry.delta, ProjectDelta::ChangeInReview { from: true, to: false })));
                }
                if flag.is_none() {
                    apply_project_flag_reconciliation(&context, &result, ruleset_id, manifest_hash, &env.db.pool, &env.db.redis_pool).await.unwrap();
                    let pending = db_models::DBProject::get_id(project_id, &*env.db.ro_pool, &env.db.redis_pool).await.unwrap().unwrap();
                    assert!(pending.inner.in_review);
                    assert_eq!(pending.inner.review_outcome, initial);
                    assert_eq!(db_models::DBProject::get_history(project_id, &env.db.ro_pool).await.unwrap().len(), history_before);
                    apply_project_flag_reconciliation_after_delay(&context, &result, ruleset_id, manifest_hash, &env.db.pool, &env.db.redis_pool, true, None, true).await.unwrap();
                    let retained = db_models::DBProject::get_id(project_id, &*env.db.ro_pool, &env.db.redis_pool).await.unwrap().unwrap();
                    assert!(retained.inner.in_review);
                    assert_eq!(retained.inner.review_outcome, initial);
                    let mut transaction = env.db.pool.begin().await.unwrap();
                    let mut state = load_project_state(project_id, &mut transaction, &env.db.pool, &env.db.redis_pool).await.unwrap();
                    assert!(state.approve_after_delay(AutoApprovalCause::ProjectFlags { ruleset_id }, &mut transaction, &env.db.redis_pool).await.unwrap());
                    PreparedMutations { states: vec![state] }.commit_without_reconciliation(transaction, &env.db.redis_pool).await.unwrap();
                    let approved = db_models::DBProject::get_id(project_id, &*env.db.ro_pool, &env.db.redis_pool).await.unwrap().unwrap();
                    assert_eq!(approved.inner.review_outcome, ProjectReviewOutcome::Approved);
                    assert!(!approved.inner.in_review);
                    assert!(!delayed_auto_approval_eligible(project_id, &env.db.pool, &env.db.redis_pool, false, None, None).await.unwrap());
                }
                if flag == Some("manual_review") {
                    let mut context = context.clone();
                    context.project = project;
                    let result = DBProjectFlagRunResult { rules: BTreeMap::new(), ..result };
                    apply_project_flag_reconciliation_after_delay(&context, &result, ruleset_id, manifest_hash, &env.db.pool, &env.db.redis_pool, true, None, true).await.unwrap();
                    let retained = db_models::DBProject::get_id(project_id, &*env.db.ro_pool, &env.db.redis_pool).await.unwrap().unwrap();
                    assert!(retained.inner.in_review);
                    assert_eq!(retained.inner.review_outcome, initial);
                    assert_eq!(db_models::DBProject::get_history(project_id, &env.db.ro_pool).await.unwrap().len(), history.len());
                }
            }
        }).await;
    }

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
            json!({"kind": "project_flags", "ruleset_id": ruleset_id})
        );
        assert!(
            matches!(serde_json::from_value::<AutoApprovalCause>(value).unwrap(), AutoApprovalCause::ProjectFlags { ruleset_id: id } if id == ruleset_id)
        );
    }

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

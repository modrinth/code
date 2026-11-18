use actix_web::{HttpRequest, HttpResponse, post, web};
use eyre::eyre;
use serde::Deserialize;
use validator::Validate;
use xredis::RedisPool;

use super::{mutation, validate};
use crate::auth::get_user_from_headers;
use crate::database::advisory_lock::AdvisoryLock;
use crate::database::models::moderation_lock_item::DBModerationLock;
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::thread_item::ThreadMessageBuilder;
use crate::database::{PgPool, models as db_models};
use crate::env::ENV;
use crate::models::notifications::NotificationBody;
use crate::models::pats::Scopes;
use crate::models::projects::{
    ApprovalReason, AutoApprovalCause, ProjectReviewStatus, ProjectStatus,
    ProjectVisibility, ReviewStatusProvenance,
};
use crate::models::teams::ProjectPermissions;
use crate::models::threads::MessageBody;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::routes::internal::delphi;
use crate::search::SearchState;
use crate::util::MinecraftMeta;
use crate::util::error::Context as _;

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct ReviewStateUpdate {
    pub review_status: ProjectReviewStatus,
    pub reason: Option<String>,
}

#[allow(clippy::too_many_arguments)]
#[utoipa::path(
    context_path = "/project",
    tag = "projects",
    request_body = ReviewStateUpdate,
    responses((status = NO_CONTENT))
)]
#[post("/{id}/review-state")]
pub async fn project_review_state(
    minecraft_meta: web::Data<MinecraftMeta>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    web::Json(update): web::Json<ReviewStateUpdate>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    if let Some(response) = crate::routes::redirect_ref(
        &req,
        "id",
        pool.as_ref(),
        redis.as_ref(),
        session_queue.as_ref(),
        Scopes::PROJECT_WRITE,
    )
    .await?
    {
        return Ok(response);
    }

    project_review_state_internal(
        minecraft_meta,
        req,
        info,
        pool,
        web::Json(update),
        redis,
        session_queue,
        search_state,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn project_review_state_internal(
    minecraft_meta: web::Data<MinecraftMeta>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    web::Json(update): web::Json<ReviewStateUpdate>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?;
    update
        .validate()
        .map_err(|err| eyre!(err))
        .wrap_request_err("validating review state update")?;
    let project =
        db_models::DBProject::get(&info.into_inner().0, &**pool, &redis)
            .await
            .wrap_internal_err("fetching project")?
            .ok_or_else(|| ApiError::NotFound(eyre!("resource not found")))?;
    let id = project.inner.id;
    let (team_member, organization_team_member) =
        db_models::DBTeamMember::get_for_project_permissions(
            &project.inner,
            user.user.id.into(),
            &**pool,
        )
        .await
        .wrap_internal_err("fetching project permissions")?;
    let permissions = ProjectPermissions::get_permissions_by_scopes(
        user.scopes,
        &team_member,
        &organization_team_member,
    )
    .ok_or_else(|| {
        ApiError::Auth(eyre!("you do not have permission to edit this project"))
    })?;
    if !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
        return Err(ApiError::Auth(eyre!(
            "you do not have permission to edit the review state of this project"
        )));
    }

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting review state transaction")?;
    delphi::tech_review_queue::lock_projects(&[id], &mut transaction).await?;
    AdvisoryLock::ProjectState(id)
        .acquire(&mut transaction)
        .await
        .wrap_internal_err("locking project review state")?;
    let locked = sqlx::query!(
        r#"
        SELECT visibility AS "visibility: ProjectVisibility",
            review_status AS "review_status: ProjectReviewStatus", slug,
            webhook_sent
        FROM mods
        WHERE id = $1
        FOR UPDATE
        "#,
        id as db_models::DBProjectId,
    )
    .fetch_one(&mut transaction)
    .await
    .wrap_internal_err("fetching locked project review state")?;
    let moderator = user.scopes.contains(Scopes::MODERATION_WRITE);
    if !moderator
        && (locked.review_status == ProjectReviewStatus::Approved
            || update.review_status != ProjectReviewStatus::InReview)
    {
        return Err(ApiError::Auth(eyre!(
            "you do not have permission to set this review state"
        )));
    }
    if locked.review_status == update.review_status {
        return Ok(HttpResponse::NoContent().finish());
    }
    if moderator
        && !user.scopes.contains(Scopes::ADMIN_WRITE)
        && locked.review_status == ProjectReviewStatus::InReview
        && let Some(lock) =
            DBModerationLock::get_with_user(id, &mut transaction)
                .await
                .wrap_internal_err("fetching moderation lock")?
        && lock.moderator_id != db_models::DBUserId::from(user.user.id)
        && !lock.expired
    {
        return Err(ApiError::Auth(eyre!(
            "this project is currently being moderated by @{}; wait for them to finish or for the lock to expire",
            lock.moderator_username
        )));
    }

    let original_status = ProjectStatus::from_visibility_review_status(
        locked.visibility,
        locked.review_status,
    );
    let submit_for_review =
        update.review_status == ProjectReviewStatus::InReview;
    let mut auto_approved = false;
    if submit_for_review {
        if !moderator {
            validate::ensure_project_is_valid_for_review(
                id,
                &pool,
                &mut transaction,
                &redis,
            )
            .await?;
        }
        let mut state =
            mutation::sync_project_state(id, &mut transaction, &pool, &redis)
                .await?;
        sqlx::query!(
            r#"
            UPDATE mods
            SET moderation_message = NULL, moderation_message_body = NULL,
                queued = NOW(), initial_queued = COALESCE(initial_queued, NOW())
            WHERE id = $1
            "#,
            id as db_models::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("updating project submission timestamps")?;
        if !moderator
            && matches!(
                locked.review_status,
                ProjectReviewStatus::Rejected | ProjectReviewStatus::Withheld
            )
        {
            auto_approved = state
                .try_auto_approve(
                    AutoApprovalCause::Resubmission {
                        user_id: user.user.id,
                    },
                    &mut transaction,
                    &redis,
                )
                .await?;
        }
    }

    let new_review_status = if auto_approved {
        ProjectReviewStatus::Approved
    } else {
        update.review_status
    };
    let new_status = ProjectStatus::from_visibility_review_status(
        locked.visibility,
        new_review_status,
    );
    if !auto_approved {
        db_models::DBProject::update_review_status(
            id,
            new_review_status,
            Some(user.user.id.into()),
            &ReviewStatusProvenance::ByUser {
                reason: update.reason,
            },
            &mut transaction,
        )
        .await
        .wrap_internal_err("updating project review status")?;
        if new_review_status == ProjectReviewStatus::Approved {
            let reason =
                serde_json::to_value(ApprovalReason::ByUser(user.user.id))
                    .wrap_internal_err("serializing project approval reason")?;
            sqlx::query!(
                r#"
                UPDATE mods
                SET approved_because = CASE WHEN approved IS NULL THEN $2::jsonb
                        ELSE approved_because END,
                    approved = COALESCE(approved, NOW()), initial_queued = NULL
                WHERE id = $1
                "#,
                id as db_models::DBProjectId,
                reason,
            )
            .execute(&mut transaction)
            .await
            .wrap_internal_err("updating project approval timestamps")?;
        }
        if team_member.as_ref().is_none_or(|member| !member.accepted) {
            let recipients = sqlx::query!(
                r#"
                SELECT user_id FROM team_members
                WHERE team_id = $1 AND accepted
                "#,
                project.inner.team_id as db_models::DBTeamId,
            )
            .fetch_all(&mut transaction)
            .await
            .wrap_internal_err("fetching review notification recipients")?
            .into_iter()
            .map(|row| db_models::DBUserId(row.user_id))
            .collect::<Vec<_>>();
            for body in [
                NotificationBody::StatusChange {
                    project_id: id.into(),
                    old_status: original_status,
                    new_status,
                },
                if new_review_status == ProjectReviewStatus::Approved {
                    NotificationBody::ProjectStatusApproved {
                        project_id: id.into(),
                    }
                } else {
                    NotificationBody::ProjectStatusNeutral {
                        project_id: id.into(),
                        old_status: original_status,
                        new_status,
                    }
                },
            ] {
                NotificationBuilder { body }
                    .insert_many(recipients.clone(), &mut transaction, &redis)
                    .await
                    .wrap_internal_err(
                        "notifying project team of review state change",
                    )?;
            }
        }
        ThreadMessageBuilder {
            author_id: Some(user.user.id.into()),
            body: MessageBody::StatusChange {
                old_status: original_status,
                new_status,
            },
            thread_id: project.thread_id,
            hide_identity: moderator,
        }
        .insert(&mut transaction)
        .await
        .wrap_internal_err("recording project review state change")?;
    }

    mutation::finalize_project_edit(
        id,
        original_status,
        locked.slug.clone(),
        submit_for_review,
        false,
        transaction,
        &pool,
        &redis,
        Some(&user),
    )
    .await?;
    if original_status.is_searchable() && !new_status.is_searchable() {
        search_state.queue.push_project_removal(id.into()).await;
    } else if !original_status.is_searchable() && new_status.is_searchable() {
        search_state
            .queue
            .push_project_with_all_versions_change(id.into())
            .await;
    } else {
        search_state.queue.push_project_change(id.into()).await;
    }

    if new_status.is_searchable()
        && !locked.webhook_sent
        && !ENV.PUBLIC_DISCORD_WEBHOOK.is_empty()
        && project.inner.components.minecraft_server.is_none()
    {
        crate::util::webhook::send_discord_webhook(
            &minecraft_meta,
            id.into(),
            &pool,
            &redis,
            &ENV.PUBLIC_DISCORD_WEBHOOK,
            None,
        )
        .await
        .ok();
        sqlx::query!(
            "UPDATE mods SET webhook_sent = TRUE WHERE id = $1",
            id as db_models::DBProjectId,
        )
        .execute(&**pool)
        .await
        .wrap_internal_err("marking project announcement webhook sent")?;
        db_models::DBProject::clear_cache(
            id,
            project.inner.slug.clone(),
            None,
            &redis,
        )
        .await
        .wrap_internal_err("clearing project webhook cache")?;
    }
    if moderator && !ENV.MODERATION_SLACK_WEBHOOK.is_empty() {
        crate::util::webhook::send_slack_project_webhook(
            &minecraft_meta,
            id.into(),
            &pool,
            &redis,
            &ENV.MODERATION_SLACK_WEBHOOK,
            Some(format!(
                "*<{}/user/{}|{}>* changed project status from *{}* to *{}*",
                ENV.SITE_URL,
                user.user.username,
                user.user.username,
                original_status.as_friendly_str(),
                new_status.as_friendly_str(),
            )),
        )
        .await
        .ok();
    }
    Ok(HttpResponse::NoContent().finish())
}

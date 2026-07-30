use actix_web::{HttpRequest, get, patch, web};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use xredis::RedisPool;

use crate::auth::checks::is_visible_project;
use crate::auth::get_user_from_headers;
use crate::database::models as db_models;
use crate::database::{PgPool, ReadOnlyPgPool};
use crate::models::disclosures::{ProjectDisclosure, ProjectDisclosureData};
use crate::models::pats::Scopes;
use crate::models::teams::ProjectPermissions;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_project_disclosures)
        .service(modify_project_disclosures);
}

#[derive(Serialize, ToSchema)]
pub struct GetProjectDisclosures {
    pub disclosures: Vec<ProjectDisclosureData>,
}

#[utoipa::path(
	context_path = "/project",
	tag = "project_disclosures",
	responses((status = OK, body = GetProjectDisclosures))
)]
#[get("/{project_id}/disclosures")]
pub async fn get_project_disclosures(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<GetProjectDisclosures>, ApiError> {
    let (string,) = info.into_inner();

    let project = db_models::DBProject::get(&string, &***ro_pool, &redis)
        .await
        .wrap_internal_err("failed to fetch project")?
        .ok_or(ApiError::NotFound)?;

    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await
    .map(|(_, user)| user)
    .ok();

    if !is_visible_project(&project.inner, &user_option, &pool, false)
        .await
        .wrap_internal_err("failed to check project visibility")?
    {
        return Err(ApiError::NotFound);
    }

    let viewer_is_moderator =
        user_option.is_some_and(|user| user.role.is_mod());

    let disclosures = db_models::DBProjectDisclosure::get_many_for_project(
        project.inner.id,
        &***ro_pool,
    )
    .await
    .wrap_internal_err("failed to fetch project disclosures")?;

    Ok(web::Json(GetProjectDisclosures {
        disclosures: disclosures
            .into_iter()
            .map(|disclosure| {
                ProjectDisclosureData::from_db(disclosure, viewer_is_moderator)
            })
            .collect(),
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct ModifyProjectDisclosures {
    pub set: Vec<ProjectDisclosure>,
    pub remove: Vec<String>,
}

#[utoipa::path(
	context_path = "/project",
	tag = "project_disclosures",
	request_body = ModifyProjectDisclosures,
	responses((status = NO_CONTENT))
)]
#[patch("/{project_id}/disclosures")]
pub async fn modify_project_disclosures(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<ModifyProjectDisclosures>,
) -> Result<(), ApiError> {
    let (string,) = info.into_inner();
    let body = body.into_inner();

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?
    .1;

    let project = db_models::DBProject::get(&string, &**pool, &redis)
        .await
        .wrap_internal_err("failed to fetch project")?
        .ok_or(ApiError::NotFound)?;

    let (team_member, organization_team_member) =
        db_models::DBTeamMember::get_for_project_permissions(
            &project.inner,
            user.id.into(),
            &**pool,
        )
        .await
        .wrap_internal_err("failed to fetch project permissions")?;

    let can_edit_details = ProjectPermissions::get_permissions_by_role(
        &user.role,
        &team_member,
        &organization_team_member,
    )
    .is_some_and(|perms| perms.contains(ProjectPermissions::EDIT_DETAILS));

    if !can_edit_details {
        return Err(ApiError::Auth(eyre!(
            "you do not have permission to edit this project's disclosures"
        )));
    }

    if !user.role.is_mod() {
        let modified_types = body
            .set
            .iter()
            .map(|disclosure| <&'static str>::from(disclosure).to_owned())
            .chain(body.remove.iter().cloned())
            .collect::<Vec<_>>();

        if db_models::DBProjectDisclosure::any_set_by_moderator(
            project.inner.id,
            &modified_types,
            &**pool,
        )
        .await
        .wrap_internal_err("failed to check moderator disclosures")?
        {
            return Err(ApiError::Auth(eyre!(
                "you cannot modify a disclosure set by a moderator"
            )));
        }
    }

    let mut transaction = pool.begin().await?;

    for disclosure in body.set {
        db_models::DBProjectDisclosure {
            project_id: project.inner.id,
            disclosure,
            updated_at: Utc::now(),
            updated_by: user.id.into(),
            set_by_moderator: user.role.is_mod(),
        }
        .upsert(&mut transaction)
        .await
        .wrap_internal_err("failed to upsert project disclosure")?;
    }

    for disclosure_type in &body.remove {
        db_models::DBProjectDisclosure::remove(
            project.inner.id,
            disclosure_type,
            &mut transaction,
        )
        .await
        .wrap_internal_err("failed to remove project disclosure")?;
    }

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit project disclosure changes")?;

    Ok(())
}

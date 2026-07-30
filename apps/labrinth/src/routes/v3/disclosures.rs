use actix_web::{HttpRequest, get, web};
use serde::Serialize;
use utoipa::ToSchema;
use xredis::RedisPool;

use crate::auth::checks::is_visible_project;
use crate::auth::get_user_from_headers;
use crate::database::models as db_models;
use crate::database::{PgPool, ReadOnlyPgPool};
use crate::models::disclosures::ProjectDisclosureData;
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_project_disclosures);
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
            .collect()
    }))
}

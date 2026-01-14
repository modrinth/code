use actix_web::web;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::Validate;

use crate::{
    auth::get_user_from_headers,
    database::models,
    models::{ids::ProjectId, v3::user_limits::UserLimits, v67},
    util::error::Context,
};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(create);
}

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum CreateError {
    #[error("project limit reached")]
    LimitReached,
    #[error("incompatible components")]
    IncompatibleComponents(v67::ComponentsIncompatibleError),
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct CreateRequest {}

/// Creates a new project.
#[utoipa::path]
#[put("/project")]
pub async fn create(
    req: HttpRequest,
    db: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    web::Json(details): web::Json<v67::ProjectCreate>,
) -> Result<(), CreateError> {
    // check that the user can make a project
    let (_, user) = get_user_from_headers(
        &req,
        &db,
        &redis,
        session_queue,
        Scopes::PROJECT_CREATE,
    )
    .await?;

    let limits = UserLimits::get_for_projects(&current_user, pool).await?;
    if limits.current >= limits.max {
        return Err(CreateError::LimitReached);
    }

    // check if the given details are valid

    v67::component_kinds_compatible(&details.component_kinds())
        .map_err(CreateError::IncompatibleComponents)?;

    details.validate()?;

    // check if this won't conflict with an existing project

    let slug_project_id_option = serde_json::from_value::<ProjectId>(
        serde_json::Value::String(details.base.slug.to_lowercase()),
    )
    .expect("should be able to deserialize");

    let mut txn = db
        .begin()
        .await
        .wrap_internal_err("failed to begin transaction")?;

    let project_id: ProjectId = models::generate_project_id(&mut txn)
        .await
        .wrap_internal_err("failed to generate project ID")?
        .into();

    Ok(())
}

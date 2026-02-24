use crate::util::guards::admin_key_guard;
use crate::{
    routes::ApiError,
    search::{SearchBackend, TasksCancelFilter},
};
use actix_web::{delete, get, web};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(tasks).service(tasks_cancel);
}

#[utoipa::path]
#[get("tasks", guard = "admin_key_guard")]
pub async fn tasks(
    search: web::Data<dyn SearchBackend>,
) -> Result<web::Json<serde_json::Value>, ApiError> {
    Ok(web::Json(search.tasks().await.map_err(ApiError::Internal)?))
}

#[utoipa::path]
#[delete("tasks", guard = "admin_key_guard")]
pub async fn tasks_cancel(
    search: web::Data<dyn SearchBackend>,
    body: web::Json<TasksCancelFilter>,
) -> Result<(), ApiError> {
    search
        .tasks_cancel(&body)
        .await
        .map_err(ApiError::Internal)?;
    Ok(())
}

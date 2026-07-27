use crate::database::PgPool;
use crate::database::models::DBUserId;
use crate::database::models::blocked_user_item::DBBlockedUser;
use crate::routes::ApiError;
use crate::util::guards::admin_key_guard;
use actix_web::{get, web};
use ariadne::ids::base62_impl::parse_base62;
use serde::Serialize;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(block_status);
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct BlockStatus {
    pub blocked: bool,
}

/// Check whether one user has blocked another.
#[utoipa::path(tag = "blocked_users", responses((status = OK, body = BlockStatus)))]
#[get("/block/{user_id}/{target_id}", guard = "admin_key_guard")]
pub async fn block_status(
    info: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
) -> Result<web::Json<BlockStatus>, ApiError> {
    let (user_id, target_id) = info.into_inner();

    let user_id =
        DBUserId(parse_base62(&user_id).map_err(|_| {
            ApiError::InvalidInput("invalid user_id".to_string())
        })? as i64);
    let target_id =
        DBUserId(parse_base62(&target_id).map_err(|_| {
            ApiError::InvalidInput("invalid target_id".to_string())
        })? as i64);

    let blocked =
        DBBlockedUser::is_blocked(user_id, target_id, &**pool).await?;

    Ok(web::Json(BlockStatus { blocked }))
}

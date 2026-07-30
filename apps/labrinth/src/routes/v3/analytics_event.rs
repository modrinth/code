use actix_web::{get, web};
use xredis::RedisPool;

use crate::{
    database::{PgPool, models::DBAnalyticsEvent},
    models::v3::analytics_event::AnalyticsEvent,
    routes::ApiError,
    util::error::Context,
};

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(analytics_events_get);
}

/// List analytics events.  
#[utoipa::path(
	context_path = "/v3/analytics-event",
	tag = "v3 analytics", responses((status = OK, body = Vec<AnalyticsEvent>))
)]
#[get("")]
pub async fn analytics_events_get(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<web::Json<Vec<AnalyticsEvent>>, ApiError> {
    let events = DBAnalyticsEvent::get_all(&**pool, &redis)
        .await
        .wrap_internal_err("failed to fetch analytics events")?
        .into_iter()
        .map(AnalyticsEvent::from)
        .collect();

    Ok(web::Json(events))
}

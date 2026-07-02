use actix_web::{HttpRequest, delete, get, patch, post, web};
use chrono::{DateTime, Utc};
use eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::{
    auth::get_user_from_headers,
    database::{
        PgPool,
        models::{
            DBAnalyticsEvent, DBAnalyticsEventId, generate_analytics_event_id,
        },
        redis::RedisPool,
    },
    models::{
        ids::AnalyticsEventId,
        pats::Scopes,
        v3::analytics_event::{AnalyticsEvent, AnalyticsEventMeta},
    },
    queue::session::AuthQueue,
    routes::ApiError,
    util::error::Context,
};

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(analytics_events_get)
        .service(analytics_event_create)
        .service(analytics_event_edit)
        .service(analytics_event_delete);
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AnalyticsEventUpsert {
    #[serde(flatten)]
    pub meta: AnalyticsEventMeta,
    pub starts: DateTime<Utc>,
    pub ends: DateTime<Utc>,
}

/// List analytics events.  
#[utoipa::path(tag = "v3 analytics", responses((status = OK, body = Vec<AnalyticsEvent>)))]
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

/// Create an analytics event.  
#[utoipa::path(tag = "v3 analytics", responses((status = OK, body = AnalyticsEvent)))]
#[post("")]
pub async fn analytics_event_create(
    req: HttpRequest,
    event: web::Json<AnalyticsEventUpsert>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<AnalyticsEvent>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::empty(),
    )
    .await?
    .1;

    if !user.role.is_admin() {
        return Err(ApiError::Auth(eyre!(
            "you do not have permission to manage analytics events"
        )));
    }

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("failed to begin transaction")?;
    let id = generate_analytics_event_id(&mut transaction)
        .await
        .wrap_internal_err("failed to generate analytics event ID")?;

    let event = DBAnalyticsEvent {
        id,
        meta: event.meta.clone(),
        starts: event.starts,
        ends: event.ends,
    };
    event
        .insert(&mut transaction)
        .await
        .wrap_internal_err("failed to insert analytics event")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;
    DBAnalyticsEvent::clear_cache(&redis)
        .await
        .wrap_internal_err("failed to clear analytics event cache")?;

    Ok(web::Json(event.into()))
}

/// Update an analytics event.  
#[utoipa::path(tag = "v3 analytics", responses((status = OK, body = AnalyticsEvent)))]
#[patch("/{id}")]
pub async fn analytics_event_edit(
    req: HttpRequest,
    id: web::Path<(AnalyticsEventId,)>,
    event: web::Json<AnalyticsEventUpsert>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<AnalyticsEvent>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::empty(),
    )
    .await?
    .1;

    if !user.role.is_admin() {
        return Err(ApiError::Auth(eyre!(
            "you do not have permission to manage analytics events"
        )));
    }

    let event = DBAnalyticsEvent {
        id: DBAnalyticsEventId::from(id.into_inner().0),
        meta: event.meta.clone(),
        starts: event.starts,
        ends: event.ends,
    };

    let updated = event
        .update(&**pool)
        .await
        .wrap_internal_err("failed to update analytics event")?;
    if !updated {
        return Err(ApiError::NotFound);
    }
    DBAnalyticsEvent::clear_cache(&redis)
        .await
        .wrap_internal_err("failed to clear analytics event cache")?;

    Ok(web::Json(event.into()))
}

/// Delete an analytics event.  
#[utoipa::path(tag = "v3 analytics", responses((status = NO_CONTENT)))]
#[delete("/{id}")]
pub async fn analytics_event_delete(
    req: HttpRequest,
    id: web::Path<(AnalyticsEventId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::empty(),
    )
    .await?
    .1;

    if !user.role.is_admin() {
        return Err(ApiError::Auth(eyre!(
            "you do not have permission to manage analytics events"
        )));
    }

    let deleted = DBAnalyticsEvent::remove(
        DBAnalyticsEventId::from(id.into_inner().0),
        &**pool,
    )
    .await
    .wrap_internal_err("failed to delete analytics event")?;
    if !deleted {
        return Err(ApiError::NotFound);
    }
    DBAnalyticsEvent::clear_cache(&redis)
        .await
        .wrap_internal_err("failed to clear analytics event cache")?;

    Ok(())
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    analytics_events_get,
    analytics_event_create,
    analytics_event_edit,
    analytics_event_delete,
))]
#[allow(dead_code)]
pub(crate) struct RouteDoc;

use actix_web::{HttpRequest, delete, patch, post, web};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

use crate::{
    auth::get_user_from_headers,
    database::{
        PgPool,
        models::{
            DBAnalyticsEvent, DBAnalyticsEventId, generate_analytics_event_id,
        },
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
    cfg.service(analytics_event_create)
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

/// Create an analytics event.
#[utoipa::path(
	context_path = "/analytics-event",
	tag = "analytics events", responses((status = OK, body = AnalyticsEvent))
)]
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
    .await
    .auth_context("authenticating API request")?;

    if !user.scopes.contains(Scopes::ADMIN_WRITE) {
        return Err(ApiError::Auth(anyhow!(
            "you do not have permission to manage analytics events"
        )));
    }

    let mut transaction = pool
        .begin()
        .await
        .internal_context("failed to begin transaction")?;
    let id = generate_analytics_event_id(&mut transaction)
        .await
        .internal_context("failed to generate analytics event ID")?;

    let event = DBAnalyticsEvent {
        id,
        meta: event.meta.clone(),
        starts: event.starts,
        ends: event.ends,
    };
    event
        .insert(&mut transaction)
        .await
        .internal_context("failed to insert analytics event")?;

    transaction
        .commit()
        .await
        .internal_context("failed to commit transaction")?;
    DBAnalyticsEvent::clear_cache(&redis)
        .await
        .internal_context("failed to clear analytics event cache")?;

    Ok(web::Json(event.into()))
}

/// Update an analytics event.
#[utoipa::path(
	context_path = "/analytics-event",
	tag = "analytics events", responses((status = OK, body = AnalyticsEvent))
)]
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
    .await
    .auth_context("authenticating API request")?;

    if !user.scopes.contains(Scopes::ADMIN_WRITE) {
        return Err(ApiError::Auth(anyhow!(
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
        .internal_context("failed to update analytics event")?;
    if !updated {
        return Err(ApiError::NotFound(anyhow!("resource not found")));
    }
    DBAnalyticsEvent::clear_cache(&redis)
        .await
        .internal_context("failed to clear analytics event cache")?;

    Ok(web::Json(event.into()))
}

/// Delete an analytics event.
#[utoipa::path(
	context_path = "/analytics-event",
	tag = "analytics events", responses((status = NO_CONTENT))
)]
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
    .await
    .auth_context("authenticating API request")?;

    if !user.scopes.contains(Scopes::ADMIN_WRITE) {
        return Err(ApiError::Auth(anyhow!(
            "you do not have permission to manage analytics events"
        )));
    }

    let deleted = DBAnalyticsEvent::remove(
        DBAnalyticsEventId::from(id.into_inner().0),
        &**pool,
    )
    .await
    .internal_context("failed to delete analytics event")?;
    if !deleted {
        return Err(ApiError::NotFound(anyhow!("resource not found")));
    }
    DBAnalyticsEvent::clear_cache(&redis)
        .await
        .internal_context("failed to clear analytics event cache")?;

    Ok(())
}

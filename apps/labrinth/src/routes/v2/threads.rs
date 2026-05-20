use std::sync::Arc;

use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::ids::{ThreadId, ThreadMessageId};
use crate::models::threads::{MessageBody, Thread};
use crate::models::v2::threads::LegacyThread;
use crate::queue::session::AuthQueue;
use crate::routes::{ApiError, v2_reroute, v3};
use actix_web::{HttpRequest, HttpResponse, delete, get, post, web};
use serde::Deserialize;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(
        utoipa_actix_web::scope("/thread")
            .service(thread_get)
            .service(thread_send_message),
    );
    cfg.service(utoipa_actix_web::scope("/message").service(message_delete));
    cfg.service(threads_get);
}

/// Get a thread by ID.
#[utoipa::path(
    get,
    operation_id = "getThread",
    params(("id" = ThreadId, Path, description = "The ID of the thread")),
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["THREAD_READ"]))
)]
#[get("/{id}")]
pub async fn thread_get(
    req: HttpRequest,
    info: web::Path<(ThreadId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::threads::thread_get(req, info, pool, redis, session_queue)
        .await
        .or_else(v2_reroute::flatten_404_error)
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ThreadIds {
    pub ids: String,
}

/// Get multiple threads by ID.
#[utoipa::path(
    get,
    operation_id = "getThreads",
    params(("ids" = String, Query, description = "The JSON array of thread IDs")),
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["THREAD_READ"]))
)]
#[get("/threads")]
pub async fn threads_get(
    req: HttpRequest,
    web::Query(ids): web::Query<ThreadIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::threads::threads_get(
        req,
        web::Query(v3::threads::ThreadIds { ids: ids.ids }),
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<Thread>>(response).await {
        Ok(threads) => {
            let threads = threads
                .into_iter()
                .map(LegacyThread::from)
                .collect::<Vec<_>>();
            Ok(HttpResponse::Ok().json(threads))
        }
        Err(response) => Ok(response),
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct NewThreadMessage {
    pub body: MessageBody,
}

/// Send a message to a thread.
#[utoipa::path(
    post,
    operation_id = "sendThreadMessage",
    params(("id" = ThreadId, Path, description = "The ID of the thread")),
    request_body = NewThreadMessage,
    responses(
        (status = 204, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error"),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["THREAD_WRITE"]))
)]
#[post("/{id}")]
pub async fn thread_send_message(
    req: HttpRequest,
    info: web::Path<(ThreadId,)>,
    pool: web::Data<PgPool>,
    new_message: web::Json<NewThreadMessage>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let new_message = new_message.into_inner();
    // Returns NoContent, so we don't need to convert the response
    v3::threads::thread_send_message(
        req,
        info,
        pool,
        web::Json(v3::threads::NewThreadMessage {
            body: new_message.body,
        }),
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

/// Delete a thread message by ID.
#[utoipa::path(
    delete,
    operation_id = "deleteThreadMessage",
    params(("id" = ThreadMessageId, Path, description = "The ID of the message")),
    responses(
        (status = 204, description = "Expected response to a valid request"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        ),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["THREAD_WRITE"]))
)]
#[delete("/{id}")]
pub async fn message_delete(
    req: HttpRequest,
    info: web::Path<(ThreadMessageId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so we don't need to convert the response
    v3::threads::message_delete(
        req,
        info,
        pool,
        redis,
        session_queue,
        file_host,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

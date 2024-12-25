use std::sync::Arc;

use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::ids::ThreadMessageId;
use crate::models::threads::{MessageBody, Thread, ThreadId};
use crate::models::v2::threads::LegacyThread;
use crate::queue::session::AuthQueue;
use crate::routes::{v2_reroute, v3, ApiError};
use ntex::web::{self, delete, get, post, HttpRequest, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("thread")
            .service(thread_get)
            .service(thread_send_message),
    );
    cfg.service(web::scope("message").service(message_delete));
    cfg.service(threads_get);
}

#[get("{id}")]
pub async fn thread_get(
    req: HttpRequest,
    info: web::types::Path<(ThreadId,)>,
    pool: web::types::State<PgPool>,
    redis: web::types::State<RedisPool>,
    session_queue: web::types::State<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::threads::thread_get(req, info, pool, redis, session_queue)
        .await
        .or_else(v2_reroute::flatten_404_error)
}

#[derive(Deserialize)]
pub struct ThreadIds {
    pub ids: String,
}

#[get("threads")]
pub async fn threads_get(
    req: HttpRequest,
    web::types::Query(ids): web::types::Query<ThreadIds>,
    pool: web::types::State<PgPool>,
    redis: web::types::State<RedisPool>,
    session_queue: web::types::State<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::threads::threads_get(
        req,
        web::types::Query(v3::threads::ThreadIds { ids: ids.ids }),
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
            Ok(HttpResponse::Ok().json(&threads))
        }
        Err(response) => Ok(response),
    }
}

#[derive(Deserialize)]
pub struct NewThreadMessage {
    pub body: MessageBody,
}

#[post("{id}")]
pub async fn thread_send_message(
    req: HttpRequest,
    info: web::types::Path<(ThreadId,)>,
    pool: web::types::State<PgPool>,
    new_message: web::types::Json<NewThreadMessage>,
    redis: web::types::State<RedisPool>,
    session_queue: web::types::State<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let new_message = new_message.into_inner();
    // Returns NoContent, so we don't need to convert the response
    v3::threads::thread_send_message(
        req,
        info,
        pool,
        web::types::Json(v3::threads::NewThreadMessage {
            body: new_message.body,
        }),
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

#[delete("{id}")]
pub async fn message_delete(
    req: HttpRequest,
    info: web::types::Path<(ThreadMessageId,)>,
    pool: web::types::State<PgPool>,
    redis: web::types::State<RedisPool>,
    session_queue: web::types::State<AuthQueue>,
    file_host: web::types::State<Arc<dyn FileHost + Send + Sync>>,
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

use std::sync::Arc;

use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::ids::{ThreadMessageId, VersionId};
use crate::models::reports::ReportId;
use crate::queue::session::AuthQueue;
use crate::routes::{v3, ApiError};
use actix_web::{post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(images_add);
}

#[derive(Serialize, Deserialize)]
pub struct ImageUpload {
    pub ext: String,

    // Context must be an allowed context
    // currently: project, version, thread_message, report
    pub context: String,

    // Optional context id to associate with
    pub project_id: Option<String>, // allow slug or id
    pub version_id: Option<VersionId>,
    pub thread_message_id: Option<ThreadMessageId>,
    pub report_id: Option<ReportId>,
}

#[post("image")]
pub async fn images_add(
    req: HttpRequest,
    web::Query(data): web::Query<ImageUpload>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    payload: web::Payload,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::images::images_add(
        req,
        web::Query(v3::images::ImageUpload {
            ext: data.ext,
            context: data.context,
            project_id: data.project_id,
            version_id: data.version_id,
            thread_message_id: data.thread_message_id,
            report_id: data.report_id,
        }),
        file_host,
        payload,
        pool,
        redis,
        session_queue,
    )
    .await
}

use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::collections::CollectionStatus;
use crate::queue::session::AuthQueue;
use crate::routes::v3::project_creation::CreateError;
use crate::routes::{v3, ApiError};
use actix_web::web::Data;
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use validator::Validate;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(collections_get);
    cfg.service(collection_create);
    cfg.service(
        web::scope("collection")
            .service(collection_get)
            .service(collection_delete)
            .service(collection_edit)
            .service(collection_icon_edit)
            .service(delete_collection_icon),
    );
}

#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct CollectionCreateData {
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    /// The title or name of the project.
    pub title: String,
    #[validate(length(min = 3, max = 255))]
    /// A short description of the collection.
    pub description: String,
    #[validate(length(max = 32))]
    #[serde(default = "Vec::new")]
    /// A list of initial projects to use with the created collection
    pub projects: Vec<String>,
}

#[post("collection")]
pub async fn collection_create(
    req: HttpRequest,
    collection_create_data: web::Json<CollectionCreateData>,
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, CreateError> {
    let collection_create_data = collection_create_data.into_inner();
    v3::collections::collection_create(
        req,
        web::Json(v3::collections::CollectionCreateData {
            title: collection_create_data.title,
            description: collection_create_data.description,
            projects: collection_create_data.projects,
        }),
        client,
        redis,
        session_queue,
    )
    .await
}

#[derive(Serialize, Deserialize)]
pub struct CollectionIds {
    pub ids: String,
}
#[get("collections")]
pub async fn collections_get(
    req: HttpRequest,
    web::Query(ids): web::Query<CollectionIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::collections::collections_get(
        req,
        web::Query(v3::collections::CollectionIds { ids: ids.ids }),
        pool,
        redis,
        session_queue,
    )
    .await
}

#[get("{id}")]
pub async fn collection_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::collections::collection_get(req, info, pool, redis, session_queue).await
}

#[derive(Deserialize, Validate)]
pub struct EditCollection {
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    pub title: Option<String>,
    #[validate(length(min = 3, max = 256))]
    pub description: Option<String>,
    pub status: Option<CollectionStatus>,
    #[validate(length(max = 64))]
    pub new_projects: Option<Vec<String>>,
}

#[patch("{id}")]
pub async fn collection_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    new_collection: web::Json<EditCollection>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let new_collection = new_collection.into_inner();
    v3::collections::collection_edit(
        req,
        info,
        pool,
        web::Json(v3::collections::EditCollection {
            title: new_collection.title,
            description: new_collection.description,
            status: new_collection.status,
            new_projects: new_collection.new_projects,
        }),
        redis,
        session_queue,
    )
    .await
}

#[derive(Serialize, Deserialize)]
pub struct Extension {
    pub ext: String,
}

#[patch("{id}/icon")]
#[allow(clippy::too_many_arguments)]
pub async fn collection_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::collections::collection_icon_edit(
        web::Query(v3::collections::Extension { ext: ext.ext }),
        req,
        info,
        pool,
        redis,
        file_host,
        payload,
        session_queue,
    )
    .await
}

#[delete("{id}/icon")]
pub async fn delete_collection_icon(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::collections::delete_collection_icon(req, info, pool, redis, file_host, session_queue).await
}

#[delete("{id}")]
pub async fn collection_delete(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::collections::collection_delete(req, info, pool, redis, session_queue).await
}

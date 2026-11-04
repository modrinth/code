use actix_web::{HttpRequest, get, put, web};
use serde::Deserialize;
use std::collections::HashSet;
use xredis::RedisPool;

use crate::{
    auth::check_is_moderator_from_headers,
    database::{
        PgPool, ReadOnlyPgPool,
        models::{
            CopyrightArchiveFile, CopyrightArchiveMeta, DBCopyrightArchive,
            DBCopyrightArchiveId, generate_copyright_archive_id,
        },
    },
    models::pats::Scopes,
    queue::session::AuthQueue,
    routes::ApiError,
    util::error::Context as _,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list).service(create);
}

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    pub limit: u16,
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u16 {
    100
}

#[derive(Deserialize)]
pub struct CreateCopyrightArchive {
    #[serde(flatten)]
    pub meta: CopyrightArchiveMeta,
    pub files: Vec<CopyrightArchiveFile>,
}

#[get("")]
pub async fn list(
    req: HttpRequest,
    query: web::Query<ListQuery>,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<DBCopyrightArchive>>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let archives = sqlx::query!(
        r#"
        SELECT id AS "id: DBCopyrightArchiveId",
               meta AS "meta: sqlx::types::Json<CopyrightArchiveMeta>"
        FROM copyright_archives
        ORDER BY id
        LIMIT $1 OFFSET $2
        "#,
        i64::from(query.limit.min(100)),
        i64::from(query.offset),
    )
    .fetch_all(&**ro_pool)
    .await
    .wrap_internal_err("fetching copyright archives")?;

    Ok(web::Json(
        archives
            .into_iter()
            .map(|row| DBCopyrightArchive {
                id: row.id,
                meta: row.meta.0,
            })
            .collect(),
    ))
}

#[put("")]
pub async fn create(
    req: HttpRequest,
    body: web::Json<CreateCopyrightArchive>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<DBCopyrightArchive>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let body = body.into_inner();
    let files: HashSet<_> = body.files.into_iter().collect();

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting copyright archive transaction")?;
    let id = generate_copyright_archive_id(&mut transaction)
        .await
        .wrap_internal_err("generating copyright archive ID")?;
    let meta = sqlx::types::Json(body.meta);

    sqlx::query!(
        r#"
        INSERT INTO copyright_archives (id, meta)
        VALUES ($1, $2)
        "#,
        id as DBCopyrightArchiveId,
        &meta as _,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("inserting copyright archive")?;

    let (paths, hashes): (Vec<_>, Vec<_>) = files
        .into_iter()
        .map(|file| (file.path, file.sha512))
        .unzip();
    sqlx::query!(
        r#"
        INSERT INTO copyright_archive_files (archive_id, path, sha512)
        SELECT $1, path, sha512
        FROM UNNEST($2::text[], $3::bytea[]) AS files(path, sha512)
        "#,
        id as DBCopyrightArchiveId,
        &paths,
        &hashes,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("inserting copyright archive files")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("committing copyright archive transaction")?;

    Ok(web::Json(DBCopyrightArchive { id, meta: meta.0 }))
}

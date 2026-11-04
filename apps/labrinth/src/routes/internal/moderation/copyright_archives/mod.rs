use actix_web::{HttpRequest, get, patch, put, web};
use eyre::eyre;
use serde::Deserialize;
use std::collections::HashMap;
use xredis::RedisPool;

use crate::{
    auth::check_is_moderator_from_headers,
    database::{
        PgPool, PgTransaction, ReadOnlyPgPool,
        models::{
            CopyrightArchive, CopyrightArchiveFile, CopyrightArchiveMeta,
            DBCopyrightArchiveId, generate_copyright_archive_id,
        },
    },
    models::{ids::CopyrightArchiveId, pats::Scopes},
    queue::session::AuthQueue,
    routes::ApiError,
    util::error::Context as _,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list).service(create).service(replace);
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

#[get("/copyright-archives")]
pub async fn list(
    req: HttpRequest,
    query: web::Query<ListQuery>,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<CopyrightArchive>>, ApiError> {
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
    .fetch_all(&***ro_pool)
    .await
    .wrap_internal_err("fetching copyright archives")?;

    Ok(web::Json(
        archives
            .into_iter()
            .map(|row| CopyrightArchive {
                id: row.id.into(),
                meta: row.meta.0,
            })
            .collect(),
    ))
}

#[put("/copyright-archives")]
pub async fn create(
    req: HttpRequest,
    body: web::Json<CreateCopyrightArchive>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<CopyrightArchiveId>, ApiError> {
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

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting copyright archive transaction")?;
    let id = generate_copyright_archive_id(&mut transaction)
        .await
        .wrap_internal_err("generating copyright archive ID")?;
    replace_archive(id, body, &mut transaction).await?;

    transaction
        .commit()
        .await
        .wrap_internal_err("committing copyright archive transaction")?;

    Ok(web::Json(id.into()))
}

#[patch("/copyright-archive/{id}")]
pub async fn replace(
    req: HttpRequest,
    id: web::Path<CopyrightArchiveId>,
    body: web::Json<CreateCopyrightArchive>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let id = DBCopyrightArchiveId::from(id.into_inner());
    let body = body.into_inner();
    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting copyright archive transaction")?;

    sqlx::query!(
        r#"
        SELECT id FROM copyright_archives WHERE id = $1 FOR UPDATE
        "#,
        id as DBCopyrightArchiveId,
    )
    .fetch_optional(&mut transaction)
    .await
    .wrap_internal_err("locking copyright archive")?
    .wrap_not_found_err("copyright archive not found")?;

    replace_archive(id, body, &mut transaction).await?;

    transaction
        .commit()
        .await
        .wrap_internal_err("committing copyright archive transaction")?;

    Ok(())
}

async fn replace_archive(
    id: DBCopyrightArchiveId,
    body: CreateCopyrightArchive,
    transaction: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
    let mut paths = HashMap::new();
    for file in &body.files {
        if let Some(previous_hash) = paths.insert(&file.path, &file.sha512) {
            return Err(ApiError::Request(eyre!(
                "duplicate archive file path `{}` with SHA-512 hashes `{}` and `{}`",
                file.path,
                hex::encode(previous_hash),
                hex::encode(&file.sha512),
            )));
        }
    }

    sqlx::query!(
        r#"
        DELETE FROM copyright_archive_files WHERE archive_id = $1
        "#,
        id as DBCopyrightArchiveId,
    )
    .execute(&mut *transaction)
    .await
    .wrap_internal_err("deleting copyright archive files")?;

    let meta = sqlx::types::Json(body.meta);
    sqlx::query!(
        r#"
        INSERT INTO copyright_archives (id, meta)
        VALUES ($1, $2)
        ON CONFLICT (id) DO UPDATE SET meta = EXCLUDED.meta
        "#,
        id as DBCopyrightArchiveId,
        &meta as _,
    )
    .execute(&mut *transaction)
    .await
    .wrap_internal_err("replacing copyright archive")?;

    let (paths, hashes): (Vec<_>, Vec<_>) = body
        .files
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
    .execute(transaction)
    .await
    .wrap_internal_err("inserting copyright archive files")?;

    Ok(())
}

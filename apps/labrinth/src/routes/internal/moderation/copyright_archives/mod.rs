use actix_web::{HttpRequest, delete, get, patch, put, web};
use eyre::eyre;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
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
    cfg.service(list)
        .service(create)
        .service(replace)
        .service(delete);
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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateCopyrightArchive {
    #[serde(flatten)]
    pub meta: CopyrightArchiveMeta,
    pub files: Vec<CopyrightArchiveFile>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ReplaceCopyrightArchive {
    pub id: CopyrightArchiveId,
    #[serde(flatten)]
    pub archive: CreateCopyrightArchive,
}

/// Lists copyright archives in ID order. Requires moderator access.
#[utoipa::path(
    context_path = "/moderation",
    tag = "moderation",
    security(("bearer_auth" = [])),
    params(
        ("limit" = Option<u16>, Query, description = "Maximum number of archives to return; defaults to 100 and is capped at 100"),
        ("offset" = Option<u32>, Query, description = "Number of archives to skip; defaults to 0"),
    ),
    responses((status = OK, body = Vec<CopyrightArchive>))
)]
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

/// Creates copyright archives atomically and returns their base62 IDs in request order.
/// Requires moderator access. Duplicate file paths cause the entire request to fail.
#[utoipa::path(
    context_path = "/moderation",
    tag = "moderation",
    security(("bearer_auth" = [])),
    request_body = Vec<CreateCopyrightArchive>,
    responses(
        (status = OK, body = Vec<CopyrightArchiveId>),
        (status = BAD_REQUEST, description = "An archive contains duplicate file paths"),
    )
)]
#[put("/copyright-archives")]
pub async fn create(
    req: HttpRequest,
    body: web::Json<Vec<CreateCopyrightArchive>>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<CopyrightArchiveId>>, ApiError> {
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
    let mut ids = Vec::with_capacity(body.len());
    for archive in body {
        let id = generate_copyright_archive_id(&mut transaction)
            .await
            .wrap_internal_err("generating copyright archive ID")?;
        replace_archive(id, archive, &mut transaction).await?;
        ids.push(id.into());
    }

    transaction
        .commit()
        .await
        .wrap_internal_err("committing copyright archive transaction")?;

    Ok(web::Json(ids))
}

/// Atomically replaces the complete metadata and file lists of existing archives.
/// Requires moderator access. Duplicate IDs, duplicate file paths, or missing archives
/// cause the entire request to fail.
#[utoipa::path(
    context_path = "/moderation",
    tag = "moderation",
    security(("bearer_auth" = [])),
    request_body = Vec<ReplaceCopyrightArchive>,
    responses(
        (status = OK, description = "All archives replaced"),
        (status = BAD_REQUEST, description = "Duplicate archive IDs or file paths"),
        (status = NOT_FOUND, description = "One or more archives do not exist; the error lists their base62 IDs"),
    )
)]
#[patch("/copyright-archives")]
pub async fn replace(
    req: HttpRequest,
    body: web::Json<Vec<ReplaceCopyrightArchive>>,
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

    let body = body.into_inner();
    let mut seen = HashSet::new();
    for archive in &body {
        if !seen.insert(archive.id) {
            return Err(ApiError::Request(eyre!(
                "duplicate copyright archive ID `{}`",
                archive.id,
            )));
        }
    }
    let ids: Vec<_> = body
        .iter()
        .map(|archive| DBCopyrightArchiveId::from(archive.id).0)
        .collect();
    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting copyright archive transaction")?;

    let archives = sqlx::query!(
        r#"
        SELECT id FROM copyright_archives
        WHERE id = ANY($1)
        ORDER BY id
        FOR UPDATE
        "#,
        &ids,
    )
    .fetch_all(&mut transaction)
    .await
    .wrap_internal_err("locking copyright archives")?;

    if archives.len() != ids.len() {
        let found: HashSet<_> =
            archives.iter().map(|archive| archive.id).collect();
        let missing: Vec<_> = body
            .iter()
            .filter(|archive| {
                !found.contains(&DBCopyrightArchiveId::from(archive.id).0)
            })
            .map(|archive| format!("`{}`", archive.id))
            .collect();
        return Err(ApiError::NotFound(eyre!(
            "copyright archives not found: {}",
            missing.join(", "),
        )));
    }

    for archive in body {
        replace_archive(archive.id.into(), archive.archive, &mut transaction)
            .await?;
    }

    transaction
        .commit()
        .await
        .wrap_internal_err("committing copyright archive transaction")?;

    Ok(())
}

/// Atomically deletes copyright archives and their files by base62 ID.
/// Requires moderator access. IDs that do not exist are ignored.
#[utoipa::path(
    context_path = "/moderation",
    tag = "moderation",
    security(("bearer_auth" = [])),
    request_body = Vec<CopyrightArchiveId>,
    responses((status = OK, description = "Archives deleted"))
)]
#[delete("/copyright-archives")]
pub async fn delete(
    req: HttpRequest,
    body: web::Json<Vec<CopyrightArchiveId>>,
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

    let ids: Vec<_> = body
        .into_inner()
        .into_iter()
        .map(|id| DBCopyrightArchiveId::from(id).0)
        .collect();
    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting copyright archive transaction")?;

    sqlx::query!(
        r#"
        SELECT id FROM copyright_archives
        WHERE id = ANY($1)
        ORDER BY id
        FOR UPDATE
        "#,
        &ids,
    )
    .fetch_all(&mut transaction)
    .await
    .wrap_internal_err("locking copyright archives")?;

    sqlx::query!(
        r#"
        DELETE FROM copyright_archives WHERE id = ANY($1)
        "#,
        &ids,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("deleting copyright archives")?;

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

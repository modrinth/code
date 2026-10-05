use super::ApiError;
use crate::auth::checks::{
    filter_visible_version_ids, filter_visible_versions, is_visible_version,
};
use crate::auth::{filter_visible_projects, get_user_from_headers};
use crate::database::PgPool;
use crate::database::ReadOnlyPgPool;
use crate::models::ids::VersionId;
use crate::models::pats::Scopes;
use crate::models::projects::{ProjectStatus, VersionStatus, VersionType};
use crate::models::teams::ProjectPermissions;
use crate::queue::session::AuthQueue;
use crate::routes::internal::delphi;
use crate::routes::{FileHash, HashAlgorithm};
use crate::util::error::ApiContext as _;
use crate::util::error::Context;
use crate::{database, models};
use actix_web::{HttpRequest, HttpResponse, delete, get, post, web};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use xredis::RedisPool;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_version_from_hash_route)
        .service(get_update_from_hash_route)
        .service(get_projects_from_hashes_route)
        .service(delete_file_route)
        .service(download_version_route)
        .service(update_files_route)
        .service(update_files_many_route)
        .service(update_individual_files_route)
        .service(get_versions_from_hashes_route);
}

/// Get version metadata by file hash.
#[utoipa::path(
	tag = "version files",
    get,
    operation_id = "v3VersionFromHash",
    params(
        ("version_id" = String, Path, description = "The hexadecimal file hash"),
        ("algorithm" = Option<String>, Query, description = "Hash algorithm to use (sha1 or sha512)"),
        ("version_id" = Option<VersionId>, Query, description = "Optional version ID when hash maps to multiple files")
    ),
    responses(
        (status = 200, description = "Expected response to a valid request", body = models::projects::Version),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[get("/version_file/{version_id}")]
pub async fn get_version_from_hash_route(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    hash_query: web::Query<HashQuery>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    get_version_from_hash(req, info, pool, redis, hash_query, session_queue)
        .await
}

pub async fn get_version_from_hash(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    hash_query: web::Query<HashQuery>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::VERSION_READ,
    )
    .await
    .map(|x| x.1)
    .ok();
    let hash = info.into_inner().0.to_lowercase();
    let algorithm = hash_query.algorithm.clone().unwrap_or_else(|| {
        default_algorithm_from_hashes(std::slice::from_ref(&hash))
    });
    let file = database::models::DBVersion::get_file_from_hash(
        algorithm,
        hash,
        hash_query.version_id.map(|x| x.into()),
        &**pool,
        &redis,
    )
    .await
    .wrap_internal_err("fetching version from database")?;
    if let Some(file) = file {
        let version =
            database::models::DBVersion::get(file.version_id, &**pool, &redis)
                .await
                .wrap_internal_err("fetching version from database")?;
        if let Some(version) = version {
            if !is_visible_version(&version.inner, &user_option, &pool, &redis)
                .await
                .wrap_api_err("checking version visibility")?
            {
                return Err(ApiError::NotFound(eyre::eyre!(
                    "resource not found"
                )));
            }

            Ok(HttpResponse::Ok()
                .json(models::projects::Version::from(version)))
        } else {
            Err(ApiError::NotFound(eyre::eyre!("resource not found")))
        }
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct HashQuery {
    pub algorithm: Option<String>, // Defaults to calculation based on size of hash
    pub version_id: Option<VersionId>,
}

// Calculates whether or not to use sha1 or sha512 based on the size of the hash
pub fn default_algorithm_from_hashes(hashes: &[String]) -> String {
    // Gets first hash, optionally
    let empty_string = "".into();
    let hash = hashes.first().unwrap_or(&empty_string);
    let hash_len = hash.len();
    // Sha1 = 40 characters
    // Sha512 = 128 characters
    // Favour sha1 as default, unless the hash is longer or equal to 128 characters
    if hash_len >= 128 {
        return "sha512".into();
    }
    "sha1".into()
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateData {
    pub loaders: Option<Vec<String>>,
    pub version_types: Option<Vec<VersionType>>,
    /*
       Loader fields to filter with:
       "game_versions": ["1.16.5", "1.17"]

       Returns if it matches any of the values
    */
    pub loader_fields: Option<HashMap<String, Vec<serde_json::Value>>>,
}

/// Get the latest matching version by file hash.
#[utoipa::path(
	tag = "version files",
    post,
    operation_id = "v3UpdateFromHash",
    params(
        ("version_id" = String, Path, description = "The hexadecimal file hash"),
        ("algorithm" = Option<String>, Query, description = "Hash algorithm to use (sha1 or sha512)"),
        ("version_id" = Option<VersionId>, Query, description = "Optional version ID when hash maps to multiple files")
    ),
    request_body = UpdateData,
    responses(
        (status = 200, description = "Expected response to a valid request", body = models::projects::Version),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[post("/version_file/{version_id}/update")]
pub async fn get_update_from_hash_route(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    hash_query: web::Query<HashQuery>,
    update_data: web::Json<UpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    get_update_from_hash(
        req,
        info,
        pool,
        redis,
        hash_query,
        update_data,
        session_queue,
    )
    .await
}

pub async fn get_update_from_hash(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    hash_query: web::Query<HashQuery>,
    update_data: web::Json<UpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user_option = get_user_from_headers(
        &req,
        &***pool,
        &redis,
        &session_queue,
        Scopes::VERSION_READ,
    )
    .await
    .map(|x| x.1)
    .ok();
    let hash = info.into_inner().0.to_lowercase();
    if let Some(file) = database::models::DBVersion::get_file_from_hash(
        hash_query.algorithm.clone().unwrap_or_else(|| {
            default_algorithm_from_hashes(std::slice::from_ref(&hash))
        }),
        hash.clone(),
        hash_query.version_id.map(|x| x.into()),
        &***pool,
        &redis,
    )
    .await
    .wrap_internal_err("querying database for `get_update_from_hash`")?
    {
        let latest = get_latest_matching_versions(
            &[LatestVersionFilter {
                hash: &hash,
                project_id: file.project_id.0,
                loaders: update_data.loaders.as_deref(),
                version_types: update_data.version_types.as_deref(),
                loader_fields: update_data.loader_fields.as_ref(),
            }],
            &pool,
        )
        .await
        .wrap_api_err("fetching latest matching version")?;

        if let Some(version_id) = latest.into_values().next()
            && let Some(first) =
                database::models::DBVersion::get(version_id, &***pool, &redis)
                    .await
                    .wrap_internal_err("fetching version from database")?
        {
            if !is_visible_version(&first.inner, &user_option, &pool, &redis)
                .await
                .wrap_api_err("checking version visibility")?
            {
                return Err(ApiError::NotFound(eyre::eyre!(
                    "resource not found"
                )));
            }

            return Ok(
                HttpResponse::Ok().json(models::projects::Version::from(first))
            );
        }
    }
    Err(ApiError::NotFound(eyre::eyre!("resource not found")))
}

// Requests above with multiple versions below
#[derive(Deserialize, utoipa::ToSchema)]
pub struct FileHashes {
    /// Hash algorithm to use (sha1 or sha512)
    #[schema(value_type = Option<HashAlgorithm>)]
    pub algorithm: Option<String>, // Defaults to calculation based on size of hash
    #[schema(value_type = Vec<FileHash>)]
    pub hashes: Vec<String>,
}

/// Get versions by file hashes.
#[utoipa::path(
	tag = "version files",
    post,
    operation_id = "v3VersionsFromHashes",
    request_body = FileHashes,
    responses(
        (status = 200, description = "Expected response to a valid request", body = HashMap<String, models::projects::Version>),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[post("/version_files")]
pub async fn get_versions_from_hashes_route(
    req: HttpRequest,
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    file_data: web::Json<FileHashes>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    get_versions_from_hashes(req, pool, redis, file_data, session_queue).await
}

pub async fn get_versions_from_hashes(
    req: HttpRequest,
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    file_data: web::Json<FileHashes>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user_option = get_user_from_headers(
        &req,
        &***pool,
        &redis,
        &session_queue,
        Scopes::VERSION_READ,
    )
    .await
    .map(|x| x.1)
    .ok();

    let algorithm = file_data
        .algorithm
        .clone()
        .unwrap_or_else(|| default_algorithm_from_hashes(&file_data.hashes));

    let files = database::models::DBVersion::get_files_from_hash(
        algorithm.clone(),
        &file_data.hashes,
        &***pool,
        &redis,
    )
    .await
    .wrap_internal_err("fetching versions from database")?;

    let version_ids = files.iter().map(|x| x.version_id).collect::<Vec<_>>();
    let versions_data = filter_visible_versions(
        database::models::DBVersion::get_many(&version_ids, &***pool, &redis)
            .await
            .wrap_internal_err("fetching versions from database")?,
        &user_option,
        &pool,
        pool.as_ref(),
        &redis,
    )
    .await
    .wrap_api_err("filtering visible versions")?;

    let mut response = HashMap::new();

    for version in versions_data {
        for file in files.iter().filter(|x| x.version_id == version.id.into()) {
            if let Some(hash) = file.hashes.get(&algorithm) {
                response.insert(hash.clone(), version.clone());
            }
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

/// Get projects by file hashes.
#[utoipa::path(
	tag = "version files",
    post,
    operation_id = "v3ProjectsFromHashes",
    request_body = FileHashes,
    responses(
        (status = 200, description = "Expected response to a valid request", body = HashMap<String, models::projects::Project>),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[post("/version_file/project")]
pub async fn get_projects_from_hashes_route(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_data: web::Json<FileHashes>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    get_projects_from_hashes(req, pool, redis, file_data, session_queue).await
}

pub async fn get_projects_from_hashes(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_data: web::Json<FileHashes>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ | Scopes::VERSION_READ,
    )
    .await
    .map(|x| x.1)
    .ok();

    let algorithm = file_data
        .algorithm
        .clone()
        .unwrap_or_else(|| default_algorithm_from_hashes(&file_data.hashes));
    let files = database::models::DBVersion::get_files_from_hash(
        algorithm.clone(),
        &file_data.hashes,
        &**pool,
        &redis,
    )
    .await
    .wrap_internal_err("fetching versions from database")?;

    let project_ids = files.iter().map(|x| x.project_id).collect::<Vec<_>>();

    let projects_data = filter_visible_projects(
        database::models::DBProject::get_many_ids(
            &project_ids,
            &**pool,
            &redis,
        )
        .await
        .wrap_internal_err("fetching projects for visibility filtering")?,
        &user_option,
        &pool,
        false,
    )
    .await
    .wrap_api_err("filtering visible projects")?;

    let mut response = HashMap::new();

    for project in projects_data {
        for file in files.iter().filter(|x| x.project_id == project.id.into()) {
            if let Some(hash) = file.hashes.get(&algorithm) {
                response.insert(hash.clone(), project.clone());
            }
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ManyUpdateData {
    pub algorithm: Option<String>, // Defaults to calculation based on size of hash
    pub hashes: Vec<String>,
    pub loaders: Option<Vec<String>>,
    pub game_versions: Option<Vec<String>>,
    pub version_types: Option<Vec<VersionType>>,
}

/// Get latest matching versions by file hashes.
#[utoipa::path(
	tag = "version files",
    post,
    operation_id = "v3UpdateFilesMany",
    request_body = ManyUpdateData,
    responses(
        (status = 200, description = "Expected response to a valid request", body = HashMap<String, Vec<models::projects::Version>>)
    )
)]
#[post("/version_files/update_many")]
pub async fn update_files_many_route(
    req: HttpRequest,
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    update_data: web::Json<ManyUpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<HashMap<String, Vec<models::projects::Version>>>, ApiError>
{
    update_files_many(req, pool, redis, update_data, session_queue).await
}

pub async fn update_files_many(
    req: HttpRequest,
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    update_data: web::Json<ManyUpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<HashMap<String, Vec<models::projects::Version>>>, ApiError>
{
    update_files_internal(req, pool, redis, update_data, session_queue)
        .await
        .map(web::Json)
}

// DEPRECATED - use `update_files_many` instead
//
// This returns a `HashMap<String, Version>` where the key is the file hash.
// But one file hash can have multiple versions associated with it.
// So you can end up in a situation where:
// - file with hash H is linked to versions V1, V2
// - user downloads mod with file hash H
// - every time the app checks for updates:
//   - it asks the backend, what is the version of H?
//   - backend says V1
//   - the app asks, is there a compatible version newer than V1?
//   - backend says, yes, V2
// - user updates to V2, but it's the same file, so it's the same hash H,
//   and the update button stays
//
// By using `update_files_many`, we can have the app know that both V1 and V2
// are linked to H
//
// This endpoint is kept for backwards compat, since it still works in 99% of
// cases where H only maps to a single version, and for older clients. This
// endpoint will only take the first version for each file hash.
/// Get the latest matching version by file hash.
#[utoipa::path(
	tag = "version files",
    post,
    operation_id = "v3UpdateFiles",
    request_body = ManyUpdateData,
    responses(
        (status = 200, description = "Expected response to a valid request", body = HashMap<String, models::projects::Version>)
    )
)]
#[post("/version_files/update")]
pub async fn update_files_route(
    req: HttpRequest,
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    update_data: web::Json<ManyUpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<HashMap<String, models::projects::Version>>, ApiError> {
    update_files(req, pool, redis, update_data, session_queue).await
}

pub async fn update_files(
    req: HttpRequest,
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    update_data: web::Json<ManyUpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<HashMap<String, models::projects::Version>>, ApiError> {
    let file_hashes_to_versions =
        update_files_internal(req, pool, redis, update_data, session_queue)
            .await
            .wrap_api_err("updating files internal")?;
    let resp = file_hashes_to_versions
        .into_iter()
        .filter_map(|(hash, versions)| {
            let first_version = versions.into_iter().next()?;
            Some((hash, first_version))
        })
        .collect();
    Ok(web::Json(resp))
}

/// Finds, for each project, the newest version matching the filters that the
/// user can see.
///
/// Visibility mirrors `filter_visible_version_ids`: moderators and members of
/// the project's team or organization see every version, everyone else only
/// sees non-hidden versions of non-hidden projects that aren't withheld for
/// missing attribution (see `get_files_missing_attribution`).
async fn update_files_internal(
    req: HttpRequest,
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    update_data: web::Json<ManyUpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HashMap<String, Vec<models::projects::Version>>, ApiError> {
    let user_option = get_user_from_headers(
        &req,
        &***pool,
        &redis,
        &session_queue,
        Scopes::VERSION_READ,
    )
    .await
    .map(|x| x.1)
    .ok();

    let algorithm = update_data
        .algorithm
        .clone()
        .unwrap_or_else(|| default_algorithm_from_hashes(&update_data.hashes));
    let files = database::models::DBVersion::get_files_from_hash(
        algorithm.clone(),
        &update_data.hashes,
        &***pool,
        &redis,
    )
    .await
    .wrap_internal_err("updating versions in database")?;

    // TODO: de-hardcode this and actually use version fields system
    let latest_version_ids = sqlx::query_scalar!(
        r#"
        SELECT latest.id AS "id!"
        FROM mods m
        CROSS JOIN LATERAL (
            SELECT
                $7::bool
                OR EXISTS (
                    SELECT 1 FROM team_members tm
                    WHERE tm.team_id = m.team_id AND tm.user_id = $8::bigint
                )
                OR EXISTS (
                    SELECT 1 FROM organizations o
                    INNER JOIN team_members tm ON tm.team_id = o.team_id
                    WHERE o.id = m.organization_id AND tm.user_id = $8::bigint
                ) AS full_access
        ) access
        CROSS JOIN LATERAL (
            SELECT v.id
            FROM versions v
            WHERE v.mod_id = m.id
                AND (cardinality($4::varchar[]) = 0 OR v.version_type = ANY($4))
                AND EXISTS (
                    SELECT 1 FROM version_fields vf
                    INNER JOIN loader_field_enum_values lfev ON lfev.id = vf.enum_value
                    WHERE vf.version_id = v.id AND vf.field_id = 3
                        AND (cardinality($2::varchar[]) = 0 OR lfev.value = ANY($2))
                )
                AND EXISTS (
                    SELECT 1 FROM loaders_versions lv
                    INNER JOIN loaders l ON l.id = lv.loader_id
                    WHERE lv.version_id = v.id
                        AND (cardinality($3::varchar[]) = 0 OR l.loader = ANY($3))
                )
                AND (
                    access.full_access
                    OR (
                        v.status = ANY($5::varchar[])
                        AND NOT EXISTS (
                            SELECT 1
                            FROM project_attribution_groups pag
                            INNER JOIN project_attribution_files paf ON paf.group_id = pag.id
                            INNER JOIN override_file_sources ofs ON ofs.sha1 = paf.sha1
                            INNER JOIN files f ON f.id = ofs.file_id
                            INNER JOIN attribution_enforced_versions aev ON aev.id = f.version_id
                            WHERE pag.project_id = v.mod_id
                                AND f.version_id = v.id
                                AND (
                                    pag.attribution IS NULL
                                    OR pag.attribution->>'kind' = 'no_permission'
                                    OR coalesce(pag.attribution->'moderation_status'->>'kind', 'approved') != 'approved'
                                )
                        )
                    )
                )
            ORDER BY v.date_published DESC, v.id DESC
            LIMIT 1
        ) latest
        WHERE m.id = ANY($1)
            AND (access.full_access OR m.status = ANY($6::varchar[]))
        "#,
        &files.iter().map(|x| x.project_id.0).collect::<Vec<_>>(),
        &update_data.game_versions.clone().unwrap_or_default(),
        &update_data.loaders.clone().unwrap_or_default(),
        &update_data
            .version_types
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>(),
        &VersionStatus::iterator()
            .filter(|x| !x.is_hidden())
            .map(|x| x.to_string())
            .collect::<Vec<_>>(),
        &ProjectStatus::iterator()
            .filter(|x| !x.is_hidden())
            .map(|x| x.to_string())
            .collect::<Vec<_>>(),
        user_option.as_ref().is_some_and(|x| x.role.is_mod()),
        user_option
            .as_ref()
            .map(|x| database::models::DBUserId::from(x.id).0),
    )
    .fetch_all(&***pool)
    .await
    .wrap_internal_err("fetching latest visible update versions")?;

    let versions = database::models::DBVersion::get_many(
        &latest_version_ids
            .into_iter()
            .map(database::models::DBVersionId)
            .collect::<Vec<_>>(),
        &***pool,
        &redis,
    )
    .await
    .wrap_internal_err("fetching update versions")?;

    let mut response = HashMap::<String, Vec<models::projects::Version>>::new();
    for file in files {
        if let Some(version) = versions
            .iter()
            .find(|x| x.inner.project_id == file.project_id)
            && let Some(hash) = file.hashes.get(&algorithm)
        {
            // add the version info for this file hash
            // note: one file hash can have multiple versions associated with it
            // just having a `HashMap<String, Version>` would mean that some version info is lost
            // so we return a vec of them instead
            response
                .entry(hash.clone())
                .or_default()
                .push(models::projects::Version::from(version.clone()));
        }
    }

    Ok(response)
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct FileUpdateData {
    pub hash: String,
    pub loaders: Option<Vec<String>>,
    pub loader_fields: Option<HashMap<String, Vec<serde_json::Value>>>,
    pub version_types: Option<Vec<VersionType>>,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct ManyFileUpdateData {
    pub algorithm: Option<String>, // Defaults to calculation based on size of hash
    pub hashes: Vec<FileUpdateData>,
}

/// Get latest matching versions by individual file filters.
#[utoipa::path(
	tag = "version files",
    post,
    operation_id = "v3UpdateIndividualFiles",
    request_body = ManyFileUpdateData,
    responses(
        (status = 200, description = "Expected response to a valid request", body = HashMap<String, models::projects::Version>)
    )
)]
#[post("/version_files/update_individual")]
pub async fn update_individual_files_route(
    req: HttpRequest,
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    update_data: web::Json<ManyFileUpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    update_individual_files(req, pool, redis, update_data, session_queue).await
}

pub async fn update_individual_files(
    req: HttpRequest,
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    update_data: web::Json<ManyFileUpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user_option = get_user_from_headers(
        &req,
        &***pool,
        &redis,
        &session_queue,
        Scopes::VERSION_READ,
    )
    .await
    .map(|x| x.1)
    .ok();

    let algorithm = update_data.algorithm.clone().unwrap_or_else(|| {
        default_algorithm_from_hashes(
            &update_data
                .hashes
                .iter()
                .map(|x| x.hash.clone())
                .collect::<Vec<_>>(),
        )
    });
    let files = database::models::DBVersion::get_files_from_hash(
        algorithm.clone(),
        &update_data
            .hashes
            .iter()
            .map(|x| x.hash.clone())
            .collect::<Vec<_>>(),
        &***pool,
        &redis,
    )
    .await
    .wrap_internal_err("updating versions in database")?;

    let filters = files
        .iter()
        .filter_map(|file| {
            let hash = file.hashes.get(&algorithm)?;
            let query_file =
                update_data.hashes.iter().find(|x| &x.hash == hash)?;
            Some(LatestVersionFilter {
                hash,
                project_id: file.project_id.0,
                loaders: query_file.loaders.as_deref(),
                version_types: query_file.version_types.as_deref(),
                loader_fields: query_file.loader_fields.as_ref(),
            })
        })
        .collect::<Vec<_>>();
    let latest = get_latest_matching_versions(&filters, &pool)
        .await
        .wrap_api_err("fetching latest matching versions")?;

    let versions = database::models::DBVersion::get_many(
        &latest.values().copied().unique().collect::<Vec<_>>(),
        &***pool,
        &redis,
    )
    .await
    .wrap_internal_err("fetching versions from database")?;
    let visible_version_ids = filter_visible_version_ids(
        versions.iter().map(|x| &x.inner).collect(),
        &user_option,
        &pool,
        &redis,
    )
    .await
    .wrap_api_err("filtering visible update versions")?;

    let response = latest
        .into_iter()
        .filter(|(_, version_id)| visible_version_ids.contains(version_id))
        .filter_map(|(hash, version_id)| {
            let version = versions.iter().find(|x| x.inner.id == version_id)?;
            Some((hash, models::projects::Version::from(version.clone())))
        })
        .collect::<HashMap<_, _>>();

    Ok(HttpResponse::Ok().json(response))
}

/// Per-hash filters for [`get_latest_matching_versions`], decoded by Postgres
/// with `jsonb_to_recordset`.
#[derive(Serialize)]
struct LatestVersionFilter<'a> {
    hash: &'a str,
    project_id: i64,
    loaders: Option<&'a [String]>,
    version_types: Option<&'a [VersionType]>,
    loader_fields: Option<&'a HashMap<String, Vec<serde_json::Value>>>,
}

/// Finds, for each filter, the newest listed version of its project matching
/// its loaders, version types and loader fields, keyed by the filter's hash.
///
/// Versions are ordered like `impl Ord for DBVersion`, and visibility is left
/// to the caller. A loader field only rules out versions that have it, using
/// the same rules as `VersionField::from_query_json`: the field must be linked
/// to one of the version's loaders, and a non-array field must have exactly one
/// value.
async fn get_latest_matching_versions(
    filters: &[LatestVersionFilter<'_>],
    pool: &PgPool,
) -> Result<HashMap<String, database::models::DBVersionId>, ApiError> {
    if filters.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query!(
        r#"
        SELECT q.hash AS "hash!", latest.id AS "version_id!"
        FROM jsonb_to_recordset($1::jsonb) AS q(
            hash text,
            project_id bigint,
            loaders varchar[],
            version_types varchar[],
            loader_fields jsonb
        )
        CROSS JOIN LATERAL (
            SELECT v.id
            FROM versions v
            WHERE v.mod_id = q.project_id
                AND v.status = ANY($2::varchar[])
                AND (q.version_types IS NULL OR v.version_type = ANY(q.version_types))
                AND (
                    q.loaders IS NULL
                    OR EXISTS (
                        SELECT 1 FROM loaders_versions lv
                        INNER JOIN loaders l ON l.id = lv.loader_id
                        WHERE lv.version_id = v.id AND l.loader = ANY(q.loaders)
                    )
                )
                AND NOT EXISTS (
                    SELECT 1
                    FROM jsonb_each(q.loader_fields) AS req(field, vals)
                    INNER JOIN loader_fields lf ON lf.field = req.field
                    WHERE EXISTS (
                        SELECT 1 FROM loaders_versions lv
                        INNER JOIN loader_fields_loaders lfl ON lfl.loader_id = lv.loader_id
                        WHERE lv.version_id = v.id AND lfl.loader_field_id = lf.id
                    )
                    AND (
                        lf.field_type IN ('array_integer', 'array_text', 'array_boolean', 'array_enum')
                        OR (
                            SELECT count(*) FROM version_fields vf
                            WHERE vf.version_id = v.id AND vf.field_id = lf.id
                        ) = 1
                    )
                    AND NOT EXISTS (
                        SELECT 1
                        FROM version_fields vf
                        LEFT JOIN loader_field_enum_values lfev ON lfev.id = vf.enum_value
                        CROSS JOIN jsonb_array_elements(req.vals) AS want(val)
                        WHERE vf.version_id = v.id AND vf.field_id = lf.id
                            AND CASE
                                WHEN lf.field_type IN ('enum', 'array_enum')
                                    THEN jsonb_typeof(want.val) = 'string' AND lfev.value = want.val #>> '{}'
                                WHEN lf.field_type IN ('text', 'array_text')
                                    THEN jsonb_typeof(want.val) = 'string' AND vf.string_value = want.val #>> '{}'
                                WHEN lf.field_type IN ('integer', 'array_integer')
                                    THEN jsonb_typeof(want.val) = 'number' AND vf.int_value = (want.val #>> '{}')::numeric
                                WHEN lf.field_type IN ('boolean', 'array_boolean')
                                    THEN jsonb_typeof(want.val) = 'boolean' AND (vf.int_value <> 0) = (want.val #>> '{}')::boolean
                            END
                    )
                )
            ORDER BY v.ordering DESC NULLS FIRST, v.date_published DESC, v.id DESC
            LIMIT 1
        ) latest
        "#,
        sqlx::types::Json(filters) as _,
        &VersionStatus::iterator()
            .filter(|x| x.is_listed())
            .map(|x| x.to_string())
            .collect::<Vec<_>>(),
    )
    .fetch_all(pool)
    .await
    .wrap_internal_err("fetching latest matching versions")?;

    Ok(rows
        .into_iter()
        .map(|row| (row.hash, database::models::DBVersionId(row.version_id)))
        .collect())
}

// under /api/v1/version_file/{hash}
/// Delete a file by hash.
#[utoipa::path(
	tag = "version files",
    delete,
    operation_id = "v3DeleteFileFromHash",
    params(
        ("version_id" = String, Path, description = "The hexadecimal file hash"),
        ("algorithm" = Option<String>, Query, description = "Hash algorithm to use (sha1 or sha512)"),
        ("version_id" = Option<VersionId>, Query, description = "Optional version ID to delete from")
    ),
    responses(
        (status = NO_CONTENT, description = "Expected response to a valid request"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        ),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[delete("/version_file/{version_id}")]
pub async fn delete_file_route(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    hash_query: web::Query<HashQuery>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    delete_file(req, info, pool, redis, hash_query, session_queue).await
}

pub async fn delete_file(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    hash_query: web::Query<HashQuery>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::VERSION_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let hash = info.into_inner().0.to_lowercase();
    let algorithm = hash_query.algorithm.clone().unwrap_or_else(|| {
        default_algorithm_from_hashes(std::slice::from_ref(&hash))
    });
    let file = database::models::DBVersion::get_file_from_hash(
        algorithm.clone(),
        hash,
        hash_query.version_id.map(|x| x.into()),
        &**pool,
        &redis,
    )
    .await
    .wrap_internal_err("fetching version from database")?;

    if let Some(row) = file {
        if !user.role.is_admin() {
            let team_member =
                database::models::DBTeamMember::get_from_user_id_version(
                    row.version_id,
                    user.id.into(),
                    &**pool,
                )
                .await
                .wrap_internal_err("fetching version team member")?;

            let organization =
                database::models::DBOrganization::get_associated_organization_project_id(
                    row.project_id,
                    &**pool,
                )
                .await
                .wrap_internal_err("fetching project organization")?;

            let organization_team_member = if let Some(organization) =
                &organization
            {
                database::models::DBTeamMember::get_from_user_id_organization(
                    organization.id,
                    user.id.into(),
                    false,
                    &**pool,
                )
                .await
                .wrap_internal_err("fetching organization team member")?
            } else {
                None
            };

            let permissions = ProjectPermissions::get_permissions_by_role(
                &user.role,
                &team_member,
                &organization_team_member,
            )
            .unwrap_or_default();

            if !permissions.contains(ProjectPermissions::DELETE_VERSION) {
                return Err(ApiError::Auth(eyre::eyre!(
                    "You don't have permission to delete this file!",
                )));
            }
        }

        let version =
            database::models::DBVersion::get(row.version_id, &**pool, &redis)
                .await
                .wrap_internal_err("fetching version from database")?;
        if let Some(version) = version {
            if version.files.len() < 2 {
                return Err(ApiError::Request(eyre::eyre!(
                    "Versions must have at least one file uploaded to them",
                )));
            }

            database::models::DBVersion::clear_cache(&version, &redis)
                .await
                .wrap_internal_err("clearing cached data from Redis")?;
        }

        let mut transaction = pool
            .begin()
            .await
            .wrap_internal_err("starting database transaction")?;

        sqlx::query!(
            "
            DELETE FROM hashes
            WHERE file_id = $1
            ",
            row.id.0
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `delete_file`")?;

        sqlx::query!(
            "
            DELETE FROM files
            WHERE files.id = $1
            ",
            row.id.0,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `delete_file`")?;

        database::models::version_item::cleanup_unused_attribution_files_and_groups(&mut transaction)
            .await.wrap_internal_err("deleting version item from database")?;

        delphi::tech_review_queue::sync_projects(
            &[row.project_id],
            delphi::tech_review_queue::TechReviewRemovalReason::FileDeleted,
            &mut transaction,
        )
        .await
        .wrap_api_err("executing `tech_review_queue::sync_projects`")?;

        transaction
            .commit()
            .await
            .wrap_internal_err("committing database transaction")?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct DownloadRedirect {
    pub url: String,
}

// under /api/v1/version_file/{hash}/download
/// Download a file by hash.
#[utoipa::path(
	tag = "version files",
    get,
    operation_id = "v3DownloadVersionFromHash",
    params(
        ("version_id" = String, Path, description = "The hexadecimal file hash"),
        ("algorithm" = Option<String>, Query, description = "Hash algorithm to use (sha1 or sha512)"),
        ("version_id" = Option<VersionId>, Query, description = "Optional version ID when hash maps to multiple files")
    ),
    responses(
        (status = 302, description = "Temporary redirect to file URL", body = DownloadRedirect),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[get("/version_file/{version_id}/download")]
pub async fn download_version_route(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    hash_query: web::Query<HashQuery>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    download_version(req, info, pool, redis, hash_query, session_queue).await
}

pub async fn download_version(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    hash_query: web::Query<HashQuery>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::VERSION_READ,
    )
    .await
    .map(|x| x.1)
    .ok();

    let hash = info.into_inner().0.to_lowercase();
    let algorithm = hash_query.algorithm.clone().unwrap_or_else(|| {
        default_algorithm_from_hashes(std::slice::from_ref(&hash))
    });
    let file = database::models::DBVersion::get_file_from_hash(
        algorithm.clone(),
        hash,
        hash_query.version_id.map(|x| x.into()),
        &**pool,
        &redis,
    )
    .await
    .wrap_internal_err("fetching version from database")?;

    if let Some(file) = file {
        let version =
            database::models::DBVersion::get(file.version_id, &**pool, &redis)
                .await
                .wrap_internal_err("fetching version from database")?;

        if let Some(version) = version {
            if !is_visible_version(&version.inner, &user_option, &pool, &redis)
                .await
                .wrap_api_err("checking version visibility")?
            {
                return Err(ApiError::NotFound(eyre::eyre!(
                    "resource not found"
                )));
            }

            Ok(HttpResponse::TemporaryRedirect()
                .append_header(("Location", &*file.url))
                .json(DownloadRedirect { url: file.url }))
        } else {
            Err(ApiError::NotFound(eyre::eyre!("resource not found")))
        }
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

use super::ApiError;
use crate::auth::{
    filter_authorized_projects, filter_authorized_versions, get_user_from_headers,
    is_authorized_version,
};
use crate::database::redis::RedisPool;
use crate::models::ids::VersionId;
use crate::models::pats::Scopes;
use crate::models::projects::VersionType;
use crate::models::teams::ProjectPermissions;
use crate::queue::session::AuthQueue;
use crate::{database, models};
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("version_file")
            .service(delete_file)
            .service(get_version_from_hash)
            .service(download_version)
            .service(get_update_from_hash)
            .service(get_projects_from_hashes),
    );

    cfg.service(
        web::scope("version_files")
            .service(get_versions_from_hashes)
            .service(update_files)
            .service(update_individual_files),
    );
}

#[derive(Serialize, Deserialize)]
pub struct HashQuery {
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
    pub version_id: Option<VersionId>,
}

fn default_algorithm() -> String {
    "sha1".into()
}

// under /api/v1/version_file/{hash}
#[get("{version_id}")]
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
        Some(&[Scopes::VERSION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();
    let hash = info.into_inner().0.to_lowercase();
    let file = database::models::Version::get_file_from_hash(
        hash_query.algorithm.clone(),
        hash,
        hash_query.version_id.map(|x| x.into()),
        &**pool,
        &redis,
    )
    .await?;
    if let Some(file) = file {
        let version = database::models::Version::get(file.version_id, &**pool, &redis).await?;
        if let Some(version) = version {
            if !is_authorized_version(&version.inner, &user_option, &pool).await? {
                return Ok(HttpResponse::NotFound().body(""));
            }

            Ok(HttpResponse::Ok().json(models::projects::Version::from(version)))
        } else {
            Ok(HttpResponse::NotFound().body(""))
        }
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(Serialize, Deserialize)]
pub struct DownloadRedirect {
    pub url: String,
}

// under /api/v1/version_file/{hash}/download
#[get("{version_id}/download")]
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
        Some(&[Scopes::VERSION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    let hash = info.into_inner().0.to_lowercase();
    let file = database::models::Version::get_file_from_hash(
        hash_query.algorithm.clone(),
        hash,
        hash_query.version_id.map(|x| x.into()),
        &**pool,
        &redis,
    )
    .await?;

    if let Some(file) = file {
        let version = database::models::Version::get(file.version_id, &**pool, &redis).await?;

        if let Some(version) = version {
            if !is_authorized_version(&version.inner, &user_option, &pool).await? {
                return Ok(HttpResponse::NotFound().body(""));
            }

            Ok(HttpResponse::TemporaryRedirect()
                .append_header(("Location", &*file.url))
                .json(DownloadRedirect { url: file.url }))
        } else {
            Ok(HttpResponse::NotFound().body(""))
        }
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

// under /api/v1/version_file/{hash}
#[delete("{version_id}")]
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
        Some(&[Scopes::VERSION_WRITE]),
    )
    .await?
    .1;

    let hash = info.into_inner().0.to_lowercase();

    let file = database::models::Version::get_file_from_hash(
        hash_query.algorithm.clone(),
        hash,
        hash_query.version_id.map(|x| x.into()),
        &**pool,
        &redis,
    )
    .await?;

    if let Some(row) = file {
        if !user.role.is_admin() {
            let team_member = database::models::TeamMember::get_from_user_id_version(
                row.version_id,
                user.id.into(),
                &**pool,
            )
            .await
            .map_err(ApiError::Database)?;

            let organization =
                database::models::Organization::get_associated_organization_project_id(
                    row.project_id,
                    &**pool,
                )
                .await
                .map_err(ApiError::Database)?;

            let organization_team_member = if let Some(organization) = &organization {
                database::models::TeamMember::get_from_user_id_organization(
                    organization.id,
                    user.id.into(),
                    &**pool,
                )
                .await
                .map_err(ApiError::Database)?
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
                return Err(ApiError::CustomAuthentication(
                    "You don't have permission to delete this file!".to_string(),
                ));
            }
        }

        let version = database::models::Version::get(row.version_id, &**pool, &redis).await?;
        if let Some(version) = version {
            if version.files.len() < 2 {
                return Err(ApiError::InvalidInput(
                    "Versions must have at least one file uploaded to them".to_string(),
                ));
            }

            database::models::Version::clear_cache(&version, &redis).await?;
        }

        let mut transaction = pool.begin().await?;

        sqlx::query!(
            "
            DELETE FROM hashes
            WHERE file_id = $1
            ",
            row.id.0
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM files
            WHERE files.id = $1
            ",
            row.id.0,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(Deserialize)]
pub struct UpdateData {
    pub loaders: Option<Vec<String>>,
    pub game_versions: Option<Vec<String>>,
    pub version_types: Option<Vec<VersionType>>,
}

#[post("{version_id}/update")]
pub async fn get_update_from_hash(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    hash_query: web::Query<HashQuery>,
    update_data: web::Json<UpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::VERSION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();
    let hash = info.into_inner().0.to_lowercase();

    if let Some(file) = database::models::Version::get_file_from_hash(
        hash_query.algorithm.clone(),
        hash,
        hash_query.version_id.map(|x| x.into()),
        &**pool,
        &redis,
    )
    .await?
    {
        if let Some(project) =
            database::models::Project::get_id(file.project_id, &**pool, &redis).await?
        {
            let mut versions =
                database::models::Version::get_many(&project.versions, &**pool, &redis)
                    .await?
                    .into_iter()
                    .filter(|x| {
                        let mut bool = true;

                        if let Some(version_types) = &update_data.version_types {
                            bool &= version_types
                                .iter()
                                .any(|y| y.as_str() == x.inner.version_type);
                        }
                        if let Some(loaders) = &update_data.loaders {
                            bool &= x.loaders.iter().any(|y| loaders.contains(y));
                        }
                        if let Some(game_versions) = &update_data.game_versions {
                            bool &= x.game_versions.iter().any(|y| game_versions.contains(y));
                        }

                        bool
                    })
                    .sorted()
                    .collect::<Vec<_>>();

            if let Some(first) = versions.pop() {
                if !is_authorized_version(&first.inner, &user_option, &pool).await? {
                    return Ok(HttpResponse::NotFound().body(""));
                }

                return Ok(HttpResponse::Ok().json(models::projects::Version::from(first)));
            }
        }
    }

    Ok(HttpResponse::NotFound().body(""))
}

// Requests above with multiple versions below
#[derive(Deserialize)]
pub struct FileHashes {
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
    pub hashes: Vec<String>,
}

// under /api/v2/version_files
#[post("")]
pub async fn get_versions_from_hashes(
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
        Some(&[Scopes::VERSION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    let files = database::models::Version::get_files_from_hash(
        file_data.algorithm.clone(),
        &file_data.hashes,
        &**pool,
        &redis,
    )
    .await?;

    let version_ids = files.iter().map(|x| x.version_id).collect::<Vec<_>>();
    let versions_data = filter_authorized_versions(
        database::models::Version::get_many(&version_ids, &**pool, &redis).await?,
        &user_option,
        &pool,
    )
    .await?;

    let mut response = HashMap::new();

    for version in versions_data {
        for file in files.iter().filter(|x| x.version_id == version.id.into()) {
            if let Some(hash) = file.hashes.get(&file_data.algorithm) {
                response.insert(hash.clone(), version.clone());
            }
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

#[post("project")]
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
        Some(&[Scopes::PROJECT_READ, Scopes::VERSION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    let files = database::models::Version::get_files_from_hash(
        file_data.algorithm.clone(),
        &file_data.hashes,
        &**pool,
        &redis,
    )
    .await?;

    let project_ids = files.iter().map(|x| x.project_id).collect::<Vec<_>>();

    let projects_data = filter_authorized_projects(
        database::models::Project::get_many_ids(&project_ids, &**pool, &redis).await?,
        &user_option,
        &pool,
    )
    .await?;

    let mut response = HashMap::new();

    for project in projects_data {
        for file in files.iter().filter(|x| x.project_id == project.id.into()) {
            if let Some(hash) = file.hashes.get(&file_data.algorithm) {
                response.insert(hash.clone(), project.clone());
            }
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Deserialize)]
pub struct ManyUpdateData {
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
    pub hashes: Vec<String>,
    pub loaders: Option<Vec<String>>,
    pub game_versions: Option<Vec<String>>,
    pub version_types: Option<Vec<VersionType>>,
}

#[post("update")]
pub async fn update_files(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    update_data: web::Json<ManyUpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::VERSION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    let files = database::models::Version::get_files_from_hash(
        update_data.algorithm.clone(),
        &update_data.hashes,
        &**pool,
        &redis,
    )
    .await?;

    let projects = database::models::Project::get_many_ids(
        &files.iter().map(|x| x.project_id).collect::<Vec<_>>(),
        &**pool,
        &redis,
    )
    .await?;
    let all_versions = database::models::Version::get_many(
        &projects
            .iter()
            .flat_map(|x| x.versions.clone())
            .collect::<Vec<_>>(),
        &**pool,
        &redis,
    )
    .await?;

    let mut response = HashMap::new();

    for project in projects {
        for file in files.iter().filter(|x| x.project_id == project.inner.id) {
            let version = all_versions
                .iter()
                .filter(|x| x.inner.project_id == file.project_id)
                .filter(|x| {
                    let mut bool = true;

                    if let Some(version_types) = &update_data.version_types {
                        bool &= version_types
                            .iter()
                            .any(|y| y.as_str() == x.inner.version_type);
                    }
                    if let Some(loaders) = &update_data.loaders {
                        bool &= x.loaders.iter().any(|y| loaders.contains(y));
                    }
                    if let Some(game_versions) = &update_data.game_versions {
                        bool &= x.game_versions.iter().any(|y| game_versions.contains(y));
                    }

                    bool
                })
                .sorted()
                .next();

            if let Some(version) = version {
                if is_authorized_version(&version.inner, &user_option, &pool).await? {
                    if let Some(hash) = file.hashes.get(&update_data.algorithm) {
                        response.insert(
                            hash.clone(),
                            models::projects::Version::from(version.clone()),
                        );
                    }
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Deserialize)]
pub struct FileUpdateData {
    pub hash: String,
    pub loaders: Option<Vec<String>>,
    pub game_versions: Option<Vec<String>>,
    pub version_types: Option<Vec<VersionType>>,
}

#[derive(Deserialize)]
pub struct ManyFileUpdateData {
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
    pub hashes: Vec<FileUpdateData>,
}

#[post("update_individual")]
pub async fn update_individual_files(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    update_data: web::Json<ManyFileUpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::VERSION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    let files = database::models::Version::get_files_from_hash(
        update_data.algorithm.clone(),
        &update_data
            .hashes
            .iter()
            .map(|x| x.hash.clone())
            .collect::<Vec<_>>(),
        &**pool,
        &redis,
    )
    .await?;

    let projects = database::models::Project::get_many_ids(
        &files.iter().map(|x| x.project_id).collect::<Vec<_>>(),
        &**pool,
        &redis,
    )
    .await?;
    let all_versions = database::models::Version::get_many(
        &projects
            .iter()
            .flat_map(|x| x.versions.clone())
            .collect::<Vec<_>>(),
        &**pool,
        &redis,
    )
    .await?;

    let mut response = HashMap::new();

    for project in projects {
        for file in files.iter().filter(|x| x.project_id == project.inner.id) {
            if let Some(hash) = file.hashes.get(&update_data.algorithm) {
                if let Some(query_file) = update_data.hashes.iter().find(|x| &x.hash == hash) {
                    let version = all_versions
                        .iter()
                        .filter(|x| x.inner.project_id == file.project_id)
                        .filter(|x| {
                            let mut bool = true;

                            if let Some(version_types) = &query_file.version_types {
                                bool &= version_types
                                    .iter()
                                    .any(|y| y.as_str() == x.inner.version_type);
                            }
                            if let Some(loaders) = &query_file.loaders {
                                bool &= x.loaders.iter().any(|y| loaders.contains(y));
                            }
                            if let Some(game_versions) = &query_file.game_versions {
                                bool &= x.game_versions.iter().any(|y| game_versions.contains(y));
                            }

                            bool
                        })
                        .sorted()
                        .next();

                    if let Some(version) = version {
                        if is_authorized_version(&version.inner, &user_option, &pool).await? {
                            response.insert(
                                hash.clone(),
                                models::projects::Version::from(version.clone()),
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

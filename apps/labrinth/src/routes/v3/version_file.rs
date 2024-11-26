use super::ApiError;
use crate::auth::checks::{filter_visible_versions, is_visible_version};
use crate::auth::{filter_visible_projects, get_user_from_headers};
use crate::database::redis::RedisPool;
use crate::models::ids::VersionId;
use crate::models::pats::Scopes;
use crate::models::projects::VersionType;
use crate::models::teams::ProjectPermissions;
use crate::queue::session::AuthQueue;
use crate::{database, models};
use actix_web::{web, HttpRequest, HttpResponse};
use dashmap::DashMap;
use futures::TryStreamExt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("version_file")
            .route("{version_id}", web::get().to(get_version_from_hash))
            .route("{version_id}/update", web::post().to(get_update_from_hash))
            .route("project", web::post().to(get_projects_from_hashes))
            .route("{version_id}", web::delete().to(delete_file))
            .route("{version_id}/download", web::get().to(download_version)),
    );
    cfg.service(
        web::scope("version_files")
            .route("update", web::post().to(update_files))
            .route("update_individual", web::post().to(update_individual_files))
            .route("", web::post().to(get_versions_from_hashes)),
    );
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
        Some(&[Scopes::VERSION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();
    let hash = info.into_inner().0.to_lowercase();
    let algorithm = hash_query
        .algorithm
        .clone()
        .unwrap_or_else(|| default_algorithm_from_hashes(&[hash.clone()]));
    let file = database::models::Version::get_file_from_hash(
        algorithm,
        hash,
        hash_query.version_id.map(|x| x.into()),
        &**pool,
        &redis,
    )
    .await?;
    if let Some(file) = file {
        let version =
            database::models::Version::get(file.version_id, &**pool, &redis)
                .await?;
        if let Some(version) = version {
            if !is_visible_version(&version.inner, &user_option, &pool, &redis)
                .await?
            {
                return Err(ApiError::NotFound);
            }

            Ok(HttpResponse::Ok()
                .json(models::projects::Version::from(version)))
        } else {
            Err(ApiError::NotFound)
        }
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Serialize, Deserialize)]
pub struct HashQuery {
    pub algorithm: Option<String>, // 默认根据哈希大小计算
    pub version_id: Option<VersionId>,
}

// 根据哈希的大小计算是否使用 sha1 或 sha512
pub fn default_algorithm_from_hashes(hashes: &[String]) -> String {
    // 可选地获取第一个哈希
    let empty_string = "".into();
    let hash = hashes.first().unwrap_or(&empty_string);
    let hash_len = hash.len();
    // Sha1 = 40 个字符
    // Sha512 = 128 个字符
    // 默认使用 sha1，除非哈希长度大于或等于 128 个字符
    if hash_len >= 128 {
        return "sha512".into();
    }
    "sha1".into()
}

#[derive(Serialize, Deserialize)]
pub struct UpdateData {
    pub loaders: Option<Vec<String>>,
    pub version_types: Option<Vec<VersionType>>,
    /*
       用于过滤的加载器字段：
       "game_versions": ["1.16.5", "1.17"]

       如果匹配任何值则返回
    */
    pub loader_fields: Option<HashMap<String, Vec<serde_json::Value>>>,
}

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
        hash_query
            .algorithm
            .clone()
            .unwrap_or_else(|| default_algorithm_from_hashes(&[hash.clone()])),
        hash,
        hash_query.version_id.map(|x| x.into()),
        &**pool,
        &redis,
    )
    .await?
    {
        if let Some(project) =
            database::models::Project::get_id(file.project_id, &**pool, &redis)
                .await?
        {
            let versions = database::models::Version::get_many(
                &project.versions,
                &**pool,
                &redis,
            )
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
                if let Some(loader_fields) = &update_data.loader_fields {
                    for (key, values) in loader_fields {
                        bool &= if let Some(x_vf) = x
                            .version_fields
                            .iter()
                            .find(|y| y.field_name == *key)
                        {
                            values
                                .iter()
                                .any(|v| x_vf.value.contains_json_value(v))
                        } else {
                            true
                        };
                    }
                }
                bool
            })
            .sorted();

            if let Some(first) = versions.last() {
                if !is_visible_version(
                    &first.inner,
                    &user_option,
                    &pool,
                    &redis,
                )
                .await?
                {
                    return Err(ApiError::NotFound);
                }

                return Ok(HttpResponse::Ok()
                    .json(models::projects::Version::from(first)));
            }
        }
    }
    Err(ApiError::NotFound)
}

// 上面的请求与下面的多个版本
#[derive(Deserialize)]
pub struct FileHashes {
    pub algorithm: Option<String>, // 默认根据哈希大小计算
    pub hashes: Vec<String>,
}

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

    let algorithm = file_data
        .algorithm
        .clone()
        .unwrap_or_else(|| default_algorithm_from_hashes(&file_data.hashes));

    let files = database::models::Version::get_files_from_hash(
        algorithm.clone(),
        &file_data.hashes,
        &**pool,
        &redis,
    )
    .await?;

    let version_ids = files.iter().map(|x| x.version_id).collect::<Vec<_>>();
    let versions_data = filter_visible_versions(
        database::models::Version::get_many(&version_ids, &**pool, &redis)
            .await?,
        &user_option,
        &pool,
        &redis,
    )
    .await?;

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

    let algorithm = file_data
        .algorithm
        .clone()
        .unwrap_or_else(|| default_algorithm_from_hashes(&file_data.hashes));
    let files = database::models::Version::get_files_from_hash(
        algorithm.clone(),
        &file_data.hashes,
        &**pool,
        &redis,
    )
    .await?;

    let project_ids = files.iter().map(|x| x.project_id).collect::<Vec<_>>();

    let projects_data = filter_visible_projects(
        database::models::Project::get_many_ids(&project_ids, &**pool, &redis)
            .await?,
        &user_option,
        &pool,
        false,
    )
    .await?;

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

#[derive(Deserialize)]
pub struct ManyUpdateData {
    pub algorithm: Option<String>, // 默认根据哈希大小计算
    pub hashes: Vec<String>,
    pub loaders: Option<Vec<String>>,
    pub game_versions: Option<Vec<String>>,
    pub version_types: Option<Vec<VersionType>>,
}
pub async fn update_files(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    update_data: web::Json<ManyUpdateData>,
) -> Result<HttpResponse, ApiError> {
    let algorithm = update_data
        .algorithm
        .clone()
        .unwrap_or_else(|| default_algorithm_from_hashes(&update_data.hashes));
    let files = database::models::Version::get_files_from_hash(
        algorithm.clone(),
        &update_data.hashes,
        &**pool,
        &redis,
    )
    .await?;

    // TODO: 取消硬编码并实际使用版本字段系统
    let update_version_ids = sqlx::query!(
        "
        SELECT v.id version_id, v.mod_id mod_id
        FROM mods m
        INNER JOIN versions v ON m.id = v.mod_id AND (cardinality($4::varchar[]) = 0 OR v.version_type = ANY($4))
        INNER JOIN version_fields vf ON vf.field_id = 3 AND v.id = vf.version_id
        INNER JOIN loader_field_enum_values lfev ON vf.enum_value = lfev.id AND (cardinality($2::varchar[]) = 0 OR lfev.value = ANY($2::varchar[]))
        INNER JOIN loaders_versions lv ON lv.version_id = v.id
        INNER JOIN loaders l on lv.loader_id = l.id AND (cardinality($3::varchar[]) = 0 OR l.loader = ANY($3::varchar[]))
        WHERE m.id = ANY($1)
        ORDER BY v.date_published ASC
        ",
        &files.iter().map(|x| x.project_id.0).collect::<Vec<_>>(),
        &update_data.game_versions.clone().unwrap_or_default(),
        &update_data.loaders.clone().unwrap_or_default(),
        &update_data.version_types.clone().unwrap_or_default().iter().map(|x| x.to_string()).collect::<Vec<_>>(),
    )
        .fetch(&**pool)
        .try_fold(DashMap::new(), |acc : DashMap<_,Vec<database::models::ids::VersionId>>, m| {
            acc.entry(database::models::ProjectId(m.mod_id))
                .or_default()
                .push(database::models::VersionId(m.version_id));
            async move { Ok(acc) }
        })
        .await?;

    let versions = database::models::Version::get_many(
        &update_version_ids
            .into_iter()
            .filter_map(|x| x.1.last().copied())
            .collect::<Vec<_>>(),
        &**pool,
        &redis,
    )
    .await?;

    let mut response = HashMap::new();
    for file in files {
        if let Some(version) = versions
            .iter()
            .find(|x| x.inner.project_id == file.project_id)
        {
            if let Some(hash) = file.hashes.get(&algorithm) {
                response.insert(
                    hash.clone(),
                    models::projects::Version::from(version.clone()),
                );
            }
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Serialize, Deserialize)]
pub struct FileUpdateData {
    pub hash: String,
    pub loaders: Option<Vec<String>>,
    pub loader_fields: Option<HashMap<String, Vec<serde_json::Value>>>,
    pub version_types: Option<Vec<VersionType>>,
}

#[derive(Serialize, Deserialize)]
pub struct ManyFileUpdateData {
    pub algorithm: Option<String>, // 默认根据哈希大小计算
    pub hashes: Vec<FileUpdateData>,
}

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

    let algorithm = update_data.algorithm.clone().unwrap_or_else(|| {
        default_algorithm_from_hashes(
            &update_data
                .hashes
                .iter()
                .map(|x| x.hash.clone())
                .collect::<Vec<_>>(),
        )
    });
    let files = database::models::Version::get_files_from_hash(
        algorithm.clone(),
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
            if let Some(hash) = file.hashes.get(&algorithm) {
                if let Some(query_file) =
                    update_data.hashes.iter().find(|x| &x.hash == hash)
                {
                    let version = all_versions
                        .iter()
                        .filter(|x| x.inner.project_id == file.project_id)
                        .filter(|x| {
                            let mut bool = true;

                            if let Some(version_types) =
                                &query_file.version_types
                            {
                                bool &= version_types.iter().any(|y| {
                                    y.as_str() == x.inner.version_type
                                });
                            }
                            if let Some(loaders) = &query_file.loaders {
                                bool &= x
                                    .loaders
                                    .iter()
                                    .any(|y| loaders.contains(y));
                            }

                            if let Some(loader_fields) =
                                &query_file.loader_fields
                            {
                                for (key, values) in loader_fields {
                                    bool &= if let Some(x_vf) = x
                                        .version_fields
                                        .iter()
                                        .find(|y| y.field_name == *key)
                                    {
                                        values.iter().any(|v| {
                                            x_vf.value.contains_json_value(v)
                                        })
                                    } else {
                                        true
                                    };
                                }
                            }
                            bool
                        })
                        .sorted()
                        .last();

                    if let Some(version) = version {
                        if is_visible_version(
                            &version.inner,
                            &user_option,
                            &pool,
                            &redis,
                        )
                        .await?
                        {
                            response.insert(
                                hash.clone(),
                                models::projects::Version::from(
                                    version.clone(),
                                ),
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

// 在 /api/v1/version_file/{hash} 下
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
    let algorithm = hash_query
        .algorithm
        .clone()
        .unwrap_or_else(|| default_algorithm_from_hashes(&[hash.clone()]));
    let file = database::models::Version::get_file_from_hash(
        algorithm.clone(),
        hash,
        hash_query.version_id.map(|x| x.into()),
        &**pool,
        &redis,
    )
  .await?;

    if let Some(row) = file {
        if !user.role.is_admin() {
            let team_member =
                database::models::TeamMember::get_from_user_id_version(
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

            let organization_team_member =
                if let Some(organization) = &organization {
                    database::models::TeamMember::get_from_user_id_organization(
                        organization.id,
                        user.id.into(),
                        false,
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
                    "您没有权限删除此文件！".to_string(),
                ));
            }
        }

        let version =
            database::models::Version::get(row.version_id, &**pool, &redis)
                .await?;
        if let Some(version) = version {
            if version.files.len() < 2 {
                return Err(ApiError::InvalidInput(
                    "版本必须至少有一个文件上传".to_string(),
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
        Err(ApiError::NotFound)
    }
}

#[derive(Serialize, Deserialize)]
pub struct DownloadRedirect {
    pub url: String,
}

// 在 /api/v1/version_file/{hash}/download 下
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
    let algorithm = hash_query
        .algorithm
        .clone()
        .unwrap_or_else(|| default_algorithm_from_hashes(&[hash.clone()]));
    let file = database::models::Version::get_file_from_hash(
        algorithm.clone(),
        hash,
        hash_query.version_id.map(|x| x.into()),
        &**pool,
        &redis,
    )
    .await?;

    if let Some(file) = file {
        let version =
            database::models::Version::get(file.version_id, &**pool, &redis)
                .await?;

        if let Some(version) = version {
            if !is_visible_version(&version.inner, &user_option, &pool, &redis)
                .await?
            {
                return Err(ApiError::NotFound);
            }

            Ok(HttpResponse::TemporaryRedirect()
                .append_header(("Location", &*file.url))
                .json(DownloadRedirect { url: file.url }))
        } else {
            Err(ApiError::NotFound)
        }
    } else {
        Err(ApiError::NotFound)
    }
}
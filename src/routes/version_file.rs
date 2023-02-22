use super::ApiError;
use crate::database::models::{version_item::QueryVersion, DatabaseError};
use crate::models::ids::VersionId;
use crate::models::projects::{GameVersion, Loader, Version};
use crate::models::teams::Permissions;
use crate::util::auth::get_user_from_headers;
use crate::util::routes::ok_or_not_found;
use crate::{database, models};
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use futures::TryStreamExt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct HashQuery {
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
    #[serde(default = "default_multiple")]
    pub multiple: bool,
    pub version_id: Option<VersionId>,
}

fn default_algorithm() -> String {
    "sha1".into()
}

fn default_multiple() -> bool {
    false
}

// under /api/v1/version_file/{hash}
#[get("{version_id}")]
pub async fn get_version_from_hash(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    hash_query: web::Query<HashQuery>,
) -> Result<HttpResponse, ApiError> {
    let hash = info.into_inner().0.to_lowercase();

    let result = sqlx::query!(
        "
        SELECT f.version_id version_id
        FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v on f.version_id = v.id AND v.status != ANY($1)
        INNER JOIN mods m on v.mod_id = m.id
        WHERE h.algorithm = $3 AND h.hash = $2 AND m.status != ANY($4)
        ORDER BY v.date_published ASC
        ",
        &*crate::models::projects::VersionStatus::iterator()
            .filter(|x| x.is_hidden())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        hash.as_bytes(),
        hash_query.algorithm,
        &*crate::models::projects::ProjectStatus::iterator()
            .filter(|x| x.is_hidden())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    )
    .fetch_all(&**pool)
    .await?;

    let versions_data = database::models::Version::get_many_full(
        result
            .iter()
            .map(|x| database::models::VersionId(x.version_id))
            .collect(),
        &**pool,
    )
    .await?;

    if let Some(first) = versions_data.first() {
        if hash_query.multiple {
            Ok(HttpResponse::Ok().json(
                versions_data
                    .into_iter()
                    .map(models::projects::Version::from)
                    .collect::<Vec<_>>(),
            ))
        } else {
            Ok(HttpResponse::Ok()
                .json(models::projects::Version::from(first.clone())))
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
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    hash_query: web::Query<HashQuery>,
) -> Result<HttpResponse, ApiError> {
    let hash = info.into_inner().0.to_lowercase();
    let mut transaction = pool.begin().await?;

    let result = sqlx::query!(
        "
        SELECT f.url url, f.id id, f.version_id version_id, v.mod_id project_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id AND v.status != ANY($1)
        INNER JOIN mods m on v.mod_id = m.id
        WHERE h.algorithm = $3 AND h.hash = $2 AND m.status != ANY($4)
        ORDER BY v.date_published ASC
        ",
        &*crate::models::projects::VersionStatus::iterator().filter(|x| x.is_hidden()).map(|x| x.to_string()).collect::<Vec<String>>(),
        hash.as_bytes(),
        hash_query.algorithm,
        &*crate::models::projects::ProjectStatus::iterator().filter(|x| x.is_hidden()).map(|x| x.to_string()).collect::<Vec<String>>(),
    )
    .fetch_optional(&mut *transaction)
    .await?;

    if let Some(id) = result {
        transaction.commit().await?;

        Ok(HttpResponse::TemporaryRedirect()
            .append_header(("Location", &*id.url))
            .json(DownloadRedirect { url: id.url }))
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
    hash_query: web::Query<HashQuery>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    let hash = info.into_inner().0.to_lowercase();

    let result = sqlx::query!(
        "
        SELECT f.id id, f.version_id version_id, f.filename filename, v.version_number version_number, v.mod_id project_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id
        WHERE h.algorithm = $2 AND h.hash = $1
        ORDER BY v.date_published ASC
        ",
        hash.as_bytes(),
        hash_query.algorithm
    )
        .fetch_all(&**pool)
        .await?;

    if let Some(row) = result.iter().find_or_first(|x| {
        hash_query.version_id.is_none()
            || Some(x.version_id) == hash_query.version_id.map(|x| x.0 as i64)
    }) {
        if !user.role.is_admin() {
            let team_member =
                database::models::TeamMember::get_from_user_id_version(
                    database::models::ids::VersionId(row.version_id),
                    user.id.into(),
                    &**pool,
                )
                .await
                .map_err(ApiError::Database)?
                .ok_or_else(|| {
                    ApiError::CustomAuthentication(
                        "You don't have permission to delete this file!"
                            .to_string(),
                    )
                })?;

            if !team_member
                .permissions
                .contains(Permissions::DELETE_VERSION)
            {
                return Err(ApiError::CustomAuthentication(
                    "You don't have permission to delete this file!"
                        .to_string(),
                ));
            }
        }

        use futures::stream::TryStreamExt;

        let files = sqlx::query!(
            "
            SELECT f.id id FROM files f
            WHERE f.version_id = $1
            ",
            row.version_id
        )
        .fetch_many(&**pool)
        .try_filter_map(|e| async { Ok(e.right().map(|_| ())) })
        .try_collect::<Vec<()>>()
        .await?;

        if files.len() < 2 {
            return Err(ApiError::InvalidInput(
                "Versions must have at least one file uploaded to them"
                    .to_string(),
            ));
        }

        let mut transaction = pool.begin().await?;

        sqlx::query!(
            "
            DELETE FROM hashes
            WHERE file_id = $1
            ",
            row.id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM files
            WHERE files.id = $1
            ",
            row.id,
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
    pub loaders: Vec<Loader>,
    pub game_versions: Vec<GameVersion>,
}

#[post("{version_id}/update")]
pub async fn get_update_from_hash(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    hash_query: web::Query<HashQuery>,
    update_data: web::Json<UpdateData>,
) -> Result<HttpResponse, ApiError> {
    let hash = info.into_inner().0.to_lowercase();

    // get version_id from hash
    // get mod_id from hash
    // get latest version satisfying conditions - if not found

    let result = sqlx::query!(
        "
        SELECT v.mod_id project_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id AND v.status != ANY($1)
        INNER JOIN mods m on v.mod_id = m.id
        WHERE h.algorithm = $3 AND h.hash = $2 AND m.status != ANY($4)
        ORDER BY v.date_published ASC
        ",
        &*crate::models::projects::VersionStatus::iterator()
            .filter(|x| x.is_hidden())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        hash.as_bytes(),
        hash_query.algorithm,
        &*crate::models::projects::ProjectStatus::iterator()
            .filter(|x| x.is_hidden())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    )
    .fetch_optional(&**pool)
    .await?;

    if let Some(id) = result {
        let version_ids = database::models::Version::get_project_versions(
            database::models::ProjectId(id.project_id),
            Some(
                update_data
                    .game_versions
                    .clone()
                    .into_iter()
                    .map(|x| x.0)
                    .collect(),
            ),
            Some(
                update_data
                    .loaders
                    .clone()
                    .into_iter()
                    .map(|x| x.0)
                    .collect(),
            ),
            None,
            None,
            None,
            &**pool,
        )
        .await?;

        if let Some(version_id) = version_ids.first() {
            let version_data =
                database::models::Version::get_full(*version_id, &**pool)
                    .await?;

            ok_or_not_found::<QueryVersion, Version>(version_data)
        } else {
            Ok(HttpResponse::NotFound().body(""))
        }
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

// Requests above with multiple versions below
#[derive(Deserialize)]
pub struct FileHashes {
    pub algorithm: String,
    pub hashes: Vec<String>,
}

// under /api/v2/version_files
#[post("")]
pub async fn get_versions_from_hashes(
    pool: web::Data<PgPool>,
    file_data: web::Json<FileHashes>,
) -> Result<HttpResponse, ApiError> {
    let hashes_parsed: Vec<Vec<u8>> = file_data
        .hashes
        .iter()
        .map(|x| x.to_lowercase().as_bytes().to_vec())
        .collect();

    let result = sqlx::query!(
        "
        SELECT h.hash hash, h.algorithm algorithm, f.version_id version_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id AND v.status != ANY($1)
        INNER JOIN mods m on v.mod_id = m.id
        WHERE h.algorithm = $3 AND h.hash = ANY($2::bytea[]) AND m.status != ANY($4)
        ",
        &*crate::models::projects::VersionStatus::iterator().filter(|x| x.is_hidden()).map(|x| x.to_string()).collect::<Vec<String>>(),
        hashes_parsed.as_slice(),
        file_data.algorithm,
        &*crate::models::projects::ProjectStatus::iterator().filter(|x| x.is_hidden()).map(|x| x.to_string()).collect::<Vec<String>>(),
    )
    .fetch_all(&**pool)
    .await?;

    let versions_data = database::models::Version::get_many_full(
        result
            .iter()
            .map(|x| database::models::VersionId(x.version_id))
            .collect(),
        &**pool,
    )
    .await?;

    let response: Result<HashMap<String, Version>, ApiError> = result
        .into_iter()
        .filter_map(|row| {
            versions_data
                .clone()
                .into_iter()
                .find(|x| x.inner.id.0 == row.version_id)
                .map(|v| {
                    if let Ok(parsed_hash) = String::from_utf8(row.hash) {
                        Ok((
                            parsed_hash,
                            crate::models::projects::Version::from(v),
                        ))
                    } else {
                        Err(ApiError::Database(DatabaseError::Other(format!(
                            "Could not parse hash for version {}",
                            row.version_id
                        ))))
                    }
                })
        })
        .collect();
    Ok(HttpResponse::Ok().json(response?))
}

#[post("download")]
pub async fn download_files(
    pool: web::Data<PgPool>,
    file_data: web::Json<FileHashes>,
) -> Result<HttpResponse, ApiError> {
    let hashes_parsed: Vec<Vec<u8>> = file_data
        .hashes
        .iter()
        .map(|x| x.to_lowercase().as_bytes().to_vec())
        .collect();

    let mut transaction = pool.begin().await?;

    let result = sqlx::query!(
        "
        SELECT f.url url, h.hash hash, h.algorithm algorithm, f.version_id version_id, v.mod_id project_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id AND v.status != ANY($1)
        INNER JOIN mods m on v.mod_id = m.id
        WHERE h.algorithm = $3 AND h.hash = ANY($2::bytea[]) AND m.status != ANY($4)
        ",
        &*crate::models::projects::VersionStatus::iterator().filter(|x| x.is_hidden()).map(|x| x.to_string()).collect::<Vec<String>>(),
        hashes_parsed.as_slice(),
        file_data.algorithm,
        &*crate::models::projects::ProjectStatus::iterator().filter(|x| x.is_hidden()).map(|x| x.to_string()).collect::<Vec<String>>(),
    )
    .fetch_all(&mut *transaction)
    .await?;

    let response = result
        .into_iter()
        .map(|row| {
            if let Ok(parsed_hash) = String::from_utf8(row.hash) {
                Ok((parsed_hash, row.url))
            } else {
                Err(ApiError::Database(DatabaseError::Other(format!(
                    "Could not parse hash for version {}",
                    row.version_id
                ))))
            }
        })
        .collect::<Result<HashMap<String, String>, ApiError>>();

    Ok(HttpResponse::Ok().json(response?))
}

#[derive(Deserialize)]
pub struct ManyUpdateData {
    pub algorithm: String,
    pub hashes: Vec<String>,
    pub loaders: Vec<Loader>,
    pub game_versions: Vec<GameVersion>,
}

#[post("update")]
pub async fn update_files(
    pool: web::Data<PgPool>,
    update_data: web::Json<ManyUpdateData>,
) -> Result<HttpResponse, ApiError> {
    let hashes_parsed: Vec<Vec<u8>> = update_data
        .hashes
        .iter()
        .map(|x| x.to_lowercase().as_bytes().to_vec())
        .collect();

    let mut transaction = pool.begin().await?;

    let result = sqlx::query!(
        "
        SELECT h.hash, v.mod_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id AND v.status != ANY($1)
        INNER JOIN mods m on v.mod_id = m.id
        WHERE h.algorithm = $3 AND h.hash = ANY($2::bytea[]) AND m.status != ANY($4)
        ",
        &*crate::models::projects::VersionStatus::iterator().filter(|x| x.is_hidden()).map(|x| x.to_string()).collect::<Vec<String>>(),
        hashes_parsed.as_slice(),
        update_data.algorithm,
        &*crate::models::projects::ProjectStatus::iterator().filter(|x| x.is_hidden()).map(|x| x.to_string()).collect::<Vec<String>>(),
    )
        .fetch_many(&mut *transaction)
        .try_filter_map(|e| async {
            Ok(e.right().map(|m| (m.hash, database::models::ids::ProjectId(m.mod_id))))
        })
        .try_collect::<Vec<_>>()
        .await?;

    let mut version_ids: HashMap<database::models::VersionId, Vec<u8>> =
        HashMap::new();

    let updated_versions = database::models::Version::get_projects_versions(
        result
            .iter()
            .map(|x| x.1)
            .collect::<Vec<database::models::ProjectId>>()
            .clone(),
        Some(
            update_data
                .game_versions
                .clone()
                .iter()
                .map(|x| x.0.clone())
                .collect(),
        ),
        Some(
            update_data
                .loaders
                .clone()
                .iter()
                .map(|x| x.0.clone())
                .collect(),
        ),
        None,
        None,
        None,
        &**pool,
    )
    .await?;

    for (hash, id) in result {
        if let Some(latest_version) =
            updated_versions.get(&id).and_then(|x| x.last())
        {
            version_ids.insert(*latest_version, hash);
        }
    }

    let versions = database::models::Version::get_many_full(
        version_ids.keys().copied().collect(),
        &**pool,
    )
    .await?;

    let mut response = HashMap::new();

    for version in versions {
        let hash = version_ids.get(&version.inner.id);

        if let Some(hash) = hash {
            if let Ok(parsed_hash) = String::from_utf8(hash.clone()) {
                response.insert(
                    parsed_hash,
                    models::projects::Version::from(version),
                );
            } else {
                let version_id: VersionId = version.inner.id.into();

                return Err(ApiError::Database(DatabaseError::Other(format!(
                    "Could not parse hash for version {version_id}"
                ))));
            }
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

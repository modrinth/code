use super::ApiError;
use crate::database::models::{version_item::QueryVersion, DatabaseError};
use crate::file_hosting::FileHost;
use crate::models::projects::{GameVersion, Loader, Version};
use crate::models::teams::Permissions;
use crate::util::auth::get_user_from_headers;
use crate::util::routes::ok_or_not_found;
use crate::{database, models};
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Deserialize)]
pub struct Algorithm {
    #[serde(default = "default_algorithm")]
    algorithm: String,
}

fn default_algorithm() -> String {
    "sha1".into()
}

// under /api/v1/version_file/{hash}
#[get("{version_id}")]
pub async fn get_version_from_hash(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    algorithm: web::Query<Algorithm>,
) -> Result<HttpResponse, ApiError> {
    let hash = info.into_inner().0.to_lowercase();

    let result = sqlx::query!(
        "
        SELECT f.version_id version_id
        FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v on f.version_id = v.id
        INNER JOIN mods m on v.mod_id = m.id
        INNER JOIN statuses s on m.status = s.id
        WHERE h.algorithm = $2 AND h.hash = $1 AND s.status != $3
        ",
        hash.as_bytes(),
        algorithm.algorithm,
        models::projects::ProjectStatus::Rejected.to_string()
    )
    .fetch_optional(&**pool)
    .await?;

    if let Some(id) = result {
        let version_data = database::models::Version::get_full(
            database::models::VersionId(id.version_id),
            &**pool,
        )
        .await?;

        if let Some(data) = version_data {
            Ok(HttpResponse::Ok().json(models::projects::Version::from(data)))
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
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    algorithm: web::Query<Algorithm>,
) -> Result<HttpResponse, ApiError> {
    let hash = info.into_inner().0.to_lowercase();
    let mut transaction = pool.begin().await?;

    let result = sqlx::query!(
        "
        SELECT f.url url, f.id id, f.version_id version_id, v.mod_id project_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id
        INNER JOIN mods m on v.mod_id = m.id
        INNER JOIN statuses s on m.status = s.id
        WHERE h.algorithm = $2 AND h.hash = $1 AND s.status != $3
        ",
        hash.as_bytes(),
        algorithm.algorithm,
        models::projects::ProjectStatus::Rejected.to_string()
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
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    algorithm: web::Query<Algorithm>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    let hash = info.into_inner().0.to_lowercase();

    let result = sqlx::query!(
        "
        SELECT f.id id, f.version_id version_id, f.filename filename, v.version_number version_number, v.mod_id project_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id
        WHERE h.algorithm = $2 AND h.hash = $1
        ",
        hash.as_bytes(),
        algorithm.algorithm
    )
        .fetch_optional(&**pool)
        .await
        ?;

    if let Some(row) = result {
        if !user.role.is_mod() {
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

        let project_id: models::projects::ProjectId =
            database::models::ids::ProjectId(row.project_id).into();
        file_host
            .delete_file_version(
                "",
                &format!(
                    "data/{}/versions/{}/{}",
                    project_id, row.version_number, row.filename
                ),
            )
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
    algorithm: web::Query<Algorithm>,
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
        INNER JOIN versions v ON v.id = f.version_id
        INNER JOIN mods m on v.mod_id = m.id
        INNER JOIN statuses s on m.status = s.id
        WHERE h.algorithm = $2 AND h.hash = $1 AND s.status != $3
        ",
        hash.as_bytes(),
        algorithm.algorithm,
        models::projects::ProjectStatus::Rejected.to_string()
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
            &**pool,
        )
        .await?;

        if let Some(version_id) = version_ids.last() {
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
        INNER JOIN versions v ON v.id = f.version_id
        INNER JOIN mods m on v.mod_id = m.id
        INNER JOIN statuses s on m.status = s.id
        WHERE h.algorithm = $2 AND h.hash = ANY($1::bytea[]) AND s.status != $3
        ",
        hashes_parsed.as_slice(),
        file_data.algorithm,
        models::projects::ProjectStatus::Rejected.to_string()
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
                .find(|x| x.id.0 == row.version_id)
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
        INNER JOIN versions v ON v.id = f.version_id
        INNER JOIN mods m on v.mod_id = m.id
        INNER JOIN statuses s on m.status = s.id
        WHERE h.algorithm = $2 AND h.hash = ANY($1::bytea[]) AND s.status != $3
        ",
        hashes_parsed.as_slice(),
        file_data.algorithm,
        models::projects::ProjectStatus::Rejected.to_string()
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
        SELECT f.url url, h.hash hash, h.algorithm algorithm, f.version_id version_id, v.mod_id project_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id
        INNER JOIN mods m on v.mod_id = m.id
        INNER JOIN statuses s on m.status = s.id
        WHERE h.algorithm = $2 AND h.hash = ANY($1::bytea[]) AND s.status != $3
        ",
        hashes_parsed.as_slice(),
        update_data.algorithm,
        models::projects::ProjectStatus::Rejected.to_string()
    )
        .fetch_all(&mut *transaction)
        .await?;

    let version_ids: RwLock<HashMap<database::models::VersionId, Vec<u8>>> =
        RwLock::new(HashMap::new());

    futures::future::try_join_all(result.into_iter().map(|row| async {
        let updated_versions = database::models::Version::get_project_versions(
            database::models::ProjectId(row.project_id),
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
            &**pool,
        )
        .await?;

        if let Some(latest_version) = updated_versions.last() {
            let mut version_ids = version_ids.write().await;

            version_ids.insert(*latest_version, row.hash);
        }

        Ok::<(), ApiError>(())
    }))
    .await?;

    let version_ids = version_ids.into_inner();

    let versions = database::models::Version::get_many_full(
        version_ids.keys().copied().collect(),
        &**pool,
    )
    .await?;

    let mut response = HashMap::new();

    for version in versions {
        let hash = version_ids.get(&version.id);

        if let Some(hash) = hash {
            if let Ok(parsed_hash) = String::from_utf8(hash.clone()) {
                response.insert(
                    parsed_hash,
                    models::projects::Version::from(version),
                );
            } else {
                let version_id: models::projects::VersionId = version.id.into();

                return Err(ApiError::Database(DatabaseError::Other(format!(
                    "Could not parse hash for version {}",
                    version_id
                ))));
            }
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

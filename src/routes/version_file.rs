use super::ApiError;
use crate::database::models::version_item::QueryVersion;
use crate::file_hosting::FileHost;
use crate::models;
use crate::models::projects::{GameVersion, Loader, Version};
use crate::models::teams::Permissions;
use crate::util::auth::get_user_from_headers;
use crate::util::routes::ok_or_not_found;
use crate::{database, Pepper};
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::Arc;

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
        SELECT f.version_id version_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        WHERE h.algorithm = $2 AND h.hash = $1
        ",
        hash.as_bytes(),
        algorithm.algorithm
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
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    algorithm: web::Query<Algorithm>,
    pepper: web::Data<Pepper>,
) -> Result<HttpResponse, ApiError> {
    let hash = info.into_inner().0.to_lowercase();
    let mut transaction = pool.begin().await?;

    let result = sqlx::query!(
        "
        SELECT f.url url, f.id id, f.version_id version_id, v.mod_id project_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id
        WHERE h.algorithm = $2 AND h.hash = $1
        ",
        hash.as_bytes(),
        algorithm.algorithm
    )
    .fetch_optional(&mut *transaction)
    .await?;

    if let Some(id) = result {
        download_version_inner(
            database::models::VersionId(id.version_id),
            database::models::ProjectId(id.project_id),
            &req,
            &mut transaction,
            &pepper,
        )
        .await?;

        transaction.commit().await?;

        Ok(HttpResponse::TemporaryRedirect()
            .append_header(("Location", &*id.url))
            .json(DownloadRedirect { url: id.url }))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

async fn download_version_inner(
    version_id: database::models::VersionId,
    project_id: database::models::ProjectId,
    req: &HttpRequest,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    pepper: &web::Data<Pepper>,
) -> Result<(), ApiError> {
    let real_ip = req.connection_info();
    let ip_option = if dotenv::var("CLOUDFLARE_INTEGRATION")
        .ok()
        .map(|i| i.parse().unwrap())
        .unwrap_or(false)
    {
        if let Some(header) = req.headers().get("CF-Connecting-IP") {
            header.to_str().ok()
        } else {
            real_ip.borrow().peer_addr()
        }
    } else {
        real_ip.borrow().peer_addr()
    };

    if let Some(ip) = ip_option {
        let hash = sha1::Sha1::from(format!("{}{}", ip, pepper.pepper)).hexdigest();

        let download_exists = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM downloads WHERE version_id = $1 AND date > (CURRENT_DATE - INTERVAL '30 minutes ago') AND identifier = $2)",
                version_id as database::models::VersionId,
                hash,
            )
            .fetch_one(&mut *transaction)
            .await
            ?
            .exists.unwrap_or(false);

        if !download_exists {
            sqlx::query!(
                "
                    INSERT INTO downloads (
                        version_id, identifier
                    )
                    VALUES (
                        $1, $2
                    )
                    ",
                version_id as database::models::VersionId,
                hash
            )
            .execute(&mut *transaction)
            .await?;

            sqlx::query!(
                "
                    UPDATE versions
                    SET downloads = downloads + 1
                    WHERE id = $1
                    ",
                version_id as database::models::VersionId,
            )
            .execute(&mut *transaction)
            .await?;

            sqlx::query!(
                "
                    UPDATE mods
                    SET downloads = downloads + 1
                    WHERE id = $1
                    ",
                project_id as database::models::ProjectId,
            )
            .execute(&mut *transaction)
            .await?;
        }
    }

    Ok(())
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
            let team_member = database::models::TeamMember::get_from_user_id_version(
                database::models::ids::VersionId(row.version_id),
                user.id.into(),
                &**pool,
            )
            .await
            .map_err(ApiError::DatabaseError)?
            .ok_or_else(|| {
                ApiError::CustomAuthenticationError(
                    "You don't have permission to delete this file!".to_string(),
                )
            })?;

            if !team_member
                .permissions
                .contains(Permissions::DELETE_VERSION)
            {
                return Err(ApiError::CustomAuthenticationError(
                    "You don't have permission to delete this file!".to_string(),
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
            return Err(ApiError::InvalidInputError(
                "Versions must have at least one file uploaded to them".to_string(),
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
        WHERE h.algorithm = $2 AND h.hash = $1
        ",
        hash.as_bytes(),
        algorithm.algorithm
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
            let version_data = database::models::Version::get_full(*version_id, &**pool).await?;

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
#[post("/")]
pub async fn get_versions_from_hashes(
    pool: web::Data<PgPool>,
    file_data: web::Json<FileHashes>,
) -> Result<HttpResponse, ApiError> {
    let hashes_parsed: Vec<Vec<u8>> = file_data
        .hashes
        .iter()
        .map(|x| x.as_bytes().to_vec())
        .collect();

    let result = sqlx::query!(
        "
        SELECT h.hash hash, h.algorithm algorithm, f.version_id version_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        WHERE h.algorithm = $2 AND h.hash = ANY($1::bytea[])
        ",
        hashes_parsed.as_slice(),
        file_data.algorithm
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

    let response: Vec<_> = result
        .into_iter()
        .filter_map(|row| {
            versions_data
                .clone()
                .into_iter()
                .find(|x| x.id.0 == row.version_id)
                .map(|v| (row.hash, crate::models::projects::Version::from(v)))
        })
        .collect();
    Ok(HttpResponse::Ok().json(response))
}

#[post("download")]
pub async fn download_files(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    file_data: web::Json<FileHashes>,
    pepper: web::Data<Pepper>,
) -> Result<HttpResponse, ApiError> {
    let hashes_parsed: Vec<Vec<u8>> = file_data
        .hashes
        .iter()
        .map(|x| x.as_bytes().to_vec())
        .collect();

    let mut transaction = pool.begin().await?;

    let result = sqlx::query!(
        "
        SELECT f.url url, h.hash hash, h.algorithm algorithm, f.version_id version_id, v.mod_id project_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id
        WHERE h.algorithm = $2 AND h.hash = ANY($1::bytea[])
        ",
        hashes_parsed.as_slice(),
        file_data.algorithm
    )
    .fetch_all(&mut *transaction)
    .await?;

    let mut response = HashMap::new();

    for row in result {
        download_version_inner(
            database::models::VersionId(row.version_id),
            database::models::ProjectId(row.project_id),
            &req,
            &mut transaction,
            &pepper,
        )
        .await?;
        response.insert(row.hash, row.url);
    }

    Ok(HttpResponse::Ok().json(response))
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
        .map(|x| x.as_bytes().to_vec())
        .collect();

    let mut transaction = pool.begin().await?;

    let result = sqlx::query!(
        "
        SELECT f.url url, h.hash hash, h.algorithm algorithm, f.version_id version_id, v.mod_id project_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id
        WHERE h.algorithm = $2 AND h.hash = ANY($1::bytea[])
        ",
        hashes_parsed.as_slice(),
        update_data.algorithm
    )
        .fetch_all(&mut *transaction)
        .await?;

    let mut version_ids = Vec::new();

    for row in &result {
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
            version_ids.push(*latest_version);
        }
    }

    let versions = database::models::Version::get_many_full(version_ids, &**pool).await?;

    let mut response = HashMap::new();

    for row in &result {
        if let Some(version) = versions.iter().find(|x| x.id.0 == row.version_id) {
            response.insert(
                row.hash.clone(),
                models::projects::Version::from(version.clone()),
            );
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

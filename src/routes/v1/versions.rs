use crate::file_hosting::FileHost;
use crate::models::ids::{ProjectId, UserId, VersionId};
use crate::models::projects::{
    Dependency, GameVersion, Loader, Version, VersionFile, VersionType,
};
use crate::models::teams::Permissions;
use crate::routes::versions::{VersionIds, VersionListFilters};
use crate::routes::ApiError;
use crate::util::auth::get_user_from_headers;
use crate::{database, models};
use actix_web::{delete, get, web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

/// A specific version of a mod
#[derive(Serialize, Deserialize)]
pub struct LegacyVersion {
    pub id: VersionId,
    pub mod_id: ProjectId,
    pub author_id: UserId,
    pub featured: bool,
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub changelog_url: Option<String>,
    pub date_published: DateTime<Utc>,
    pub downloads: u32,
    pub version_type: VersionType,
    pub files: Vec<VersionFile>,
    pub dependencies: Vec<Dependency>,
    pub game_versions: Vec<GameVersion>,
    pub loaders: Vec<Loader>,
}

fn convert_to_legacy(version: Version) -> LegacyVersion {
    LegacyVersion {
        id: version.id,
        mod_id: version.project_id,
        author_id: version.author_id,
        featured: version.featured,
        name: version.name,
        version_number: version.version_number,
        changelog: version.changelog,
        changelog_url: version.changelog_url,
        date_published: version.date_published,
        downloads: version.downloads,
        version_type: version.version_type,
        files: version.files,
        dependencies: version.dependencies,
        game_versions: version.game_versions,
        loaders: version.loaders,
    }
}

#[get("version")]
pub async fn version_list(
    info: web::Path<(String,)>,
    web::Query(filters): web::Query<VersionListFilters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;

    let result = database::models::Project::get_from_slug_or_project_id(
        &string, &**pool,
    )
    .await?;

    if let Some(project) = result {
        let id = project.id;

        let version_ids = database::models::Version::get_project_versions(
            id,
            filters
                .game_versions
                .as_ref()
                .map(|x| serde_json::from_str(x).unwrap_or_default()),
            filters
                .loaders
                .as_ref()
                .map(|x| serde_json::from_str(x).unwrap_or_default()),
            &**pool,
        )
        .await?;

        let mut versions =
            database::models::Version::get_many_full(version_ids, &**pool)
                .await?;

        let mut response = versions
            .iter()
            .cloned()
            .filter(|version| {
                filters
                    .featured
                    .map(|featured| featured == version.featured)
                    .unwrap_or(true)
            })
            .map(Version::from)
            .map(convert_to_legacy)
            .collect::<Vec<_>>();

        versions.sort_by(|a, b| b.date_published.cmp(&a.date_published));

        // Attempt to populate versions with "auto featured" versions
        if response.is_empty()
            && !versions.is_empty()
            && filters.featured.unwrap_or(false)
        {
            let loaders =
                database::models::categories::Loader::list(&**pool).await?;
            let game_versions =
                database::models::categories::GameVersion::list_filter(
                    None,
                    Some(true),
                    &**pool,
                )
                .await?;

            let mut joined_filters = Vec::new();
            for game_version in &game_versions {
                for loader in &loaders {
                    joined_filters.push((game_version, loader))
                }
            }

            joined_filters.into_iter().for_each(|filter| {
                versions
                    .iter()
                    .find(|version| {
                        version.game_versions.contains(&filter.0.version)
                            && version.loaders.contains(&filter.1.loader)
                    })
                    .map(|version| {
                        response.push(convert_to_legacy(Version::from(
                            version.clone(),
                        )))
                    })
                    .unwrap_or(());
            });

            if response.is_empty() {
                versions.into_iter().for_each(|version| {
                    response.push(convert_to_legacy(Version::from(version)))
                });
            }
        }

        response.sort_by(|a, b| b.date_published.cmp(&a.date_published));
        response.dedup_by(|a, b| a.id == b.id);

        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("versions")]
pub async fn versions_get(
    ids: web::Query<VersionIds>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let version_ids = serde_json::from_str::<Vec<VersionId>>(&*ids.ids)?
        .into_iter()
        .map(|x| x.into())
        .collect();
    let versions_data =
        database::models::Version::get_many_full(version_ids, &**pool).await?;

    let mut versions = Vec::new();

    for version_data in versions_data {
        versions.push(convert_to_legacy(Version::from(version_data)));
    }

    Ok(HttpResponse::Ok().json(versions))
}

#[get("{version_id}")]
pub async fn version_get(
    info: web::Path<(VersionId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let version_data =
        database::models::Version::get_full(id.into(), &**pool).await?;

    if let Some(data) = version_data {
        Ok(HttpResponse::Ok().json(convert_to_legacy(Version::from(data))))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

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
            Ok(HttpResponse::Ok()
                .json(crate::models::projects::Version::from(data)))
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
#[allow(clippy::await_holding_refcell_ref)]
#[get("{version_id}/download")]
pub async fn download_version(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    algorithm: web::Query<Algorithm>,
) -> Result<HttpResponse, ApiError> {
    let hash = info.into_inner().0;

    let result = sqlx::query!(
        "
        SELECT f.url url, f.id id, f.version_id version_id, v.mod_id mod_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id
        WHERE h.algorithm = $2 AND h.hash = $1
        ",
        hash.as_bytes(),
        algorithm.algorithm
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|e| ApiError::Database(e.into()))?;

    if let Some(id) = result {
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

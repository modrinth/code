use crate::database;
use crate::models::ids::{ProjectId, UserId, VersionId};
use crate::models::projects::{
    Dependency, GameVersion, Loader, Version, VersionFile, VersionType,
};
use crate::routes::versions::{VersionIds, VersionListFilters};
use crate::routes::ApiError;
use actix_web::{get, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

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
        name: format!("[STOP USING API v1] {}", version.name),
        version_number: version.version_number,
        changelog: format!("# STOP USING API v1 - whatever application you're using right now is likely deprecated or abandoned\n{}", version.changelog),
        changelog_url: None,
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
            filters.version_type,
            filters.limit,
            filters.offset,
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
                    .map(|featured| featured == version.inner.featured)
                    .unwrap_or(true)
            })
            .map(Version::from)
            .map(convert_to_legacy)
            .collect::<Vec<_>>();

        versions.sort_by(|a, b| {
            b.inner.date_published.cmp(&a.inner.date_published)
        });

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
    let version_ids = serde_json::from_str::<Vec<VersionId>>(&ids.ids)?
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

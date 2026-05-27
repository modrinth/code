use std::collections::HashSet;

use actix_web::{HttpRequest, post, web};
use serde::Serialize;

use super::{DownloadSource, GetRequest, normalize_download_source};
use crate::{
    auth::get_user_from_headers,
    database::{
        PgPool,
        models::{DBProjectId, DBUser, DBVersionId},
        redis::RedisPool,
    },
    models::{ids::VersionId, pats::Scopes, v3::analytics::DownloadReason},
    queue::session::AuthQueue,
    routes::ApiError,
};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(fetch_facets);
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FacetsResponse {
    pub facets: AnalyticsFacets,
}

#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
pub struct AnalyticsFacets {
    pub project_views: ProjectViewsFacets,
    pub project_downloads: ProjectDownloadsFacets,
    pub project_playtime: ProjectPlaytimeFacets,
}

#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
pub struct ProjectViewsFacets {
    pub domain: Vec<String>,
    pub site_path: Vec<String>,
    pub monetized: Vec<bool>,
    pub country: Vec<String>,
}

#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
pub struct ProjectDownloadsFacets {
    pub domain: Vec<String>,
    pub user_agent: Vec<DownloadSource>,
    pub version_id: Vec<VersionId>,
    pub monetized: Vec<bool>,
    pub country: Vec<String>,
    pub reason: Vec<DownloadReason>,
    pub game_version: Vec<String>,
    pub loader: Vec<String>,
}

#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
pub struct ProjectPlaytimeFacets {
    pub version_id: Vec<VersionId>,
    pub loader: Vec<String>,
    pub game_version: Vec<String>,
    pub country: Vec<String>,
}

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct StringFacetRow {
    value: String,
}

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct VersionFacetRow {
    value: DBVersionId,
}

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct BoolFacetRow {
    value: bool,
}

#[utoipa::path(
    responses((status = OK, body = inline(FacetsResponse))),
)]
#[post("/facets")]
pub async fn fetch_facets(
    http_req: HttpRequest,
    req: web::Json<GetRequest>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    clickhouse: web::Data<clickhouse::Client>,
) -> Result<web::Json<FacetsResponse>, ApiError> {
    let user = get_user_from_headers(
        &http_req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::ANALYTICS,
    )
    .await?
    .1;

    let project_ids = if req.project_ids.is_empty() {
        DBUser::get_projects(user.id.into(), &**pool, &redis).await?
    } else {
        req.project_ids
            .iter()
            .map(|id| DBProjectId::from(*id))
            .collect::<Vec<_>>()
    };
    let project_ids =
        super::filter_allowed_project_ids(&project_ids, &user, &pool, &redis)
            .await?;

    let parent_version_ids =
        fetch_project_version_ids(&project_ids, &pool).await?;

    Ok(web::Json(FacetsResponse {
        facets: AnalyticsFacets {
            project_views: fetch_project_views_facets(
                &clickhouse,
                &project_ids,
            )
            .await?,
            project_downloads: fetch_project_downloads_facets(
                &clickhouse,
                &project_ids,
            )
            .await?,
            project_playtime: fetch_project_playtime_facets(
                &clickhouse,
                &project_ids,
                &parent_version_ids,
            )
            .await?,
        },
    }))
}

async fn fetch_project_version_ids(
    project_ids: &[DBProjectId],
    pool: &PgPool,
) -> Result<Vec<DBVersionId>, ApiError> {
    let project_id_values =
        project_ids.iter().map(|id| id.0).collect::<Vec<_>>();
    Ok(sqlx::query!(
        "
        SELECT id
        FROM versions
        WHERE mod_id = ANY($1)
        ",
        &project_id_values,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| DBVersionId(row.id))
    .collect())
}

async fn fetch_project_views_facets(
    clickhouse: &clickhouse::Client,
    project_ids: &[DBProjectId],
) -> Result<ProjectViewsFacets, ApiError> {
    Ok(ProjectViewsFacets {
        domain: fetch_string_facet(
            clickhouse,
            "SELECT DISTINCT domain AS value FROM views WHERE project_id IN {project_ids: Array(UInt64)} AND domain != '' ORDER BY value",
            project_ids,
        )
        .await?,
        site_path: fetch_string_facet(
            clickhouse,
            "SELECT DISTINCT site_path AS value FROM views WHERE project_id IN {project_ids: Array(UInt64)} AND site_path != '' ORDER BY value",
            project_ids,
        )
        .await?,
        monetized: fetch_bool_facet(
            clickhouse,
            "SELECT DISTINCT monetized AS value FROM views WHERE project_id IN {project_ids: Array(UInt64)} ORDER BY value",
            project_ids,
        )
        .await?,
        country: fetch_string_facet(
            clickhouse,
            "SELECT DISTINCT country AS value FROM views WHERE project_id IN {project_ids: Array(UInt64)} AND country != '' ORDER BY value",
            project_ids,
        )
        .await?,
    })
}

async fn fetch_project_downloads_facets(
    clickhouse: &clickhouse::Client,
    project_ids: &[DBProjectId],
) -> Result<ProjectDownloadsFacets, ApiError> {
    let user_agents = fetch_string_facet(
        clickhouse,
        "SELECT DISTINCT user_agent AS value FROM downloads WHERE project_id IN {project_ids: Array(UInt64)} AND user_agent != ''",
        project_ids,
    )
    .await?;
    let user_agent = normalize_download_source_facets(&user_agents);

    Ok(ProjectDownloadsFacets {
        domain: fetch_string_facet(
            clickhouse,
            "SELECT DISTINCT domain AS value FROM downloads WHERE project_id IN {project_ids: Array(UInt64)} AND domain != '' ORDER BY value",
            project_ids,
        )
        .await?,
        user_agent,
        version_id: fetch_version_facet(
            clickhouse,
            "SELECT DISTINCT version_id AS value FROM downloads WHERE project_id IN {project_ids: Array(UInt64)} AND version_id != 0 ORDER BY value",
            project_ids,
        )
        .await?,
        monetized: fetch_bool_facet(
            clickhouse,
            "SELECT DISTINCT user_id != 0 AS value FROM downloads WHERE project_id IN {project_ids: Array(UInt64)} ORDER BY value",
            project_ids,
        )
        .await?,
        country: fetch_string_facet(
            clickhouse,
            "SELECT DISTINCT country AS value FROM downloads WHERE project_id IN {project_ids: Array(UInt64)} AND country != '' ORDER BY value",
            project_ids,
        )
        .await?,
        reason: fetch_string_facet(
            clickhouse,
            "SELECT DISTINCT reason AS value FROM downloads WHERE project_id IN {project_ids: Array(UInt64)} AND reason != '' ORDER BY value",
            project_ids,
        )
        .await?
        .into_iter()
        .filter_map(|reason| reason.parse().ok())
        .collect(),
        game_version: fetch_string_facet(
            clickhouse,
            "SELECT DISTINCT game_version AS value FROM downloads WHERE project_id IN {project_ids: Array(UInt64)} AND game_version != '' ORDER BY value",
            project_ids,
        )
        .await?,
        loader: fetch_string_facet(
            clickhouse,
            "SELECT DISTINCT loader AS value FROM downloads WHERE project_id IN {project_ids: Array(UInt64)} AND loader != '' ORDER BY value",
            project_ids,
        )
        .await?,
    })
}

fn normalize_download_source_facets(
    user_agents: &[String],
) -> Vec<DownloadSource> {
    let mut sources = user_agents
        .iter()
        .filter_map(|user_agent| normalize_download_source(user_agent))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    sources.sort_by(|a, b| {
        download_source_sort_key(a).cmp(download_source_sort_key(b))
    });
    sources
}

fn download_source_sort_key(source: &DownloadSource) -> &str {
    match source {
        DownloadSource::Named(name) => name,
        DownloadSource::Website => "website",
        DownloadSource::ModrinthApp => "modrinth_app",
        DownloadSource::ModrinthHosting => "modrinth_hosting",
        DownloadSource::ModrinthMaven => "modrinth_maven",
        DownloadSource::Other => "other",
    }
}

async fn fetch_project_playtime_facets(
    clickhouse: &clickhouse::Client,
    project_ids: &[DBProjectId],
    parent_version_ids: &[DBVersionId],
) -> Result<ProjectPlaytimeFacets, ApiError> {
    Ok(ProjectPlaytimeFacets {
        version_id: fetch_playtime_version_facet(
            clickhouse,
            project_ids,
            parent_version_ids,
        )
        .await?,
        loader: fetch_playtime_string_facet(
            clickhouse,
            "loader",
            project_ids,
            parent_version_ids,
        )
        .await?,
        game_version: fetch_playtime_string_facet(
            clickhouse,
            "game_version",
            project_ids,
            parent_version_ids,
        )
        .await?,
        country: fetch_playtime_string_facet(
            clickhouse,
            "country",
            project_ids,
            parent_version_ids,
        )
        .await?,
    })
}

async fn fetch_string_facet(
    clickhouse: &clickhouse::Client,
    query: &str,
    project_ids: &[DBProjectId],
) -> Result<Vec<String>, ApiError> {
    let mut rows = clickhouse
        .query(query)
        .param("project_ids", project_ids)
        .fetch::<StringFacetRow>()?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(row.value);
    }
    Ok(values)
}

async fn fetch_version_facet(
    clickhouse: &clickhouse::Client,
    query: &str,
    project_ids: &[DBProjectId],
) -> Result<Vec<VersionId>, ApiError> {
    let mut rows = clickhouse
        .query(query)
        .param("project_ids", project_ids)
        .fetch::<VersionFacetRow>()?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(row.value.into());
    }
    Ok(values)
}

async fn fetch_bool_facet(
    clickhouse: &clickhouse::Client,
    query: &str,
    project_ids: &[DBProjectId],
) -> Result<Vec<bool>, ApiError> {
    let mut rows = clickhouse
        .query(query)
        .param("project_ids", project_ids)
        .fetch::<BoolFacetRow>()?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(row.value);
    }
    Ok(values)
}

async fn fetch_playtime_string_facet(
    clickhouse: &clickhouse::Client,
    column: &str,
    project_ids: &[DBProjectId],
    parent_version_ids: &[DBVersionId],
) -> Result<Vec<String>, ApiError> {
    let query = format!(
        "SELECT DISTINCT {column} AS value
        FROM playtime
        WHERE (project_id IN {{project_ids: Array(UInt64)}} OR parent IN {{parent_version_ids: Array(UInt64)}})
            AND {column} != ''
        ORDER BY value"
    );
    let mut rows = clickhouse
        .query(&query)
        .param("project_ids", project_ids)
        .param("parent_version_ids", parent_version_ids)
        .fetch::<StringFacetRow>()?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(row.value);
    }
    Ok(values)
}

async fn fetch_playtime_version_facet(
    clickhouse: &clickhouse::Client,
    project_ids: &[DBProjectId],
    parent_version_ids: &[DBVersionId],
) -> Result<Vec<VersionId>, ApiError> {
    let mut rows = clickhouse
        .query(
            "SELECT DISTINCT version_id AS value
            FROM playtime
            WHERE (project_id IN {project_ids: Array(UInt64)} OR parent IN {parent_version_ids: Array(UInt64)})
                AND version_id != 0
            ORDER BY value",
        )
        .param("project_ids", project_ids)
        .param("parent_version_ids", parent_version_ids)
        .fetch::<VersionFacetRow>()?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(row.value.into());
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_agent_facets_use_normalized_sources() {
        let user_agents = vec![
            "MultiMC/5.0".to_string(),
            "MultiMC/6.0".to_string(),
            "PrismLauncher/6.1".to_string(),
            "curl/8.7.1".to_string(),
            "Mozilla/5.0 AppleWebKit/537.36".to_string(),
        ];

        assert_eq!(
            normalize_download_source_facets(&user_agents),
            vec![
                DownloadSource::Named("MultiMC".into()),
                DownloadSource::Named("Prism Launcher".into()),
                DownloadSource::Website,
            ],
        );
    }
}

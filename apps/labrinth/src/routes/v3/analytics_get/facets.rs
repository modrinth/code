use std::collections::HashMap;

use actix_web::{HttpRequest, post, web};
use serde::Serialize;

use super::{DownloadSource, GetRequest, TimeRange, normalize_download_source};
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
    pub domain: Vec<FacetValue<String>>,
    pub site_path: Vec<FacetValue<String>>,
    pub monetized: Vec<FacetValue<bool>>,
    pub country: Vec<FacetValue<String>>,
}

#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
pub struct ProjectDownloadsFacets {
    pub domain: Vec<FacetValue<String>>,
    pub user_agent: Vec<FacetValue<DownloadSource>>,
    pub version_id: Vec<FacetValue<VersionId>>,
    pub monetized: Vec<FacetValue<bool>>,
    pub country: Vec<FacetValue<String>>,
    pub reason: Vec<FacetValue<DownloadReason>>,
    pub game_version: Vec<FacetValue<String>>,
    pub loader: Vec<FacetValue<String>>,
}

#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
pub struct ProjectPlaytimeFacets {
    pub version_id: Vec<FacetValue<VersionId>>,
    pub loader: Vec<FacetValue<String>>,
    pub game_version: Vec<FacetValue<String>>,
    pub country: Vec<FacetValue<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, utoipa::ToSchema)]
pub struct FacetValue<T> {
    pub value: T,
    pub count: u64,
}

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct StringFacetRow {
    value: String,
    count: u64,
}

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct VersionFacetRow {
    value: DBVersionId,
    count: u64,
}

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct BoolFacetRow {
    value: bool,
    count: u64,
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
                &req.time_range,
            )
            .await?,
            project_downloads: fetch_project_downloads_facets(
                &clickhouse,
                &project_ids,
                &req.time_range,
            )
            .await?,
            project_playtime: fetch_project_playtime_facets(
                &clickhouse,
                &project_ids,
                &parent_version_ids,
                &req.time_range,
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
    time_range: &TimeRange,
) -> Result<ProjectViewsFacets, ApiError> {
    Ok(ProjectViewsFacets {
        domain: fetch_string_facet(
            clickhouse,
            "SELECT domain AS value, COUNT(*) AS count FROM views WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND domain != '' GROUP BY value ORDER BY value",
            project_ids,
            time_range,
        )
        .await?,
        site_path: fetch_string_facet(
            clickhouse,
            "SELECT site_path AS value, COUNT(*) AS count FROM views WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND site_path != '' GROUP BY value ORDER BY value",
            project_ids,
            time_range,
        )
        .await?,
        monetized: fetch_bool_facet(
            clickhouse,
            "SELECT monetized AS value, COUNT(*) AS count FROM views WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} GROUP BY value ORDER BY value",
            project_ids,
            time_range,
        )
        .await?,
        country: fetch_string_facet(
            clickhouse,
            "SELECT country AS value, COUNT(*) AS count FROM views WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND country != '' GROUP BY value ORDER BY value",
            project_ids,
            time_range,
        )
		.await?,
	})
}

async fn fetch_project_downloads_facets(
    clickhouse: &clickhouse::Client,
    project_ids: &[DBProjectId],
    time_range: &TimeRange,
) -> Result<ProjectDownloadsFacets, ApiError> {
    let user_agents = fetch_string_facet(
        clickhouse,
        "SELECT user_agent AS value, COUNT(*) AS count FROM downloads WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND user_agent != '' GROUP BY value",
        project_ids,
        time_range,
    )
	.await?;
    let user_agent = normalize_download_source_facets(&user_agents);

    Ok(ProjectDownloadsFacets {
        domain: fetch_string_facet(
            clickhouse,
            "SELECT domain AS value, COUNT(*) AS count FROM downloads WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND domain != '' GROUP BY value ORDER BY value",
            project_ids,
            time_range,
        )
		.await?,
		user_agent,
        version_id: fetch_version_facet(
            clickhouse,
            "SELECT version_id AS value, COUNT(*) AS count FROM downloads WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND version_id != 0 GROUP BY value ORDER BY value",
            project_ids,
            time_range,
        )
		.await?,
        monetized: fetch_bool_facet(
            clickhouse,
            "SELECT user_id != 0 AS value, COUNT(*) AS count FROM downloads WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} GROUP BY value ORDER BY value",
            project_ids,
            time_range,
        )
		.await?,
        country: fetch_string_facet(
            clickhouse,
            "SELECT country AS value, COUNT(*) AS count FROM downloads WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND country != '' GROUP BY value ORDER BY value",
            project_ids,
            time_range,
        )
		.await?,
        reason: fetch_string_facet(
            clickhouse,
            "SELECT reason AS value, COUNT(*) AS count FROM downloads WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND reason != '' GROUP BY value ORDER BY value",
            project_ids,
            time_range,
        )
        .await?
        .into_iter()
        .filter_map(|reason| {
            reason.value.parse().ok().map(|value| FacetValue {
                value,
                count: reason.count,
            })
        })
        .collect(),
        game_version: fetch_string_facet(
            clickhouse,
            "SELECT game_version AS value, COUNT(*) AS count FROM downloads WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND game_version != '' GROUP BY value ORDER BY value",
            project_ids,
            time_range,
        )
		.await?,
        loader: fetch_string_facet(
            clickhouse,
            "SELECT loader AS value, COUNT(*) AS count FROM downloads WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND loader != '' GROUP BY value ORDER BY value",
            project_ids,
            time_range,
        )
		.await?,
	})
}

fn normalize_download_source_facets(
    user_agents: &[FacetValue<String>],
) -> Vec<FacetValue<DownloadSource>> {
    let mut counts = HashMap::<DownloadSource, u64>::new();
    for user_agent in user_agents {
        if let Some(source) = normalize_download_source(&user_agent.value) {
            *counts.entry(source).or_default() += user_agent.count;
        }
    }

    let mut sources = counts
        .into_iter()
        .map(|(value, count)| FacetValue { value, count })
        .collect::<Vec<_>>();
    sources.sort_by(|a, b| {
        download_source_sort_key(&a.value)
            .cmp(download_source_sort_key(&b.value))
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
    time_range: &TimeRange,
) -> Result<ProjectPlaytimeFacets, ApiError> {
    Ok(ProjectPlaytimeFacets {
        version_id: fetch_playtime_version_facet(
            clickhouse,
            project_ids,
            parent_version_ids,
            time_range,
        )
        .await?,
        loader: fetch_playtime_string_facet(
            clickhouse,
            "loader",
            project_ids,
            parent_version_ids,
            time_range,
        )
        .await?,
        game_version: fetch_playtime_string_facet(
            clickhouse,
            "game_version",
            project_ids,
            parent_version_ids,
            time_range,
        )
        .await?,
        country: fetch_playtime_string_facet(
            clickhouse,
            "country",
            project_ids,
            parent_version_ids,
            time_range,
        )
        .await?,
    })
}

async fn fetch_string_facet(
    clickhouse: &clickhouse::Client,
    query: &str,
    project_ids: &[DBProjectId],
    time_range: &TimeRange,
) -> Result<Vec<FacetValue<String>>, ApiError> {
    let mut rows = clickhouse
        .query(query)
        .param("time_range_start", time_range.start.timestamp())
        .param("time_range_end", time_range.end.timestamp())
        .param("project_ids", project_ids)
        .fetch::<StringFacetRow>()?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(FacetValue {
            value: row.value,
            count: row.count,
        });
    }
    Ok(values)
}

async fn fetch_version_facet(
    clickhouse: &clickhouse::Client,
    query: &str,
    project_ids: &[DBProjectId],
    time_range: &TimeRange,
) -> Result<Vec<FacetValue<VersionId>>, ApiError> {
    let mut rows = clickhouse
        .query(query)
        .param("time_range_start", time_range.start.timestamp())
        .param("time_range_end", time_range.end.timestamp())
        .param("project_ids", project_ids)
        .fetch::<VersionFacetRow>()?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(FacetValue {
            value: row.value.into(),
            count: row.count,
        });
    }
    Ok(values)
}

async fn fetch_bool_facet(
    clickhouse: &clickhouse::Client,
    query: &str,
    project_ids: &[DBProjectId],
    time_range: &TimeRange,
) -> Result<Vec<FacetValue<bool>>, ApiError> {
    let mut rows = clickhouse
        .query(query)
        .param("time_range_start", time_range.start.timestamp())
        .param("time_range_end", time_range.end.timestamp())
        .param("project_ids", project_ids)
        .fetch::<BoolFacetRow>()?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(FacetValue {
            value: row.value,
            count: row.count,
        });
    }
    Ok(values)
}

async fn fetch_playtime_string_facet(
    clickhouse: &clickhouse::Client,
    column: &str,
    project_ids: &[DBProjectId],
    parent_version_ids: &[DBVersionId],
    time_range: &TimeRange,
) -> Result<Vec<FacetValue<String>>, ApiError> {
    let query = format!(
		"SELECT {column} AS value, COUNT(*) AS count
		FROM playtime
		WHERE recorded >= {{time_range_start: Int64}}
			AND recorded < {{time_range_end: Int64}}
			AND (project_id IN {{project_ids: Array(UInt64)}} OR parent IN {{parent_version_ids: Array(UInt64)}})
			AND {column} != ''
        GROUP BY value
		ORDER BY value"
    );
    let mut rows = clickhouse
        .query(&query)
        .param("time_range_start", time_range.start.timestamp())
        .param("time_range_end", time_range.end.timestamp())
        .param("project_ids", project_ids)
        .param("parent_version_ids", parent_version_ids)
        .fetch::<StringFacetRow>()?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(FacetValue {
            value: row.value,
            count: row.count,
        });
    }
    Ok(values)
}

async fn fetch_playtime_version_facet(
    clickhouse: &clickhouse::Client,
    project_ids: &[DBProjectId],
    parent_version_ids: &[DBVersionId],
    time_range: &TimeRange,
) -> Result<Vec<FacetValue<VersionId>>, ApiError> {
    let mut rows = clickhouse
		.query(
			"SELECT version_id AS value, COUNT(*) AS count
			FROM playtime
			WHERE recorded >= {time_range_start: Int64}
				AND recorded < {time_range_end: Int64}
				AND (project_id IN {project_ids: Array(UInt64)} OR parent IN {parent_version_ids: Array(UInt64)})
				AND version_id != 0
            GROUP BY value
			ORDER BY value",
        )
		.param("time_range_start", time_range.start.timestamp())
		.param("time_range_end", time_range.end.timestamp())
		.param("project_ids", project_ids)
		.param("parent_version_ids", parent_version_ids)
        .fetch::<VersionFacetRow>()?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(FacetValue {
            value: row.value.into(),
            count: row.count,
        });
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_agent_facets_use_normalized_sources() {
        let user_agents = vec![
            FacetValue {
                value: "MultiMC/5.0".to_string(),
                count: 2,
            },
            FacetValue {
                value: "MultiMC/6.0".to_string(),
                count: 3,
            },
            FacetValue {
                value: "PrismLauncher/6.1".to_string(),
                count: 5,
            },
            FacetValue {
                value: "curl/8.7.1".to_string(),
                count: 7,
            },
            FacetValue {
                value: "Mozilla/5.0 AppleWebKit/537.36".to_string(),
                count: 11,
            },
        ];

        assert_eq!(
            normalize_download_source_facets(&user_agents),
            vec![
                FacetValue {
                    value: DownloadSource::Named("MultiMC".into()),
                    count: 5,
                },
                FacetValue {
                    value: DownloadSource::Named("Prism Launcher".into()),
                    count: 5,
                },
                FacetValue {
                    value: DownloadSource::Website,
                    count: 11,
                },
            ],
        );
    }
}

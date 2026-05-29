use std::collections::HashMap;

use super::{
    AnalyticsFacets, FacetValue, ProjectDownloadsFacets, ProjectPlaytimeFacets,
    ProjectViewsFacets,
};
use crate::{
    database::{
        PgPool,
        models::{DBProjectId, DBUser},
        redis::RedisPool,
    },
    models::users::User,
    routes::ApiError,
};

const FACET_LIMIT: u64 = 100;

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct StringFacetRow {
    value: String,
    count: u64,
}

pub async fn fetch(
    req: super::super::GetRequest,
    user: &User,
    pool: &PgPool,
    redis: &RedisPool,
    clickhouse: &clickhouse::Client,
) -> Result<AnalyticsFacets, ApiError> {
    let project_ids = if req.project_ids.is_empty() {
        DBUser::get_projects(user.id.into(), pool, redis).await?
    } else {
        req.project_ids
            .iter()
            .map(|id| DBProjectId::from(*id))
            .collect::<Vec<_>>()
    };
    let project_ids = super::super::filter_allowed_project_ids(
        &project_ids,
        user,
        pool,
        redis,
    )
    .await?;

    Ok(AnalyticsFacets {
        project_views: fetch_project_views_facets(
            clickhouse,
            &project_ids,
            &req.time_range,
        )
        .await?,
        project_downloads: fetch_project_downloads_facets(
            clickhouse,
            &project_ids,
            &req.time_range,
        )
        .await?,
        project_playtime: fetch_project_playtime_facets(
            clickhouse,
            &project_ids,
            &req.time_range,
        )
        .await?,
    })
}

async fn fetch_project_views_facets(
    clickhouse: &clickhouse::Client,
    project_ids: &[DBProjectId],
    time_range: &super::super::TimeRange,
) -> Result<ProjectViewsFacets, ApiError> {
    Ok(ProjectViewsFacets {
		country: fetch_string_facet(
			clickhouse,
			"SELECT country AS value, COUNT(*) AS count FROM views WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND country != '' GROUP BY value ORDER BY count DESC, value LIMIT {facet_limit: UInt64}",
			project_ids,
			time_range,
		)
		.await?,
		..Default::default()
	})
}

async fn fetch_project_downloads_facets(
    clickhouse: &clickhouse::Client,
    project_ids: &[DBProjectId],
    time_range: &super::super::TimeRange,
) -> Result<ProjectDownloadsFacets, ApiError> {
    let user_agents = fetch_string_facet(
		clickhouse,
		"SELECT user_agent AS value, COUNT(*) AS count FROM downloads WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND user_agent != '' GROUP BY value ORDER BY count DESC, value LIMIT {facet_limit: UInt64}",
		project_ids,
		time_range,
	)
	.await?;
    let user_agent = normalize_download_source_facets(&user_agents);

    Ok(ProjectDownloadsFacets {
		user_agent,
		country: fetch_string_facet(
			clickhouse,
			"SELECT country AS value, COUNT(*) AS count FROM downloads WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND country != '' GROUP BY value ORDER BY count DESC, value LIMIT {facet_limit: UInt64}",
			project_ids,
			time_range,
		)
		.await?,
		..Default::default()
	})
}

fn normalize_download_source_facets(
    user_agents: &[FacetValue<String>],
) -> Vec<FacetValue<super::super::DownloadSource>> {
    let mut counts = HashMap::<super::super::DownloadSource, u64>::new();
    for user_agent in user_agents {
        if let Some(source) =
            super::super::normalize_download_source(&user_agent.value)
        {
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

fn download_source_sort_key(source: &super::super::DownloadSource) -> &str {
    match source {
        super::super::DownloadSource::Named(name) => name,
        super::super::DownloadSource::Website => "website",
        super::super::DownloadSource::ModrinthApp => "modrinth_app",
        super::super::DownloadSource::ModrinthHosting => "modrinth_hosting",
        super::super::DownloadSource::ModrinthMaven => "modrinth_maven",
        super::super::DownloadSource::Other => "other",
    }
}

async fn fetch_project_playtime_facets(
    clickhouse: &clickhouse::Client,
    project_ids: &[DBProjectId],
    time_range: &super::super::TimeRange,
) -> Result<ProjectPlaytimeFacets, ApiError> {
    Ok(ProjectPlaytimeFacets {
		country: fetch_string_facet(
			clickhouse,
			"SELECT country AS value, COUNT(*) AS count FROM playtime WHERE recorded >= {time_range_start: Int64} AND recorded < {time_range_end: Int64} AND project_id IN {project_ids: Array(UInt64)} AND country != '' GROUP BY value ORDER BY count DESC, value LIMIT {facet_limit: UInt64}",
			project_ids,
			time_range,
		)
		.await?,
		..Default::default()
	})
}

async fn fetch_string_facet(
    clickhouse: &clickhouse::Client,
    query: &str,
    project_ids: &[DBProjectId],
    time_range: &super::super::TimeRange,
) -> Result<Vec<FacetValue<String>>, ApiError> {
    let mut rows = clickhouse
        .query(query)
        .param("time_range_start", time_range.start.timestamp())
        .param("time_range_end", time_range.end.timestamp())
        .param("project_ids", project_ids)
        .param("facet_limit", FACET_LIMIT)
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
                    value: super::super::DownloadSource::Named(
                        "MultiMC".into()
                    ),
                    count: 5,
                },
                FacetValue {
                    value: super::super::DownloadSource::Named(
                        "Prism Launcher".into()
                    ),
                    count: 5,
                },
                FacetValue {
                    value: super::super::DownloadSource::Website,
                    count: 11,
                },
            ],
        );
    }
}

use std::sync::Arc;

use crate::{models::ids::ProjectId, routes::ApiError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(clickhouse::Row, Serialize, Deserialize, Clone, Debug)]
pub struct ReturnPlaytimes {
    pub time: u32,
    pub id: u64,
    pub total_seconds: u64,
}

#[derive(clickhouse::Row, Serialize, Deserialize, Clone, Debug)]
pub struct ReturnCountry {
    pub country: String,
    pub id: u64,
    pub total_views: u64,
    pub total_downloads: u64,
}

#[derive(clickhouse::Row, Serialize, Deserialize, Clone, Debug)]
pub struct ReturnViews {
    pub time: u32,
    pub id: u64,
    pub total_views: u64,
}

#[derive(clickhouse::Row, Serialize, Deserialize, Clone, Debug)]
pub struct ReturnDownloads {
    pub time: u32,
    pub id: u64,
    pub total_downloads: u64,
}

// Only one of project_id or version_id should be used
// Fetches playtimes as a Vec of ReturnPlaytimes
pub async fn fetch_playtimes(
    projects: Vec<ProjectId>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    resolution_minute: u32,
    client: Arc<clickhouse::Client>,
) -> Result<Vec<ReturnPlaytimes>, ApiError> {
    let query = client
        .query(
            "
            SELECT
                toUnixTimestamp(toStartOfInterval(recorded, toIntervalMinute(?))) AS time,
                project_id AS id,
                SUM(seconds) AS total_seconds
            FROM playtime
            WHERE recorded BETWEEN ? AND ?
            AND project_id IN ?
            GROUP BY
                time,
                project_id
            ",
        )
        .bind(resolution_minute)
        .bind(start_date.timestamp())
        .bind(end_date.timestamp())
        .bind(projects.iter().map(|x| x.0).collect::<Vec<_>>());

    Ok(query.fetch_all().await?)
}

// Fetches views as a Vec of ReturnViews
pub async fn fetch_views(
    projects: Vec<ProjectId>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    resolution_minutes: u32,
    client: Arc<clickhouse::Client>,
) -> Result<Vec<ReturnViews>, ApiError> {
    let query = client
        .query(
            "
            SELECT  
                toUnixTimestamp(toStartOfInterval(recorded, toIntervalMinute(?))) AS time,
                project_id AS id,
                count(1) AS total_views
            FROM views
            WHERE recorded BETWEEN ? AND ?
                  AND project_id IN ?
            GROUP BY
            time, project_id
            ",
        )
        .bind(resolution_minutes)
        .bind(start_date.timestamp())
        .bind(end_date.timestamp())
        .bind(projects.iter().map(|x| x.0).collect::<Vec<_>>());

    Ok(query.fetch_all().await?)
}

// Fetches downloads as a Vec of ReturnDownloads
pub async fn fetch_downloads(
    projects: Vec<ProjectId>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    resolution_minutes: u32,
    client: Arc<clickhouse::Client>,
) -> Result<Vec<ReturnDownloads>, ApiError> {
    let query = client
        .query(
            "
            SELECT  
                toUnixTimestamp(toStartOfInterval(recorded, toIntervalMinute(?))) AS time,
                project_id as id,
                count(1) AS total_downloads
            FROM downloads
            WHERE recorded BETWEEN ? AND ?
                  AND project_id IN ?
            GROUP BY time, project_id
            ",
        )
        .bind(resolution_minutes)
        .bind(start_date.timestamp())
        .bind(end_date.timestamp())
        .bind(projects.iter().map(|x| x.0).collect::<Vec<_>>());

    Ok(query.fetch_all().await?)
}

// Fetches countries as a Vec of ReturnCountry
pub async fn fetch_countries(
    projects: Vec<ProjectId>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    client: Arc<clickhouse::Client>,
) -> Result<Vec<ReturnCountry>, ApiError> {
    let query = client.query(
            "
            WITH view_grouping AS (
            SELECT
                country,
                project_id,
                count(1) AS total_views
            FROM views
            WHERE recorded BETWEEN ? AND ?
            GROUP BY
                country,
                project_id
            ),
            download_grouping AS (
            SELECT
                country,
                project_id,
                count(1) AS total_downloads
            FROM downloads
            WHERE recorded BETWEEN ? AND ?
            GROUP BY
                country,
                project_id
            )

            SELECT
                v.country,
                v.project_id,
                v.total_views,
                d.total_downloads
            FROM view_grouping AS v
            LEFT JOIN download_grouping AS d ON (v.country = d.country) AND (v.project_id = d.project_id)
            WHERE project_id IN ?
            "
        )
        .bind(start_date.timestamp())
        .bind(end_date.timestamp())
        .bind(start_date.timestamp())
        .bind(end_date.timestamp())
        .bind(projects.iter().map(|x| x.0).collect::<Vec<_>>());

    Ok(query.fetch_all().await?)
}

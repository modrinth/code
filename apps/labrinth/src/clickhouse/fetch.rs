use std::sync::Arc;

use crate::{models::ids::ProjectId, routes::ApiError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(clickhouse::Row, Serialize, Deserialize, Clone, Debug)]
pub struct ReturnIntervals {
    pub time: u32,
    pub id: u64,
    pub total: u64,
}

#[derive(clickhouse::Row, Serialize, Deserialize, Clone, Debug)]
pub struct ReturnCountry {
    pub country: String,
    pub id: u64,
    pub total: u64,
}

// Only one of project_id or version_id should be used
// Fetches playtimes as a Vec of ReturnPlaytimes
pub async fn fetch_playtimes(
    projects: Vec<ProjectId>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    resolution_minute: u32,
    client: Arc<clickhouse::Client>,
) -> Result<Vec<ReturnIntervals>, ApiError> {
    let query = client
        .query(
            "
            SELECT
                toUnixTimestamp(toStartOfInterval(recorded, toIntervalMinute(?))) AS time,
                project_id AS id,
                SUM(seconds) AS total
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
) -> Result<Vec<ReturnIntervals>, ApiError> {
    let query = client
        .query(
            "
            SELECT  
                toUnixTimestamp(toStartOfInterval(recorded, toIntervalMinute(?))) AS time,
                project_id AS id,
                count(1) AS total
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
) -> Result<Vec<ReturnIntervals>, ApiError> {
    let query = client
        .query(
            "
            SELECT  
                toUnixTimestamp(toStartOfInterval(recorded, toIntervalMinute(?))) AS time,
                project_id as id,
                count(1) AS total
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

pub async fn fetch_countries_downloads(
    projects: Vec<ProjectId>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    client: Arc<clickhouse::Client>,
) -> Result<Vec<ReturnCountry>, ApiError> {
    let query = client
        .query(
            "
            SELECT
                country,
                project_id,
                count(1) AS total
            FROM downloads
            WHERE recorded BETWEEN ? AND ? AND project_id IN ?
            GROUP BY
                country,
                project_id
            ",
        )
        .bind(start_date.timestamp())
        .bind(end_date.timestamp())
        .bind(projects.iter().map(|x| x.0).collect::<Vec<_>>());

    Ok(query.fetch_all().await?)
}

pub async fn fetch_countries_views(
    projects: Vec<ProjectId>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    client: Arc<clickhouse::Client>,
) -> Result<Vec<ReturnCountry>, ApiError> {
    let query = client
        .query(
            "
            SELECT
                country,
                project_id,
                count(1) AS total
            FROM views
            WHERE recorded BETWEEN ? AND ? AND project_id IN ?
            GROUP BY
                country,
                project_id
            ",
        )
        .bind(start_date.timestamp())
        .bind(end_date.timestamp())
        .bind(projects.iter().map(|x| x.0).collect::<Vec<_>>());

    Ok(query.fetch_all().await?)
}

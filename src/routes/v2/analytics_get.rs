use super::ApiError;
use crate::database::redis::RedisPool;
use crate::routes::{v2_reroute, v3};
use crate::{models::ids::VersionId, queue::session::AuthQueue};
use actix_web::{get, web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("analytics")
            .service(playtimes_get)
            .service(views_get)
            .service(downloads_get)
            .service(revenue_get)
            .service(countries_downloads_get)
            .service(countries_views_get),
    );
}

/// The json data to be passed to fetch analytic data
/// Either a list of project_ids or version_ids can be used, but not both. Unauthorized projects/versions will be filtered out.
/// start_date and end_date are optional, and default to two weeks ago, and the maximum date respectively
/// start_date and end_date are inclusive
/// resolution_minutes is optional. This refers to the window by which we are looking (every day, every minute, etc) and defaults to 1440 (1 day)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetData {
    // only one of project_ids or version_ids should be used
    // if neither are provided, all projects the user has access to will be used
    pub project_ids: Option<String>,
    pub version_ids: Option<String>,

    pub start_date: Option<DateTime<Utc>>, // defaults to 2 weeks ago
    pub end_date: Option<DateTime<Utc>>,   // defaults to now

    pub resolution_minutes: Option<u32>, // defaults to 1 day. Ignored in routes that do not aggregate over a resolution (eg: /countries)
}

/// Get playtime data for a set of projects or versions
/// Data is returned as a hashmap of project/version ids to a hashmap of days to playtime data
/// eg:
/// {
///     "4N1tEhnO": {
///         "20230824": 23
///    }
///}
/// Either a list of project_ids or version_ids can be used, but not both. Unauthorized projects/versions will be filtered out.
#[derive(Serialize, Deserialize, Clone)]
pub struct FetchedPlaytime {
    pub time: u64,
    pub total_seconds: u64,
    pub loader_seconds: HashMap<String, u64>,
    pub game_version_seconds: HashMap<String, u64>,
    pub parent_seconds: HashMap<VersionId, u64>,
}
#[get("playtime")]
pub async fn playtimes_get(
    req: HttpRequest,
    clickhouse: web::Data<clickhouse::Client>,
    data: web::Query<GetData>,
    session_queue: web::Data<AuthQueue>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let data = data.into_inner();
    v3::analytics_get::playtimes_get(
        req,
        clickhouse,
        web::Query(v3::analytics_get::GetData {
            project_ids: data.project_ids,
            version_ids: data.version_ids,
            start_date: data.start_date,
            end_date: data.end_date,
            resolution_minutes: data.resolution_minutes,
        }),
        session_queue,
        pool,
        redis,
    )
    .await
}

/// Get view data for a set of projects or versions
/// Data is returned as a hashmap of project/version ids to a hashmap of days to views
/// eg:
/// {
///     "4N1tEhnO": {
///         "20230824": 1090
///    }
///}
/// Either a list of project_ids or version_ids can be used, but not both. Unauthorized projects/versions will be filtered out.
#[get("views")]
pub async fn views_get(
    req: HttpRequest,
    clickhouse: web::Data<clickhouse::Client>,
    data: web::Query<GetData>,
    session_queue: web::Data<AuthQueue>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let data = data.into_inner();
    v3::analytics_get::views_get(
        req,
        clickhouse,
        web::Query(v3::analytics_get::GetData {
            project_ids: data.project_ids,
            version_ids: data.version_ids,
            start_date: data.start_date,
            end_date: data.end_date,
            resolution_minutes: data.resolution_minutes,
        }),
        session_queue,
        pool,
        redis,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

/// Get download data for a set of projects or versions
/// Data is returned as a hashmap of project/version ids to a hashmap of days to downloads
/// eg:
/// {
///     "4N1tEhnO": {
///         "20230824": 32
///    }
///}
/// Either a list of project_ids or version_ids can be used, but not both. Unauthorized projects/versions will be filtered out.
#[get("downloads")]
pub async fn downloads_get(
    req: HttpRequest,
    clickhouse: web::Data<clickhouse::Client>,
    data: web::Query<GetData>,
    session_queue: web::Data<AuthQueue>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let data = data.into_inner();
    v3::analytics_get::downloads_get(
        req,
        clickhouse,
        web::Query(v3::analytics_get::GetData {
            project_ids: data.project_ids,
            version_ids: data.version_ids,
            start_date: data.start_date,
            end_date: data.end_date,
            resolution_minutes: data.resolution_minutes,
        }),
        session_queue,
        pool,
        redis,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

/// Get payout data for a set of projects
/// Data is returned as a hashmap of project ids to a hashmap of days to amount earned per day
/// eg:
/// {
///     "4N1tEhnO": {
///         "20230824": 0.001
///    }
///}
/// ONLY project IDs can be used. Unauthorized projects will be filtered out.
#[get("revenue")]
pub async fn revenue_get(
    req: HttpRequest,
    data: web::Query<GetData>,
    session_queue: web::Data<AuthQueue>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let data = data.into_inner();
    v3::analytics_get::revenue_get(
        req,
        web::Query(v3::analytics_get::GetData {
            project_ids: data.project_ids,
            version_ids: None,
            start_date: data.start_date,
            end_date: data.end_date,
            resolution_minutes: data.resolution_minutes,
        }),
        session_queue,
        pool,
        redis,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

/// Get country data for a set of projects or versions
/// Data is returned as a hashmap of project/version ids to a hashmap of coutnry to downloads.
/// Unknown countries are labeled "".
/// This is usuable to see significant performing countries per project
/// eg:
/// {
///     "4N1tEhnO": {
///         "CAN":  22
///    }
///}
/// Either a list of project_ids or version_ids can be used, but not both. Unauthorized projects/versions will be filtered out.
/// For this endpoint, provided dates are a range to aggregate over, not specific days to fetch
#[get("countries/downloads")]
pub async fn countries_downloads_get(
    req: HttpRequest,
    clickhouse: web::Data<clickhouse::Client>,
    data: web::Query<GetData>,
    session_queue: web::Data<AuthQueue>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let data = data.into_inner();
    v3::analytics_get::countries_downloads_get(
        req,
        clickhouse,
        web::Query(v3::analytics_get::GetData {
            project_ids: data.project_ids,
            version_ids: data.version_ids,
            start_date: data.start_date,
            end_date: data.end_date,
            resolution_minutes: data.resolution_minutes,
        }),
        session_queue,
        pool,
        redis,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

/// Get country data for a set of projects or versions
/// Data is returned as a hashmap of project/version ids to a hashmap of coutnry to views.
/// Unknown countries are labeled "".
/// This is usuable to see significant performing countries per project
/// eg:
/// {
///     "4N1tEhnO": {
///         "CAN":  56165
///    }
///}
/// Either a list of project_ids or version_ids can be used, but not both. Unauthorized projects/versions will be filtered out.
/// For this endpoint, provided dates are a range to aggregate over, not specific days to fetch
#[get("countries/views")]
pub async fn countries_views_get(
    req: HttpRequest,
    clickhouse: web::Data<clickhouse::Client>,
    data: web::Query<GetData>,
    session_queue: web::Data<AuthQueue>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let data = data.into_inner();
    v3::analytics_get::countries_views_get(
        req,
        clickhouse,
        web::Query(v3::analytics_get::GetData {
            project_ids: data.project_ids,
            version_ids: data.version_ids,
            start_date: data.start_date,
            end_date: data.end_date,
            resolution_minutes: data.resolution_minutes,
        }),
        session_queue,
        pool,
        redis,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

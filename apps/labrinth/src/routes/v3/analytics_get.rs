//! # Design rationale
//!
//! - different metrics require different scopes
//!   - views, downloads, playtime requires `Scopes::ANALYTICS`
//!   - revenue requires `Scopes::PAYOUTS_READ`
//! - each request returns an array of N elements; if you have to make multiple
//!   requests, you have to zip together M arrays of N elements
//!   - this makes it inconvenient to have separate endpoints

mod old;

use std::num::NonZeroU64;

use actix_web::{HttpRequest, post, web};
use chrono::{DateTime, TimeDelta, Utc};
use futures::StreamExt;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    auth::{AuthenticationError, get_user_from_headers},
    database::{
        self, DBProject,
        models::{DBProjectId, DBUser, DBUserId, DBVersionId},
        redis::RedisPool,
    },
    models::{
        ids::{ProjectId, VersionId},
        pats::Scopes,
        teams::ProjectPermissions,
    },
    queue::session::AuthQueue,
    routes::ApiError,
};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(fetch_analytics);
    cfg.configure(old::config);
}

// request

/// Requests analytics data, aggregating over all possible analytics sources
/// like projects and affiliate codes, returning the data in a list of time
/// slices.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GetRequest {
    /// What time range to return statistics for.
    pub time_range: TimeRange,
    /// What analytics metrics to return data for.
    pub return_metrics: ReturnMetrics,
}

/// Time range for fetching analytics.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TimeRange {
    /// When to start including data.
    pub start: DateTime<Utc>,
    /// When to stop including data.
    pub end: DateTime<Utc>,
    /// Determines how many time slices between the start and end will be
    /// included, and how fine-grained those time slices will be.
    ///
    /// This must fall within the bounds of [`MIN_RESOLUTION`] and
    /// [`MAX_TIME_SLICES`].
    pub resolution: TimeRangeResolution,
}

/// Determines how many time slices between the start and end will be
/// included, and how fine-grained those time slices will be.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimeRangeResolution {
    /// Use a set number of time slices, with the resolution being determined
    /// automatically.
    #[schema(value_type = u64)]
    Slices(NonZeroU64),
    /// Each time slice will be a set number of minutes long, and the number of
    /// slices is determined automatically.
    #[schema(value_type = u64)]
    Minutes(NonZeroU64),
}

/// What metrics the caller would like to receive from this analytics get
/// request.
#[derive(Debug, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReturnMetrics {
    /// How many times a project page has been viewed.
    pub project_views: Option<Metrics<ProjectViewsField>>,
    /// How many times a project has been downloaded.
    pub project_downloads: Option<Metrics<ProjectDownloadsField>>,
    /// How long users have been playing a project.
    pub project_playtime: Option<Metrics<ProjectPlaytimeField>>,
    /// How much payout revenue a project has generated.
    pub project_revenue: Option<Metrics<Unit>>,
}

/// Replacement for `()` because of a `utoipa` limitation.
#[derive(Debug, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Unit {}

/// See [`ReturnMetrics`].
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Metrics<F> {
    /// When collecting metrics, what fields do we want to group the results by?
    ///
    /// For example, if we have two views entries:
    /// - `{ "project_id": "abcdefgh", "domain": "youtube.com", "count": 5 }`
    /// - `{ "project_id": "abcdefgh", "domain": "discord.com", "count": 3 }`
    ///
    /// If we bucket by `domain`, then we will get two results:
    /// - `{ "project_id": "abcdefgh", "domain": "youtube.com", "count": 5 }`
    /// - `{ "project_id": "abcdefgh", "domain": "discord.com", "count": 3 }`
    ///
    /// If we do not bucket by `domain`, we will only get one, which is an
    /// aggregate of the two rows:
    /// - `{ "project_id": "abcdefgh", "count": 8 }`
    #[serde(default = "Vec::default")]
    pub bucket_by: Vec<F>,
}

/// Fields for [`ReturnMetrics::project_views`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProjectViewsField {
    /// Project ID.
    ProjectId,
    /// Referrer domain which linked to this project.
    Domain,
    /// Modrinth site path which was visited, e.g. `/mod/foo`.
    SitePath,
    /// Whether these views were monetized or not.
    Monetized,
    /// What country these views came from.
    ///
    /// To anonymize the data, the country may be reported as `XX`.
    Country,
}

/// Fields for [`ReturnMetrics::project_downloads`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProjectDownloadsField {
    /// Project ID.
    ProjectId,
    /// Version ID of this project.
    VersionId,
    /// Referrer domain which linked to this project.
    Domain,
    /// Modrinth site path which was visited, e.g. `/mod/foo`.
    SitePath,
    /// What country these views came from.
    ///
    /// To anonymize the data, the country may be reported as `XX`.
    Country,
}

/// Fields for [`ReturnMetrics::project_playtime`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPlaytimeField {
    /// Project ID.
    ProjectId,
    /// Version ID of this project.
    VersionId,
    /// Game mod loader which was used to count this playtime, e.g. Fabric.
    Loader,
    /// Game version which this project was played on.
    GameVersion,
}

/// Minimum width of a [`TimeSlice`], controlled by [`TimeRange::resolution`].
pub const MIN_RESOLUTION: TimeDelta = TimeDelta::minutes(60);

/// Maximum number of [`TimeSlice`]s in a [`GetResponse`], controlled by
/// [`TimeRange::resolution`].
pub const MAX_TIME_SLICES: usize = 1024;

// response

/// Response for a [`GetRequest`].
///
/// This is a list of N [`TimeSlice`]s, where each slice represents an equal
/// time interval of metrics collection. The number of slices is determined
/// by [`GetRequest::time_range`].
#[derive(Debug, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FetchResponse(pub Vec<TimeSlice>);

/// Single time interval of metrics collection.
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TimeSlice(pub Vec<AnalyticsData>);

/// Metrics collected in a single [`TimeSlice`].
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(untagged)] // the presence of `source_project`, `source_affiliate_code` determines the kind
pub enum AnalyticsData {
    /// Project metrics.
    Project(ProjectAnalytics),
    // AffiliateCode(AffiliateCodeAnalytics),
}

/// Project metrics.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectAnalytics {
    /// What project these metrics are for.
    source_project: ProjectId,
    /// Metrics collected.
    #[serde(flatten)]
    metrics: ProjectMetrics,
}

impl ProjectAnalytics {
    /// Get the project ID for these analytics.
    pub fn project_id(&self) -> &ProjectId {
        &self.source_project
    }
}

/// Project metrics of a specific kind.
///
/// If a field is not included in [`Metrics::bucket_by`], it will be [`None`].
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case", tag = "metric_kind")]
pub enum ProjectMetrics {
    /// [`ReturnMetrics::project_views`].
    Views(ProjectViews),
    /// [`ReturnMetrics::project_downloads`].
    Downloads(ProjectDownloads),
    /// [`ReturnMetrics::project_playtime`].
    Playtime(ProjectPlaytime),
    /// [`ReturnMetrics::project_revenue`].
    Revenue(ProjectRevenue),
}

/// [`ReturnMetrics::project_views`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectViews {
    /// [`ProjectViewsField::Domain`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// [`ProjectViewsField::SitePath`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_path: Option<String>,
    /// [`ProjectViewsField::Monetized`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monetized: Option<bool>,
    /// [`ProjectViewsField::Country`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// Total number of views for this bucket.
    pub views: u64,
}

/// [`ReturnMetrics::project_downloads`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectDownloads {
    /// [`ProjectDownloadsField::Domain`].
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    /// [`ProjectDownloadsField::SitePath`].
    #[serde(skip_serializing_if = "Option::is_none")]
    site_path: Option<String>,
    /// [`ProjectDownloadsField::VersionId`].
    #[serde(skip_serializing_if = "Option::is_none")]
    version_id: Option<VersionId>,
    /// [`ProjectDownloadsField::Country`].
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<String>,
    /// Total number of downloads for this bucket.
    downloads: u64,
}

/// [`ReturnMetrics::project_playtime`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectPlaytime {
    /// [`ProjectPlaytimeField::VersionId`].
    #[serde(skip_serializing_if = "Option::is_none")]
    version_id: Option<VersionId>,
    /// [`ProjectPlaytimeField::Loader`].
    #[serde(skip_serializing_if = "Option::is_none")]
    loader: Option<String>,
    /// [`ProjectPlaytimeField::GameVersion`].
    #[serde(skip_serializing_if = "Option::is_none")]
    game_version: Option<String>,
    /// Total number of seconds of playtime for this bucket.
    seconds: u64,
}

/// [`ReturnMetrics::project_revenue`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectRevenue {
    /// Total revenue for this bucket.
    revenue: Decimal,
}

// logic

/// Clickhouse queries - separate from [`sqlx`] queries.
mod query {
    use crate::database::models::{DBProjectId, DBVersionId};
    use const_format::formatcp;

    const TIME_RANGE_START: &str = "{time_range_start: UInt64}";
    const TIME_RANGE_END: &str = "{time_range_end: UInt64}";
    const TIME_SLICES: &str = "{time_slices: UInt64}";
    const PROJECT_IDS: &str = "{project_ids: Array(UInt64)}";

    #[derive(Debug, clickhouse::Row, serde::Deserialize)]
    pub struct ViewRow {
        pub bucket: u64,
        pub project_id: DBProjectId,
        pub domain: String,
        pub site_path: String,
        pub monetized: i8,
        pub country: String,
        pub views: u64,
    }

    pub const VIEWS: &str = {
        const USE_PROJECT_ID: &str = "{use_project_id: Bool}";
        const USE_DOMAIN: &str = "{use_domain: Bool}";
        const USE_SITE_PATH: &str = "{use_site_path: Bool}";
        const USE_MONETIZED: &str = "{use_monetized: Bool}";
        const USE_COUNTRY: &str = "{use_country: Bool}";

        formatcp!(
            "SELECT
                widthBucket(toUnixTimestamp(recorded), {TIME_RANGE_START}, {TIME_RANGE_END}, {TIME_SLICES}) AS bucket,
                if({USE_PROJECT_ID}, project_id, 0) AS project_id,
                if({USE_DOMAIN}, domain, '') AS domain,
                if({USE_SITE_PATH}, site_path, '') AS site_path,
                if({USE_MONETIZED}, CAST(monetized AS Int8), -1) AS monetized,
                if({USE_COUNTRY}, country, '') AS country,
                COUNT(*) AS views
            FROM views
            WHERE
                recorded BETWEEN {TIME_RANGE_START} AND {TIME_RANGE_END}
                -- make sure that the REAL project id is included,
                -- not the possibly-zero one,
                -- by using `views.project_id` instead of `project_id`
                AND views.project_id IN {PROJECT_IDS}
            GROUP BY
                bucket, project_id, domain, site_path, monetized, country"
        )
    };

    #[derive(Debug, clickhouse::Row, serde::Deserialize)]
    pub struct DownloadRow {
        pub bucket: u64,
        pub project_id: DBProjectId,
        pub domain: String,
        pub site_path: String,
        pub version_id: DBVersionId,
        pub country: String,
        pub downloads: u64,
    }

    pub const DOWNLOADS: &str = {
        const USE_PROJECT_ID: &str = "{use_project_id: Bool}";
        const USE_DOMAIN: &str = "{use_domain: Bool}";
        const USE_SITE_PATH: &str = "{use_site_path: Bool}";
        const USE_VERSION_ID: &str = "{use_version_id: Bool}";
        const USE_COUNTRY: &str = "{use_country: Bool}";

        formatcp!(
            "SELECT
                widthBucket(toUnixTimestamp(recorded), {TIME_RANGE_START}, {TIME_RANGE_END}, {TIME_SLICES}) AS bucket,
                if({USE_PROJECT_ID}, project_id, 0) AS project_id,
                if({USE_DOMAIN}, domain, '') AS domain,
                if({USE_SITE_PATH}, site_path, '') AS site_path,
                if({USE_VERSION_ID}, version_id, 0) AS version_id,
                if({USE_COUNTRY}, country, '') AS country,
                COUNT(*) AS downloads
            FROM downloads
            WHERE
                recorded BETWEEN {TIME_RANGE_START} AND {TIME_RANGE_END}
                -- make sure that the REAL project id is included,
                -- not the possibly-zero one,
                -- by using `downloads.project_id` instead of `project_id`
                AND downloads.project_id IN {PROJECT_IDS}
            GROUP BY
                bucket, project_id, domain, site_path, version_id, country"
        )
    };

    #[derive(Debug, clickhouse::Row, serde::Deserialize)]
    pub struct PlaytimeRow {
        pub bucket: u64,
        pub project_id: DBProjectId,
        pub version_id: DBVersionId,
        pub loader: String,
        pub game_version: String,
        pub seconds: u64,
    }

    pub const PLAYTIME: &str = {
        const USE_PROJECT_ID: &str = "{use_project_id: Bool}";
        const USE_VERSION_ID: &str = "{use_version_id: Bool}";
        const USE_LOADER: &str = "{use_loader: Bool}";
        const USE_GAME_VERSION: &str = "{use_game_version: Bool}";

        formatcp!(
            "SELECT
                widthBucket(toUnixTimestamp(recorded), {TIME_RANGE_START}, {TIME_RANGE_END}, {TIME_SLICES}) AS bucket,
                if({USE_PROJECT_ID}, project_id, 0) AS project_id,
                if({USE_VERSION_ID}, version_id, 0) AS version_id,
                if({USE_LOADER}, loader, '') AS loader,
                if({USE_GAME_VERSION}, game_version, '') AS game_version,
                SUM(seconds) AS seconds
            FROM playtime
            WHERE
                recorded BETWEEN {TIME_RANGE_START} AND {TIME_RANGE_END}
                -- make sure that the REAL project id is included,
                -- not the possibly-zero one,
                -- by using `playtime.project_id` instead of `project_id`
                AND playtime.project_id IN {PROJECT_IDS}
            GROUP BY
                bucket, project_id, version_id, loader, game_version"
        )
    };
}

/// Fetches analytics data for the authorized user's projects.
#[utoipa::path(
    responses((status = OK, body = inline(FetchResponse))),
)]
#[post("")]
pub async fn fetch_analytics(
    http_req: HttpRequest,
    req: web::Json<GetRequest>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    clickhouse: web::Data<clickhouse::Client>,
) -> Result<web::Json<FetchResponse>, ApiError> {
    let (scopes, user) = get_user_from_headers(
        &http_req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::ANALYTICS,
    )
    .await?;

    let full_time_range = req.time_range.end - req.time_range.start;
    if full_time_range < TimeDelta::zero() {
        return Err(ApiError::InvalidInput(
            "End date must be after start date".into(),
        ));
    }

    let (num_time_slices, resolution) = match req.time_range.resolution {
        TimeRangeResolution::Slices(slices) => {
            let slices = i32::try_from(slices.get()).map_err(|_| {
                ApiError::InvalidInput(
                    "Number of slices must fit into an `i32`".into(),
                )
            })?;
            let resolution = full_time_range / slices;
            (slices as usize, resolution)
        }
        TimeRangeResolution::Minutes(resolution_minutes) => {
            let resolution_minutes = i64::try_from(resolution_minutes.get())
                .map_err(|_| {
                    ApiError::InvalidInput(
                        "Resolution must fit into a `i64`".into(),
                    )
                })?;
            let resolution = TimeDelta::try_minutes(resolution_minutes)
                .ok_or_else(|| {
                    ApiError::InvalidInput("Resolution overflow".into())
                })?;

            let num_slices =
                full_time_range.as_seconds_f64() / resolution.as_seconds_f64();

            (num_slices as usize, resolution)
        }
    };

    if num_time_slices > MAX_TIME_SLICES {
        return Err(ApiError::InvalidInput(format!(
            "Resolution is too fine or range is too large - maximum of {MAX_TIME_SLICES} time slices, was {num_time_slices}"
        )));
    }
    if resolution < MIN_RESOLUTION {
        return Err(ApiError::InvalidInput(format!(
            "Resolution must be at least {MIN_RESOLUTION}, was {resolution}",
        )));
    }

    let mut time_slices = vec![TimeSlice::default(); num_time_slices];

    // TODO fetch from req
    let project_ids =
        DBUser::get_projects(user.id.into(), &**pool, &redis).await?;

    let project_ids =
        filter_allowed_project_ids(&project_ids, &user, &pool, &redis).await?;

    let mut query_clickhouse_cx = QueryClickhouseContext {
        clickhouse: &clickhouse,
        req: &req,
        time_slices: &mut time_slices,
        project_ids: &project_ids,
    };

    if let Some(metrics) = &req.return_metrics.project_views {
        use ProjectViewsField as F;
        let uses = |field| metrics.bucket_by.contains(&field);

        query_clickhouse::<query::ViewRow>(
            &mut query_clickhouse_cx,
            query::VIEWS,
            &[
                ("use_project_id", uses(F::ProjectId)),
                ("use_domain", uses(F::Domain)),
                ("use_site_path", uses(F::SitePath)),
                ("use_monetized", uses(F::Monetized)),
                ("use_country", uses(F::Country)),
            ],
            |row| row.bucket,
            |row| {
                let country = if uses(F::Country) {
                    Some(condense_country(row.country, row.views))
                } else {
                    None
                };
                AnalyticsData::Project(ProjectAnalytics {
                    source_project: row.project_id.into(),
                    metrics: ProjectMetrics::Views(ProjectViews {
                        domain: none_if_empty(row.domain),
                        site_path: none_if_empty(row.site_path),
                        monetized: match row.monetized {
                            0 => Some(false),
                            1 => Some(true),
                            _ => None,
                        },
                        country,
                        views: row.views,
                    }),
                })
            },
        )
        .await?;
    }

    if let Some(metrics) = &req.return_metrics.project_downloads {
        use ProjectDownloadsField as F;
        let uses = |field| metrics.bucket_by.contains(&field);

        query_clickhouse::<query::DownloadRow>(
            &mut query_clickhouse_cx,
            query::DOWNLOADS,
            &[
                ("use_project_id", uses(F::ProjectId)),
                ("use_domain", uses(F::Domain)),
                ("use_site_path", uses(F::SitePath)),
                ("use_version_id", uses(F::VersionId)),
                ("use_country", uses(F::Country)),
            ],
            |row| row.bucket,
            |row| {
                let country = if uses(F::Country) {
                    Some(condense_country(row.country, row.downloads))
                } else {
                    None
                };
                AnalyticsData::Project(ProjectAnalytics {
                    source_project: row.project_id.into(),
                    metrics: ProjectMetrics::Downloads(ProjectDownloads {
                        domain: none_if_empty(row.domain),
                        site_path: none_if_empty(row.site_path),
                        version_id: none_if_zero_version_id(row.version_id),
                        country,
                        downloads: row.downloads,
                    }),
                })
            },
        )
        .await?;
    }

    if let Some(metrics) = &req.return_metrics.project_playtime {
        use ProjectPlaytimeField as F;
        let uses = |field| metrics.bucket_by.contains(&field);

        query_clickhouse::<query::PlaytimeRow>(
            &mut query_clickhouse_cx,
            query::PLAYTIME,
            &[
                ("use_project_id", uses(F::ProjectId)),
                ("use_version_id", uses(F::VersionId)),
                ("use_loader", uses(F::Loader)),
                ("use_game_version", uses(F::GameVersion)),
            ],
            |row| row.bucket,
            |row| {
                AnalyticsData::Project(ProjectAnalytics {
                    source_project: row.project_id.into(),
                    metrics: ProjectMetrics::Playtime(ProjectPlaytime {
                        version_id: none_if_zero_version_id(row.version_id),
                        loader: none_if_empty(row.loader),
                        game_version: none_if_empty(row.game_version),
                        seconds: row.seconds,
                    }),
                })
            },
        )
        .await?;
    }

    if req.return_metrics.project_revenue.is_some() {
        if !scopes.contains(Scopes::PAYOUTS_READ) {
            return Err(AuthenticationError::InvalidCredentials.into());
        }

        let mut rows = sqlx::query!(
            "SELECT
                WIDTH_BUCKET(
                    EXTRACT(EPOCH FROM created)::bigint,
                    EXTRACT(EPOCH FROM $1::timestamp with time zone AT TIME ZONE 'UTC')::bigint,
                    EXTRACT(EPOCH FROM $2::timestamp with time zone AT TIME ZONE 'UTC')::bigint,
                    $3::integer
                ) AS bucket,
                COALESCE(mod_id, 0) AS mod_id,
                SUM(amount) amount_sum
            FROM payouts_values
            WHERE
                user_id = $4
                AND created BETWEEN $1 AND $2
            GROUP BY bucket, mod_id",
            req.time_range.start,
            req.time_range.end,
            num_time_slices as i64,
            DBUserId::from(user.id) as DBUserId,
        )
        .fetch(&**pool);
        while let Some(row) = rows.next().await.transpose()? {
            let bucket = row.bucket.ok_or_else(|| {
                ApiError::InvalidInput(
                    "bucket should be non-null - query bug!".into(),
                )
            })?;
            let bucket = usize::try_from(bucket).map_err(|_| {
                ApiError::InvalidInput(
                    "bucket value {bucket} does not fit into `usize` - query bug!".into(),
                )
            })?;

            if let Some(source_project) =
                row.mod_id.map(DBProjectId).map(ProjectId::from)
                && let Some(revenue) = row.amount_sum
            {
                add_to_time_slice(
                    &mut time_slices,
                    bucket,
                    AnalyticsData::Project(ProjectAnalytics {
                        source_project,
                        metrics: ProjectMetrics::Revenue(ProjectRevenue {
                            revenue,
                        }),
                    }),
                )?;
            }
        }
    }

    Ok(web::Json(FetchResponse(time_slices)))
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

fn none_if_zero_version_id(v: DBVersionId) -> Option<VersionId> {
    if v.0 == 0 { None } else { Some(v.into()) }
}

fn condense_country(country: String, count: u64) -> String {
    // Every country under '50' (view or downloads) should be condensed into 'XX'
    if count < 50 {
        "XX".to_string()
    } else {
        country
    }
}

struct QueryClickhouseContext<'a> {
    clickhouse: &'a clickhouse::Client,
    req: &'a GetRequest,
    time_slices: &'a mut [TimeSlice],
    project_ids: &'a [DBProjectId],
}

async fn query_clickhouse<Row>(
    cx: &mut QueryClickhouseContext<'_>,
    query: &str,
    use_columns: &[(&str, bool)],
    // I hate using the hidden type Row::Value here, but it's what next() returns, so I see no other option
    row_get_bucket: impl Fn(&Row::Value<'_>) -> u64,
    row_to_analytics: impl Fn(Row::Value<'_>) -> AnalyticsData,
) -> Result<(), ApiError>
where
    Row: clickhouse::RowRead + serde::de::DeserializeOwned + std::fmt::Debug,
{
    let mut query = cx
        .clickhouse
        .query(query)
        .param("time_range_start", cx.req.time_range.start.timestamp())
        .param("time_range_end", cx.req.time_range.end.timestamp())
        .param("time_slices", cx.time_slices.len())
        .param("project_ids", cx.project_ids);
    for (param_name, used) in use_columns {
        query = query.param(param_name, used)
    }
    let mut cursor = query.fetch::<Row>()?;

    while let Some(row) = cursor.next().await? {
        let bucket = row_get_bucket(&row) as usize;
        add_to_time_slice(cx.time_slices, bucket, row_to_analytics(row))?;
    }

    Ok(())
}

fn add_to_time_slice(
    time_slices: &mut [TimeSlice],
    bucket: usize,
    data: AnalyticsData,
) -> Result<(), ApiError> {
    // row.recorded <  time_range_start => bucket = 0
    // row.recorded >= time_range_end   => bucket = num_time_slices
    //   (note: this is out of range of `time_slices`!)
    let Some(bucket) = bucket.checked_sub(1) else {
        return Ok(());
    };

    let num_time_slices = time_slices.len();
    let slice = time_slices.get_mut(bucket).ok_or_else(|| {
        ApiError::InvalidInput(
            format!("bucket {bucket} returned by query out of range for {num_time_slices} - query bug!")
        )
    })?;

    slice.0.push(data);
    Ok(())
}

async fn filter_allowed_project_ids(
    project_ids: &[DBProjectId],
    user: &crate::models::users::User,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Vec<DBProjectId>, ApiError> {
    let projects = DBProject::get_many_ids(project_ids, pool, redis).await?;

    let team_ids = projects
        .iter()
        .map(|x| x.inner.team_id)
        .collect::<Vec<database::models::DBTeamId>>();
    let team_members = database::models::DBTeamMember::get_from_team_full_many(
        &team_ids, pool, redis,
    )
    .await?;

    let organization_ids = projects
        .iter()
        .filter_map(|x| x.inner.organization_id)
        .collect::<Vec<database::models::DBOrganizationId>>();
    let organizations = database::models::DBOrganization::get_many_ids(
        &organization_ids,
        pool,
        redis,
    )
    .await?;

    let organization_team_ids = organizations
        .iter()
        .map(|x| x.team_id)
        .collect::<Vec<database::models::DBTeamId>>();
    let organization_team_members =
        database::models::DBTeamMember::get_from_team_full_many(
            &organization_team_ids,
            pool,
            redis,
        )
        .await?;

    Ok(projects
        .into_iter()
        .filter(|project| {
            let team_member = team_members.iter().find(|x| {
                x.team_id == project.inner.team_id
                    && x.user_id == user.id.into()
            });

            let organization = project
                .inner
                .organization_id
                .and_then(|oid| organizations.iter().find(|x| x.id == oid));

            let organization_team_member =
                if let Some(organization) = organization {
                    organization_team_members.iter().find(|x| {
                        x.team_id == organization.team_id
                            && x.user_id == user.id.into()
                    })
                } else {
                    None
                };

            let permissions = ProjectPermissions::get_permissions_by_role(
                &user.role,
                &team_member.cloned(),
                &organization_team_member.cloned(),
            )
            .unwrap_or_default();

            permissions.contains(ProjectPermissions::VIEW_ANALYTICS)
        })
        .map(|project| project.inner.id)
        .collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn response_format() {
        let test_project_1 = ProjectId(123);
        let test_project_2 = ProjectId(456);
        let test_project_3 = ProjectId(789);

        let src = FetchResponse(vec![
            TimeSlice(vec![
                AnalyticsData::Project(ProjectAnalytics {
                    source_project: test_project_1,
                    metrics: ProjectMetrics::Views(ProjectViews {
                        domain: Some("youtube.com".into()),
                        views: 100,
                        ..Default::default()
                    }),
                }),
                AnalyticsData::Project(ProjectAnalytics {
                    source_project: test_project_2,
                    metrics: ProjectMetrics::Downloads(ProjectDownloads {
                        domain: Some("discord.com".into()),
                        downloads: 150,
                        ..Default::default()
                    }),
                }),
            ]),
            TimeSlice(vec![AnalyticsData::Project(ProjectAnalytics {
                source_project: test_project_3,
                metrics: ProjectMetrics::Revenue(ProjectRevenue {
                    revenue: Decimal::new(20000, 2),
                }),
            })]),
        ]);
        let target = json!([
            [
                {
                    "source_project": test_project_1.to_string(),
                    "metric_kind": "views",
                    "domain": "youtube.com",
                    "views": 100,
                },
                {
                    "source_project": test_project_2.to_string(),
                    "metric_kind": "downloads",
                    "domain": "discord.com",
                    "downloads": 150,
                }
            ],
            [
                {
                    "source_project": test_project_3.to_string(),
                    "metric_kind": "revenue",
                    "revenue": "200.00",
                }
            ]
        ]);

        assert_eq!(serde_json::to_value(src).unwrap(), target);
    }
}

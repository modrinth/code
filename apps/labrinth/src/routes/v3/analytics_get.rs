//! # Design rationale
//!
//! - different metrics require different scopes
//!   - views, downloads, playtime requires `Scopes::ANALYTICS`
//!   - revenue requires `Scopes::PAYOUTS_READ`
//! - each request returns an array of N elements; if you have to make multiple
//!   requests, you have to zip together M arrays of N elements
//!   - this makes it inconvenient to have separate endpoints

use std::num::NonZeroU64;

use actix_web::{HttpRequest, web};
use chrono::{DateTime, Utc};
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

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("analytics").route("", web::post().to(get)));
}

// request

#[derive(Debug, Serialize, Deserialize)]
struct GetRequest {
    time_range: TimeRange,
    return_metrics: ReturnMetrics,
    // filters: Filters,
}

#[derive(Debug, Serialize, Deserialize)]
struct TimeRange {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    resolution: TimeRangeResolution,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeRangeResolution {
    Minutes(NonZeroU64),
    Slices(NonZeroU64),
}

#[derive(Debug, Serialize, Deserialize)]
struct ReturnMetrics {
    project_views: Option<Metrics<ProjectViewsField>>,
    project_downloads: Option<Metrics<ProjectDownloadsField>>,
    project_playtime: Option<Metrics<ProjectPlaytimeField>>,
    project_revenue: Option<Metrics<()>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Metrics<F> {
    #[serde(default = "Vec::default")]
    bucket_by: Vec<F>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProjectViewsField {
    ProjectId,
    Domain,
    SitePath,
    Monetized,
    Country,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProjectDownloadsField {
    ProjectId,
    VersionId,
    Domain,
    SitePath,
    Country,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProjectPlaytimeField {
    ProjectId,
    VersionId,
    Loader,
    GameVersion,
}

// response

#[derive(Debug, Default, Serialize, Deserialize)]
struct GetResponse(Vec<TimeSlice>);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TimeSlice(Vec<AnalyticsData>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)] // the presence of `source_project`, `source_affiliate_code` determines the kind
enum AnalyticsData {
    Project(ProjectAnalytics),
    // AffiliateCode(AffiliateCodeAnalytics),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectAnalytics {
    source_project: ProjectId,
    #[serde(flatten)]
    metrics: ProjectMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "metric_kind")]
enum ProjectMetrics {
    Views {
        #[serde(skip_serializing_if = "Option::is_none")]
        domain: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        site_path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        monetized: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        country: Option<String>,
        views: u64,
    },
    Downloads {
        #[serde(skip_serializing_if = "Option::is_none")]
        domain: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        site_path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        version_id: Option<VersionId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        country: Option<String>,
        downloads: u64,
    },
    Playtime {
        #[serde(skip_serializing_if = "Option::is_none")]
        version_id: Option<VersionId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        loader: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        game_version: Option<String>,
        seconds: u64,
    },
    Revenue {
        revenue: Decimal,
    },
}

impl From<ProjectAnalytics> for AnalyticsData {
    fn from(value: ProjectAnalytics) -> Self {
        Self::Project(value)
    }
}

// logic

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

async fn get(
    http_req: HttpRequest,
    req: web::Json<GetRequest>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    clickhouse: web::Data<clickhouse::Client>,
) -> Result<web::Json<GetResponse>, ApiError> {
    const MIN_RESOLUTION_MINUTES: u64 = 60;
    const MAX_TIME_SLICES: usize = 1024;

    let (scopes, user) = get_user_from_headers(
        &http_req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::ANALYTICS,
    )
    .await?;

    let num_time_slices = match req.time_range.resolution {
        TimeRangeResolution::Slices(slices) => slices.get(),
        TimeRangeResolution::Minutes(resolution_minutes) => {
            if resolution_minutes.get() < MIN_RESOLUTION_MINUTES {
                return Err(ApiError::InvalidInput(format!(
                    "resolution must be at least {} minutes",
                    MIN_RESOLUTION_MINUTES
                )));
            }

            let range_minutes = u64::try_from(
                (req.time_range.end - req.time_range.start).num_minutes(),
            )
            .map_err(|_| {
                ApiError::InvalidInput(
                    "time range end must be after start".into(),
                )
            })?;

            range_minutes / resolution_minutes
        }
    };
    let num_time_slices = usize::try_from(num_time_slices)
        .expect("u64 should fit within a usize");

    if num_time_slices > MAX_TIME_SLICES {
        return Err(ApiError::InvalidInput(format!(
            "resolution is too fine or range is too large - maximum of {MAX_TIME_SLICES} time slices"
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
                ProjectAnalytics {
                    source_project: row.project_id.into(),
                    metrics: ProjectMetrics::Views {
                        domain: none_if_empty(row.domain),
                        site_path: none_if_empty(row.site_path),
                        monetized: match row.monetized {
                            0 => Some(false),
                            1 => Some(true),
                            _ => None,
                        },
                        country,
                        views: row.views,
                    },
                }
                .into()
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
                ProjectAnalytics {
                    source_project: row.project_id.into(),
                    metrics: ProjectMetrics::Downloads {
                        domain: none_if_empty(row.domain),
                        site_path: none_if_empty(row.site_path),
                        version_id: none_if_zero_version_id(row.version_id),
                        country,
                        downloads: row.downloads,
                    },
                }
                .into()
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
                ProjectAnalytics {
                    source_project: row.project_id.into(),
                    metrics: ProjectMetrics::Playtime {
                        version_id: none_if_zero_version_id(row.version_id),
                        loader: none_if_empty(row.loader),
                        game_version: none_if_empty(row.game_version),
                        seconds: row.seconds,
                    },
                }
                .into()
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
                    ProjectAnalytics {
                        source_project,
                        metrics: ProjectMetrics::Revenue { revenue },
                    }
                    .into(),
                )?;
            }
        }
    }

    Ok(web::Json(GetResponse(time_slices)))
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
    row_get_bucket: impl Fn(&Row) -> u64,
    row_to_analytics: impl Fn(Row) -> AnalyticsData,
) -> Result<(), ApiError>
where
    Row: clickhouse::Row + serde::de::DeserializeOwned + std::fmt::Debug,
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

// async fn filter_allowed_ids(
//     mut project_ids: Option<Vec<String>>,
//     user: crate::models::users::User,
//     pool: &PgPool,
//     redis: &RedisPool,
//     // remove_defaults: Option<bool>,
// ) -> Result<Option<Vec<ProjectId>>, ApiError> {
//     // If no project_ids or version_ids are provided, we default to all projects the user has *public* access to
//     let project_ids = if let Some(ids) = project_ids {
//         ids.into_iter().filter_map()
//     } else {
//         DBUser::get_projects(user.id.into(), &**pool, redis)
//             .await?
//             .into_iter()
//             .map(ProjectId::from)
//     };

//     if project_ids.is_none() && !remove_defaults.unwrap_or(false) {
//         project_ids = Some(
//             user_item::DBUser::get_projects(user.id.into(), &***pool, redis)
//                 .await?
//                 .into_iter()
//                 .map(|x| ProjectId::from(x).to_string())
//                 .collect(),
//         );
//     }

//     // Convert String list to list of ProjectIds or VersionIds
//     // - Filter out unauthorized projects/versions
//     let project_ids = if let Some(project_strings) = project_ids {
//         let projects_data = database::models::DBProject::get_many(
//             &project_strings,
//             &***pool,
//             redis,
//         )
//         .await?;

//         let team_ids = projects_data
//             .iter()
//             .map(|x| x.inner.team_id)
//             .collect::<Vec<database::models::DBTeamId>>();
//         let team_members =
//             database::models::DBTeamMember::get_from_team_full_many(
//                 &team_ids, &***pool, redis,
//             )
//             .await?;

//         let organization_ids = projects_data
//             .iter()
//             .filter_map(|x| x.inner.organization_id)
//             .collect::<Vec<database::models::DBOrganizationId>>();
//         let organizations = database::models::DBOrganization::get_many_ids(
//             &organization_ids,
//             &***pool,
//             redis,
//         )
//         .await?;

//         let organization_team_ids = organizations
//             .iter()
//             .map(|x| x.team_id)
//             .collect::<Vec<database::models::DBTeamId>>();
//         let organization_team_members =
//             database::models::DBTeamMember::get_from_team_full_many(
//                 &organization_team_ids,
//                 &***pool,
//                 redis,
//             )
//             .await?;

//         let ids = projects_data
//             .into_iter()
//             .filter(|project| {
//                 let team_member = team_members.iter().find(|x| {
//                     x.team_id == project.inner.team_id
//                         && x.user_id == user.id.into()
//                 });

//                 let organization = project
//                     .inner
//                     .organization_id
//                     .and_then(|oid| organizations.iter().find(|x| x.id == oid));

//                 let organization_team_member =
//                     if let Some(organization) = organization {
//                         organization_team_members.iter().find(|x| {
//                             x.team_id == organization.team_id
//                                 && x.user_id == user.id.into()
//                         })
//                     } else {
//                         None
//                     };

//                 let permissions = ProjectPermissions::get_permissions_by_role(
//                     &user.role,
//                     &team_member.cloned(),
//                     &organization_team_member.cloned(),
//                 )
//                 .unwrap_or_default();

//                 permissions.contains(ProjectPermissions::VIEW_ANALYTICS)
//             })
//             .map(|x| x.inner.id.into())
//             .collect::<Vec<_>>();

//         Some(ids)
//     } else {
//         None
//     };
//     // Only one of project_ids or version_ids will be Some
//     Ok(project_ids)
// }

// #[cfg(test)]
// mod tests {
//     use serde_json::json;

//     use super::*;

//     #[test]
//     fn response_format() {
//         let test_project_1 = ProjectId(123);
//         let test_project_2 = ProjectId(456);
//         let test_affiliate_code = AffiliateCodeId(789);

//         let src = GetResponse(vec![
//             TimeSlice(vec![
//                 ProjectAnalytics {
//                     game_versions: [
//                         ("1.20.1".to_string(), 400),
//                         ("1.20.2".to_string(), 300),
//                     ]
//                     .into_iter()
//                     .collect(),
//                     ..ProjectAnalytics::new(test_project_1)
//                 }
//                 .into(),
//                 ProjectAnalytics {
//                     game_versions: [
//                         ("1.20.1".to_string(), 200),
//                         ("1.20.2".to_string(), 100),
//                     ]
//                     .into_iter()
//                     .collect(),
//                     ..ProjectAnalytics::new(test_project_2)
//                 }
//                 .into(),
//                 AffiliateCodeAnalytics {
//                     clicks: Some(300),
//                     conversions: Some(200),
//                     ..AffiliateCodeAnalytics::new(test_affiliate_code)
//                 }
//                 .into(),
//             ]),
//             TimeSlice(vec![]),
//         ]);
//         let target = json!([
//             [
//                 {
//                     "source_project": test_project_1.to_string(),
//                     "game_versions": {
//                         "1.20.1": 400,
//                         "1.20.2": 300,
//                     }
//                 },
//                 {
//                     "source_project": test_project_2.to_string(),
//                     "game_versions": {
//                         "1.20.1": 200,
//                         "1.20.2": 100,
//                     }
//                 },
//                 {
//                     "source_affiliate_code": test_affiliate_code.to_string(),
//                     "clicks": 300,
//                     "conversions": 200
//                 }
//             ],
//             []
//         ]);

//         assert_eq!(serde_json::to_value(src).unwrap(), target);
//     }
// }

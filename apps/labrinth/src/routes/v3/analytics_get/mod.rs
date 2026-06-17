//! # Design rationale
//!
//! - different metrics require different scopes
//!   - views, downloads, playtime requires `Scopes::ANALYTICS`
//!   - revenue requires `Scopes::PAYOUTS_READ`
//! - each request returns an array of N elements; if you have to make multiple
//!   requests, you have to zip together M arrays of N elements
//!   - this makes it inconvenient to have separate endpoints

mod facets;
mod metrics;
mod old;

use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU64,
};

use crate::database::{PgPool, models::DBUserId};
use actix_web::{HttpRequest, post, web};
use chrono::{DateTime, TimeDelta, Utc};
use eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::{
    auth::{
        AuthenticationError,
        checks::{filter_visible_project_ids, filter_visible_version_ids},
        get_user_from_headers,
    },
    database::{
        self, DBProject,
        models::version_item::VersionQueryResult,
        models::{
            DBAffiliateCode, DBAffiliateCodeId, DBProjectId, DBUser, DBVersion,
            DBVersionId,
        },
        redis::RedisPool,
    },
    models::{
        ids::{AffiliateCodeId, ProjectId, VersionId},
        pats::Scopes,
        projects::ProjectStatus,
        teams::ProjectPermissions,
        threads::MessageBody,
        v3::{analytics::DownloadReason, projects::Project},
    },
    queue::session::AuthQueue,
    routes::ApiError,
};

#[cfg(test)]
pub(crate) use metrics::normalize_download_source;
pub use metrics::*;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(fetch_analytics);
    cfg.configure(facets::config);
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
    #[serde(default)]
    pub return_metrics: ReturnMetrics,
    /// What project IDs to return data for.
    ///
    /// If this is empty, all of the user's projects will be included.
    #[serde(default)]
    pub project_ids: Vec<ProjectId>,
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

/// Minimum width of a [`TimeSlice`], controlled by [`TimeRange::resolution`].
pub const MIN_RESOLUTION: TimeDelta = TimeDelta::minutes(60);

/// Maximum number of [`TimeSlice`]s in a [`GetResponse`], controlled by
/// [`TimeRange::resolution`].
pub const MAX_TIME_SLICES: usize = 1024;
pub(crate) const UNKNOWN_LOADER: &str = "unknown";
pub(crate) const UNKNOWN_COUNTRY: &str = "XX";
pub(crate) const COUNTRY_PRIVACY_FLOOR: u64 = 50;
pub(crate) const COUNTRY_PLAYTIME_PRIVACY_FLOOR_SECONDS: u64 = 4 * 60 * 60;

// response

/// Response for a [`GetRequest`].
#[derive(Debug, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GetResponse {
    /// List of N [`TimeSlice`]s, where each slice represents an equal
    /// time interval of metrics collection. The number of slices is determined
    /// by [`GetRequest::time_range`].
    pub metrics: Vec<TimeSlice>,
    /// Project metadata for projects referenced in the response metrics.
    #[serde(default)]
    pub projects: HashMap<ProjectId, Project>,
    /// List of events associated with projects that were requested.
    pub project_events: Vec<ProjectAnalyticsEvent>,
}

/// Single time interval of metrics collection.
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TimeSlice(pub Vec<AnalyticsData>);

/// Notable update to a project which may reflect in analytics metrics.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectAnalyticsEvent {
    /// ID of the event's project.
    pub project_id: ProjectId,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    #[serde(flatten)]
    pub kind: ProjectAnalyticsEventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectAnalyticsEventKind {
    /// New version of this project was uploaded.
    VersionUploaded {
        version_id: VersionId,
        version_name: String,
        version_number: String,
    },
    /// Project changed status.
    StatusChanged {
        status_from: ProjectStatus,
        status_to: ProjectStatus,
    },
}

// logic

/// Fetches analytics data for the authorized user's projects.
#[utoipa::path(
    responses((status = OK, body = inline(GetResponse))),
)]
#[post("")]
pub async fn fetch_analytics(
    http_req: HttpRequest,
    req: web::Json<GetRequest>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    clickhouse: web::Data<clickhouse::Client>,
) -> Result<web::Json<GetResponse>, ApiError> {
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
        return Err(ApiError::Request(eyre!(
            "Resolution is too fine or range is too large - maximum of {MAX_TIME_SLICES} time slices, was {num_time_slices}"
        )));
    }
    if resolution < MIN_RESOLUTION {
        return Err(ApiError::Request(eyre!(
            "Resolution must be at least {MIN_RESOLUTION}, was {resolution}",
        )));
    }

    let mut time_slices = vec![TimeSlice::default(); num_time_slices];

    let project_ids = {
        if req.project_ids.is_empty() {
            DBUser::get_projects(user.id.into(), &**pool, &redis).await?
        } else {
            req.project_ids
                .iter()
                .map(|id| DBProjectId::from(*id))
                .collect::<Vec<_>>()
        }
    };

    let project_ids =
        filter_allowed_project_ids(&project_ids, &user, &pool, &redis).await?;

    let project_id_values =
        project_ids.iter().map(|id| id.0).collect::<Vec<_>>();
    let parent_versions = sqlx::query!(
        "
        SELECT id, mod_id
        FROM versions
        WHERE mod_id = ANY($1)
        ",
        &project_id_values,
    )
    .fetch_all(&**pool)
    .await?;
    let parent_version_ids = parent_versions
        .iter()
        .map(|version| DBVersionId(version.id))
        .collect::<Vec<_>>();
    let parent_version_projects = parent_versions
        .iter()
        .map(|version| (DBVersionId(version.id), DBProjectId(version.mod_id)))
        .collect::<HashMap<_, _>>();
    let parent_version_data =
        DBVersion::get_many(&parent_version_ids, &**pool, &redis).await?;
    let visible_version_ids = filter_visible_version_ids(
        parent_version_data
            .iter()
            .map(|version| &version.inner)
            .collect(),
        &Some(user.clone()),
        &pool,
        &redis,
    )
    .await?;
    let mut project_events = parent_version_data
        .iter()
        .filter(|version| {
            visible_version_ids.contains(&version.inner.id)
                && version.inner.date_published >= req.time_range.start
                && version.inner.date_published < req.time_range.end
        })
        .map(|version| ProjectAnalyticsEvent {
            project_id: version.inner.project_id.into(),
            timestamp: version.inner.date_published,
            kind: ProjectAnalyticsEventKind::VersionUploaded {
                version_id: version.inner.id.into(),
                version_name: version.inner.name.clone(),
                version_number: version.inner.version_number.clone(),
            },
        })
        .collect::<Vec<_>>();
    project_events.extend(
        fetch_project_status_change_events(
            &project_ids,
            &req.time_range,
            &pool,
        )
        .await?,
    );
    project_events.sort_by_key(|event| event.timestamp);

    let affiliate_code_ids =
        DBAffiliateCode::get_by_affiliate(user.id.into(), &**pool)
            .await?
            .into_iter()
            .map(|code| code.id)
            .collect::<Vec<_>>();
    let project_loaders = project_loader_map(&parent_version_data);

    let mut query_clickhouse_cx = QueryClickhouseContext {
        clickhouse: &clickhouse,
        pool: &pool,
        redis: &redis,
        req: &req,
        time_slices: &mut time_slices,
        project_ids: &project_ids,
        parent_version_ids: &parent_version_ids,
        affiliate_code_ids: &affiliate_code_ids,
        project_loaders: &project_loaders,
    };

    if let Some(metrics) = &req.return_metrics.project_views {
        metrics::fetch_project_views(&mut query_clickhouse_cx, metrics).await?;
    }

    if let Some(metrics) = &req.return_metrics.project_downloads {
        metrics::fetch_project_downloads(&mut query_clickhouse_cx, metrics)
            .await?;
    }

    if let Some(metrics) = &req.return_metrics.project_playtime {
        metrics::fetch_project_playtime(
            &mut query_clickhouse_cx,
            &parent_version_projects,
            metrics,
        )
        .await?;
    }

    if let Some(metrics) = &req.return_metrics.affiliate_code_clicks {
        metrics::fetch_affiliate_code_clicks(&mut query_clickhouse_cx, metrics)
            .await?;
    }

    if let Some(metrics) = &req.return_metrics.project_revenue {
        if !scopes.contains(Scopes::PAYOUTS_READ) {
            return Err(AuthenticationError::InvalidCredentials.into());
        }

        let user_id_bucket_project_ids = sqlx::query!(
            "
            SELECT m.id
            FROM mods m
            INNER JOIN team_members tm ON tm.team_id = m.team_id
            WHERE
                m.id = ANY($1)
                AND tm.user_id = $2
                AND tm.accepted
            ",
            &project_id_values,
            DBUserId::from(user.id).0,
        )
        .fetch_all(&**pool)
        .await?
        .into_iter()
        .map(|row| row.id)
        .collect::<Vec<_>>();

        metrics::fetch_project_revenue(
            &pool,
            &mut time_slices,
            &req,
            num_time_slices,
            &project_id_values,
            &user_id_bucket_project_ids,
            user.role.is_mod(),
            metrics,
        )
        .await?;
    }

    if let Some(metrics) = &req.return_metrics.affiliate_code_conversions {
        metrics::fetch_affiliate_code_conversions(
            &pool,
            &mut time_slices,
            &req,
            user.id.into(),
            num_time_slices,
            metrics,
        )
        .await?;
    }

    if let Some(metrics) = &req.return_metrics.affiliate_code_revenue {
        if !scopes.contains(Scopes::PAYOUTS_READ) {
            return Err(AuthenticationError::InvalidCredentials.into());
        }

        metrics::fetch_affiliate_code_revenue(
            &pool,
            &mut time_slices,
            &req,
            user.id.into(),
            num_time_slices,
            metrics,
        )
        .await?;
    }

    let projects =
        fetch_response_projects(&mut time_slices, &user, &pool, &redis).await?;

    Ok(web::Json(GetResponse {
        metrics: time_slices,
        projects,
        project_events,
    }))
}

pub(crate) fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

pub(crate) fn none_if_zero_version_id(v: DBVersionId) -> Option<VersionId> {
    if v.0 == 0 { None } else { Some(v.into()) }
}

pub(crate) fn apply_country_privacy(
    country: &mut Option<String>,
    country_filter_applied: bool,
    count: u64,
    floor: u64,
) -> bool {
    if count >= floor {
        return true;
    }

    if country_filter_applied {
        return false;
    }

    if country.is_some() {
        *country = Some(UNKNOWN_COUNTRY.to_string());
    }

    true
}

pub(crate) fn project_loader_map(
    versions: &[VersionQueryResult],
) -> HashMap<DBProjectId, HashSet<String>> {
    let mut loaders = HashMap::<DBProjectId, HashSet<String>>::new();

    for version in versions {
        loaders
            .entry(version.inner.project_id)
            .or_default()
            .extend(version.loaders.iter().cloned());
    }

    loaders
}

pub(crate) fn normalize_loader_for_project(
    loader: String,
    project_id: DBProjectId,
    project_loaders: &HashMap<DBProjectId, HashSet<String>>,
) -> String {
    if loader.is_empty() {
        return loader;
    }

    let loader_is_valid = project_loaders
        .get(&project_id)
        .is_some_and(|loaders| loaders.contains(&loader));

    if loader_is_valid {
        loader
    } else {
        UNKNOWN_LOADER.to_string()
    }
}

async fn fetch_response_projects(
    time_slices: &mut [TimeSlice],
    user: &crate::models::users::User,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<HashMap<ProjectId, Project>, ApiError> {
    let mut project_ids = HashSet::<DBProjectId>::new();

    for time_slice in &*time_slices {
        for data in &time_slice.0 {
            let AnalyticsData::Project(project) = data else {
                continue;
            };

            let source_project_id = DBProjectId::from(project.source_project);
            if source_project_id.0 != 0 {
                project_ids.insert(source_project_id);
            }
            if let ProjectMetrics::Downloads(downloads) = &project.metrics
                && let Some(dependent_project_id) =
                    downloads.dependent_project_id
            {
                project_ids.insert(dependent_project_id.into());
            }
        }
    }

    let project_ids = project_ids.into_iter().collect::<Vec<_>>();
    let projects = DBProject::get_many_ids(&project_ids, pool, redis).await?;
    let visible_project_ids = filter_visible_project_ids(
        projects.iter().map(|project| &project.inner).collect(),
        &Some(user.clone()),
        pool,
        false,
    )
    .await?
    .into_iter()
    .collect::<HashSet<_>>();

    filter_response_project_ids(time_slices, &visible_project_ids);

    Ok(projects
        .into_iter()
        .filter(|project| visible_project_ids.contains(&project.inner.id))
        .map(|project| {
            let project_id = project.inner.id.into();
            (project_id, Project::from(project))
        })
        .collect())
}

fn filter_response_project_ids(
    time_slices: &mut [TimeSlice],
    visible_project_ids: &HashSet<DBProjectId>,
) {
    for time_slice in time_slices {
        time_slice.0.retain_mut(|data| {
            let AnalyticsData::Project(project) = data else {
                return true;
            };

            let source_project_id = DBProjectId::from(project.source_project);
            if source_project_id.0 != 0
                && !visible_project_ids.contains(&source_project_id)
            {
                return false;
            }

            if let ProjectMetrics::Downloads(downloads) = &mut project.metrics
                && let Some(dependent_project_id) =
                    downloads.dependent_project_id
                && !visible_project_ids
                    .contains(&DBProjectId::from(dependent_project_id))
            {
                downloads.dependent_project_id = None;
            }

            true
        });
    }
}

async fn fetch_project_status_change_events(
    project_ids: &[DBProjectId],
    time_range: &TimeRange,
    pool: &PgPool,
) -> Result<Vec<ProjectAnalyticsEvent>, ApiError> {
    let project_id_values =
        project_ids.iter().map(|id| id.0).collect::<Vec<_>>();

    let rows = sqlx::query!(
        r#"
        SELECT
            t.mod_id AS "project_id!",
            tm.created,
            tm.body AS "body: sqlx::types::Json<MessageBody>"
        FROM threads_messages tm
        INNER JOIN threads t ON t.id = tm.thread_id
        WHERE
            t.mod_id = ANY($1)
            AND tm.body->>'type' = 'status_change'
            AND tm.created >= $2
            AND tm.created < $3
        "#,
        &project_id_values,
        time_range.start,
        time_range.end,
    )
    .fetch_all(&**pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let MessageBody::StatusChange {
                old_status,
                new_status,
            } = row.body.0
            else {
                return None;
            };

            Some(ProjectAnalyticsEvent {
                project_id: DBProjectId(row.project_id).into(),
                timestamp: row.created,
                kind: ProjectAnalyticsEventKind::StatusChanged {
                    status_from: old_status,
                    status_to: new_status,
                },
            })
        })
        .collect())
}

pub(crate) struct QueryClickhouseContext<'a> {
    pub(crate) clickhouse: &'a clickhouse::Client,
    pub(crate) pool: &'a PgPool,
    pub(crate) redis: &'a RedisPool,
    pub(crate) req: &'a GetRequest,
    pub(crate) time_slices: &'a mut [TimeSlice],
    pub(crate) project_ids: &'a [DBProjectId],
    pub(crate) parent_version_ids: &'a [DBVersionId],
    pub(crate) affiliate_code_ids: &'a [DBAffiliateCodeId],
    pub(crate) project_loaders: &'a HashMap<DBProjectId, HashSet<String>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ClickhouseQueryParams {
    pub(crate) project_ids: bool,
    pub(crate) parent_version_ids: bool,
    pub(crate) affiliate_code_ids: bool,
}

pub(crate) enum ClickhouseFilterParam<'a> {
    String(&'a [String]),
    Bool(&'static str, &'a [bool]),
    VersionId(&'a [VersionId]),
    AffiliateCodeId(&'a [AffiliateCodeId]),
    DownloadReason(&'a [DownloadReason]),
}

impl ClickhouseFilterParam<'_> {
    pub(crate) fn bind(
        self,
        query: clickhouse::query::Query,
    ) -> clickhouse::query::Query {
        match self {
            Self::String(values) => query.bind(values),
            Self::Bool(name, values) => {
                let value = match values {
                    [false] => 0,
                    [true] => 1,
                    _ => 2,
                };
                query.param(name, value)
            }
            Self::VersionId(values) => {
                let values = values
                    .iter()
                    .map(|id| DBVersionId::from(*id))
                    .collect::<Vec<_>>();
                query.bind(values)
            }
            Self::AffiliateCodeId(values) => {
                let values = values
                    .iter()
                    .map(|id| DBAffiliateCodeId::from(*id))
                    .collect::<Vec<_>>();
                query.bind(values)
            }
            Self::DownloadReason(values) => {
                let values =
                    values.iter().map(ToString::to_string).collect::<Vec<_>>();
                query.bind(values)
            }
        }
    }
}

impl ClickhouseQueryParams {
    pub(crate) const fn empty() -> Self {
        Self {
            project_ids: false,
            parent_version_ids: false,
            affiliate_code_ids: false,
        }
    }
}

impl std::ops::BitOr for ClickhouseQueryParams {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            project_ids: self.project_ids || rhs.project_ids,
            parent_version_ids: self.parent_version_ids
                || rhs.parent_version_ids,
            affiliate_code_ids: self.affiliate_code_ids
                || rhs.affiliate_code_ids,
        }
    }
}

pub(crate) async fn query_clickhouse<Row>(
    cx: &mut QueryClickhouseContext<'_>,
    query: &str,
    params: ClickhouseQueryParams,
    use_columns: &[(&str, bool)],
    filter_params: Vec<ClickhouseFilterParam<'_>>,
    row_filter: impl Fn(&Row::Value<'_>) -> bool,
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
        .param("time_slices", cx.time_slices.len());
    if params.project_ids {
        query = query.bind(cx.project_ids);
    }
    if params.parent_version_ids {
        query = query.bind(cx.parent_version_ids);
    }
    if params.affiliate_code_ids {
        query = query.bind(cx.affiliate_code_ids);
    }
    for (param_name, used) in use_columns {
        query = query.param(param_name, used)
    }
    for filter_param in filter_params {
        query = filter_param.bind(query);
    }
    let mut cursor = query.fetch::<Row>()?;

    while let Some(row) = cursor.next().await? {
        if !row_filter(&row) {
            continue;
        }
        let bucket = row_get_bucket(&row) as usize;
        add_to_time_slice(cx.time_slices, bucket, row_to_analytics(row))?;
    }

    Ok(())
}

pub(crate) fn add_to_time_slice(
    time_slices: &mut [TimeSlice],
    bucket: usize,
    data: AnalyticsData,
) -> Result<(), ApiError> {
    // Bucketed analytics queries must filter time ranges as `[start, end)`.
    // `widthBucket` returns `num_time_slices + 1` for values at or after
    // `end`, which is outside the response slice array.
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
    use rust_decimal::Decimal;
    use serde_json::json;

    use super::*;

    #[test]
    fn normalizes_download_sources() {
        let cases = [
            ("MultiMC/5.0", Some(DownloadSource::Named("MultiMC".into()))),
            (
                "PrismLauncher/6.1",
                Some(DownloadSource::Named("Prism Launcher".into())),
            ),
            (
                "modrinth/theseus/0.8.6 (support@modrinth.com)",
                Some(DownloadSource::ModrinthApp),
            ),
            (
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15",
                Some(DownloadSource::Website),
            ),
            ("curl/8.7.1", None),
        ];

        for (user_agent, source) in cases {
            assert_eq!(normalize_download_source(user_agent), source);
        }
    }

    #[test]
    fn download_source_serializes_as_raw_string() {
        assert_eq!(
            serde_json::to_value(DownloadSource::Named("MultiMC".into()))
                .unwrap(),
            json!("MultiMC")
        );
        assert_eq!(
            serde_json::to_value(DownloadSource::Website).unwrap(),
            json!("website")
        );
        assert_eq!(
            serde_json::to_value(DownloadSource::ModrinthApp).unwrap(),
            json!("modrinth_app")
        );
        assert_eq!(
            serde_json::to_value(DownloadSource::Other).unwrap(),
            json!("other")
        );
    }

    #[test]
    fn country_privacy_floor_suppresses_small_constrained_buckets() {
        let mut country = None;
        assert!(apply_country_privacy(
            &mut country,
            false,
            1,
            COUNTRY_PRIVACY_FLOOR
        ));
        assert_eq!(country, None);

        let mut country = Some("US".into());
        assert!(apply_country_privacy(
            &mut country,
            false,
            49,
            COUNTRY_PRIVACY_FLOOR
        ));
        assert_eq!(country, Some("XX".into()));

        let mut country = Some("US".into());
        assert!(!apply_country_privacy(
            &mut country,
            true,
            49,
            COUNTRY_PRIVACY_FLOOR
        ));
        assert_eq!(country, Some("US".into()));

        let mut country = Some("US".into());
        assert!(apply_country_privacy(
            &mut country,
            true,
            50,
            COUNTRY_PRIVACY_FLOOR
        ));
        assert_eq!(country, Some("US".into()));
    }

    #[test]
    fn response_format() {
        let test_project_1 = ProjectId(123);
        let test_project_2 = ProjectId(456);
        let test_project_3 = ProjectId(789);

        let src = GetResponse {
            metrics: vec![
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
                        user_id: None,
                        revenue: Decimal::new(20000, 2),
                    }),
                })]),
            ],
            projects: HashMap::new(),
            project_events: vec![],
        };
        let target = json!({
            "metrics": [
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
            ],
            "projects": {},
            "project_events": []
        });

        assert_eq!(serde_json::to_value(src).unwrap(), target);
    }
}

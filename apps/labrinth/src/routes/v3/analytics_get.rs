use std::num::NonZeroU32;

use actix_web::{HttpRequest, web};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    auth::get_user_from_headers,
    database::{
        self, DBProject,
        models::{DBProjectId, DBUser},
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
    include: Vec<MetricField>,
    // filters: Filters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MetricField {
    ProjectId,
    ViewDomain,
    ViewSitePath,
    DownloadDomain,
    DownloadSitePath,
    DownloadVersionId,
}

#[derive(Debug, Serialize, Deserialize)]
struct TimeRange {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    num_slices: NonZeroU32,
    // resolution_minutes: NonZeroU32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Filters {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum AnalyticsKind {}

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
        domain: Option<String>,
        site_path: Option<String>,
        views: u64,
    },
    Downloads {
        domain: Option<String>,
        site_path: Option<String>,
        version_id: Option<VersionId>,
        downloads: u64,
    },
}

impl From<ProjectAnalytics> for AnalyticsData {
    fn from(value: ProjectAnalytics) -> Self {
        Self::Project(value)
    }
}

// #[derive(Debug, Clone, Default, Serialize, Deserialize)]
// struct AffiliateCodeAnalytics {
//     #[serde(skip_serializing_if = "Option::is_none")]
//     clicks: Option<u64>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     conversions: Option<u64>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     revenue: Option<u64>,
// }

// impl AffiliateCodeAnalytics {
//     fn new(source_affiliate_code: AffiliateCodeId) -> Self {
//         Self {
//             source_affiliate_code,
//             clicks: None,
//             conversions: None,
//             revenue: None,
//         }
//     }
// }

// impl From<AffiliateCodeAnalytics> for AnalyticsData {
//     fn from(value: AffiliateCodeAnalytics) -> Self {
//         Self::AffiliateCode(value)
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
// #[serde(rename_all = "snake_case")]
// enum RevenueSource {
//     Adverts,
//     ModrinthPlus,
// }

// logic

mod query {
    use super::MetricField;
    use const_format::formatcp;

    const TIME_RANGE_START: &str = "{time_range_start: UInt64}";
    const TIME_RANGE_END: &str = "{time_range_end: UInt64}";
    const TIME_SLICES: &str = "{time_slices: UInt64}";
    const INCLUDE_FIELDS: &str = "{include_fields: UInt64}";
    const PROJECT_IDS: &str = "{project_ids: Array(UInt64)}";

    bitflags::bitflags! {
        pub struct ViewsField: u64 {
            const PROJECT_ID = 1 << 1;
            const DOMAIN = 1 << 2;
            const SITE_PATH = 1 << 3;
        }
    }

    impl ViewsField {
        pub fn from_metric(field: MetricField) -> Option<Self> {
            match field {
                MetricField::ProjectId => Some(Self::PROJECT_ID),
                MetricField::ViewDomain => Some(Self::DOMAIN),
                MetricField::ViewSitePath => Some(Self::SITE_PATH),
                _ => None,
            }
        }
    }

    #[derive(Debug, clickhouse::Row, serde::Deserialize)]
    pub struct ViewRow {
        pub bucket: u64,
        pub project_id: u64,
        pub domain: String,
        pub site_path: String,
        pub views: u64,
    }

    pub const VIEWS: &str = {
        const HAS_PROJECT_ID: &str = formatcp!("bitTest({INCLUDE_FIELDS}, 0)");
        const HAS_DOMAIN: &str = formatcp!("bitTest({INCLUDE_FIELDS}, 1)");
        const HAS_SITE_PATH: &str = formatcp!("bitTest({INCLUDE_FIELDS}, 2)");

        formatcp!(
            "
            SELECT
                widthBucket(toUnixTimestamp(recorded), {TIME_RANGE_START}, {TIME_RANGE_END}, {TIME_SLICES}) AS bucket,
                if({HAS_PROJECT_ID}, project_id, 0) AS project_id,
                if({HAS_DOMAIN}, domain, '') AS domain,
                if({HAS_SITE_PATH}, site_path, '') AS site_path,
                COUNT(*) AS views
            FROM views
            WHERE
                recorded BETWEEN {TIME_RANGE_START} AND {TIME_RANGE_END}
                AND project_id IN {PROJECT_IDS}
            GROUP BY
                bucket, project_id, domain, site_path
            HAVING
                ({HAS_DOMAIN} AND domain != '') OR
                ({HAS_SITE_PATH} AND site_path != '') OR
                {HAS_PROJECT_ID}"
        )
    };

    bitflags::bitflags! {
        pub struct DownloadsField: u64 {
            const PROJECT_ID = 1 << 1;
            const DOMAIN = 1 << 2;
            const SITE_PATH = 1 << 3;
            const VERSION_ID = 1 << 4;
        }
    }

    impl DownloadsField {
        pub fn from_metric(field: MetricField) -> Option<Self> {
            match field {
                MetricField::ProjectId => Some(Self::PROJECT_ID),
                MetricField::DownloadDomain => Some(Self::DOMAIN),
                MetricField::DownloadSitePath => Some(Self::SITE_PATH),
                MetricField::DownloadVersionId => Some(Self::VERSION_ID),
                _ => None,
            }
        }
    }

    #[derive(Debug, clickhouse::Row, serde::Deserialize)]
    pub struct DownloadRow {
        pub bucket: u64,
        pub project_id: u64,
        pub domain: String,
        pub site_path: String,
        pub version_id: u64,
        pub downloads: u64,
    }

    pub const DOWNLOADS: &str = {
        const HAS_PROJECT_ID: &str = formatcp!("bitTest({INCLUDE_FIELDS}, 0)");
        const HAS_DOMAIN: &str = formatcp!("bitTest({INCLUDE_FIELDS}, 1)");
        const HAS_SITE_PATH: &str = formatcp!("bitTest({INCLUDE_FIELDS}, 2)");
        const HAS_VERSION_ID: &str = formatcp!("bitTest({INCLUDE_FIELDS}, 3)");

        formatcp!(
            "
            SELECT
                widthBucket(toUnixTimestamp(recorded), {TIME_RANGE_START}, {TIME_RANGE_END}, {TIME_SLICES}) AS bucket,
                if({HAS_PROJECT_ID}, project_id, 0) AS project_id,
                if({HAS_DOMAIN}, domain, '') AS domain,
                if({HAS_SITE_PATH}, site_path, '') AS site_path,
                if({HAS_VERSION_ID}, version_id, 0) AS version_id,
                COUNT(*) AS downloads
            FROM downloads
            WHERE
                recorded BETWEEN {TIME_RANGE_START} AND {TIME_RANGE_END}
                AND project_id IN {PROJECT_IDS}
            GROUP BY
                bucket, project_id, domain, site_path, version_id
            HAVING
                ({HAS_DOMAIN} AND domain != '') OR
                ({HAS_SITE_PATH} AND site_path != '') OR
                ({HAS_VERSION_ID} AND version_id != 0) OR
                {HAS_PROJECT_ID}"
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
    let (_, user) = get_user_from_headers(
        &http_req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::ANALYTICS,
    )
    .await?;

    let time_range_start = req.time_range.start.timestamp();
    let time_range_end = req.time_range.end.timestamp();
    let num_time_slices = req.time_range.num_slices.get() as usize;
    if num_time_slices > 256 {
        return Err(ApiError::InvalidInput("too many time slices".to_string()));
    }
    let mut time_slices = vec![TimeSlice::default(); num_time_slices];

    let project_ids =
        DBUser::get_projects(user.id.into(), &**pool, &redis).await?;

    let project_ids =
        filter_allowed_project_ids(&project_ids, &user, &pool, &redis).await?;

    let mut cursor = clickhouse
        .query(query::VIEWS)
        .param("time_range_start", time_range_start)
        .param("time_range_end", time_range_end)
        .param("time_slices", num_time_slices)
        .param(
            "include_fields",
            req.include
                .iter()
                .copied()
                .filter_map(query::ViewsField::from_metric)
                .collect::<query::ViewsField>()
                .bits(),
        )
        .param("project_ids", &project_ids)
        .fetch::<query::ViewRow>()?;

    while let Some(row) = cursor.next().await? {
        // row.recorded <  time_range_start => bucket = 0
        // row.recorded >= time_range_end   => bucket = num_time_slices
        //   (note: this is out of range of `time_slices`!)
        let Some(bucket) = (row.bucket as usize).checked_sub(1) else {
            continue;
        };

        let slice = time_slices.get_mut(bucket).ok_or_else(|| {
            ApiError::InvalidInput(format!(
                "bucket {} returned by query out of range for {num_time_slices} - query bug!",
                row.bucket,
            ))
        })?;

        slice.0.push(
            ProjectAnalytics {
                source_project: ProjectId(row.project_id),
                metrics: ProjectMetrics::Views {
                    domain: Some(row.domain),
                    site_path: Some(row.site_path),
                    views: row.views,
                },
            }
            .into(),
        );
    }

    Ok(web::Json(GetResponse(time_slices)))

    // "
    // -- SELECT bucket, project_id, 1 AS metric_type, COUNT(*) AS total
    // -- FROM (
    // --     SELECT
    // --         widthBucket(
    // --             toUnixTimestamp(recorded),
    // --             {time_range_start: UInt64},
    // --             {time_range_end: UInt64},
    // --             {time_slices: UInt64}
    // --         ) AS bucket,
    // --         project_id
    // --     FROM views
    // --     WHERE
    // --         recorded BETWEEN toDateTime({time_range_start: UInt64}) AND toDateTime({time_range_end: UInt64})
    // --         AND project_id IN {project_ids: Array(UInt64)}
    // -- )
    // -- GROUP BY bucket, project_id

    // -- UNION ALL

    // SELECT bucket, project_id, 2 AS metric_type, COUNT(*) AS total
    // FROM (
    //     SELECT
    //         widthBucket(
    //             toUnixTimestamp(recorded),
    //             {time_range_start: UInt64},
    //             {time_range_end: UInt64},
    //             {time_slices: UInt64}
    //         ) AS bucket,
    //         if(bitTest({include_fields: UInt64}, 0)) project_id
    //     FROM downloads
    //     WHERE
    //         recorded BETWEEN toDateTime({time_range_start: UInt64}) AND toDateTime({time_range_end: UInt64})
    //         AND project_id IN {project_ids: Array(UInt64)}
    // )
    // GROUP BY
    //     bucket, project_id, domain, site_path
    // HAVING
    //     (bitTest({include_fields: UInt64}, 0) AND domain != '') OR
    //     (bitTest({include_fields: UInt64}, 1) AND site_path != '') OR

    // UNION ALL

    // SELECT bucket, project_id, 3 AS metric_type, SUM(seconds) AS total
    // FROM (
    //     SELECT
    //         widthBucket(
    //             toUnixTimestamp(recorded),
    //             {time_range_start: UInt64},
    //             {time_range_end: UInt64},
    //             {time_slices: UInt64}
    //         ) AS bucket,
    //         project_id,
    //         seconds
    //     FROM playtime
    //     WHERE
    //         recorded BETWEEN toDateTime({time_range_start: UInt64}) AND toDateTime({time_range_end: UInt64})
    //         AND project_id IN {project_ids: Array(UInt64)}
    // )
    // GROUP BY bucket, project_id
    // ORDER BY bucket, metric_type, project_id"
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

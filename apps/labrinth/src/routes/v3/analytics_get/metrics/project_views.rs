use std::collections::HashMap;

use const_format::formatcp;
use serde::{Deserialize, Serialize};

use crate::{database::models::DBProjectId, routes::ApiError};

use super::super::{
    COUNTRY_PRIVACY_FLOOR, ClickhouseFilterParam, QueryClickhouseContext,
    add_to_time_slice, apply_country_privacy, none_if_empty,
};
use super::{AnalyticsData, Metrics, ProjectAnalytics, ProjectMetrics};

const TIME_RANGE_START: &str = "{time_range_start: UInt64}";
const TIME_RANGE_END: &str = "{time_range_end: UInt64}";
const TIME_SLICES: &str = "{time_slices: UInt64}";
const PROJECT_IDS: &str = "project_ids";

/// Fields for [`super::ReturnMetrics::project_views`].
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

/// Filters for [`super::ReturnMetrics::project_views`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectViewsFilters {
    /// Referrer domains to include.
    #[serde(default)]
    pub domain: Vec<String>,
    /// Modrinth site paths to include.
    #[serde(default)]
    pub site_path: Vec<String>,
    /// Monetization states to include.
    #[serde(default)]
    pub monetized: Vec<bool>,
    /// Country codes to include.
    #[serde(default)]
    pub country: Vec<String>,
}

/// [`super::ReturnMetrics::project_views`].
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

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct ViewRow {
    bucket: u64,
    project_id: DBProjectId,
    domain: String,
    site_path: String,
    monetized: i8,
    country: String,
    views: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ViewBucket {
    bucket: u64,
    project_id: DBProjectId,
    domain: Option<String>,
    site_path: Option<String>,
    monetized: Option<bool>,
    country: Option<String>,
}

const VIEWS: &str = {
    const USE_PROJECT_ID: &str = "{use_project_id: Bool}";
    const USE_DOMAIN: &str = "{use_domain: Bool}";
    const USE_SITE_PATH: &str = "{use_site_path: Bool}";
    const USE_MONETIZED: &str = "{use_monetized: Bool}";
    const USE_COUNTRY: &str = "{use_country: Bool}";
    const FILTER_DOMAIN: &str = "filter_domain";
    const FILTER_SITE_PATH: &str = "filter_site_path";
    const FILTER_MONETIZED: &str = "{filter_monetized: UInt8}";
    const FILTER_COUNTRY: &str = "filter_country";

    formatcp!(
        "WITH
            ? AS {PROJECT_IDS},
            ? AS {FILTER_DOMAIN},
            ? AS {FILTER_SITE_PATH},
            ? AS {FILTER_COUNTRY}
        SELECT
            widthBucket(toUnixTimestamp(recorded), {TIME_RANGE_START}, {TIME_RANGE_END}, {TIME_SLICES}) AS bucket,
            if({USE_PROJECT_ID}, project_id, 0) AS project_id,
            if({USE_DOMAIN}, domain, '') AS domain,
            if({USE_SITE_PATH}, site_path, '') AS site_path,
            if({USE_MONETIZED}, CAST(monetized AS Int8), -1) AS monetized,
            if({USE_COUNTRY}, country, '') AS country,
            COUNT(*) AS views
        FROM views
        WHERE
            recorded >= {TIME_RANGE_START}
            AND recorded < {TIME_RANGE_END}
            -- make sure that the REAL project id is included,
            -- not the possibly-zero one,
            -- by using `views.project_id` instead of `project_id`
            AND views.project_id IN {PROJECT_IDS}
            AND (empty({FILTER_DOMAIN}) OR views.domain IN {FILTER_DOMAIN})
            AND (empty({FILTER_SITE_PATH}) OR views.site_path IN {FILTER_SITE_PATH})
            AND ({FILTER_MONETIZED} = 2 OR CAST(views.monetized AS UInt8) = {FILTER_MONETIZED})
            AND (empty({FILTER_COUNTRY}) OR views.country IN {FILTER_COUNTRY})
        GROUP BY bucket, project_id, domain, site_path, monetized, country
        "
    )
};

pub(crate) async fn fetch(
    cx: &mut QueryClickhouseContext<'_>,
    metrics: &Metrics<ProjectViewsField, ProjectViewsFilters>,
) -> Result<(), ApiError> {
    use ProjectViewsField as F;
    let uses = |field| metrics.bucket_by.contains(&field);
    let use_columns = &[
        ("use_project_id", uses(F::ProjectId)),
        ("use_domain", uses(F::Domain)),
        ("use_site_path", uses(F::SitePath)),
        ("use_monetized", uses(F::Monetized)),
        ("use_country", uses(F::Country)),
    ];
    let uses_column = |name| {
        use_columns
            .iter()
            .any(|(column_name, used)| *column_name == name && *used)
    };

    let mut query = cx
        .clickhouse
        .query(VIEWS)
        .param("time_range_start", cx.req.time_range.start.timestamp())
        .param("time_range_end", cx.req.time_range.end.timestamp())
        .param("time_slices", cx.time_slices.len())
        .bind(cx.project_ids);
    for (param_name, used) in use_columns {
        query = query.param(param_name, used)
    }
    for filter_param in [
        ClickhouseFilterParam::String(&metrics.filter_by.domain),
        ClickhouseFilterParam::String(&metrics.filter_by.site_path),
        ClickhouseFilterParam::Bool(
            "filter_monetized",
            &metrics.filter_by.monetized,
        ),
        ClickhouseFilterParam::String(&metrics.filter_by.country),
    ] {
        query = filter_param.bind(query);
    }

    let mut cursor = query.fetch::<ViewRow>()?;
    let mut buckets = HashMap::<ViewBucket, u64>::new();

    while let Some(row) = cursor.next().await? {
        let key = ViewBucket {
            bucket: row.bucket,
            project_id: row.project_id,
            domain: uses_column("use_domain").then(|| row.domain.clone()),
            site_path: uses_column("use_site_path")
                .then(|| row.site_path.clone()),
            monetized: if uses_column("use_monetized") {
                match row.monetized {
                    0 => Some(false),
                    1 => Some(true),
                    _ => None,
                }
            } else {
                None
            },
            country: uses_column("use_country").then(|| row.country.clone()),
        };

        *buckets.entry(key).or_default() += row.views;
    }

    let mut output_buckets = HashMap::<ViewBucket, u64>::new();
    for (mut key, views) in buckets {
        if !apply_country_privacy(
            &mut key.country,
            !metrics.filter_by.country.is_empty(),
            views,
            COUNTRY_PRIVACY_FLOOR,
        ) {
            continue;
        }
        *output_buckets.entry(key).or_default() += views;
    }

    for (key, views) in output_buckets {
        add_to_time_slice(
            cx.time_slices,
            key.bucket as usize,
            AnalyticsData::Project(ProjectAnalytics {
                source_project: key.project_id.into(),
                metrics: ProjectMetrics::Views(ProjectViews {
                    domain: key.domain.and_then(none_if_empty),
                    site_path: key.site_path.and_then(none_if_empty),
                    monetized: key.monetized,
                    country: key.country,
                    views,
                }),
            }),
        )?;
    }

    Ok(())
}

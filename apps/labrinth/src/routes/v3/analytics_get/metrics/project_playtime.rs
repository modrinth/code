use std::collections::HashMap;

use const_format::formatcp;
use serde::{Deserialize, Serialize};

use crate::{
    database::models::{DBProjectId, DBVersionId},
    models::ids::VersionId,
    routes::ApiError,
};

use super::super::{
    ClickhouseFilterParam, QueryClickhouseContext, add_to_time_slice,
    none_if_empty, none_if_zero_version_id, normalize_loader_for_project,
    passes_country_privacy_floor,
};
use super::{AnalyticsData, Metrics, ProjectAnalytics, ProjectMetrics};

const TIME_RANGE_START: &str = "{time_range_start: UInt64}";
const TIME_RANGE_END: &str = "{time_range_end: UInt64}";
const TIME_SLICES: &str = "{time_slices: UInt64}";
const PROJECT_IDS: &str = "project_ids";

/// Fields for [`super::ReturnMetrics::project_playtime`].
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
    /// What country this playtime came from.
    ///
    /// To anonymize the data, the country may be reported as `XX`.
    Country,
}

/// Filters for [`super::ReturnMetrics::project_playtime`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectPlaytimeFilters {
    /// Version IDs to include.
    #[serde(default)]
    pub version_id: Vec<VersionId>,
    /// Loaders to include.
    #[serde(default)]
    pub loader: Vec<String>,
    /// Game versions to include.
    #[serde(default)]
    pub game_version: Vec<String>,
    /// Country codes to include.
    #[serde(default)]
    pub country: Vec<String>,
}

/// [`super::ReturnMetrics::project_playtime`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectPlaytime {
    /// [`ProjectPlaytimeField::VersionId`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) version_id: Option<VersionId>,
    /// [`ProjectPlaytimeField::Loader`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) loader: Option<String>,
    /// [`ProjectPlaytimeField::GameVersion`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) game_version: Option<String>,
    /// [`ProjectPlaytimeField::Country`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) country: Option<String>,
    /// Total number of seconds of playtime for this bucket.
    pub(crate) seconds: u64,
}

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct PlaytimeRow {
    bucket: u64,
    source_project_id: DBProjectId,
    project_id: DBProjectId,
    parent_version_id: DBVersionId,
    version_id: DBVersionId,
    loader: String,
    game_version: String,
    country: String,
    seconds: u64,
}

const PLAYTIME: &str = {
    const USE_PROJECT_ID: &str = "{use_project_id: Bool}";
    const USE_VERSION_ID: &str = "{use_version_id: Bool}";
    const USE_LOADER: &str = "{use_loader: Bool}";
    const USE_GAME_VERSION: &str = "{use_game_version: Bool}";
    const USE_COUNTRY: &str = "{use_country: Bool}";
    const PARENT_VERSION_IDS: &str = "parent_version_ids";
    const FILTER_VERSION_ID: &str = "filter_version_id";
    const FILTER_LOADER: &str = "filter_loader";
    const FILTER_GAME_VERSION: &str = "filter_game_version";
    const FILTER_COUNTRY: &str = "filter_country";

    formatcp!(
        "WITH
            ? AS {PROJECT_IDS},
            ? AS {PARENT_VERSION_IDS},
            ? AS {FILTER_VERSION_ID},
            ? AS {FILTER_LOADER},
            ? AS {FILTER_GAME_VERSION},
            ? AS {FILTER_COUNTRY}
        SELECT
            bucket,
            source_project_id,
            if({USE_PROJECT_ID}, source_project_id, 0) AS project_id,
            parent_version_id,
            version_id,
            loader,
            game_version,
            country,
            SUM(seconds) AS seconds
        FROM (
            SELECT
                widthBucket(toUnixTimestamp(recorded), {TIME_RANGE_START}, {TIME_RANGE_END}, {TIME_SLICES}) AS bucket,
                project_id AS source_project_id,
                0 AS parent_version_id,
                if({USE_VERSION_ID}, version_id, 0) AS version_id,
                if({USE_LOADER}, loader, '') AS loader,
                if({USE_GAME_VERSION}, game_version, '') AS game_version,
                if({USE_COUNTRY}, country, '') AS country,
                seconds
            FROM playtime
            WHERE
                recorded >= {TIME_RANGE_START}
                AND recorded < {TIME_RANGE_END}
                AND playtime.project_id IN {PROJECT_IDS}
                AND (empty({FILTER_VERSION_ID}) OR playtime.version_id IN {FILTER_VERSION_ID})
                AND (empty({FILTER_LOADER}) OR playtime.loader IN {FILTER_LOADER})
                AND (empty({FILTER_GAME_VERSION}) OR playtime.game_version IN {FILTER_GAME_VERSION})
                AND (empty({FILTER_COUNTRY}) OR playtime.country IN {FILTER_COUNTRY})

            UNION ALL

            SELECT
                widthBucket(toUnixTimestamp(recorded), {TIME_RANGE_START}, {TIME_RANGE_END}, {TIME_SLICES}) AS bucket,
                0 AS source_project_id,
                parent AS parent_version_id,
                if({USE_VERSION_ID}, version_id, 0) AS version_id,
                if({USE_LOADER}, loader, '') AS loader,
                if({USE_GAME_VERSION}, game_version, '') AS game_version,
                if({USE_COUNTRY}, country, '') AS country,
                seconds
            FROM playtime
            WHERE
                recorded >= {TIME_RANGE_START}
                AND recorded < {TIME_RANGE_END}
                AND parent IN {PARENT_VERSION_IDS}
                AND (empty({FILTER_VERSION_ID}) OR playtime.version_id IN {FILTER_VERSION_ID})
                AND (empty({FILTER_LOADER}) OR playtime.loader IN {FILTER_LOADER})
                AND (empty({FILTER_GAME_VERSION}) OR playtime.game_version IN {FILTER_GAME_VERSION})
                AND (empty({FILTER_COUNTRY}) OR playtime.country IN {FILTER_COUNTRY})
        )
        GROUP BY bucket, source_project_id, project_id, parent_version_id, version_id, loader, game_version, country"
    )
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PlaytimeBucket {
    bucket: u64,
    project_id: DBProjectId,
    version_id: Option<DBVersionId>,
    loader: Option<String>,
    game_version: Option<String>,
    country: Option<String>,
}

pub(crate) async fn fetch(
    cx: &mut QueryClickhouseContext<'_>,
    parent_version_projects: &HashMap<DBVersionId, DBProjectId>,
    metrics: &Metrics<ProjectPlaytimeField, ProjectPlaytimeFilters>,
) -> Result<(), ApiError> {
    use ProjectPlaytimeField as F;
    let uses = |field| metrics.bucket_by.contains(&field);
    let use_columns = &[
        ("use_project_id", uses(F::ProjectId)),
        ("use_version_id", uses(F::VersionId)),
        ("use_loader", uses(F::Loader)),
        ("use_game_version", uses(F::GameVersion)),
        ("use_country", uses(F::Country)),
    ];
    let uses_column = |name| {
        use_columns
            .iter()
            .any(|(column_name, used)| *column_name == name && *used)
    };

    let mut query = cx
        .clickhouse
        .query(PLAYTIME)
        .param("time_range_start", cx.req.time_range.start.timestamp())
        .param("time_range_end", cx.req.time_range.end.timestamp())
        .param("time_slices", cx.time_slices.len())
        .bind(cx.project_ids)
        .bind(cx.parent_version_ids);
    for (param_name, used) in use_columns {
        query = query.param(param_name, used)
    }
    for filter_param in [
        ClickhouseFilterParam::VersionId(&metrics.filter_by.version_id),
        ClickhouseFilterParam::String(&metrics.filter_by.loader),
        ClickhouseFilterParam::String(&metrics.filter_by.game_version),
        ClickhouseFilterParam::String(&metrics.filter_by.country),
    ] {
        query = filter_param.bind(query);
    }

    let mut cursor = query.fetch::<PlaytimeRow>()?;
    let mut buckets = HashMap::<PlaytimeBucket, u64>::new();

    while let Some(row) = cursor.next().await? {
        let project_id =
            if uses_column("use_project_id") && row.project_id.0 == 0 {
                parent_version_projects
                    .get(&row.parent_version_id)
                    .copied()
                    .unwrap_or(row.project_id)
            } else {
                row.project_id
            };
        let source_project_id = if row.source_project_id.0 == 0 {
            parent_version_projects
                .get(&row.parent_version_id)
                .copied()
                .unwrap_or(row.source_project_id)
        } else {
            row.source_project_id
        };
        let key = PlaytimeBucket {
            bucket: row.bucket,
            project_id,
            version_id: uses_column("use_version_id").then_some(row.version_id),
            loader: uses_column("use_loader").then(|| {
                normalize_loader_for_project(
                    row.loader.clone(),
                    source_project_id,
                    cx.project_loaders,
                )
            }),
            game_version: uses_column("use_game_version")
                .then(|| row.game_version.clone()),
            country: uses_column("use_country").then(|| row.country.clone()),
        };

        *buckets.entry(key).or_default() += row.seconds;
    }

    for (key, seconds) in buckets {
        if !passes_country_privacy_floor(
            key.country.is_some() || !metrics.filter_by.country.is_empty(),
            seconds,
        ) {
            continue;
        }

        add_to_time_slice(
            cx.time_slices,
            key.bucket as usize,
            AnalyticsData::Project(ProjectAnalytics {
                source_project: key.project_id.into(),
                metrics: ProjectMetrics::Playtime(ProjectPlaytime {
                    version_id: key
                        .version_id
                        .and_then(none_if_zero_version_id),
                    loader: key.loader.and_then(none_if_empty),
                    game_version: key.game_version.and_then(none_if_empty),
                    country: key.country,
                    seconds,
                }),
            }),
        )?;
    }

    Ok(())
}

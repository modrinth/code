use std::{
    collections::HashMap,
    sync::{
        LazyLock,
        atomic::{AtomicUsize, Ordering},
    },
};

use const_format::formatcp;
use dashmap::DashMap;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

use crate::{
    database::models::{DBProjectId, DBVersionId},
    models::{ids::VersionId, v3::analytics::DownloadReason},
    routes::ApiError,
};

use super::super::{
    ClickhouseFilterParam, QueryClickhouseContext, add_to_time_slice,
    condense_country, none_if_empty, none_if_zero_version_id,
    normalize_loader_for_project,
};
use super::{AnalyticsData, Metrics, ProjectAnalytics, ProjectMetrics};

const TIME_RANGE_START: &str = "{time_range_start: UInt64}";
const TIME_RANGE_END: &str = "{time_range_end: UInt64}";
const TIME_SLICES: &str = "{time_slices: UInt64}";
const PROJECT_IDS: &str = "project_ids";

/// Fields for [`super::ReturnMetrics::project_downloads`].
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
    /// Normalized user agent used to download this project.
    UserAgent,
    /// Whether these downloads were monetized or not.
    Monetized,
    /// What country these downloads came from.
    ///
    /// To anonymize the data, the country may be reported as `XX`.
    Country,
    /// Download reason.
    Reason,
    /// Game version used for this download.
    GameVersion,
    /// Mod loader used for this download.
    Loader,
}

/// Filters for [`super::ReturnMetrics::project_downloads`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectDownloadsFilters {
    /// Version IDs to include.
    #[serde(default)]
    pub version_id: Vec<VersionId>,
    /// Referrer domains to include.
    #[serde(default)]
    pub domain: Vec<String>,
    /// Normalized download sources to include.
    #[serde(default)]
    pub user_agent: Vec<DownloadSource>,
    /// Monetization states to include.
    #[serde(default)]
    pub monetized: Vec<bool>,
    /// Country codes to include.
    #[serde(default)]
    pub country: Vec<String>,
    /// Download reasons to include.
    #[serde(default)]
    pub reason: Vec<DownloadReason>,
    /// Game versions to include.
    #[serde(default)]
    pub game_version: Vec<String>,
    /// Loaders to include.
    #[serde(default)]
    pub loader: Vec<String>,
}

/// [`super::ReturnMetrics::project_downloads`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectDownloads {
    /// [`ProjectDownloadsField::Domain`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) domain: Option<String>,
    /// [`ProjectDownloadsField::UserAgent`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) user_agent: Option<DownloadSource>,
    /// [`ProjectDownloadsField::VersionId`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) version_id: Option<VersionId>,
    /// [`ProjectDownloadsField::Monetized`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) monetized: Option<bool>,
    /// [`ProjectDownloadsField::Country`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) country: Option<String>,
    /// [`ProjectDownloadsField::Reason`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reason: Option<DownloadReason>,
    /// [`ProjectDownloadsField::GameVersion`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) game_version: Option<String>,
    /// [`ProjectDownloadsField::Loader`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) loader: Option<String>,
    /// Total number of downloads for this bucket.
    pub(crate) downloads: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, utoipa::ToSchema)]
pub enum DownloadSource {
    Website,
    ModrinthApp,
    ModrinthHosting,
    ModrinthMaven,
    Other,
    Named(String),
}

impl Serialize for DownloadSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Named(name) => serializer.serialize_str(name),
            Self::Website => serializer.serialize_str("website"),
            Self::ModrinthApp => serializer.serialize_str("modrinth_app"),
            Self::ModrinthHosting => {
                serializer.serialize_str("modrinth_hosting")
            }
            Self::ModrinthMaven => serializer.serialize_str("modrinth_maven"),
            Self::Other => serializer.serialize_str("other"),
        }
    }
}

impl<'de> Deserialize<'de> for DownloadSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let source = String::deserialize(deserializer)?;
        Ok(match source.as_str() {
            "website" => Self::Website,
            "modrinth_app" => Self::ModrinthApp,
            "modrinth_hosting" => Self::ModrinthHosting,
            "modrinth_maven" => Self::ModrinthMaven,
            "other" => Self::Other,
            _ if !source.is_empty() => Self::Named(source),
            _ => {
                return Err(D::Error::custom(
                    "download source cannot be empty",
                ));
            }
        })
    }
}

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct DownloadRow {
    bucket: u64,
    source_project_id: DBProjectId,
    project_id: DBProjectId,
    domain: String,
    user_agent: String,
    version_id: DBVersionId,
    monetized: i8,
    country: String,
    reason: String,
    game_version: String,
    loader: String,
    downloads: u64,
}

const DOWNLOADS: &str = {
    const USE_PROJECT_ID: &str = "{use_project_id: Bool}";
    const USE_DOMAIN: &str = "{use_domain: Bool}";
    const USE_USER_AGENT: &str = "{use_user_agent: Bool}";
    const USE_VERSION_ID: &str = "{use_version_id: Bool}";
    const USE_MONETIZED: &str = "{use_monetized: Bool}";
    const USE_COUNTRY: &str = "{use_country: Bool}";
    const USE_REASON: &str = "{use_reason: Bool}";
    const USE_GAME_VERSION: &str = "{use_game_version: Bool}";
    const USE_LOADER: &str = "{use_loader: Bool}";
    const FILTER_DOMAIN: &str = "filter_domain";
    const FILTER_VERSION_ID: &str = "filter_version_id";
    const FILTER_MONETIZED: &str = "{filter_monetized: UInt8}";
    const FILTER_COUNTRY: &str = "filter_country";
    const FILTER_REASON: &str = "filter_reason";
    const FILTER_GAME_VERSION: &str = "filter_game_version";
    const FILTER_LOADER: &str = "filter_loader";

    formatcp!(
        "WITH
            ? AS {PROJECT_IDS},
            ? AS {FILTER_DOMAIN},
            ? AS {FILTER_VERSION_ID},
            ? AS {FILTER_COUNTRY},
            ? AS {FILTER_REASON},
            ? AS {FILTER_GAME_VERSION},
            ? AS {FILTER_LOADER}
        SELECT
            widthBucket(toUnixTimestamp(recorded), {TIME_RANGE_START}, {TIME_RANGE_END}, {TIME_SLICES}) AS bucket,
            downloads.project_id AS source_project_id,
            if({USE_PROJECT_ID}, downloads.project_id, 0) AS project_id,
            if({USE_DOMAIN}, domain, '') AS domain,
            if({USE_USER_AGENT}, user_agent, '') AS user_agent,
            if({USE_VERSION_ID}, version_id, 0) AS version_id,
            if({USE_MONETIZED}, CAST(user_id != 0 AS Int8), -1) AS monetized,
            if({USE_COUNTRY}, country, '') AS country,
            if({USE_REASON}, reason, '') AS reason,
            if({USE_GAME_VERSION}, game_version, '') AS game_version,
            if({USE_LOADER}, loader, '') AS loader,
            COUNT(*) AS downloads
        FROM downloads
        WHERE
            recorded >= {TIME_RANGE_START}
            AND recorded < {TIME_RANGE_END}
            -- make sure that the REAL project id is included,
            -- not the possibly-zero one,
            -- by using `downloads.project_id` instead of `project_id`
            AND downloads.project_id IN {PROJECT_IDS}
            AND (empty({FILTER_DOMAIN}) OR downloads.domain IN {FILTER_DOMAIN})
            AND (empty({FILTER_VERSION_ID}) OR downloads.version_id IN {FILTER_VERSION_ID})
            AND ({FILTER_MONETIZED} = 2 OR CAST(downloads.user_id != 0 AS UInt8) = {FILTER_MONETIZED})
            AND (empty({FILTER_COUNTRY}) OR downloads.country IN {FILTER_COUNTRY})
            AND (empty({FILTER_REASON}) OR downloads.reason IN {FILTER_REASON})
            AND (empty({FILTER_GAME_VERSION}) OR downloads.game_version IN {FILTER_GAME_VERSION})
            AND (empty({FILTER_LOADER}) OR downloads.loader IN {FILTER_LOADER})
        GROUP BY bucket, source_project_id, project_id, domain, user_agent, version_id, monetized, country, reason, game_version, loader"
    )
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DownloadBucket {
    bucket: u64,
    project_id: DBProjectId,
    domain: Option<String>,
    user_agent: Option<DownloadSource>,
    version_id: Option<DBVersionId>,
    monetized: Option<bool>,
    country: Option<String>,
    reason: Option<DownloadReason>,
    game_version: Option<String>,
    loader: Option<String>,
}

pub(crate) async fn fetch(
    cx: &mut QueryClickhouseContext<'_>,
    metrics: &Metrics<ProjectDownloadsField, ProjectDownloadsFilters>,
) -> Result<(), ApiError> {
    use ProjectDownloadsField as F;
    let uses = |field| metrics.bucket_by.contains(&field);
    let use_columns = &[
        ("use_project_id", uses(F::ProjectId)),
        ("use_domain", uses(F::Domain)),
        (
            "use_user_agent",
            uses(F::UserAgent) || !metrics.filter_by.user_agent.is_empty(),
        ),
        ("use_version_id", uses(F::VersionId)),
        ("use_monetized", uses(F::Monetized)),
        ("use_country", uses(F::Country)),
        ("use_reason", uses(F::Reason)),
        ("use_game_version", uses(F::GameVersion)),
        ("use_loader", uses(F::Loader)),
    ];

    let mut query = cx
        .clickhouse
        .query(DOWNLOADS)
        .param("time_range_start", cx.req.time_range.start.timestamp())
        .param("time_range_end", cx.req.time_range.end.timestamp())
        .param("time_slices", cx.time_slices.len())
        .bind(cx.project_ids);
    for (param_name, used) in use_columns {
        query = query.param(param_name, used)
    }
    for filter_param in [
        ClickhouseFilterParam::String(&metrics.filter_by.domain),
        ClickhouseFilterParam::VersionId(&metrics.filter_by.version_id),
        ClickhouseFilterParam::Bool(
            "filter_monetized",
            &metrics.filter_by.monetized,
        ),
        ClickhouseFilterParam::String(&metrics.filter_by.country),
        ClickhouseFilterParam::DownloadReason(&metrics.filter_by.reason),
        ClickhouseFilterParam::String(&metrics.filter_by.game_version),
        ClickhouseFilterParam::String(&metrics.filter_by.loader),
    ] {
        query = filter_param.bind(query);
    }

    let uses_column = |name| {
        use_columns
            .iter()
            .any(|(column_name, used)| *column_name == name && *used)
    };
    let mut cursor = query.fetch::<DownloadRow>()?;
    let mut buckets = HashMap::<DownloadBucket, u64>::new();

    while let Some(row) = cursor.next().await? {
        let normalized_source = normalize_download_source(&row.user_agent);
        if !metrics.filter_by.user_agent.is_empty()
            && !normalized_source.as_ref().is_some_and(|source| {
                metrics.filter_by.user_agent.contains(source)
            })
        {
            continue;
        }

        let key = DownloadBucket {
            bucket: row.bucket,
            project_id: row.project_id,
            domain: uses_column("use_domain").then(|| row.domain.clone()),
            user_agent: uses(F::UserAgent)
                .then_some(normalized_source)
                .flatten(),
            version_id: uses_column("use_version_id").then_some(row.version_id),
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
            reason: if uses_column("use_reason") {
                none_if_empty(row.reason.clone()).and_then(|s| s.parse().ok())
            } else {
                None
            },
            game_version: uses_column("use_game_version")
                .then(|| row.game_version.clone()),
            loader: uses_column("use_loader").then(|| {
                normalize_loader_for_project(
                    row.loader.clone(),
                    row.source_project_id,
                    cx.project_loaders,
                )
            }),
        };

        *buckets.entry(key).or_default() += row.downloads;
    }

    for (key, downloads) in buckets {
        add_to_time_slice(
            cx.time_slices,
            key.bucket as usize,
            AnalyticsData::Project(ProjectAnalytics {
                source_project: key.project_id.into(),
                metrics: ProjectMetrics::Downloads(ProjectDownloads {
                    domain: key.domain.and_then(none_if_empty),
                    user_agent: key.user_agent,
                    version_id: key
                        .version_id
                        .and_then(none_if_zero_version_id),
                    monetized: key.monetized,
                    country: key
                        .country
                        .map(|country| condense_country(country, downloads)),
                    reason: key.reason,
                    game_version: key.game_version.and_then(none_if_empty),
                    loader: key.loader.and_then(none_if_empty),
                    downloads,
                }),
            }),
        )?;
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum DownloadSourcePattern {
    Named(&'static str),
    Website,
    ModrinthApp,
    ModrinthHosting,
    ModrinthMaven,
}

impl DownloadSourcePattern {
    fn into_source(self) -> DownloadSource {
        match self {
            Self::Named(name) => DownloadSource::Named(name.into()),
            Self::Website => DownloadSource::Website,
            Self::ModrinthApp => DownloadSource::ModrinthApp,
            Self::ModrinthHosting => DownloadSource::ModrinthHosting,
            Self::ModrinthMaven => DownloadSource::ModrinthMaven,
        }
    }
}

pub(crate) fn all_download_sources() -> Vec<DownloadSource> {
    let mut sources = DOWNLOAD_SOURCE_PATTERNS
        .iter()
        .map(|(_, source)| source.into_source())
        .collect::<Vec<_>>();
    sources.push(DownloadSource::Other);
    sources.sort_by(|a, b| {
        download_source_sort_key(a).cmp(download_source_sort_key(b))
    });
    sources.dedup();
    sources
}

fn download_source_sort_key(source: &DownloadSource) -> &str {
    match source {
        DownloadSource::Named(name) => name,
        DownloadSource::Website => "website",
        DownloadSource::ModrinthApp => "modrinth_app",
        DownloadSource::ModrinthHosting => "modrinth_hosting",
        DownloadSource::ModrinthMaven => "modrinth_maven",
        DownloadSource::Other => "other",
    }
}

static DOWNLOAD_SOURCE_PATTERNS: LazyLock<Vec<(Regex, DownloadSourcePattern)>> =
    LazyLock::new(|| {
        use DownloadSourcePattern as P;

        [
            (r"^modrinth/kyros/", P::ModrinthHosting),
            (r"^modrinth/theseus/", P::ModrinthApp),
            (r"^(Gradle/|Apache-Maven/)", P::ModrinthMaven),
            (r"^MultiMC/", P::Named("MultiMC")),
            (r"^PrismLauncher/", P::Named("Prism Launcher")),
            (r"^PolyMC/", P::Named("PolyMC")),
            (r"^FCL/", P::Named("FCL")),
            (r"^PCL2/", P::Named("PCL2")),
            (r"^HMCL/", P::Named("HMCL")),
            (r"^Lunar Client Launcher", P::Named("Lunar Client")),
            (r"^PojavLauncher", P::Named("PojavLauncher")),
            (r"^ATLauncher/", P::Named("ATLauncher")),
            (r"FeatherLauncher/", P::Named("Feather Client")),
            (
                r"^FeatherMC/Feather Client Rust Launcher/",
                P::Named("Feather Client"),
            ),
            (r"Feather/[0-9A-Za-z]+", P::Named("Feather Client")),
            (r"^PandoraLauncher/", P::Named("Pandora Launcher")),
            (r"^unsup", P::Named("unsup")),
            (r"nothub/mrpack-install", P::Named("mrpack-install")),
            (r"^(packwiz-installer|packwiz/)", P::Named("Packwiz")),
            (
                r"^(Mozilla/|Chrome/|Chromium/|Firefox/|Safari/|AppleWebKit/|Edg/|OPR/)",
                P::Website,
            ),
        ]
        .into_iter()
        .map(|(pattern, source)| {
            (
                Regex::new(pattern)
                    .expect("download source regex should be valid"),
                source,
            )
        })
        .collect()
    });

const MAX_DOWNLOAD_SOURCE_CACHE_BYTES: usize = 100 * 1024 * 1024;

static DOWNLOAD_SOURCE_CACHE: LazyLock<
    DashMap<String, Option<DownloadSource>>,
> = LazyLock::new(DashMap::new);

static DOWNLOAD_SOURCE_CACHE_BYTES: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn normalize_download_source(
    user_agent: &str,
) -> Option<DownloadSource> {
    if let Some(source) = DOWNLOAD_SOURCE_CACHE.get(user_agent) {
        return source.clone();
    }

    let source = normalize_download_source_uncached(user_agent);

    let key_bytes = user_agent.len();
    let previous_bytes =
        DOWNLOAD_SOURCE_CACHE_BYTES.fetch_add(key_bytes, Ordering::Relaxed);
    if previous_bytes + key_bytes <= MAX_DOWNLOAD_SOURCE_CACHE_BYTES {
        DOWNLOAD_SOURCE_CACHE.insert(user_agent.to_owned(), source.clone());
    } else {
        DOWNLOAD_SOURCE_CACHE_BYTES.fetch_sub(key_bytes, Ordering::Relaxed);
    }

    source
}

fn normalize_download_source_uncached(
    user_agent: &str,
) -> Option<DownloadSource> {
    DOWNLOAD_SOURCE_PATTERNS.iter().find_map(|(regex, source)| {
        regex.is_match(user_agent).then(|| source.into_source())
    })
}

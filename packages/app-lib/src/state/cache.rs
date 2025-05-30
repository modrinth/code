use crate::config::{META_URL, MODRINTH_API_URL, MODRINTH_API_URL_V3};
use crate::state::ProjectType;
use crate::util::fetch::{FetchSemaphore, fetch_json, sha1_async};
use chrono::{DateTime, Utc};
use dashmap::DashSet;
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::path::{Path, PathBuf};

// 1 day
const DEFAULT_ID: &str = "0";

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CacheValueType {
    Project,
    Version,
    User,
    Team,
    Organization,
    File,
    LoaderManifest,
    MinecraftManifest,
    Categories,
    ReportTypes,
    Loaders,
    GameVersions,
    DonationPlatforms,
    FileHash,
    FileUpdate,
    SearchResults,
}

impl CacheValueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CacheValueType::Project => "project",
            CacheValueType::Version => "version",
            CacheValueType::User => "user",
            CacheValueType::Team => "team",
            CacheValueType::Organization => "organization",
            CacheValueType::File => "file",
            CacheValueType::LoaderManifest => "loader_manifest",
            CacheValueType::MinecraftManifest => "minecraft_manifest",
            CacheValueType::Categories => "categories",
            CacheValueType::ReportTypes => "report_types",
            CacheValueType::Loaders => "loaders",
            CacheValueType::GameVersions => "game_versions",
            CacheValueType::DonationPlatforms => "donation_platforms",
            CacheValueType::FileHash => "file_hash",
            CacheValueType::FileUpdate => "file_update",
            CacheValueType::SearchResults => "search_results",
        }
    }

    pub fn from_string(val: &str) -> CacheValueType {
        match val {
            "project" => CacheValueType::Project,
            "version" => CacheValueType::Version,
            "user" => CacheValueType::User,
            "team" => CacheValueType::Team,
            "organization" => CacheValueType::Organization,
            "file" => CacheValueType::File,
            "loader_manifest" => CacheValueType::LoaderManifest,
            "minecraft_manifest" => CacheValueType::MinecraftManifest,
            "categories" => CacheValueType::Categories,
            "report_types" => CacheValueType::ReportTypes,
            "loaders" => CacheValueType::Loaders,
            "game_versions" => CacheValueType::GameVersions,
            "donation_platforms" => CacheValueType::DonationPlatforms,
            "file_hash" => CacheValueType::FileHash,
            "file_update" => CacheValueType::FileUpdate,
            "search_results" => CacheValueType::SearchResults,
            _ => CacheValueType::Project,
        }
    }

    pub fn expiry(&self) -> i64 {
        match self {
            CacheValueType::File => 60 * 60 * 24 * 30, // 30 days
            CacheValueType::FileHash => 60 * 60 * 24 * 30, // 30 days
            _ => 60 * 60 * 30,                         // 30 minutes
        }
    }

    pub fn get_empty_entry(self, key: String) -> CachedEntry {
        CachedEntry {
            id: key,
            alias: None,
            expires: Utc::now().timestamp() + self.expiry(),
            type_: self,
            data: None,
        }
    }

    pub fn case_sensitive_alias(&self) -> Option<bool> {
        match self {
            CacheValueType::Project
            | CacheValueType::User
            | CacheValueType::Organization => Some(false),

            CacheValueType::FileHash => Some(true),

            CacheValueType::MinecraftManifest
            | CacheValueType::Categories
            | CacheValueType::ReportTypes
            | CacheValueType::Loaders
            | CacheValueType::GameVersions
            | CacheValueType::DonationPlatforms
            | CacheValueType::Version
            | CacheValueType::Team
            | CacheValueType::File
            | CacheValueType::LoaderManifest
            | CacheValueType::FileUpdate
            | CacheValueType::SearchResults => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum CacheValue {
    Project(Project),

    Version(Version),

    User(User),

    Team(Vec<TeamMember>),

    Organization(Organization),

    File(CachedFile),

    LoaderManifest(CachedLoaderManifest),
    MinecraftManifest(daedalus::minecraft::VersionManifest),

    Categories(Vec<Category>),
    ReportTypes(Vec<String>),
    Loaders(Vec<Loader>),
    GameVersions(Vec<GameVersion>),
    DonationPlatforms(Vec<DonationPlatform>),

    FileHash(CachedFileHash),
    FileUpdate(CachedFileUpdate),
    SearchResults(SearchResults),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResults {
    pub search: String,
    pub result: SearchResult,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    pub hits: Vec<SearchEntry>,
    pub offset: u32,
    pub limit: u32,
    pub total_hits: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchEntry {
    pub project_id: String,
    pub project_type: String,
    pub slug: Option<String>,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub display_categories: Vec<String>,
    pub versions: Vec<String>,
    pub downloads: i32,
    pub follows: i32,
    pub icon_url: String,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub latest_version: String,
    pub license: String,
    pub client_side: String,
    pub server_side: String,
    pub gallery: Vec<String>,
    pub featured_gallery: Option<String>,
    pub color: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedFileUpdate {
    pub hash: String,
    pub game_version: String,
    pub loaders: Vec<String>,
    pub update_version_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedFileHash {
    pub path: String,
    pub size: u64,
    pub hash: String,
    pub project_type: Option<ProjectType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedLoaderManifest {
    pub loader: String,
    pub manifest: daedalus::modded::Manifest,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedFile {
    pub hash: String,
    pub project_id: String,
    pub version_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub id: String,
    pub slug: Option<String>,
    pub project_type: String,
    pub team: String,
    pub organization: Option<String>,
    pub title: String,
    pub description: String,
    pub body: String,

    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub approved: Option<DateTime<Utc>>,

    pub status: String,

    pub license: License,

    pub client_side: SideType,
    pub server_side: SideType,

    pub downloads: u32,
    pub followers: u32,

    pub categories: Vec<String>,
    pub additional_categories: Vec<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,

    pub versions: Vec<String>,

    pub icon_url: Option<String>,

    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub donation_urls: Option<Vec<DonationLink>>,
    pub gallery: Vec<GalleryItem>,
    pub color: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct License {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GalleryItem {
    pub url: String,
    pub raw_url: String,
    pub featured: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub ordering: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DonationLink {
    pub id: String,
    pub platform: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SideType {
    Required,
    Optional,
    Unsupported,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Version {
    pub id: String,
    pub project_id: String,
    pub author_id: String,

    pub featured: bool,

    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub changelog_url: Option<String>,

    pub date_published: DateTime<Utc>,
    pub downloads: u32,
    pub version_type: String,

    pub files: Vec<VersionFile>,
    pub dependencies: Vec<Dependency>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VersionFile {
    pub hashes: HashMap<String, String>,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u32,
    pub file_type: Option<FileType>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum FileType {
    RequiredResourcePack,
    OptionalResourcePack,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: DependencyType,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TeamMember {
    pub team_id: String,
    pub user: User,
    pub is_owner: bool,
    pub role: String,
    pub ordering: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created: DateTime<Utc>,
    pub role: String,
    #[serde(default)]
    pub badges: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Organization {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub team_id: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub color: Option<u32>,
    pub members: Vec<TeamMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub project_type: String,
    pub header: String,
    pub icon: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loader {
    pub name: String,
    pub icon: PathBuf,
    pub supported_project_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonationPlatform {
    pub short: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVersion {
    pub version: String,
    pub version_type: String,
    pub date: String,
    pub major: bool,
}

impl CacheValue {
    pub fn get_entry(self) -> CachedEntry {
        CachedEntry {
            id: self.get_key(),
            alias: self.get_alias(),
            type_: self.get_type(),
            expires: Utc::now().timestamp() + self.get_type().expiry(),
            data: Some(self),
        }
    }

    pub fn get_type(&self) -> CacheValueType {
        match self {
            CacheValue::Project(_) => CacheValueType::Project,
            CacheValue::Version(_) => CacheValueType::Version,
            CacheValue::User(_) => CacheValueType::User,
            CacheValue::Team { .. } => CacheValueType::Team,
            CacheValue::Organization(_) => CacheValueType::Organization,
            CacheValue::File { .. } => CacheValueType::File,
            CacheValue::LoaderManifest { .. } => CacheValueType::LoaderManifest,
            CacheValue::MinecraftManifest(_) => {
                CacheValueType::MinecraftManifest
            }
            CacheValue::Categories(_) => CacheValueType::Categories,
            CacheValue::ReportTypes(_) => CacheValueType::ReportTypes,
            CacheValue::Loaders(_) => CacheValueType::Loaders,
            CacheValue::GameVersions(_) => CacheValueType::GameVersions,
            CacheValue::DonationPlatforms(_) => {
                CacheValueType::DonationPlatforms
            }
            CacheValue::FileHash(_) => CacheValueType::FileHash,
            CacheValue::FileUpdate(_) => CacheValueType::FileUpdate,
            CacheValue::SearchResults(_) => CacheValueType::SearchResults,
        }
    }

    fn get_key(&self) -> String {
        match self {
            CacheValue::Project(project) => project.id.clone(),
            CacheValue::Version(version) => version.id.clone(),
            CacheValue::User(user) => user.id.clone(),
            CacheValue::Team(members) => members
                .iter()
                .next()
                .map(|x| x.team_id.as_str())
                .unwrap_or(DEFAULT_ID)
                .to_string(),
            CacheValue::Organization(org) => org.id.clone(),
            CacheValue::File(file) => file.hash.clone(),
            CacheValue::LoaderManifest(loader) => loader.loader.clone(),
            // These values can only have one key/val pair, so we specify the same key
            CacheValue::MinecraftManifest(_)
            | CacheValue::Categories(_)
            | CacheValue::ReportTypes(_)
            | CacheValue::Loaders(_)
            | CacheValue::GameVersions(_)
            | CacheValue::DonationPlatforms(_) => DEFAULT_ID.to_string(),

            CacheValue::FileHash(hash) => {
                format!(
                    "{}-{}",
                    hash.size,
                    hash.path.trim_end_matches(".disabled")
                )
            }
            CacheValue::FileUpdate(hash) => {
                format!(
                    "{}-{}-{}",
                    hash.hash,
                    hash.loaders.join("+"),
                    hash.game_version
                )
            }
            CacheValue::SearchResults(search) => search.search.clone(),
        }
    }

    fn get_alias(&self) -> Option<String> {
        match self {
            CacheValue::Project(project) => project.slug.clone(),
            CacheValue::User(user) => Some(user.username.clone()),
            CacheValue::Organization(org) => Some(org.slug.clone()),

            CacheValue::FileHash(_) => {
                Some(format!("{}.disabled", self.get_key()))
            }

            CacheValue::MinecraftManifest(_)
            | CacheValue::Categories(_)
            | CacheValue::ReportTypes(_)
            | CacheValue::Loaders(_)
            | CacheValue::GameVersions(_)
            | CacheValue::DonationPlatforms(_)
            | CacheValue::Version(_)
            | CacheValue::Team { .. }
            | CacheValue::File { .. }
            | CacheValue::LoaderManifest { .. }
            | CacheValue::FileUpdate(_)
            | CacheValue::SearchResults(_) => None,
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CacheBehaviour {
    /// Serve expired data. If fetch fails / launcher is offline, errors are ignored
    /// and expired data is served
    StaleWhileRevalidateSkipOffline,
    // Serve expired data, revalidate in background
    StaleWhileRevalidate,
    // Must revalidate if data is expired
    MustRevalidate,
    // Ignore cache- always fetch updated data from origin
    Bypass,
}

impl Default for CacheBehaviour {
    fn default() -> Self {
        Self::StaleWhileRevalidateSkipOffline
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEntry {
    id: String,
    alias: Option<String>,
    #[serde(rename = "data_type")]
    pub type_: CacheValueType,
    data: Option<CacheValue>,
    pub expires: i64,
}

macro_rules! impl_cache_methods {
    ($(($variant:ident, $type:ty)),*) => {
        impl CachedEntry {
            $(
                paste::paste! {
                    #[tracing::instrument(skip(pool, fetch_semaphore))]
                    #[allow(dead_code)]
                    pub async fn [<get_ $variant:snake>](
                        id: &str,
                        cache_behaviour: Option<CacheBehaviour>,
                        pool: &SqlitePool,
                        fetch_semaphore: &FetchSemaphore,
                    ) -> crate::Result<Option<$type>>
                    {
                        Ok(Self::[<get_ $variant:snake _many>](&[id], cache_behaviour, pool, fetch_semaphore).await?.into_iter().next())
                    }

                    #[tracing::instrument(skip(pool, fetch_semaphore))]
                    #[allow(dead_code)]
                    pub async fn [<get_ $variant:snake _many>](
                        ids: &[&str],
                        cache_behaviour: Option<CacheBehaviour>,
                        pool: &SqlitePool,
                        fetch_semaphore: &FetchSemaphore,
                    ) -> crate::Result<Vec<$type>>
                    {
                        let entry =
                            CachedEntry::get_many(CacheValueType::$variant, ids, cache_behaviour, pool, fetch_semaphore).await?;

                        Ok(entry.into_iter().filter_map(|x| if let Some(CacheValue::$variant(value)) = x.data {
                            Some(value)
                        } else {
                            None
                        }).collect())
                    }
                }
            )*
        }
    }
}

macro_rules! impl_cache_method_singular {
    ($(($variant:ident, $type:ty)),*) => {
        impl CachedEntry {
            $(
                paste::paste! {
                    #[tracing::instrument(skip(pool, fetch_semaphore))]
                    #[allow(dead_code)]
                    pub async fn [<get_ $variant:snake>] (
                        cache_behaviour: Option<CacheBehaviour>,
                        pool: &SqlitePool,
                        fetch_semaphore: &FetchSemaphore,
                    ) -> crate::Result<Option<$type>>
                    {
                        let entry =
                            CachedEntry::get(CacheValueType::$variant, DEFAULT_ID, cache_behaviour, pool, fetch_semaphore).await?;

                        if let Some(CacheValue::$variant(value)) = entry.map(|x| x.data).flatten() {
                            Ok(Some(value))
                        } else {
                            Ok(None)
                        }
                    }
                }
            )*
        }
    }
}

impl_cache_methods!(
    (Project, Project),
    (Version, Version),
    (User, User),
    (Team, Vec<TeamMember>),
    (Organization, Organization),
    (File, CachedFile),
    (LoaderManifest, CachedLoaderManifest),
    (FileHash, CachedFileHash),
    (FileUpdate, CachedFileUpdate),
    (SearchResults, SearchResults)
);

impl_cache_method_singular!(
    (MinecraftManifest, daedalus::minecraft::VersionManifest),
    (Categories, Vec<Category>),
    (ReportTypes, Vec<String>),
    (Loaders, Vec<Loader>),
    (GameVersions, Vec<GameVersion>),
    (DonationPlatforms, Vec<DonationPlatform>)
);

impl CachedEntry {
    #[tracing::instrument(skip(pool, fetch_semaphore))]
    pub async fn get(
        type_: CacheValueType,
        key: &str,
        cache_behaviour: Option<CacheBehaviour>,
        pool: &SqlitePool,
        fetch_semaphore: &FetchSemaphore,
    ) -> crate::Result<Option<Self>> {
        Ok(Self::get_many(
            type_,
            &[key],
            cache_behaviour,
            pool,
            fetch_semaphore,
        )
        .await?
        .into_iter()
        .next())
    }

    #[tracing::instrument(skip(pool, fetch_semaphore))]
    pub async fn get_many(
        type_: CacheValueType,
        keys: &[&str],
        cache_behaviour: Option<CacheBehaviour>,
        pool: &SqlitePool,
        fetch_semaphore: &FetchSemaphore,
    ) -> crate::Result<Vec<Self>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let cache_behaviour = cache_behaviour.unwrap_or_default();

        let remaining_keys = DashSet::new();
        for key in keys {
            remaining_keys.insert(*key);
        }

        let mut return_vals = Vec::new();
        let expired_keys = DashSet::new();

        if cache_behaviour != CacheBehaviour::Bypass {
            let type_str = type_.as_str();
            let serialized_keys = serde_json::to_string(&keys)?;
            let alias_keys = if type_.case_sensitive_alias().unwrap_or(true) {
                serialized_keys.clone()
            } else {
                serde_json::to_string(
                    &keys.iter().map(|x| x.to_lowercase()).collect::<Vec<_>>(),
                )?
            };

            // unsupported type NULL of column #3 ("data"), so cannot be compile time type checked
            // https://github.com/launchbadge/sqlx/issues/1979
            let query = sqlx::query!(
                r#"
                SELECT id, data_type, json(data) as "data?: serde_json::Value", alias, expires
                FROM cache
                WHERE data_type = $1 AND (
                    id IN (SELECT value FROM json_each($2))
                    OR
                    alias IN (SELECT value FROM json_each($3))
                )
                "#,
                type_str,
                serialized_keys,
                alias_keys
            )
            .fetch_all(pool)
            .await?;

            for row in query {
                let row_exists = row.data.is_some();
                let parsed_data = row
                    .data
                    .and_then(|x| serde_json::from_value::<CacheValue>(x).ok());

                // If data is corrupted/failed to parse ignore it
                if row_exists && parsed_data.is_none() {
                    continue;
                }

                if row.expires <= Utc::now().timestamp() {
                    if cache_behaviour == CacheBehaviour::MustRevalidate {
                        continue;
                    } else {
                        expired_keys.insert(row.id.clone());
                    }
                }

                remaining_keys.retain(|x| {
                    x != &&*row.id
                        && !row
                            .alias
                            .as_ref()
                            .map(|y| {
                                if type_.case_sensitive_alias().unwrap_or(true)
                                {
                                    x == y
                                } else {
                                    y.to_lowercase() == x.to_lowercase()
                                }
                            })
                            .unwrap_or(false)
                });

                if let Some(data) = parsed_data {
                    return_vals.push(Self {
                        id: row.id,
                        alias: row.alias,
                        type_: CacheValueType::from_string(&row.data_type),
                        data: Some(data),
                        expires: row.expires,
                    });
                }
            }
        }

        if !remaining_keys.is_empty() {
            let res = Self::fetch_many(
                type_,
                remaining_keys.clone(),
                fetch_semaphore,
                pool,
            )
            .await;

            if res.is_err()
                && cache_behaviour
                    == CacheBehaviour::StaleWhileRevalidateSkipOffline
            {
                for key in remaining_keys {
                    expired_keys.insert(key.to_string());
                }
            } else {
                let values = res?;

                Self::upsert_many(
                    &values.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
                    pool,
                )
                .await?;

                if !values.is_empty() {
                    return_vals.append(
                        &mut values
                            .into_iter()
                            .filter(|(_, include)| *include)
                            .map(|x| x.0)
                            .collect::<Vec<_>>(),
                    );
                }
            }
        }

        if !expired_keys.is_empty()
            && (cache_behaviour == CacheBehaviour::StaleWhileRevalidate
                || cache_behaviour
                    == CacheBehaviour::StaleWhileRevalidateSkipOffline)
        {
            tokio::task::spawn(async move {
                // TODO: if possible- find a way to do this without invoking state get
                let state = crate::state::State::get().await?;

                let values = Self::fetch_many(
                    type_,
                    expired_keys,
                    &state.api_semaphore,
                    &state.pool,
                )
                .await?
                .into_iter()
                .map(|x| x.0)
                .collect::<Vec<_>>();

                if !values.is_empty() {
                    Self::upsert_many(&values, &state.pool).await?;
                }

                Ok::<(), crate::Error>(())
            });
        }

        Ok(return_vals)
    }

    async fn fetch_many(
        type_: CacheValueType,
        keys: DashSet<impl Display + Eq + Hash + Serialize>,
        fetch_semaphore: &FetchSemaphore,
        pool: &SqlitePool,
    ) -> crate::Result<Vec<(Self, bool)>> {
        async fn fetch_many_batched<T: DeserializeOwned>(
            method: Method,
            api_url: &str,
            url: &str,
            keys: &DashSet<impl Display + Eq + Hash + Serialize>,
            fetch_semaphore: &FetchSemaphore,
            pool: &SqlitePool,
        ) -> crate::Result<Vec<T>> {
            const MAX_REQUEST_SIZE: usize = 800;

            let urls = keys
                .iter()
                .collect::<Vec<_>>()
                .chunks(MAX_REQUEST_SIZE)
                .map(|chunk| {
                    serde_json::to_string(&chunk)
                        .map(|keys| format!("{api_url}{url}{keys}"))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let res = futures::future::try_join_all(urls.iter().map(|url| {
                fetch_json::<Vec<_>>(
                    method.clone(),
                    url,
                    None,
                    None,
                    fetch_semaphore,
                    pool,
                )
            }))
            .await?;

            Ok(res.into_iter().flatten().collect())
        }

        macro_rules! fetch_original_values {
            ($type:ident, $api_url:expr, $url_suffix:expr, $cache_variant:path) => {{
                let mut results = fetch_many_batched(
                    Method::GET,
                    $api_url,
                    &format!("{}?ids=", $url_suffix),
                    &keys,
                    &fetch_semaphore,
                    &pool,
                )
                .await?
                .into_iter()
                .map($cache_variant)
                .collect::<Vec<_>>();

                let mut values = vec![];
                let visited_keys = DashSet::new();

                for key in keys {
                    let key = key.to_string();
                    let lower_case_key = key.to_lowercase();
                    let case_sensitive = CacheValueType::$type
                        .case_sensitive_alias()
                        .unwrap_or(true);

                    if let Some(position) = results.iter().position(|x| {
                        x.get_key() == key
                            || x.get_alias()
                                .map(|x| {
                                    if case_sensitive {
                                        x == key
                                    } else {
                                        x == lower_case_key
                                    }
                                })
                                .unwrap_or(false)
                    }) {
                        visited_keys.insert(key);
                        if !case_sensitive {
                            visited_keys.insert(lower_case_key);
                        }

                        let result = results.remove(position);

                        values.push((result.get_entry(), true));
                    } else if !visited_keys.contains(&key)
                        && (case_sensitive
                            || !visited_keys.contains(&lower_case_key))
                    {
                        values.push((
                            CacheValueType::$type.get_empty_entry(key),
                            true,
                        ));
                    }
                }

                values
            }};
        }

        macro_rules! fetch_original_value {
            ($type:ident, $api_url:expr, $url_suffix:expr, $cache_variant:path) => {{
                vec![(
                    $cache_variant(
                        fetch_json(
                            Method::GET,
                            &*format!("{}{}", $api_url, $url_suffix),
                            None,
                            None,
                            &fetch_semaphore,
                            pool,
                        )
                        .await?,
                    )
                    .get_entry(),
                    true,
                )]
            }};
        }

        Ok(match type_ {
            CacheValueType::Project => {
                fetch_original_values!(
                    Project,
                    MODRINTH_API_URL,
                    "projects",
                    CacheValue::Project
                )
            }
            CacheValueType::Version => {
                fetch_original_values!(
                    Version,
                    MODRINTH_API_URL,
                    "versions",
                    CacheValue::Version
                )
            }
            CacheValueType::User => {
                fetch_original_values!(
                    User,
                    MODRINTH_API_URL,
                    "users",
                    CacheValue::User
                )
            }
            CacheValueType::Team => {
                let mut teams = fetch_many_batched::<Vec<TeamMember>>(
                    Method::GET,
                    MODRINTH_API_URL_V3,
                    "teams?ids=",
                    &keys,
                    fetch_semaphore,
                    pool,
                )
                .await?;

                let mut values = vec![];
                for key in keys {
                    let key = key.to_string();

                    if let Some(position) = teams.iter().position(|x| {
                        x.first().map(|x| x.team_id == key).unwrap_or(false)
                    }) {
                        let team = teams.remove(position);

                        for member in &team {
                            values.push((
                                CacheValue::User(member.user.clone())
                                    .get_entry(),
                                false,
                            ));
                        }

                        values.push((CacheValue::Team(team).get_entry(), true))
                    } else {
                        values.push((
                            CacheValueType::Team.get_empty_entry(key),
                            true,
                        ))
                    }
                }

                values
            }
            CacheValueType::Organization => {
                let mut orgs = fetch_many_batched::<Organization>(
                    Method::GET,
                    MODRINTH_API_URL_V3,
                    "organizations?ids=",
                    &keys,
                    fetch_semaphore,
                    pool,
                )
                .await?;

                let mut values = vec![];
                let visited_keys = DashSet::new();

                for key in keys {
                    let id = key.to_string();
                    let slug = id.to_lowercase();

                    if let Some(position) = orgs.iter().position(|x| {
                        x.id == id || x.slug.to_lowercase() == slug
                    }) {
                        visited_keys.insert(id);
                        visited_keys.insert(slug);

                        let org = orgs.remove(position);

                        for member in &org.members {
                            values.push((
                                CacheValue::User(member.user.clone())
                                    .get_entry(),
                                false,
                            ));
                        }

                        values.push((
                            CacheValue::Team(org.members.clone()).get_entry(),
                            false,
                        ));

                        values.push((
                            CacheValue::Organization(org).get_entry(),
                            true,
                        ));
                    } else if !visited_keys.contains(&id)
                        && !visited_keys.contains(&slug)
                    {
                        values.push((
                            CacheValueType::Organization.get_empty_entry(id),
                            true,
                        ));
                    }
                }

                values
            }
            CacheValueType::File => {
                let mut versions = fetch_json::<HashMap<String, Version>>(
                    Method::POST,
                    &format!("{MODRINTH_API_URL}version_files"),
                    None,
                    Some(serde_json::json!({
                        "algorithm": "sha1",
                        "hashes": &keys,
                    })),
                    fetch_semaphore,
                    pool,
                )
                .await?;

                let mut vals = Vec::new();

                for key in keys {
                    let hash = key.to_string();

                    if let Some(version) = versions.remove(&hash) {
                        let version_id = version.id.clone();
                        let project_id = version.project_id.clone();
                        vals.push((
                            CacheValue::Version(version).get_entry(),
                            false,
                        ));

                        vals.push((
                            CacheValue::File(CachedFile {
                                hash,
                                version_id,
                                project_id,
                            })
                            .get_entry(),
                            true,
                        ))
                    } else {
                        vals.push((
                            Self {
                                id: hash,
                                alias: None,
                                type_: CacheValueType::File,
                                data: None,
                                expires: Utc::now().timestamp()
                                    + CacheValueType::File.expiry(),
                            },
                            true,
                        ))
                    };
                }

                vals
            }
            CacheValueType::LoaderManifest => {
                let fetch_urls = keys
                    .iter()
                    .map(|x| {
                        (
                            x.key().to_string(),
                            format!("{META_URL}{}/v0/manifest.json", x.key()),
                        )
                    })
                    .collect::<Vec<_>>();

                futures::future::try_join_all(fetch_urls.iter().map(
                    |(_, url)| {
                        fetch_json(
                            Method::GET,
                            url,
                            None,
                            None,
                            fetch_semaphore,
                            pool,
                        )
                    },
                ))
                .await?
                .into_iter()
                .enumerate()
                .map(|(index, metadata)| {
                    (
                        CacheValue::LoaderManifest(CachedLoaderManifest {
                            loader: fetch_urls[index].0.to_string(),
                            manifest: metadata,
                        })
                        .get_entry(),
                        true,
                    )
                })
                .collect()
            }
            CacheValueType::MinecraftManifest => {
                fetch_original_value!(
                    MinecraftManifest,
                    META_URL,
                    format!(
                        "minecraft/v{}/manifest.json",
                        daedalus::minecraft::CURRENT_FORMAT_VERSION
                    ),
                    CacheValue::MinecraftManifest
                )
            }
            CacheValueType::Categories => {
                fetch_original_value!(
                    Categories,
                    MODRINTH_API_URL,
                    "tag/category",
                    CacheValue::Categories
                )
            }
            CacheValueType::ReportTypes => {
                fetch_original_value!(
                    ReportTypes,
                    MODRINTH_API_URL,
                    "tag/report_type",
                    CacheValue::ReportTypes
                )
            }
            CacheValueType::Loaders => {
                fetch_original_value!(
                    Loaders,
                    MODRINTH_API_URL,
                    "tag/loader",
                    CacheValue::Loaders
                )
            }
            CacheValueType::GameVersions => {
                fetch_original_value!(
                    GameVersions,
                    MODRINTH_API_URL,
                    "tag/game_version",
                    CacheValue::GameVersions
                )
            }
            CacheValueType::DonationPlatforms => {
                fetch_original_value!(
                    DonationPlatforms,
                    MODRINTH_API_URL,
                    "tag/donation_platform",
                    CacheValue::DonationPlatforms
                )
            }
            CacheValueType::FileHash => {
                // TODO: Replace state call here
                let state = crate::State::get().await?;
                let profiles_dir = state.directories.profiles_dir();

                async fn hash_file(
                    profiles_dir: &Path,
                    key: String,
                ) -> crate::Result<(CachedEntry, bool)> {
                    let path =
                        key.split_once('-').map(|x| x.1).unwrap_or_default();

                    let full_path = profiles_dir.join(path);

                    let mut file = tokio::fs::File::open(&full_path).await?;
                    let size = file.metadata().await?.len();

                    let mut hasher = sha1_smol::Sha1::new();

                    let mut buffer = vec![0u8; 262144]; // 256KiB
                    loop {
                        use tokio::io::AsyncReadExt;
                        let bytes_read = file.read(&mut buffer).await?;
                        if bytes_read == 0 {
                            break;
                        }
                        hasher.update(&buffer[..bytes_read]);
                    }

                    let hash = hasher.digest().to_string();

                    Ok((
                        CacheValue::FileHash(CachedFileHash {
                            path: path.to_string(),
                            size,
                            hash,
                            project_type: ProjectType::get_from_parent_folder(
                                &full_path,
                            ),
                        })
                        .get_entry(),
                        true,
                    ))
                }

                use futures::stream::StreamExt;
                let results: Vec<_> = futures::stream::iter(keys)
                    .map(|x| hash_file(&profiles_dir, x.to_string()))
                    .buffer_unordered(64) // hash 64 files at once
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .filter_map(|x| x.ok())
                    .collect();

                results
            }
            CacheValueType::FileUpdate => {
                let mut vals = Vec::new();

                // TODO: switch to update individual once back-end route exists
                let mut filtered_keys: Vec<((String, String), Vec<String>)> =
                    Vec::new();
                keys.iter().for_each(|x| {
                    let string = x.key().to_string();
                    let key = string.splitn(3, '-').collect::<Vec<_>>();

                    if key.len() == 3 {
                        let hash = key[0];
                        let loaders_key = key[1];
                        let game_version = key[2];

                        if let Some(values) =
                            filtered_keys.iter_mut().find(|x| {
                                x.0.0 == loaders_key && x.0.1 == game_version
                            })
                        {
                            values.1.push(hash.to_string());
                        } else {
                            filtered_keys.push((
                                (
                                    loaders_key.to_string(),
                                    game_version.to_string(),
                                ),
                                vec![hash.to_string()],
                            ))
                        }
                    } else {
                        vals.push((
                            CacheValueType::FileUpdate.get_empty_entry(string),
                            true,
                        ))
                    }
                });

                let version_update_url =
                    format!("{MODRINTH_API_URL}version_files/update");
                let variations =
                    futures::future::try_join_all(filtered_keys.iter().map(
                        |((loaders_key, game_version), hashes)| {
                            fetch_json::<HashMap<String, Version>>(
                                Method::POST,
                                &version_update_url,
                                None,
                                Some(serde_json::json!({
                                    "algorithm": "sha1",
                                    "hashes": hashes,
                                    "loaders": loaders_key.split('+').collect::<Vec<_>>(),
                                    "game_versions": [game_version]
                                })),
                                fetch_semaphore,
                                pool,
                            )
                        },
                    ))
                    .await?;

                for (index, mut variation) in variations.into_iter().enumerate()
                {
                    let ((loaders_key, game_version), hashes) =
                        &filtered_keys[index];

                    for hash in hashes {
                        let version = variation.remove(hash);

                        if let Some(version) = version {
                            let version_id = version.id.clone();
                            vals.push((
                                CacheValue::Version(version).get_entry(),
                                false,
                            ));

                            vals.push((
                                CacheValue::FileUpdate(CachedFileUpdate {
                                    hash: hash.clone(),
                                    game_version: game_version.clone(),
                                    loaders: loaders_key
                                        .split('+')
                                        .map(|x| x.to_string())
                                        .collect(),
                                    update_version_id: version_id,
                                })
                                .get_entry(),
                                true,
                            ));
                        } else {
                            vals.push((
                                CacheValueType::FileUpdate.get_empty_entry(
                                    format!(
                                        "{hash}-{loaders_key}-{game_version}"
                                    ),
                                ),
                                true,
                            ))
                        };
                    }
                }

                vals
            }
            CacheValueType::SearchResults => {
                let fetch_urls = keys
                    .iter()
                    .map(|x| {
                        (
                            x.key().to_string(),
                            format!("{MODRINTH_API_URL}search{}", x.key()),
                        )
                    })
                    .collect::<Vec<_>>();

                futures::future::try_join_all(fetch_urls.iter().map(
                    |(_, url)| {
                        fetch_json(
                            Method::GET,
                            url,
                            None,
                            None,
                            fetch_semaphore,
                            pool,
                        )
                    },
                ))
                .await?
                .into_iter()
                .enumerate()
                .map(|(index, result)| {
                    (
                        CacheValue::SearchResults(SearchResults {
                            search: fetch_urls[index].0.to_string(),
                            result,
                        })
                        .get_entry(),
                        true,
                    )
                })
                .collect()
            }
        })
    }

    pub(crate) async fn upsert_many(
        items: &[Self],
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let items = serde_json::to_string(items)?;

        sqlx::query!(
            "
            INSERT INTO cache (id, data_type, alias, data, expires)
                SELECT
                    json_extract(value, '$.id') AS id,
                    json_extract(value, '$.data_type') AS data_type,
                    json_extract(value, '$.alias') AS alias,
                    json_extract(value, '$.data') AS data,
                    json_extract(value, '$.expires') AS expires
                FROM
                    json_each($1)
                WHERE TRUE
            ON CONFLICT (id, data_type) DO UPDATE SET
                alias = excluded.alias,
                data = excluded.data,
                expires = excluded.expires
            ",
            items,
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn purge_cache_types(
        cache_types: &[CacheValueType],
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let cache_types = serde_json::to_string(&cache_types)?;

        sqlx::query!(
            "
            DELETE FROM cache
            WHERE data_type IN (SELECT value FROM json_each($1))
            ",
            cache_types,
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}

pub async fn cache_file_hash(
    bytes: bytes::Bytes,
    profile_path: &str,
    path: &str,
    known_hash: Option<&str>,
    project_type: Option<ProjectType>,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<()> {
    let size = bytes.len();

    let hash = if let Some(known_hash) = known_hash {
        known_hash.to_string()
    } else {
        sha1_async(bytes).await?
    };

    CachedEntry::upsert_many(
        &[CacheValue::FileHash(CachedFileHash {
            path: format!("{profile_path}/{path}"),
            size: size as u64,
            hash,
            project_type,
        })
        .get_entry()],
        exec,
    )
    .await?;

    Ok(())
}

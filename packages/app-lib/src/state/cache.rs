use crate::config::{META_URL, MODRINTH_API_URL, MODRINTH_API_URL_V3};
use crate::util::fetch::{fetch_json, FetchSemaphore};
use chrono::{DateTime, Utc};
use dashmap::DashSet;
use reqwest::Method;
use serde::{Deserialize, Serialize};
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
        }
    }

    pub fn from_str(val: &str) -> CacheValueType {
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
            _ => CacheValueType::Project,
        }
    }

    pub fn expiry(&self) -> i64 {
        match self {
            CacheValueType::File => 60 * 60 * 24 * 30, // 30 days
            CacheValueType::FileHash => 60 * 60 * 24 * 30, // 30 days
            _ => 60 * 60 * 24,                         // 24 hours
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedFileUpdate {
    pub hash: String,
    pub game_version: String,
    pub loader: String,
    pub update_version_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedFileHash {
    pub path: String,
    pub file_name: String,
    pub size: u64,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedLoaderManifest {
    pub loader: String,
    pub manifest: daedalus::modded::Manifest,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedFile {
    pub hash: String,
    pub metadata: FileMetadata,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FileMetadata {
    Modrinth {
        project_id: String,
        version_id: String,
    },
    Inferred {
        title: Option<String>,
        description: Option<String>,
        authors: Vec<String>,
        version: Option<String>,
        icon: Option<PathBuf>,
        project_type: Option<String>,
    },
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub id: String,
    pub slug: Option<String>,
    pub project_type: String,
    pub team: String,
    pub title: String,
    pub description: String,
    pub body: String,

    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,

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
    fn get_type(&self) -> CacheValueType {
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
                format!("{}-{}", hash.size, hash.path)
            }
            CacheValue::FileUpdate(hash) => {
                format!("{}-{}-{}", hash.hash, hash.loader, hash.game_version)
            }
        }
    }

    fn get_alias(&self) -> Option<&str> {
        match self {
            CacheValue::Project(project) => project.slug.as_deref(),
            CacheValue::User(user) => Some(&user.username),
            CacheValue::Organization(org) => Some(&org.slug),

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
            | CacheValue::FileHash(_)
            | CacheValue::FileUpdate(_) => None,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum CacheBehaviour {
    // Serve expired data, revalidate in background
    StaleWhileRevalidate,
    // Must revalidate if data is expired
    MustRevalidate,
    // Ignore cache- always fetch updated data from origin
    Bypass,
}

impl Default for CacheBehaviour {
    fn default() -> Self {
        Self::StaleWhileRevalidate
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEntry {
    id: String,
    alias: Option<String>,
    #[serde(rename = "data_type")]
    type_: CacheValueType,
    data: CacheValue,
    expires: i64,
}

macro_rules! impl_cache_methods {
    ($(($variant:ident, $type:ty)),*) => {
        impl CachedEntry {
            $(
                paste::paste! {
                    #[tracing::instrument(skip(exec, fetch_semaphore))]
                    pub async fn [<get_ $variant:snake>] <'a, E>(
                        id: &str,
                        cache_behaviour: Option<CacheBehaviour>,
                        exec: E,
                        fetch_semaphore: &FetchSemaphore,
                    ) -> crate::Result<Option<$type>>
                    where
                        E: sqlx::Acquire<'a, Database = sqlx::Sqlite>,
                    {
                        Ok(Self::[<get_ $variant:snake _many>](&[id], cache_behaviour, exec, fetch_semaphore).await?.into_iter().next())
                    }

                    #[tracing::instrument(skip(exec, fetch_semaphore))]
                    pub async fn [<get_ $variant:snake _many>] <'a, E>(
                        ids: &[&str],
                        cache_behaviour: Option<CacheBehaviour>,
                        exec: E,
                        fetch_semaphore: &FetchSemaphore,
                    ) -> crate::Result<Vec<$type>>
                    where
                        E: sqlx::Acquire<'a, Database = sqlx::Sqlite>,
                    {
                        let entry =
                            CachedEntry::get_many(CacheValueType::$variant, ids, cache_behaviour, exec, fetch_semaphore).await?;

                        Ok(entry.into_iter().filter_map(|x| if let CacheValue::$variant(value) = x.data {
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
                    #[tracing::instrument(skip(exec, fetch_semaphore))]
                    pub async fn [<get_ $variant:snake>] <'a, E>(
                        cache_behaviour: Option<CacheBehaviour>,
                        exec: E,
                        fetch_semaphore: &FetchSemaphore,
                    ) -> crate::Result<Option<$type>>
                    where
                        E: sqlx::Acquire<'a, Database = sqlx::Sqlite>,
                    {
                        let entry =
                            CachedEntry::get(CacheValueType::$variant, DEFAULT_ID, cache_behaviour, exec, fetch_semaphore).await?;

                        if let Some(CacheValue::$variant(value)) = entry.map(|x| x.data) {
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
    (FileUpdate, CachedFileUpdate)
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
    #[tracing::instrument(skip(exec, fetch_semaphore))]
    pub async fn get<'a, E>(
        type_: CacheValueType,
        key: &str,
        cache_behaviour: Option<CacheBehaviour>,
        exec: E,
        fetch_semaphore: &FetchSemaphore,
    ) -> crate::Result<Option<Self>>
    where
        E: sqlx::Acquire<'a, Database = sqlx::Sqlite>,
    {
        Ok(Self::get_many(
            type_,
            &[key],
            cache_behaviour,
            exec,
            fetch_semaphore,
        )
        .await?
        .into_iter()
        .next())
    }

    #[tracing::instrument(skip(conn, fetch_semaphore))]
    pub async fn get_many<'a, E>(
        type_: CacheValueType,
        keys: &[&str],
        cache_behaviour: Option<CacheBehaviour>,
        conn: E,
        fetch_semaphore: &FetchSemaphore,
    ) -> crate::Result<Vec<Self>>
    where
        E: sqlx::Acquire<'a, Database = sqlx::Sqlite>,
    {
        let cache_behaviour = cache_behaviour.unwrap_or_default();

        let remaining_keys = DashSet::new();
        for key in keys {
            remaining_keys.insert(*key);
        }

        let mut return_vals = Vec::new();
        let expired_keys = DashSet::new();

        let mut exec = conn.acquire().await?;

        if cache_behaviour != CacheBehaviour::Bypass {
            let type_ = type_.as_str();
            let keys = serde_json::to_string(&keys)?;

            // unsupported type NULL of column #3 ("data"), so cannot be compile time type checked
            // https://github.com/launchbadge/sqlx/issues/1979
            let query = sqlx::query!(
                r#"
                SELECT id, data_type, json(data) as "data!: serde_json::Value", alias, expires
                FROM cache
                WHERE data_type = $1 AND (
                    id IN (SELECT value FROM json_each($2))
                    OR
                    alias IN (SELECT value FROM json_each($2))
                )
                "#,
                type_,
                keys
            )
            .fetch_all(&mut *exec)
            .await?;

            for row in query {
                if let Ok(data) = serde_json::from_value(row.data) {
                    if row.expires <= Utc::now().timestamp() {
                        if cache_behaviour == CacheBehaviour::MustRevalidate {
                            continue;
                        } else {
                            expired_keys.insert(row.id.clone());
                        }
                    }

                    remaining_keys.remove(&*row.id);
                    if let Some(alias) = row.alias.as_deref() {
                        remaining_keys.remove(alias);
                    }

                    return_vals.push(Self {
                        id: row.id,
                        alias: row.alias,
                        type_: CacheValueType::from_str(&row.data_type),
                        data,
                        expires: row.expires,
                    });
                }
            }
        }

        if !remaining_keys.is_empty() {
            let mut values =
                Self::fetch_many(type_, remaining_keys, fetch_semaphore)
                    .await?;

            if !values.is_empty() {
                Self::upsert_many(&values, &mut *exec).await?;

                return_vals.append(&mut values);
            }
        }

        if !expired_keys.is_empty()
            && cache_behaviour == CacheBehaviour::StaleWhileRevalidate
        {
            tokio::task::spawn(async move {
                // TODO: if possible- find a way to do this without invoking state get
                let state = crate::state::State::get().await?;

                let values = Self::fetch_many(
                    type_,
                    expired_keys,
                    &state.fetch_semaphore,
                )
                .await?;

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
    ) -> crate::Result<Vec<Self>> {
        macro_rules! fetch_original_values {
            ($type:ident, $api_url:expr, $url_suffix:expr, $cache_variant:path) => {{
                fetch_json::<Vec<_>>(
                    Method::GET,
                    &*format!(
                        "{}{}?ids={}",
                        $api_url,
                        $url_suffix,
                        serde_json::to_string(&keys)?
                    ),
                    None,
                    None,
                    &fetch_semaphore,
                )
                .await?
                .into_iter()
                .map(|x| {
                    let data = $cache_variant(x);

                    Self {
                        id: data.get_key().to_string(),
                        alias: data.get_alias().map(|x| x.to_string()),
                        type_: CacheValueType::$type,
                        data,
                        expires: Utc::now().timestamp()
                            + CacheValueType::$type.expiry(),
                    }
                })
                .collect()
            }};
        }

        macro_rules! fetch_original_value {
            ($type:ident, $api_url:expr, $url_suffix:expr, $cache_variant:path) => {{
                vec![Self {
                    id: DEFAULT_ID.to_string(),
                    alias: None,
                    type_: CacheValueType::$type,
                    data: $cache_variant(
                        fetch_json(
                            Method::GET,
                            &*format!("{}{}", $api_url, $url_suffix),
                            None,
                            None,
                            &fetch_semaphore,
                        )
                        .await?,
                    ),
                    expires: Utc::now().timestamp()
                        + CacheValueType::$type.expiry(),
                }]
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
                fetch_original_values!(
                    Team,
                    MODRINTH_API_URL,
                    "teams",
                    CacheValue::Team
                )
            }
            CacheValueType::Organization => {
                fetch_original_values!(
                    Organization,
                    MODRINTH_API_URL_V3,
                    "organizations",
                    CacheValue::Organization
                )
            }
            CacheValueType::File => {
                let mut versions = fetch_json::<HashMap<String, Version>>(
                    Method::POST,
                    &format!("{}version_files", MODRINTH_API_URL),
                    None,
                    Some(serde_json::json!({
                        "algorithm": "sha1",
                        "hashes": &keys,
                    })),
                    fetch_semaphore,
                )
                .await?;

                let mut vals = Vec::new();

                for key in keys {
                    let hash = key.to_string();

                    let metadata = if let Some(version) = versions.remove(&hash)
                    {
                        FileMetadata::Modrinth {
                            project_id: version.project_id,
                            version_id: version.id,
                        }
                    } else {
                        FileMetadata::Unknown
                    };

                    vals.push(Self {
                        id: hash.clone(),
                        alias: None,
                        type_: CacheValueType::File,
                        data: CacheValue::File(CachedFile { hash, metadata }),
                        expires: Utc::now().timestamp()
                            + CacheValueType::File.expiry(),
                    })
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
                        )
                    },
                ))
                .await?
                .into_iter()
                .enumerate()
                .map(|(index, metadata)| Self {
                    id: fetch_urls[index].0.to_string(),
                    alias: None,
                    type_: CacheValueType::LoaderManifest,
                    data: CacheValue::LoaderManifest(CachedLoaderManifest {
                        loader: fetch_urls[index].0.to_string(),
                        manifest: metadata,
                    }),
                    expires: Utc::now().timestamp()
                        + CacheValueType::LoaderManifest.expiry(),
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
                let profiles_dir = state.directories.profiles_dir().await;

                async fn hash_file(
                    profiles_dir: &Path,
                    path: String,
                ) -> crate::Result<CachedEntry> {
                    let path =
                        path.split_once('-').map(|x| x.1).unwrap_or_default();

                    let full_path = profiles_dir.join(path);

                    let mut file = tokio::fs::File::open(&full_path).await?;
                    let size = file.metadata().await?.len();

                    let mut hasher = sha1_smol::Sha1::new();

                    let mut buffer = [0u8; 65536]; // 64KiB
                    loop {
                        use tokio::io::AsyncReadExt;
                        let bytes_read = file.read(&mut buffer).await?;
                        if bytes_read == 0 {
                            break;
                        }
                        hasher.update(&buffer[..bytes_read]);
                    }

                    let hash = hasher.digest().to_string();

                    let data = CacheValue::FileHash(CachedFileHash {
                        path: path.to_string(),
                        file_name: full_path
                            .file_name()
                            .and_then(|x| x.to_str())
                            .unwrap_or_default()
                            .to_string(),
                        size,
                        hash,
                    });

                    Ok(CachedEntry {
                        id: path.to_string(),
                        alias: None,
                        type_: CacheValueType::FileHash,
                        data,
                        expires: Utc::now().timestamp()
                            + CacheValueType::FileHash.expiry(),
                    })
                }

                use futures::stream::StreamExt;
                let results: Vec<_> = futures::stream::iter(keys)
                    .map(|x| hash_file(&profiles_dir, x.to_string()))
                    .buffer_unordered(16) // hash 16 files at once
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .filter_map(|x| x.ok())
                    .collect();

                results
            }
            CacheValueType::FileUpdate => {
                // TODO: switch to update individual once back-end route exists
                let mut filtered_keys: Vec<((String, String), Vec<String>)> =
                    Vec::new();
                keys.iter().for_each(|x| {
                    let string = x.key().to_string();
                    let key = string.splitn(3, '-').collect::<Vec<_>>();

                    if key.len() == 3 {
                        let hash = key[0];
                        let loader = key[1];
                        let game_version = key[2];

                        if let Some(values) =
                            filtered_keys.iter_mut().find(|x| {
                                x.0 .0 == loader && x.0 .1 == game_version
                            })
                        {
                            values.1.push(hash.to_string());
                        } else {
                            filtered_keys.push((
                                (loader.to_string(), game_version.to_string()),
                                vec![hash.to_string()],
                            ))
                        }
                    }
                });

                let version_update_url =
                    format!("{}version_files/update", MODRINTH_API_URL);
                let variations =
                    futures::future::try_join_all(filtered_keys.iter().map(
                        |((loader, game_version), hashes)| {
                            fetch_json::<HashMap<String, Version>>(
                                Method::POST,
                                &version_update_url,
                                None,
                                Some(serde_json::json!({
                                    "algorithm": "sha1",
                                    "hashes": hashes,
                                    "loaders": [loader],
                                    "game_versions": [game_version]
                                })),
                                fetch_semaphore,
                            )
                        },
                    ))
                    .await?;

                let mut vals = Vec::new();

                for (index, mut variation) in variations.into_iter().enumerate()
                {
                    let ((loader, game_version), hashes) =
                        &filtered_keys[index];

                    for hash in hashes {
                        let version_id = variation.remove(hash).map(|x| x.id);

                        let data = CacheValue::FileUpdate(CachedFileUpdate {
                            hash: hash.clone(),
                            game_version: game_version.clone(),
                            loader: loader.clone(),
                            update_version_id: version_id,
                        });

                        vals.push(Self {
                            id: data.get_key(),
                            alias: None,
                            type_: CacheValueType::FileUpdate,
                            data,
                            expires: Utc::now().timestamp()
                                + CacheValueType::FileUpdate.expiry(),
                        })
                    }
                }

                vals
            }
        })
    }

    /// Update/sets a value in the cache to the given value. Avoid using if possible:
    /// stick to `Self::get` and `Self::get_many`.
    pub(crate) async fn upsert(
        self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        Self::upsert_many(&[self], exec).await
    }

    /// Update/sets values in the cache to the given values. Avoid using if possible:
    /// stick to `Self::get` and `Self::get_many`.
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
}

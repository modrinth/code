use crate::data::{Dependency, ProjectType, User, Version};
use crate::jre::check_jre;
use crate::prelude::ModLoader;
use crate::state;
use crate::state::{
    CacheValue, CachedEntry, CachedFile, CachedFileHash, CachedFileUpdate,
    Credentials, DefaultPage, DependencyType, DeviceToken, DeviceTokenKey,
    DeviceTokenPair, FileType, Hooks, LauncherFeatureVersion, LinkedData,
    MemorySettings, ModrinthCredentials, Profile, ProfileInstallStage,
    TeamMember, Theme, VersionFile, WindowSize,
};
use crate::util::fetch::{IoSemaphore, read_json};
use chrono::{DateTime, Utc};
use p256::ecdsa::SigningKey;
use p256::pkcs8::DecodePrivateKey;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::Semaphore;
use uuid::Uuid;

pub async fn migrate_legacy_data<'a, E>(exec: E) -> crate::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Sqlite> + Copy,
{
    let mut settings = state::Settings::get(exec).await?;

    if settings.migrated {
        return Ok(());
    };

    let Some(old_launcher_root) = default_settings_dir() else {
        return Ok(());
    };
    let old_launcher_root_str = old_launcher_root.to_string_lossy().to_string();

    let io_semaphore = IoSemaphore(Semaphore::new(10));
    let settings_path = old_launcher_root.join("settings.json");

    if let Ok(legacy_settings) =
        read_json::<LegacySettings>(&settings_path, &io_semaphore).await
    {
        settings.max_concurrent_writes = legacy_settings.max_concurrent_writes;
        settings.max_concurrent_downloads =
            legacy_settings.max_concurrent_downloads;
        settings.theme = match legacy_settings.theme {
            LegacyTheme::Dark => Theme::Dark,
            LegacyTheme::Light => Theme::Light,
            LegacyTheme::Oled => Theme::Oled,
        };
        settings.default_page = match legacy_settings.default_page {
            LegacyDefaultPage::Home => DefaultPage::Home,
            LegacyDefaultPage::Library => DefaultPage::Library,
        };
        settings.collapsed_navigation = legacy_settings.collapsed_navigation;
        settings.advanced_rendering = legacy_settings.advanced_rendering;
        settings.native_decorations = legacy_settings.native_decorations;
        settings.telemetry = !legacy_settings.opt_out_analytics;
        settings.discord_rpc = !legacy_settings.disable_discord_rpc;
        settings.developer_mode = legacy_settings.developer_mode;
        settings.onboarded = legacy_settings.fully_onboarded;
        settings.extra_launch_args = legacy_settings.custom_java_args;
        settings.custom_env_vars = legacy_settings.custom_env_args;
        settings.memory.maximum = legacy_settings.memory.maximum;
        settings.force_fullscreen = legacy_settings.force_fullscreen;
        settings.game_resolution.0 = legacy_settings.game_resolution.0;
        settings.game_resolution.1 = legacy_settings.game_resolution.1;
        settings.hide_on_process_start = legacy_settings.hide_on_process;
        settings.hooks.pre_launch = legacy_settings.hooks.pre_launch;
        settings.hooks.wrapper = legacy_settings.hooks.wrapper;
        settings.hooks.post_exit = legacy_settings.hooks.post_exit;

        if let Some(path) = legacy_settings
            .loaded_config_dir
            .clone()
            .and_then(|x| x.to_str().map(|x| x.to_string()))
        {
            if path != old_launcher_root_str {
                settings.custom_dir = Some(path);
            }
        }

        settings.prev_custom_dir = Some(old_launcher_root_str.clone());

        for (_, legacy_version) in legacy_settings.java_globals.0 {
            if let Ok(java_version) =
                check_jre(PathBuf::from(legacy_version.path)).await
            {
                java_version.upsert(exec).await?;
            }
        }

        let modrinth_auth_path =
            old_launcher_root.join("caches/metadata/auth.json");
        if let Ok(creds) = read_json::<LegacyModrinthCredentials>(
            &modrinth_auth_path,
            &io_semaphore,
        )
        .await
        {
            ModrinthCredentials {
                session: creds.session,
                expires: creds.expires_at,
                user_id: creds.user.id,
                active: true,
            }
            .upsert(exec)
            .await?;
        }

        let minecraft_auth_path =
            old_launcher_root.join("caches/metadata/minecraft_auth.json");
        if let Ok(minecraft_auth) = read_json::<LegacyMinecraftAuthStore>(
            &minecraft_auth_path,
            &io_semaphore,
        )
        .await
        {
            let minecraft_users_len = minecraft_auth.users.len();
            for (uuid, credential) in minecraft_auth.users {
                Credentials {
                    id: credential.id,
                    username: credential.username,
                    access_token: credential.access_token,
                    refresh_token: credential.refresh_token,
                    expires: credential.expires,
                    active: minecraft_auth.default_user == Some(uuid)
                        || minecraft_users_len == 1,
                }
                .upsert(exec)
                .await?;
            }

            if let Some(device_token) = minecraft_auth.token {
                if let Ok(private_key) =
                    SigningKey::from_pkcs8_pem(&device_token.private_key)
                {
                    if let Ok(uuid) = Uuid::parse_str(&device_token.id) {
                        DeviceTokenPair {
                            token: DeviceToken {
                                issue_instant: device_token.token.issue_instant,
                                not_after: device_token.token.not_after,
                                token: device_token.token.token,
                                display_claims: device_token
                                    .token
                                    .display_claims,
                            },
                            key: DeviceTokenKey {
                                id: uuid,
                                key: private_key,
                                x: device_token.x,
                                y: device_token.y,
                            },
                        }
                        .upsert(exec)
                        .await?;
                    }
                }
            }
        }

        let mut cached_entries = vec![];

        if let Ok(profiles_dir) = std::fs::read_dir(
            legacy_settings
                .loaded_config_dir
                .clone()
                .unwrap_or_else(|| old_launcher_root.clone())
                .join("profiles"),
        ) {
            for entry in profiles_dir.flatten() {
                if !entry.path().is_dir() {
                    continue;
                }

                let profile_path = entry.path().join("profile.json");

                let Ok(profile) =
                    read_json::<LegacyProfile>(&profile_path, &io_semaphore)
                        .await
                else {
                    continue;
                };

                for (path, project) in profile.projects {
                    let full_path = legacy_settings
                        .loaded_config_dir
                        .clone()
                        .unwrap_or_else(|| old_launcher_root.clone())
                        .join("profiles")
                        .join(&profile.path)
                        .join(&path);

                    if !full_path.exists() || !full_path.is_file() {
                        continue;
                    }
                    let sha512 = project.sha512;

                    if let LegacyProjectMetadata::Modrinth {
                        version,
                        members,
                        update_version,
                        ..
                    } = project.metadata
                    {
                        if let Some(file) = version
                            .files
                            .iter()
                            .find(|x| x.hashes.get("sha512") == Some(&sha512))
                        {
                            if let Some(sha1) = file.hashes.get("sha1") {
                                if let Ok(metadata) = full_path.metadata() {
                                    let file_name = format!(
                                        "{}/{}",
                                        profile.path,
                                        path.replace('\\', "/")
                                            .replace(".disabled", "")
                                    );

                                    cached_entries.push(CacheValue::FileHash(
                                        CachedFileHash {
                                            path: file_name,
                                            size: metadata.len(),
                                            hash: sha1.clone(),
                                            project_type: ProjectType::get_from_parent_folder(&full_path),
                                        },
                                    ));
                                }

                                cached_entries.push(CacheValue::File(
                                    CachedFile {
                                        hash: sha1.clone(),
                                        project_id: version.project_id.clone(),
                                        version_id: version.id.clone(),
                                    },
                                ));

                                if let Some(update_version) = update_version {
                                    let mod_loader: ModLoader =
                                        profile.metadata.loader.into();
                                    cached_entries.push(
                                        CacheValue::FileUpdate(
                                            CachedFileUpdate {
                                                hash: sha1.clone(),
                                                game_version: profile
                                                    .metadata
                                                    .game_version
                                                    .clone(),
                                                loaders: vec![
                                                    mod_loader
                                                        .as_str()
                                                        .to_string(),
                                                ],
                                                update_version_id:
                                                    update_version.id.clone(),
                                            },
                                        ),
                                    );

                                    cached_entries.push(CacheValue::Version(
                                        (*update_version).into(),
                                    ));
                                }

                                let members = members
                                    .into_iter()
                                    .map(|x| {
                                        let user = User {
                                            id: x.user.id,
                                            username: x.user.username,
                                            avatar_url: x.user.avatar_url,
                                            bio: x.user.bio,
                                            created: x.user.created,
                                            role: x.user.role,
                                            badges: 0,
                                        };

                                        cached_entries.push(CacheValue::User(
                                            user.clone(),
                                        ));

                                        TeamMember {
                                            team_id: x.team_id,
                                            user,
                                            is_owner: x.role == "Owner",
                                            role: x.role,
                                            ordering: x.ordering,
                                        }
                                    })
                                    .collect::<Vec<_>>();

                                cached_entries.push(CacheValue::Team(members));

                                cached_entries.push(CacheValue::Version(
                                    (*version).into(),
                                ));
                            }
                        }
                    }
                }

                Profile {
                    path: profile.path,
                    install_stage: match profile.install_stage {
                        LegacyProfileInstallStage::Installed => {
                            ProfileInstallStage::Installed
                        }
                        LegacyProfileInstallStage::Installing => {
                            ProfileInstallStage::MinecraftInstalling
                        }
                        LegacyProfileInstallStage::PackInstalling => {
                            ProfileInstallStage::PackInstalling
                        }
                        LegacyProfileInstallStage::NotInstalled => {
                            ProfileInstallStage::NotInstalled
                        }
                    },
                    launcher_feature_version: LauncherFeatureVersion::None,
                    name: profile.metadata.name,
                    icon_path: profile.metadata.icon,
                    game_version: profile.metadata.game_version,
                    protocol_version: None,
                    loader: profile.metadata.loader.into(),
                    loader_version: profile
                        .metadata
                        .loader_version
                        .map(|x| x.id),
                    groups: profile.metadata.groups,
                    linked_data: profile.metadata.linked_data.and_then(|x| {
                        if let Some(project_id) = x.project_id {
                            if let Some(version_id) = x.version_id {
                                if let Some(locked) = x.locked {
                                    return Some(LinkedData {
                                        project_id,
                                        version_id,
                                        locked,
                                    });
                                }
                            }
                        }

                        None
                    }),
                    created: profile.metadata.date_created,
                    modified: profile.metadata.date_modified,
                    last_played: profile.metadata.last_played,
                    submitted_time_played: profile
                        .metadata
                        .submitted_time_played,
                    recent_time_played: profile.metadata.recent_time_played,
                    java_path: profile.java.as_ref().and_then(|x| {
                        x.override_version.clone().map(|x| x.path)
                    }),
                    extra_launch_args: profile
                        .java
                        .as_ref()
                        .and_then(|x| x.extra_arguments.clone()),
                    custom_env_vars: profile
                        .java
                        .and_then(|x| x.custom_env_args),
                    memory: profile
                        .memory
                        .map(|x| MemorySettings { maximum: x.maximum }),
                    force_fullscreen: profile.fullscreen,
                    game_resolution: profile
                        .resolution
                        .map(|x| WindowSize(x.0, x.1)),
                    hooks: Hooks {
                        pre_launch: profile
                            .hooks
                            .as_ref()
                            .and_then(|x| x.pre_launch.clone()),
                        wrapper: profile
                            .hooks
                            .as_ref()
                            .and_then(|x| x.wrapper.clone()),
                        post_exit: profile.hooks.and_then(|x| x.post_exit),
                    },
                }
                .upsert(exec)
                .await?;
            }
        }

        CachedEntry::upsert_many(
            &cached_entries
                .into_iter()
                .map(|x| {
                    let mut entry = x.get_entry();
                    entry.expires =
                        Utc::now().timestamp() - entry.type_.expiry();
                    entry
                })
                .collect::<Vec<_>>(),
            exec,
        )
        .await?;

        settings.migrated = true;
        settings.update(exec).await?;
    }

    Ok(())
}

#[derive(Deserialize, Debug, Clone)]
struct LegacySettings {
    pub theme: LegacyTheme,
    pub memory: LegacyMemorySettings,
    #[serde(default)]
    pub force_fullscreen: bool,
    pub game_resolution: LegacyWindowSize,
    pub custom_java_args: Vec<String>,
    pub custom_env_args: Vec<(String, String)>,
    pub java_globals: LegacyJavaGlobals,
    pub hooks: LegacyHooks,
    pub max_concurrent_downloads: usize,
    pub max_concurrent_writes: usize,
    pub collapsed_navigation: bool,
    #[serde(default)]
    pub disable_discord_rpc: bool,
    #[serde(default)]
    pub hide_on_process: bool,
    #[serde(default)]
    pub native_decorations: bool,
    #[serde(default)]
    pub default_page: LegacyDefaultPage,
    #[serde(default)]
    pub developer_mode: bool,
    #[serde(default)]
    pub opt_out_analytics: bool,
    #[serde(default)]
    pub advanced_rendering: bool,
    #[serde(default)]
    pub fully_onboarded: bool,
    #[serde(default = "default_settings_dir")]
    pub loaded_config_dir: Option<PathBuf>,
}

fn default_settings_dir() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("com.modrinth.theseus"))
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyTheme {
    Dark,
    Light,
    Oled,
}

#[derive(Deserialize, Default, Debug, Clone, Copy)]
enum LegacyDefaultPage {
    #[default]
    Home,
    Library,
}

#[derive(Deserialize, Debug, Clone)]
struct LegacyHooks {
    pub pre_launch: Option<String>,
    pub wrapper: Option<String>,
    pub post_exit: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
struct LegacyMemorySettings {
    pub maximum: u32,
}

#[derive(Deserialize, Debug, Clone, Copy)]
struct LegacyWindowSize(pub u16, pub u16);

#[derive(Debug, Deserialize, Clone)]
struct LegacyJavaGlobals(HashMap<String, LegacyJavaVersion>);

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Clone)]
struct LegacyJavaVersion {
    pub path: String,
    pub version: String,
    pub architecture: String,
}

#[derive(Deserialize, Clone, Debug)]
struct LegacyModrinthUser {
    pub id: String,
    pub username: String,
    // pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created: DateTime<Utc>,
    pub role: String,
}

#[derive(Deserialize, Clone, Debug)]
struct LegacyModrinthCredentials {
    pub session: String,
    pub expires_at: DateTime<Utc>,
    pub user: LegacyModrinthUser,
}

#[derive(Deserialize, Debug)]
struct LegacyMinecraftAuthStore {
    pub users: HashMap<Uuid, LegacyCredentials>,
    pub token: Option<LegacySaveDeviceToken>,
    pub default_user: Option<Uuid>,
}

#[derive(Deserialize, Clone, Debug)]
struct LegacyCredentials {
    pub id: Uuid,
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
struct LegacySaveDeviceToken {
    pub id: String,
    pub private_key: String,
    pub x: String,
    pub y: String,
    pub token: LegacyDeviceToken,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct LegacyDeviceToken {
    pub issue_instant: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub token: String,
    pub display_claims: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Clone, Debug)]
struct LegacyProfile {
    #[serde(default)]
    pub install_stage: LegacyProfileInstallStage,
    #[serde(default)]
    pub path: String,
    pub metadata: LegacyProfileMetadata,
    pub java: Option<LegacyJavaSettings>,
    pub memory: Option<LegacyMemorySettings>,
    pub resolution: Option<LegacyWindowSize>,
    pub fullscreen: Option<bool>,
    pub hooks: Option<LegacyHooks>,
    pub projects: HashMap<String, LegacyProject>,
}

#[derive(Deserialize, Clone, Debug)]
struct LegacyProject {
    pub sha512: String,
    // pub disabled: bool,
    pub metadata: LegacyProjectMetadata,
    // pub file_name: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LegacyProjectMetadata {
    Modrinth {
        // project: Box<LegacyModrinthProject>,
        version: Box<LegacyModrinthVersion>,
        members: Vec<LegacyModrinthTeamMember>,
        update_version: Option<Box<LegacyModrinthVersion>>,
    },
    Inferred,
    Unknown,
}

// #[derive(Deserialize, Clone, Debug)]
// struct LegacyModrinthProject {
//     pub id: String,
//     pub slug: Option<String>,
//     pub project_type: String,
//     pub team: String,
//     pub title: String,
//     pub description: String,
//     pub body: String,
//
//     pub published: DateTime<Utc>,
//     pub updated: DateTime<Utc>,
//
//     pub client_side: LegacySideType,
//     pub server_side: LegacySideType,
//
//     pub downloads: u32,
//     pub followers: u32,
//
//     pub categories: Vec<String>,
//     pub additional_categories: Vec<String>,
//     pub game_versions: Vec<String>,
//     pub loaders: Vec<String>,
//
//     pub versions: Vec<String>,
//
//     pub icon_url: Option<String>,
// }

#[derive(Deserialize, Clone, Debug)]
struct LegacyModrinthVersion {
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

    pub files: Vec<LegacyModrinthVersionFile>,
    pub dependencies: Vec<LegacyDependency>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

impl From<LegacyModrinthVersion> for Version {
    fn from(value: LegacyModrinthVersion) -> Self {
        Version {
            id: value.id,
            project_id: value.project_id,
            author_id: value.author_id,
            featured: value.featured,
            name: value.name,
            version_number: value.version_number,
            changelog: value.changelog,
            changelog_url: value.changelog_url,
            date_published: value.date_published,
            downloads: value.downloads,
            version_type: value.version_type,
            files: value
                .files
                .into_iter()
                .map(|x| VersionFile {
                    hashes: x.hashes,
                    url: x.url,
                    filename: x.filename,
                    primary: x.primary,
                    size: x.size,
                    file_type: x.file_type.map(|x| match x {
                        LegacyFileType::RequiredResourcePack => {
                            FileType::RequiredResourcePack
                        }
                        LegacyFileType::OptionalResourcePack => {
                            FileType::OptionalResourcePack
                        }
                        LegacyFileType::Unknown => FileType::Unknown,
                    }),
                })
                .collect::<Vec<_>>(),
            dependencies: value
                .dependencies
                .into_iter()
                .map(|x| Dependency {
                    version_id: x.version_id,
                    project_id: x.project_id,
                    file_name: x.file_name,
                    dependency_type: match x.dependency_type {
                        LegacyDependencyType::Required => {
                            DependencyType::Required
                        }
                        LegacyDependencyType::Optional => {
                            DependencyType::Optional
                        }
                        LegacyDependencyType::Incompatible => {
                            DependencyType::Incompatible
                        }
                        LegacyDependencyType::Embedded => {
                            DependencyType::Embedded
                        }
                    },
                })
                .collect::<Vec<_>>(),
            game_versions: value.game_versions,
            loaders: value.loaders,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
struct LegacyModrinthVersionFile {
    pub hashes: HashMap<String, String>,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u32,
    pub file_type: Option<LegacyFileType>,
}

#[derive(Deserialize, Clone, Debug)]
struct LegacyDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: LegacyDependencyType,
}

#[derive(Deserialize, Clone, Debug)]
struct LegacyModrinthTeamMember {
    pub team_id: String,
    pub user: LegacyModrinthUser,
    pub role: String,
    pub ordering: i64,
}

#[derive(Deserialize, Copy, Clone, Debug)]
#[serde(rename_all = "lowercase")]
enum LegacyDependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

// #[derive(Deserialize, Clone, Debug, Eq, PartialEq)]
// #[serde(rename_all = "kebab-case")]
// enum LegacySideType {
//     Required,
//     Optional,
//     Unsupported,
//     Unknown,
// }

#[derive(Deserialize, Copy, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
enum LegacyFileType {
    RequiredResourcePack,
    OptionalResourcePack,
    Unknown,
}

#[derive(Deserialize, Clone, Debug)]
struct LegacyProfileMetadata {
    pub name: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub groups: Vec<String>,

    pub game_version: String,
    #[serde(default)]
    pub loader: LegacyModLoader,
    pub loader_version: Option<LegacyLoaderVersion>,

    pub linked_data: Option<LegacyLinkedData>,

    #[serde(default)]
    pub date_created: DateTime<Utc>,
    #[serde(default)]
    pub date_modified: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,
    #[serde(default)]
    pub submitted_time_played: u64,
    #[serde(default)]
    pub recent_time_played: u64,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum LegacyModLoader {
    #[default]
    Vanilla,
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

impl From<LegacyModLoader> for ModLoader {
    fn from(value: LegacyModLoader) -> Self {
        match value {
            LegacyModLoader::Vanilla => ModLoader::Vanilla,
            LegacyModLoader::Forge => ModLoader::Forge,
            LegacyModLoader::Fabric => ModLoader::Fabric,
            LegacyModLoader::Quilt => ModLoader::Quilt,
            LegacyModLoader::NeoForge => ModLoader::NeoForge,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
struct LegacyLinkedData {
    pub project_id: Option<String>,
    pub version_id: Option<String>,

    #[serde(default = "default_locked")]
    pub locked: Option<bool>,
}

fn default_locked() -> Option<bool> {
    Some(true)
}

#[derive(Deserialize, Clone, Debug)]
struct LegacyJavaSettings {
    pub override_version: Option<LegacyJavaVersion>,
    pub extra_arguments: Option<Vec<String>>,
    pub custom_env_args: Option<Vec<(String, String)>>,
}

#[derive(Deserialize, Clone, Debug)]
struct LegacyLoaderVersion {
    pub id: String,
}

#[derive(Deserialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum LegacyProfileInstallStage {
    Installed,
    Installing,
    PackInstalling,
    #[default]
    NotInstalled,
}

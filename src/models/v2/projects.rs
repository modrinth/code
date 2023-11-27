use super::super::ids::OrganizationId;
use super::super::teams::TeamId;
use super::super::users::UserId;
use crate::database::models::legacy_loader_fields::MinecraftGameVersion;
use crate::database::models::{version_item, DatabaseError};
use crate::database::redis::RedisPool;
use crate::models::ids::{ProjectId, VersionId};
use crate::models::projects::{
    Dependency, DonationLink, GalleryItem, License, Loader, ModeratorMessage, MonetizationStatus,
    Project, ProjectStatus, Version, VersionFile, VersionStatus, VersionType,
};
use crate::models::threads::ThreadId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A project returned from the API
#[derive(Serialize, Deserialize, Clone)]
pub struct LegacyProject {
    /// Relevant V2 fields- these were removed or modfified in V3,
    /// and are now part of the dynamic fields system
    /// The support range for the client project*
    pub client_side: LegacySideType,
    /// The support range for the server project
    pub server_side: LegacySideType,
    /// A list of game versions this project supports
    pub game_versions: Vec<String>,

    // All other fields are the same as V3
    // If they change, or their constituent types change, we may need to
    // add a new struct for them here.
    pub id: ProjectId,
    pub slug: Option<String>,
    pub project_type: String,
    pub team: TeamId,
    pub organization: Option<OrganizationId>,
    pub title: String,
    pub description: String,
    pub body: String,
    pub body_url: Option<String>,
    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub approved: Option<DateTime<Utc>>,
    pub queued: Option<DateTime<Utc>>,
    pub status: ProjectStatus,
    pub requested_status: Option<ProjectStatus>,
    pub moderator_message: Option<ModeratorMessage>,
    pub license: License,
    pub downloads: u32,
    pub followers: u32,
    pub categories: Vec<String>,
    pub additional_categories: Vec<String>,
    pub loaders: Vec<String>,
    pub versions: Vec<VersionId>,
    pub icon_url: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub donation_urls: Option<Vec<DonationLink>>,
    pub gallery: Vec<GalleryItem>,
    pub color: Option<u32>,
    pub thread_id: ThreadId,
    pub monetization_status: MonetizationStatus,
}

impl LegacyProject {
    // Convert from a standard V3 project to a V2 project
    // Requires any queried versions to be passed in, to get access to certain version fields contained within.
    // - This can be any version, because the fields are ones that used to be on the project itself.
    // - Its conceivable that certain V3 projects that have many different ones may not have the same fields on all of them.
    // TODO: Should this return an error instead for v2 users?
    // It's safe to use a db version_item for this as the only info is side types, game versions, and loader fields (for loaders), which used to be public on project anyway.
    pub fn from(data: Project, versions_item: Option<version_item::QueryVersion>) -> Self {
        let mut client_side = LegacySideType::Unknown;
        let mut server_side = LegacySideType::Unknown;
        let mut game_versions = Vec::new();

        // V2 versions only have one project type- v3 versions can rarely have multiple.
        // We'll just use the first one.
        let mut project_type = data
            .project_types
            .first()
            .cloned()
            .unwrap_or("unknown".to_string());
        let mut loaders = data.loaders;

        if let Some(versions_item) = versions_item {
            client_side = versions_item
                .version_fields
                .iter()
                .find(|f| f.field_name == "client_side")
                .and_then(|f| {
                    Some(LegacySideType::from_string(
                        f.value.serialize_internal().as_str()?,
                    ))
                })
                .unwrap_or(LegacySideType::Unknown);
            server_side = versions_item
                .version_fields
                .iter()
                .find(|f| f.field_name == "server_side")
                .and_then(|f| {
                    Some(LegacySideType::from_string(
                        f.value.serialize_internal().as_str()?,
                    ))
                })
                .unwrap_or(LegacySideType::Unknown);
            game_versions = versions_item
                .version_fields
                .iter()
                .find(|f| f.field_name == "game_versions")
                .and_then(|f| MinecraftGameVersion::try_from_version_field(f).ok())
                .map(|v| v.into_iter().map(|v| v.version).collect())
                .unwrap_or(Vec::new());

            // - if loader is mrpack, this is a modpack
            // the loaders are whatever the corresponding loader fields are
            if versions_item.loaders == vec!["mrpack".to_string()] {
                project_type = "modpack".to_string();
                if let Some(mrpack_loaders) = versions_item
                    .version_fields
                    .iter()
                    .find(|f| f.field_name == "mrpack_loaders")
                {
                    loaders = mrpack_loaders.value.as_strings();
                }
            }
        }

        Self {
            id: data.id,
            slug: data.slug,
            project_type,
            team: data.team,
            organization: data.organization,
            title: data.title,
            description: data.description,
            body: data.body,
            body_url: data.body_url,
            published: data.published,
            updated: data.updated,
            approved: data.approved,
            queued: data.queued,
            status: data.status,
            requested_status: data.requested_status,
            moderator_message: data.moderator_message,
            license: data.license,
            downloads: data.downloads,
            followers: data.followers,
            categories: data.categories,
            additional_categories: data.additional_categories,
            loaders,
            versions: data.versions,
            icon_url: data.icon_url,
            issues_url: data.issues_url,
            source_url: data.source_url,
            wiki_url: data.wiki_url,
            discord_url: data.discord_url,
            donation_urls: data.donation_urls,
            gallery: data.gallery,
            color: data.color,
            thread_id: data.thread_id,
            monetization_status: data.monetization_status,
            client_side,
            server_side,
            game_versions,
        }
    }

    // Because from needs a version_item, this is a helper function to get many from one db query.
    pub async fn from_many<'a, E>(
        data: Vec<Project>,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let version_ids: Vec<_> = data
            .iter()
            .filter_map(|p| p.versions.first().map(|i| (*i).into()))
            .collect();
        let example_versions = version_item::Version::get_many(&version_ids, exec, redis).await?;
        let mut legacy_projects = Vec::new();
        for project in data {
            let version_item = example_versions
                .iter()
                .find(|v| v.inner.project_id == project.id.into())
                .cloned();
            let project = LegacyProject::from(project, version_item);
            legacy_projects.push(project);
        }
        Ok(legacy_projects)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum LegacySideType {
    Required,
    Optional,
    Unsupported,
    Unknown,
}

impl std::fmt::Display for LegacySideType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl LegacySideType {
    // These are constant, so this can remove unneccessary allocations (`to_string`)
    pub fn as_str(&self) -> &'static str {
        match self {
            LegacySideType::Required => "required",
            LegacySideType::Optional => "optional",
            LegacySideType::Unsupported => "unsupported",
            LegacySideType::Unknown => "unknown",
        }
    }

    pub fn from_string(string: &str) -> LegacySideType {
        match string {
            "required" => LegacySideType::Required,
            "optional" => LegacySideType::Optional,
            "unsupported" => LegacySideType::Unsupported,
            _ => LegacySideType::Unknown,
        }
    }
}

/// A specific version of a project
#[derive(Serialize, Deserialize, Clone)]
pub struct LegacyVersion {
    /// Relevant V2 fields- these were removed or modfified in V3,
    /// and are now part of the dynamic fields system
    /// A list of game versions this project supports
    pub game_versions: Vec<String>,

    /// A list of loaders this project supports (has a newtype struct)
    pub loaders: Vec<Loader>,

    // TODO: should we remove this? as this is a v3 field and tests for it should be isolated to v3
    // it allows us to keep tests that use this struct in common
    pub ordering: Option<i32>,

    pub id: VersionId,
    pub project_id: ProjectId,
    pub author_id: UserId,
    pub featured: bool,
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub changelog_url: Option<String>,
    pub date_published: DateTime<Utc>,
    pub downloads: u32,
    pub version_type: VersionType,
    pub status: VersionStatus,
    pub requested_status: Option<VersionStatus>,
    pub files: Vec<VersionFile>,
    pub dependencies: Vec<Dependency>,
}

impl From<Version> for LegacyVersion {
    fn from(data: Version) -> Self {
        let mut game_versions = Vec::new();
        if let Some(value) = data.fields.get("game_versions").and_then(|v| v.as_array()) {
            for gv in value {
                if let Some(game_version) = gv.as_str() {
                    game_versions.push(game_version.to_string());
                }
            }
        }

        // - if loader is mrpack, this is a modpack
        // the v2 loaders are whatever the corresponding loader fields are
        let mut loaders = data.loaders.into_iter().map(|l| l.0).collect::<Vec<_>>();
        if loaders == vec!["mrpack".to_string()] {
            if let Some((_, mrpack_loaders)) = data
                .fields
                .into_iter()
                .find(|(key, _)| key == "mrpack_loaders")
            {
                if let Ok(mrpack_loaders) = serde_json::from_value(mrpack_loaders) {
                    loaders = mrpack_loaders;
                }
            }
        }
        let loaders = loaders.into_iter().map(Loader).collect::<Vec<_>>();

        Self {
            id: data.id,
            project_id: data.project_id,
            author_id: data.author_id,
            featured: data.featured,
            name: data.name,
            version_number: data.version_number,
            changelog: data.changelog,
            changelog_url: data.changelog_url,
            date_published: data.date_published,
            downloads: data.downloads,
            version_type: data.version_type,
            status: data.status,
            requested_status: data.requested_status,
            files: data.files,
            dependencies: data.dependencies,
            game_versions,
            ordering: data.ordering,
            loaders,
        }
    }
}

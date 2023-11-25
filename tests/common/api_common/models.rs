use chrono::{DateTime, Utc};
use labrinth::models::{
    notifications::{NotificationAction, NotificationBody, NotificationId},
    organizations::OrganizationId,
    projects::{
        Dependency, DonationLink, GalleryItem, License, ModeratorMessage, MonetizationStatus,
        ProjectId, ProjectStatus, VersionFile, VersionId, VersionStatus, VersionType,
    },
    teams::{OrganizationPermissions, ProjectPermissions, TeamId},
    threads::ThreadId,
    users::{User, UserId},
};
use rust_decimal::Decimal;
use serde::Deserialize;

// Fields shared by every version of the API.
// No struct in here should have ANY field that
// is not present in *every* version of the API.

// These are used for common tests- tests that can be used on both V2 AND v3 of the API and have the same results.

// Any test that requires version-specific fields should have its own test that is not done for each version,
// as the environment generator for both uses common fields.

#[derive(Deserialize)]
pub struct CommonProject {
    // For example, for CommonProject, we do not include:
    // - game_versions (v2 only)
    // - loader_fields (v3 only)
    // - etc.
    // For any tests that require those fields, we make a separate test with separate API functions tht do not use Common models.
    pub id: ProjectId,
    pub slug: Option<String>,
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
#[derive(Deserialize, Clone)]
pub struct CommonVersion {
    pub id: VersionId,
    pub loaders: Vec<String>,
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

    // TODO: should ordering be in v2?
    pub ordering: Option<i32>,
}

#[derive(Deserialize)]
pub struct CommonImageData {
    pub filename: String,
    pub extension: String,
    pub icon: Vec<u8>,
}

#[derive(Deserialize)]
pub struct CommonLoaderData {
    pub icon: String,
    pub name: String,
    pub supported_project_types: Vec<String>,
}

#[derive(Deserialize)]
pub struct CommonCategoryData {
    pub icon: String,
    pub name: String,
    pub project_type: String,
    pub header: String,
}

/// A member of a team
#[derive(Deserialize)]
pub struct CommonTeamMember {
    pub team_id: TeamId,
    pub user: User,
    pub role: String,

    // TODO: Should these be removed from the Common?
    pub permissions: Option<ProjectPermissions>,
    pub organization_permissions: Option<OrganizationPermissions>,

    pub accepted: bool,
    pub payouts_split: Option<Decimal>,
    pub ordering: i64,
}

#[derive(Deserialize)]
pub struct CommonNotification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub read: bool,
    pub created: DateTime<Utc>,
    pub body: NotificationBody,

    // DEPRECATED: use body field instead
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub title: String,
    pub text: String,
    pub link: String,
    pub actions: Vec<NotificationAction>,
}

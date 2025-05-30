use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use labrinth::models::ids::{
    ImageId, NotificationId, OrganizationId, ProjectId, ReportId, TeamId,
    ThreadId, ThreadMessageId, VersionId,
};
use labrinth::{
    auth::AuthProvider,
    models::{
        projects::{
            Dependency, GalleryItem, License, ModeratorMessage,
            MonetizationStatus, ProjectStatus, VersionFile, VersionStatus,
            VersionType,
        },
        teams::ProjectPermissions,
        users::{Badges, Role, User, UserPayoutData},
    },
};
use rust_decimal::Decimal;
use serde::Deserialize;
// Fields shared by every version of the API.
// No struct in here should have ANY field that
// is not present in *every* version of the API.

// Exceptions are fields that *should* be changing across the API, and older versions
// should be unsupported on API version increase- for example, payouts related financial fields.

// These are used for common tests- tests that can be used on both V2 AND v3 of the API and have the same results.

// Any test that requires version-specific fields should have its own test that is not done for each version,
// as the environment generator for both uses common fields.

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CommonProject {
    // For example, for CommonProject, we do not include:
    // - game_versions (v2 only)
    // - loader_fields (v3 only)
    // - etc.
    // For any tests that require those fields, we make a separate test with separate API functions tht do not use Common models.
    pub id: ProjectId,
    pub slug: Option<String>,
    pub organization: Option<OrganizationId>,
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
    pub gallery: Vec<GalleryItem>,
    pub color: Option<u32>,
    pub thread_id: ThreadId,
    pub monetization_status: MonetizationStatus,
}
#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct CommonVersion {
    pub id: VersionId,
    pub loaders: Vec<String>,
    pub project_id: ProjectId,
    pub author_id: UserId,
    pub featured: bool,
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub date_published: DateTime<Utc>,
    pub downloads: u32,
    pub version_type: VersionType,
    pub status: VersionStatus,
    pub requested_status: Option<VersionStatus>,
    pub files: Vec<VersionFile>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CommonLoaderData {
    pub icon: String,
    pub name: String,
    pub supported_project_types: Vec<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CommonCategoryData {
    pub icon: String,
    pub name: String,
    pub project_type: String,
    pub header: String,
}

/// A member of a team
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CommonTeamMember {
    pub team_id: TeamId,
    pub user: User,
    pub role: String,

    pub permissions: Option<ProjectPermissions>,

    pub accepted: bool,
    pub payouts_split: Option<Decimal>,
    pub ordering: i64,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CommonNotification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub read: bool,
    pub created: DateTime<Utc>,
    // Body is absent as one of the variants differs
    pub text: String,
    pub link: String,
    pub actions: Vec<CommonNotificationAction>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CommonNotificationAction {
    pub action_route: (String, String),
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum CommonItemType {
    Project,
    Version,
    User,
    Unknown,
}

impl CommonItemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommonItemType::Project => "project",
            CommonItemType::Version => "version",
            CommonItemType::User => "user",
            CommonItemType::Unknown => "unknown",
        }
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CommonReport {
    pub id: ReportId,
    pub report_type: String,
    pub item_id: String,
    pub item_type: CommonItemType,
    pub reporter: UserId,
    pub body: String,
    pub created: DateTime<Utc>,
    pub closed: bool,
    pub thread_id: ThreadId,
}

#[derive(Deserialize)]
pub enum LegacyItemType {
    Project,
    Version,
    User,
    Unknown,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CommonThread {
    pub id: ThreadId,
    #[serde(rename = "type")]
    pub type_: CommonThreadType,
    pub project_id: Option<ProjectId>,
    pub report_id: Option<ReportId>,
    pub messages: Vec<CommonThreadMessage>,
    pub members: Vec<User>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CommonThreadMessage {
    pub id: ThreadMessageId,
    pub author_id: Option<UserId>,
    pub body: CommonMessageBody,
    pub created: DateTime<Utc>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub enum CommonMessageBody {
    Text {
        body: String,
        #[serde(default)]
        private: bool,
        replying_to: Option<ThreadMessageId>,
        #[serde(default)]
        associated_images: Vec<ImageId>,
    },
    StatusChange {
        new_status: ProjectStatus,
        old_status: ProjectStatus,
    },
    ThreadClosure,
    ThreadReopen,
    Deleted,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub enum CommonThreadType {
    Report,
    Project,
    DirectMessage,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CommonUser {
    pub id: UserId,
    pub username: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created: DateTime<Utc>,
    pub role: Role,
    pub badges: Badges,
    pub auth_providers: Option<Vec<AuthProvider>>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub has_password: Option<bool>,
    pub has_totp: Option<bool>,
    pub payout_data: Option<UserPayoutData>,
    pub github_id: Option<u64>,
}

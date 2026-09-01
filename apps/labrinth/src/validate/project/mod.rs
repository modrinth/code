use serde::{Deserialize, Serialize};

use crate::models::projects::{Project, Version};

mod description;
mod disclosures;
mod gallery;
mod icon;
mod license;
mod links;
mod moderation;
mod name;
mod permissions;
mod server_settings;
mod summary;
mod tags;
mod text;
mod versions;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    strum::EnumIter,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProjectNagKind {
    // Project name
    ProjectNameProfanity,
    ProjectNameNonStandardText,
    ProjectNameVersion,
    MinecraftTitleClause,

    // Project summary
    ProjectSummaryProfanity,
    ProjectSummaryNonStandardText,
    ProjectSummaryNonEnglish,
    ProjectSummaryMatchesTitle,
    SummaryTooShort,
    ProjectSummarySpam,
    SummarySpecialFormatting,
    ProjectSummaryLinks,

    // Project icon
    AddIcon,

    // Gallery
    GalleryTextProfanity,
    GalleryTextNonStandard,
    UploadGalleryImage,
    FeatureGalleryImage,

    // Project description
    ProjectDescriptionProfanity,
    ProjectDescriptionNonStandardText,
    ProjectDescriptionNonEnglish,
    AddDescription,
    DescriptionTooShort,
    ProjectDescriptionSpam,
    ProjectDescriptionBannedLink,
    LongHeaders,
    MissingAltText,

    // License
    SelectLicense,
    AddCustomLicenseDetails,
    InvalidLicenseUrl,

    // External links
    AddLinks,
    AddLinksServer,
    IdenticalLinks,
    VerifyExternalLinks,
    MisusedDiscordLink,
    BannedLinkUsage,
    GplLicenseSourceRequired,

    // Permissions
    ReviewPermissions,

    // Server settings
    SelectCountry,
    AllLanguages,
    AddJavaAddress,
    SelectCompatibility,
    TooManyLanguages,
    SelectLanguage,

    // Tags
    SelectTags,
    TooManyTags,
    TooManyTagsServer,
    MultipleResolutionTags,
    AllTagsSelected,

    // Versions
    UploadVersion,
    SelectEnvironment,

    // Disclosures
    CheckDisclosures,

    // Moderation
    ModeratorFeedback,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ProjectNagSeverity {
    Required,
    Warning,
    Suggestion,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
pub struct ProjectNag {
    pub kind: ProjectNagKind,
    pub severity: ProjectNagSeverity,
}

impl ProjectNag {
    pub(super) fn new(
        kind: ProjectNagKind,
        severity: ProjectNagSeverity,
    ) -> Self {
        Self { kind, severity }
    }
}

pub fn validate(project: &Project, versions: &[Version]) -> Vec<ProjectNag> {
    let mut nags = [
        name::validate,
        summary::validate,
        icon::validate,
        gallery::validate,
        description::validate,
        license::validate,
    ]
    .into_iter()
    .flat_map(|validate| validate(project))
    .collect::<Vec<_>>();

    nags.extend(links::validate(project, versions));
    nags.extend(permissions::validate(versions));
    nags.extend(versions::validate(project, versions));
    nags.extend(
        [
            server_settings::validate,
            tags::validate,
            disclosures::validate,
            moderation::validate,
        ]
        .into_iter()
        .flat_map(|validate| validate(project)),
    );
    nags
}

pub fn has_required_nags(project: &Project, versions: &[Version]) -> bool {
    validate(project, versions)
        .iter()
        .any(|nag| nag.severity == ProjectNagSeverity::Required)
}

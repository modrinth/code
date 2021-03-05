use super::ids::Base62Id;
use super::teams::TeamId;
use super::users::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The ID of a specific mod, encoded as base62 for usage in the API
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct ModId(pub u64);

/// The ID of a specific version of a mod
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct VersionId(pub u64);

/// A mod returned from the API
#[derive(Serialize, Deserialize)]
pub struct Mod {
    /// The ID of the mod, encoded as a base62 string.
    pub id: ModId,
    /// The slug of a mod, used for vanity URLs
    pub slug: Option<String>,
    /// The team of people that has ownership of this mod.
    pub team: TeamId,
    /// The title or name of the mod.
    pub title: String,
    /// A short description of the mod.
    pub description: String,
    /// A long form description of the mod.
    pub body: String,
    /// The link to the long description of the mod. (Deprecated), being replaced by `body`
    pub body_url: Option<String>,
    /// The date at which the mod was first published.
    pub published: DateTime<Utc>,
    /// The date at which the mod was first published.
    pub updated: DateTime<Utc>,
    /// The status of the mod
    pub status: ModStatus,
    /// The license of this mod
    pub license: License,

    /// The support range for the client mod
    pub client_side: SideType,
    /// The support range for the server mod
    pub server_side: SideType,

    /// The total number of downloads the mod has had.
    pub downloads: u32,
    /// The total number of followers this mod has accumulated
    pub followers: u32,

    /// A list of the categories that the mod is in.
    pub categories: Vec<String>,
    /// A list of ids for versions of the mod.
    pub versions: Vec<VersionId>,
    /// The URL of the icon of the mod
    pub icon_url: Option<String>,
    /// An optional link to where to submit bugs or issues with the mod.
    pub issues_url: Option<String>,
    /// An optional link to the source code for the mod.
    pub source_url: Option<String>,
    /// An optional link to the mod's wiki page or other relevant information.
    pub wiki_url: Option<String>,
    /// An optional link to the mod's discord
    pub discord_url: Option<String>,
    /// An optional list of all donation links the mod has
    pub donation_urls: Option<Vec<DonationLink>>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum SideType {
    Required,
    Optional,
    Unsupported,
    Unknown,
}

impl std::fmt::Display for SideType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl SideType {
    // These are constant, so this can remove unneccessary allocations (`to_string`)
    pub fn as_str(&self) -> &'static str {
        match self {
            SideType::Required => "required",
            SideType::Optional => "optional",
            SideType::Unsupported => "unsupported",
            SideType::Unknown => "unknown",
        }
    }

    pub fn from_str(string: &str) -> SideType {
        match string {
            "required" => SideType::Required,
            "optional" => SideType::Optional,
            "unsupported" => SideType::Unsupported,
            _ => SideType::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct License {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DonationLink {
    pub id: String,
    pub platform: String,
    pub url: String,
}

/// A status decides the visbility of a mod in search, URLs, and the whole site itself.
/// Approved - Mod is displayed on search, and accessible by URL
/// Rejected - Mod is not displayed on search, and not accessible by URL (Temporary state, mod can reapply)
/// Draft - Mod is not displayed on search, and not accessible by URL
/// Unlisted - Mod is not displayed on search, but accessible by URL
/// Processing - Mod is not displayed on search, and not accessible by URL (Temporary state, mod under review)
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModStatus {
    Approved,
    Rejected,
    Draft,
    Unlisted,
    Processing,
    Unknown,
}

impl std::fmt::Display for ModStatus {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl ModStatus {
    pub fn from_str(string: &str) -> ModStatus {
        match string {
            "processing" => ModStatus::Processing,
            "rejected" => ModStatus::Rejected,
            "approved" => ModStatus::Approved,
            "draft" => ModStatus::Draft,
            "unlisted" => ModStatus::Unlisted,
            _ => ModStatus::Unknown,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            ModStatus::Approved => "approved",
            ModStatus::Rejected => "rejected",
            ModStatus::Draft => "draft",
            ModStatus::Unlisted => "unlisted",
            ModStatus::Processing => "processing",
            ModStatus::Unknown => "unknown",
        }
    }

    pub fn is_hidden(&self) -> bool {
        match self {
            ModStatus::Approved => false,
            ModStatus::Rejected => true,
            ModStatus::Draft => true,
            ModStatus::Unlisted => false,
            ModStatus::Processing => true,
            ModStatus::Unknown => true,
        }
    }

    pub fn is_searchable(&self) -> bool {
        matches!(self, ModStatus::Approved)
    }
}

/// A specific version of a mod
#[derive(Serialize, Deserialize)]
pub struct Version {
    /// The ID of the version, encoded as a base62 string.
    pub id: VersionId,
    /// The ID of the mod this version is for.
    pub mod_id: ModId,
    /// The ID of the author who published this version
    pub author_id: UserId,
    /// Whether the version is featured or not
    pub featured: bool,

    /// The name of this version
    pub name: String,
    /// The version number. Ideally will follow semantic versioning
    pub version_number: String,
    /// The changelog for this version of the mod.
    pub changelog: String,
    /// A link to the changelog for this version of the mod. (Deprecated), being replaced by `changelog`
    pub changelog_url: Option<String>,
    /// The date that this version was published.
    pub date_published: DateTime<Utc>,
    /// The number of downloads this specific version has had.
    pub downloads: u32,
    /// The type of the release - `Alpha`, `Beta`, or `Release`.
    pub version_type: VersionType,

    /// A list of files available for download for this version.
    pub files: Vec<VersionFile>,
    /// A list of mods that this version depends on.
    pub dependencies: Vec<Dependency>,
    /// A list of versions of Minecraft that this version of the mod supports.
    pub game_versions: Vec<GameVersion>,
    /// The loaders that this version works on
    pub loaders: Vec<ModLoader>,
}

/// A single mod file, with a url for the file and the file's hash
#[derive(Serialize, Deserialize)]
pub struct VersionFile {
    /// A map of hashes of the file.  The key is the hashing algorithm
    /// and the value is the string version of the hash.
    pub hashes: std::collections::HashMap<String, String>,
    /// A direct link to the file for downloading it.
    pub url: String,
    /// The filename of the file.
    pub filename: String,
    /// Whether the file is the primary file of a version
    pub primary: bool,
}

/// A dependency which describes what versions are required, break support, or are optional to the
/// version's functionality
#[derive(Serialize, Deserialize, Clone)]
pub struct Dependency {
    /// The filename of the file.
    pub version_id: VersionId,
    /// Whether the file is the primary file of a version
    pub dependency_type: DependencyType,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum VersionType {
    Release,
    Beta,
    Alpha,
}

impl std::fmt::Display for VersionType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VersionType::Release => write!(fmt, "release"),
            VersionType::Beta => write!(fmt, "beta"),
            VersionType::Alpha => write!(fmt, "alpha"),
        }
    }
}

impl VersionType {
    // These are constant, so this can remove unneccessary allocations (`to_string`)
    pub fn as_str(&self) -> &'static str {
        match self {
            VersionType::Release => "release",
            VersionType::Beta => "beta",
            VersionType::Alpha => "alpha",
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DependencyType::Required => write!(fmt, "required"),
            DependencyType::Optional => write!(fmt, "optional"),
            DependencyType::Incompatible => write!(fmt, "incompatible"),
        }
    }
}

impl DependencyType {
    // These are constant, so this can remove unneccessary allocations (`to_string`)
    pub fn as_str(&self) -> &'static str {
        match self {
            DependencyType::Required => "required",
            DependencyType::Optional => "optional",
            DependencyType::Incompatible => "incompatible",
        }
    }

    pub fn from_str(string: &str) -> DependencyType {
        match string {
            "required" => DependencyType::Required,
            "optional" => DependencyType::Optional,
            "incompatible" => DependencyType::Incompatible,
            _ => DependencyType::Required,
        }
    }
}

/// A specific version of Minecraft
#[derive(Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct GameVersion(pub String);

/// A mod loader
#[derive(Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct ModLoader(pub String);

// These fields must always succeed parsing; deserialize errors aren't
// processed correctly (don't return JSON errors)
#[derive(Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: Option<String>,
    /// Must match a json 2 deep array of strings `[["categories:misc"]]`
    // TODO: We may want to have a better representation of this, so that
    // we are less likely to break backwards compatibility
    pub facets: Option<String>,
    pub filters: Option<String>,
    pub version: Option<String>,
    pub offset: Option<String>,
    pub index: Option<String>,
    pub limit: Option<String>,
}

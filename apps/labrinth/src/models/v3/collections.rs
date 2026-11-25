use super::ids::ProjectId;
use crate::bitflags_serde_impl;
use crate::database;
use crate::models::ids::CollectionId;
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    /// Staff-assigned collection flags.
    ///
    /// Stored and exposed as a bitfield. `1` is featured, `2` is potw.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CollectionFlags: i64 {
        const FEATURED = 1 << 0;
        const POTW = 1 << 1;
    }
}

impl Default for CollectionFlags {
    fn default() -> Self {
        Self::empty()
    }
}

impl CollectionFlags {
    pub fn from_db(bits: i64) -> Self {
        Self::from_bits_truncate(bits)
    }

    pub fn to_db(self) -> i64 {
        self.bits()
    }
}

impl utoipa::PartialSchema for CollectionFlags {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        i64::schema()
    }
}

impl utoipa::ToSchema for CollectionFlags {
    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        i64::schemas(schemas);
    }
}

bitflags_serde_impl!(CollectionFlags, i64);

/// A `listed` collection with the `featured` flag is public for project metadata.
pub fn is_public_featured(
    status: CollectionStatus,
    flags: CollectionFlags,
) -> bool {
    status.is_searchable() && flags.contains(CollectionFlags::FEATURED)
}

/// A collection returned from the API
#[derive(Serialize, Deserialize, Clone)]
pub struct Collection {
    /// The ID of the collection, encoded as a base62 string.
    pub id: CollectionId,
    /// The person that has ownership of this collection.
    pub user: UserId,
    /// The title or name of the collection.
    pub name: String,
    /// A short description of the collection.
    pub description: Option<String>,

    /// An icon URL for the collection.
    pub icon_url: Option<String>,
    /// The raw icon URL for the collection.
    pub raw_icon_url: Option<String>,
    /// Color of the collection.
    pub color: Option<u32>,

    /// The status of the collection (eg: whether collection is public or not)
    pub status: CollectionStatus,

    /// Staff-assigned flags. Empty unless set by a moderator.
    #[serde(default)]
    pub collection_flags: CollectionFlags,

    /// The date at which the collection was first published.
    pub created: DateTime<Utc>,

    /// The date at which the collection was updated.
    pub updated: DateTime<Utc>,

    /// The number of users following this collection.
    pub follows: u32,

    /// A list of ProjectIds that are in this collection.
    pub projects: Vec<ProjectId>,
}

impl From<database::models::DBCollection> for Collection {
    fn from(c: database::models::DBCollection) -> Self {
        Self {
            id: c.id.into(),
            user: c.user_id.into(),
            created: c.created,
            name: c.name,
            description: c.description,
            updated: c.updated,
            projects: c.projects.into_iter().map(|x| x.into()).collect(),
            icon_url: c.icon_url,
            raw_icon_url: c.raw_icon_url,
            color: c.color,
            status: c.status,
            collection_flags: c.flags,
            follows: c.follows as u32,
        }
    }
}

/// A status decides the visibility of a collection in search, URLs, and the whole site itself.
/// Listed - collection is displayed on search, and accessible by URL (for if/when search is implemented for collections)
/// Unlisted - collection is not displayed on search, but accessible by URL
/// Rejected - collection is disabled
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CollectionStatus {
    Listed,
    Unlisted,
    Private,
    Rejected,
    Unknown,
}

impl std::fmt::Display for CollectionStatus {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl CollectionStatus {
    pub fn from_string(string: &str) -> CollectionStatus {
        match string {
            "listed" => CollectionStatus::Listed,
            "unlisted" => CollectionStatus::Unlisted,
            "private" => CollectionStatus::Private,
            "rejected" => CollectionStatus::Rejected,
            _ => CollectionStatus::Unknown,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            CollectionStatus::Listed => "listed",
            CollectionStatus::Unlisted => "unlisted",
            CollectionStatus::Private => "private",
            CollectionStatus::Rejected => "rejected",
            CollectionStatus::Unknown => "unknown",
        }
    }

    // Collection pages + info cannot be viewed
    pub fn is_hidden(&self) -> bool {
        match self {
            CollectionStatus::Rejected => true,
            CollectionStatus::Private => true,
            CollectionStatus::Listed => false,
            CollectionStatus::Unlisted => false,
            CollectionStatus::Unknown => false,
        }
    }

    // Collection can be displayed in on user page
    pub fn is_searchable(&self) -> bool {
        matches!(self, CollectionStatus::Listed)
    }

    pub fn is_approved(&self) -> bool {
        match self {
            CollectionStatus::Listed => true,
            CollectionStatus::Private => true,
            CollectionStatus::Unlisted => true,
            CollectionStatus::Rejected => false,
            CollectionStatus::Unknown => false,
        }
    }

    pub fn can_be_requested(&self) -> bool {
        match self {
            CollectionStatus::Listed => true,
            CollectionStatus::Private => true,
            CollectionStatus::Unlisted => true,
            CollectionStatus::Rejected => false,
            CollectionStatus::Unknown => false,
        }
    }
}

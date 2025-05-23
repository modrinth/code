use super::ids::ProjectId;
use crate::database;
use crate::models::ids::CollectionId;
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    /// Color of the collection.
    pub color: Option<u32>,

    /// The status of the collectin (eg: whether collection is public or not)
    pub status: CollectionStatus,

    /// The date at which the collection was first published.
    pub created: DateTime<Utc>,

    /// The date at which the collection was updated.
    pub updated: DateTime<Utc>,

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
            color: c.color,
            status: c.status,
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

    // Project pages + info cannot be viewed
    pub fn is_hidden(&self) -> bool {
        match self {
            CollectionStatus::Rejected => true,
            CollectionStatus::Private => true,
            CollectionStatus::Listed => false,
            CollectionStatus::Unlisted => false,
            CollectionStatus::Unknown => false,
        }
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

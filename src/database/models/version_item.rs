use crate::database::models::Item;
use crate::database::Result;
use bson::{Bson, Document};
use serde::{Deserialize, Serialize};

//TODO: Files should probably be moved to their own table
#[derive(Deserialize, Serialize)]
pub struct Version {
    ///The unqiue VersionId of this version
    pub version_id: i32,
    /// The ModId of the mod that this version belongs to
    pub mod_id: i32,
    pub name: String,
    pub number: String,
    pub changelog_url: Option<String>,
    pub date_published: String,
    pub downloads: i32,
    pub files: Vec<VersionFile>,
    pub dependencies: Vec<i32>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub version_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct VersionFile {
    pub game_versions: Vec<String>,
    pub hashes: Vec<FileHash>,
    pub url: String,
}

/// A hash of a mod's file
#[derive(Serialize, Deserialize)]
pub struct FileHash {
    pub algorithm: String,
    pub hash: String,
}

impl Item for Version {
    fn get_collection() -> &'static str {
        "versions"
    }

    fn from_doc(elem: Document) -> Result<Box<Version>> {
        let version: Version = bson::from_bson(Bson::from(elem))?;
        Ok(Box::from(version))
    }
}

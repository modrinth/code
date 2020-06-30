use crate::database::models::Item;
use crate::database::Result;
use bson::{Bson, Document};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Version {
    pub version_id: i32,
    pub mod_id: i32,
    pub title: String,
    pub changelog_path: String,
    pub files_path: Vec<String>,
    pub date_published: String,
    pub author: String,
    pub downloads: i32,
    pub dependencies: Vec<String>,
    pub game_versions: Vec<String>,
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

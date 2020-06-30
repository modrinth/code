use crate::database::models::Item;
use crate::database::Result;
use bson::{Bson, Document};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Mod {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub published: String,
    pub author: String,
    pub downloads: i32,
    pub categories: Vec<String>,
    pub body_path: String,
    pub icon_path: String,
}
impl Item for Mod {
    fn get_collection() -> &'static str {
        "mods"
    }

    fn from_doc(elem: Document) -> Result<Box<Mod>> {
        let result: Mod = bson::from_bson(Bson::from(elem))?;
        Ok(Box::from(result))
    }
}

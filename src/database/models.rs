use serde::{Serialize, Deserialize};

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

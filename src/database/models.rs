use chrono::NaiveDate;

#[derive(Queryable)]
pub struct Mod {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub published: NaiveDate,
    pub author: String,
    pub downloads: i32,
    pub categories: Vec<String>,
    pub body_path: String,
    pub icon_path: String,
}

#[derive(Queryable)]
pub struct Version {
    pub version_id: i32,
    pub mod_id: i32,
    pub title: String,
    pub changelog_path: String,
    pub files_path: Vec<String>,
    pub date_published: NaiveDate,
    pub author: String,
    pub downloads: i32,
    pub dependencies: Vec<String>,
    pub game_versions: Vec<String>,
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct ModrinthModMetadata {
    pub name: String,
    pub version_number: String,
    pub files: Vec<ModrinthVersionFile>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct ModrinthVersionFile {
    pub hashes: HashMap<String, String>,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i32,
}
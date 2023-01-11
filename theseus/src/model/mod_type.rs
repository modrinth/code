use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModType {
    Modrinth,
    Jar,
}

impl std::fmt::Display for ModType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            &Self::Modrinth => "Modrinth",
            &Self::Jar => "Jar",
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
// Represent a mod downloaded from Modrinth.
pub struct ModrinthMod {
    pub mod_id: String,
    pub version_number: String,
    pub jar_loaded_mod: JARLoadedMod,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
// Represent a mod loaded from an external JAR.
pub struct JARLoadedMod {
    pub file_hash: String,
    pub absolute_path: String,
    //date_added: DateTime<TimeZone>,
    pub mod_name: String,
    pub description: String,
}
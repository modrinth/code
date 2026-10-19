use std::collections::HashMap;

use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::model;

pub const CATALOG_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";

/// Forge loader manifest for all game versions, mapped to all loader versions
/// available for that game version.
///
/// Available at [`CATALOG_URL`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog(
    pub HashMap<GameVersionName, Vec<model::ForgelikeInstallerName>>,
);

/// Name of a Minecraft game version.
#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct GameVersionName(pub String);

use crate::state::ModLoader;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstanceInstallTarget {
    pub game_version: String,
    pub loader: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct InstanceInstallCandidate {
    pub id: String,
    pub name: String,
    pub icon_path: Option<String>,
    pub game_version: String,
    pub loader: ModLoader,
    pub installed: bool,
    pub compatible: bool,
}

use derive_more::Display;
use serde::{Deserialize, Serialize};
use toasty::Embed;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Embed,
)]
pub enum MinecraftLoader {
    Fabric,
    Forge,
    Neoforge,
    Quilt,
}

#[derive(
    Debug,
    Display,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Embed,
)]
pub struct MinecraftVersionName(pub String);

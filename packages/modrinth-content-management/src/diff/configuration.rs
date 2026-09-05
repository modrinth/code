use serde::{Deserialize, Serialize};

use crate::diff::Change;

/// The linked modpack, Minecraft version, and loader settings to compare.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContentSetConfiguration {
    pub modpack_version_id: Option<String>,
    pub game_version: String,
    pub loader: LoaderReference,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LoaderReference {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "change", rename_all = "snake_case")]
pub enum ConfigurationDiff {
    Modpack(Change<String>),
    GameVersion(Change<String>),
    Loader(Change<LoaderReference>),
}

/// Lists changes to the linked modpack, Minecraft version, and mod loader.
///
/// Some sources use an empty string where others use `None` for an unset
/// modpack ID or loader version. Treat them the same so this difference alone
/// does not show an update.
pub fn diff_configuration(
    before: &ContentSetConfiguration,
    after: &ContentSetConfiguration,
) -> Vec<ConfigurationDiff> {
    let mut changes = Vec::new();
    if let Some(change) = Change::between(
        before
            .modpack_version_id
            .as_ref()
            .filter(|id| !id.is_empty()),
        after
            .modpack_version_id
            .as_ref()
            .filter(|id| !id.is_empty()),
    ) {
        changes.push(ConfigurationDiff::Modpack(change));
    }
    if let Some(change) =
        Change::between(Some(&before.game_version), Some(&after.game_version))
    {
        changes.push(ConfigurationDiff::GameVersion(change));
    }
    let normalize_loader = |loader: &LoaderReference| LoaderReference {
        name: loader.name.to_lowercase(),
        version: loader.version.clone().filter(|version| !version.is_empty()),
    };
    if let Some(change) = Change::between(
        Some(&normalize_loader(&before.loader)),
        Some(&normalize_loader(&after.loader)),
    ) {
        changes.push(ConfigurationDiff::Loader(change));
    }
    changes
}

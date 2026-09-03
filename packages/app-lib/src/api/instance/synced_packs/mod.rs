mod operations;
mod reconciliation;
mod selection;
mod selection_compatibility;
mod storage;

pub use operations::{
    desync_pack, get_pack_sync_preview, list_synced_packs, remove_synced_pack,
    set_synced_pack_enabled, sync_pack, upload_synced_pack,
};
pub(super) use operations::{seed_from_instance, sync_new_pack};
pub(crate) use reconciliation::reconcile_after_change;
pub(super) use reconciliation::{
    capture_resource_pack_selection_change, decorate_content, detach,
    prepare_instance_update, reconcile, reconcile_after_content_change,
};

use crate::state::{
    ContentItem, InstanceMetadata, ProjectType, SyncedOption, Version,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Default, Serialize, Deserialize)]
struct PackLibrary {
    packs: BTreeMap<String, SyncedPack>,
    instances: BTreeMap<String, BTreeMap<String, PackPlacement>>,
    #[serde(default)]
    resource_pack_order: Vec<String>,
    #[serde(default)]
    resource_pack_observations: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    resource_pack_incompatible_observations: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SyncedPack {
    item: ContentItem,
    sha1: String,
    game_versions: Vec<String>,
    #[serde(default)]
    selected: Option<bool>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct PackPlacement {
    path: String,
    sha1: String,
    enabled: bool,
    excluded: bool,
    suspended: bool,
    pending: bool,
    content_set_id: String,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    resource_pack_selection_pending: bool,
    /// The pack path used by the last reconciled resourcePacks selection.
    #[serde(default)]
    resource_pack_selection_path: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct PackSyncTarget {
    pub instance_id: String,
    pub name: String,
    pub game_version: String,
    pub compatible: bool,
    pub participating: bool,
}

#[derive(Clone, Serialize)]
pub struct PackSyncPreview {
    pub pack: ContentItem,
    pub instances: Vec<PackSyncTarget>,
}

fn pack_option(project_type: ProjectType) -> crate::Result<SyncedOption> {
    let option = match project_type {
        ProjectType::ResourcePack => SyncedOption::ResourcePacks,
        ProjectType::DataPack => SyncedOption::DataPacks,
        _ => {
            return Err(crate::ErrorKind::InputError(
                "Only resource packs and data packs can be synced.".to_string(),
            )
            .into());
        }
    };
    if !option.is_available() {
        return Err(crate::ErrorKind::InputError(
            "Data pack syncing is currently disabled.".to_string(),
        )
        .into());
    }
    Ok(option)
}

fn version_compatible(
    pack: &SyncedPack,
    version: &Version,
    metadata: &InstanceMetadata,
) -> bool {
    pack.item
        .project
        .as_ref()
        .is_some_and(|project| project.id == version.project_id)
        && version
            .game_versions
            .contains(&metadata.applied_content_set.game_version)
        && version.loaders.iter().any(|loader| {
            pack.item
                .project_type
                .get_loaders()
                .contains(&loader.as_str())
        })
}

fn same_path(left: &str, right: &str) -> bool {
    left.trim_end_matches(".disabled") == right.trim_end_matches(".disabled")
}

fn pack_path(pack: &SyncedPack, file_name: &str) -> String {
    format!(
        "{}/{}{}",
        pack.item.project_type.get_folder(),
        file_name,
        if pack.item.enabled { "" } else { ".disabled" },
    )
}

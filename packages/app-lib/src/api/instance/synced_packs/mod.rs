mod operations;
mod reconciliation;
mod storage;

pub use operations::{
    desync_pack, get_pack_sync_preview, list_synced_packs, remove_synced_pack,
    set_synced_pack_enabled, sync_pack, upload_synced_pack,
};
pub(super) use operations::{seed_from_instance, sync_new_pack};
pub(crate) use reconciliation::reconcile_after_change;
pub(super) use reconciliation::{
    decorate_content, detach, reconcile, reconcile_after_content_change,
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
}

#[derive(Clone, Serialize, Deserialize)]
struct SyncedPack {
    item: ContentItem,
    sha1: String,
    game_versions: Vec<String>,
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
    match project_type {
        ProjectType::ResourcePack => Ok(SyncedOption::ResourcePacks),
        ProjectType::DataPack => Ok(SyncedOption::DataPacks),
        _ => Err(crate::ErrorKind::InputError(
            "Only resource packs and data packs can be synced.".to_string(),
        )
        .into()),
    }
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

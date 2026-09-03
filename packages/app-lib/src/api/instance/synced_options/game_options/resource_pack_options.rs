//! Reads and merges Minecraft's active resource-pack list.

use super::options_file::{input_error, options_path, read_document};
use crate::state::{InstanceMetadata, State};
use crate::util::io;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::api::instance) struct ResourcePackOptions {
    pub entries: Vec<String>,
    pub incompatible: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::api::instance) enum ResourcePackOptionsUpdate {
    Deferred,
    Applied(ResourcePackOptions),
}

fn parse_entries(raw: &str) -> crate::Result<Vec<String>> {
    serde_json::from_str(raw).map_err(|_| {
        input_error("options.txt contains an invalid resourcePacks value")
    })
}

pub(in crate::api::instance) fn merge_resource_pack_order(
    entries: &[String],
    managed: &BTreeSet<String>,
    selected: &[String],
) -> Vec<String> {
    let mut merged = Vec::new();
    let mut remaining = selected.iter();
    let mut seen = BTreeSet::new();
    for entry in entries {
        if !managed.contains(entry) {
            merged.push(entry.clone());
        } else if selected.contains(entry)
            && seen.insert(entry)
            && let Some(next) = remaining.next()
        {
            merged.push(next.clone());
        }
    }
    merged.extend(remaining.cloned());
    merged
}

pub(in crate::api::instance) async fn read_resource_pack_entries(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<Option<ResourcePackOptions>> {
    let path = options_path(metadata, state);
    if !path.exists() {
        return Ok(None);
    }
    let (document, _) = read_document(&path).await?;
    let Some(raw) = document.value("resourcePacks") else {
        return Ok(None);
    };
    Ok(Some(ResourcePackOptions {
        entries: parse_entries(raw)?,
        incompatible: document
            .value("incompatibleResourcePacks")
            .map(parse_entries)
            .transpose()?
            .unwrap_or_default(),
    }))
}

pub(in crate::api::instance) async fn merge_resource_pack_entries(
    metadata: &InstanceMetadata,
    managed: &BTreeSet<String>,
    selected: &[String],
    state: &State,
) -> crate::Result<ResourcePackOptionsUpdate> {
    let path = options_path(metadata, state);
    if !path.exists() {
        return Ok(ResourcePackOptionsUpdate::Deferred);
    }
    let (mut document, input_bytes) = read_document(&path).await?;
    let Some(raw) = document.value("resourcePacks") else {
        return Ok(ResourcePackOptionsUpdate::Deferred);
    };
    let original = parse_entries(raw)?;
    let entries = merge_resource_pack_order(&original, managed, selected);
    let options = ResourcePackOptions {
        entries,
        incompatible: document
            .value("incompatibleResourcePacks")
            .map(parse_entries)
            .transpose()?
            .unwrap_or_default(),
    };
    if options.entries == original {
        return Ok(ResourcePackOptionsUpdate::Applied(options));
    }
    let raw = serde_json::to_string(&options.entries)?;
    document.set("resourcePacks", &raw, false)?;
    let output_bytes = document.serialize()?;
    if output_bytes == input_bytes {
        return Ok(ResourcePackOptionsUpdate::Applied(options));
    }
    io::write(path, output_bytes).await?;
    Ok(ResourcePackOptionsUpdate::Applied(options))
}

use super::HOTBAR_FILE;
use super::files::{
    CheckpointStatus, begin_checkpoint, checkpoint, ensure_link,
    finish_checkpoint, instance_dir, nbt_from_bytes, nbt_to_bytes,
    read_nbt_file, safe_instance_id, sha1_bytes, sha1_file,
};
use super::orchestration::{
    backup_bytes, create_synced_directories, option_effective,
    seed_from_instance, synced_options_path,
};
use crate::state::{InstanceMetadata, SyncedOption};
use crate::util::io;
use crate::{ErrorKind, State};
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use std::path::PathBuf;

const COMPONENTS_DATA_VERSION_FLOOR: i32 = 3837;
pub(super) const HOTBAR_SCHEMA_VERSION: i64 = 2;

pub(super) struct HotbarState {
    pub schema_version: i64,
    pub revision: i64,
    pub nbt: NbtCompound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HotbarFamily {
    Legacy,
    Components,
}

impl HotbarFamily {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Legacy => "legacy",
            Self::Components => "components",
        }
    }

    fn other(self) -> Self {
        match self {
            Self::Legacy => Self::Components,
            Self::Components => Self::Legacy,
        }
    }
}

pub(super) async fn hotbar_family(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<HotbarFamily> {
    let (manifest, version_index) =
        crate::launcher::resolve_minecraft_manifest(
            &metadata.applied_content_set.game_version,
            state,
        )
        .await?;
    let cutoff = manifest
        .versions
        .iter()
        .find(|item| item.id == "1.20.5")
        .ok_or_else(|| {
            ErrorKind::LauncherError(
                "Minecraft 1.20.5 is missing from the version manifest"
                    .to_string(),
            )
        })?;
    Ok(
        if manifest.versions[version_index].release_time >= cutoff.release_time
        {
            HotbarFamily::Components
        } else {
            HotbarFamily::Legacy
        },
    )
}

pub(super) async fn backup_shared_hotbars(state: &State) -> crate::Result<()> {
    let sync_state = read_hotbar_state(state).await?;
    let legacy = hotbar_family_root(&sync_state.nbt, HotbarFamily::Legacy);
    let components =
        hotbar_family_root(&sync_state.nbt, HotbarFamily::Components);
    backup_bytes(
        "shared",
        "hotbar-legacy.nbt",
        &nbt_to_bytes(&legacy)?,
        state,
    )
    .await?;
    backup_bytes(
        "shared",
        "hotbar-components.nbt",
        &nbt_to_bytes(&components)?,
        state,
    )
    .await?;
    backup_bytes(
        "shared",
        "hotbars-state.nbt",
        &nbt_to_bytes(&sync_state.nbt)?,
        state,
    )
    .await
}

pub(super) async fn ensure_hotbar(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    create_synced_directories(state).await?;
    if !hotbar_state_exists(state).await? {
        seed_from_instance(metadata, SyncedOption::CreativeHotbars, state)
            .await?;
    }
    write_hotbar_projection(metadata, state).await
}

pub(super) async fn reconcile_hotbar(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    if !option_effective(metadata, SyncedOption::CreativeHotbars, state).await?
    {
        return Ok(());
    }
    let local = instance_dir(metadata, state).join(HOTBAR_FILE);
    if !local.exists() {
        return ensure_hotbar(metadata, state).await;
    }
    let family = hotbar_family(metadata, state).await?;
    let current_checkpoint = checkpoint(
        &metadata.instance.id,
        SyncedOption::CreativeHotbars,
        family.as_str(),
        state,
    )
    .await?;
    if current_checkpoint
        .as_ref()
        .is_some_and(|value| value.status == CheckpointStatus::Pending)
    {
        return write_hotbar_projection(metadata, state).await;
    }
    let actual = sha1_file(&local).await?;
    let mut sync_state = read_hotbar_state(state).await?;
    if current_checkpoint
        .as_ref()
        .map(|value| value.expected_sha1.as_str())
        == Some(actual.as_str())
    {
        if current_checkpoint
            .as_ref()
            .is_some_and(|value| value.source_revision == sync_state.revision)
        {
            return Ok(());
        }
        return write_hotbar_projection(metadata, state).await;
    }

    let (changed_family, checkpoint) = if current_checkpoint.is_none() {
        let previous_family = family.other();
        let previous_checkpoint = checkpoint(
            &metadata.instance.id,
            SyncedOption::CreativeHotbars,
            previous_family.as_str(),
            state,
        )
        .await?;
        if let Some(previous_checkpoint) = previous_checkpoint {
            if previous_checkpoint.status == CheckpointStatus::Pending
                || previous_checkpoint.expected_sha1 == actual
            {
                return write_hotbar_projection(metadata, state).await;
            }
            (previous_family, Some(previous_checkpoint))
        } else {
            (family, None)
        }
    } else {
        (family, current_checkpoint)
    };

    let changed = read_nbt_file(&local).await?;
    let merge_base = checkpoint
        .and_then(|value| value.merge_base)
        .map(nbt_from_bytes)
        .transpose()?
        .unwrap_or_else(|| hotbar_family_root(&sync_state.nbt, changed_family));
    if merge_hotbar_family(
        &mut sync_state.nbt,
        changed_family,
        &merge_base,
        &changed,
    ) {
        increment_hotbar_revision(&mut sync_state);
        write_hotbar_state(state, &sync_state).await?;
    }
    regenerate_hotbars(state).await
}

pub(super) async fn instance_hotbars_differ_from_synced(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<bool> {
    let local_path = instance_dir(metadata, state).join(HOTBAR_FILE);
    if !local_path.exists() {
        return Ok(false);
    }
    let local = read_nbt_file(&local_path).await?;
    let family = hotbar_family(metadata, state).await?;
    let synced =
        hotbar_family_root(&read_hotbar_state(state).await?.nbt, family);
    Ok((0..81)
        .any(|slot| hotbar_slot(&local, slot) != hotbar_slot(&synced, slot)))
}

pub(super) fn merge_hotbar_family(
    state: &mut NbtCompound,
    family: HotbarFamily,
    merge_base: &NbtCompound,
    changed: &NbtCompound,
) -> bool {
    let family_key = family_state_key(family);
    let mut current = state
        .get::<_, &NbtCompound>(family_key)
        .ok()
        .cloned()
        .unwrap_or_else(empty_hotbar_root);
    let other_family = match family {
        HotbarFamily::Legacy => HotbarFamily::Components,
        HotbarFamily::Components => HotbarFamily::Legacy,
    };
    let other_key = family_state_key(other_family);
    let other = state.get::<_, &NbtCompound>(other_key).ok().cloned();
    let seed_components_with_legacy =
        other.is_none() && family == HotbarFamily::Legacy;
    let mut other_root = if seed_components_with_legacy {
        (*changed).clone()
    } else {
        other.unwrap_or_else(empty_hotbar_root)
    };
    let mut revisions = state
        .get::<_, &NbtCompound>("Revisions")
        .ok()
        .cloned()
        .unwrap_or_default();
    let family_versions_key = family_data_versions_key(family);
    let other_versions_key = family_data_versions_key(other_family);
    let mut family_versions = state
        .get::<_, &NbtCompound>(family_versions_key)
        .ok()
        .cloned()
        .unwrap_or_default();
    let mut other_versions = state
        .get::<_, &NbtCompound>(other_versions_key)
        .ok()
        .cloned()
        .unwrap_or_default();
    let writer_data_version = hotbar_data_version(changed).max(match family {
        HotbarFamily::Legacy => 1,
        HotbarFamily::Components => COMPONENTS_DATA_VERSION_FLOOR,
    });
    let current_data_version = hotbar_data_version(&current);
    let other_data_version = hotbar_data_version(&other_root);
    let mut changed_any = false;

    for slot in 0..81 {
        let old_slot = hotbar_slot(merge_base, slot);
        let new_slot = hotbar_slot(changed, slot);
        if old_slot == new_slot {
            continue;
        }
        let current_origin = family_versions
            .get::<_, i32>(&slot.to_string())
            .unwrap_or(current_data_version);
        if writer_data_version > 0
            && current_origin > 0
            && writer_data_version < current_origin
        {
            continue;
        }
        set_hotbar_slot_optional(&mut current, slot, new_slot.clone());
        family_versions.insert(slot.to_string(), writer_data_version);
        let revision = revisions
            .get::<_, i64>(&slot.to_string())
            .unwrap_or(0)
            .saturating_add(1);
        revisions.insert(slot.to_string(), revision);
        if seed_components_with_legacy {
            other_versions.insert(slot.to_string(), writer_data_version);
        } else if let Some(slot_value) = new_slot
            .and_then(|value| convert_hotbar_slot(value, family, other_family))
        {
            let other_origin = other_versions
                .get::<_, i32>(&slot.to_string())
                .unwrap_or(other_data_version);
            if writer_data_version <= 0
                || other_origin <= 0
                || writer_data_version >= other_origin
            {
                set_hotbar_slot(&mut other_root, slot, slot_value);
                other_versions.insert(slot.to_string(), writer_data_version);
            }
        }
        changed_any = true;
    }

    if writer_data_version > current_data_version {
        current.insert("DataVersion", writer_data_version);
        changed_any = true;
    }
    if seed_components_with_legacy
        && writer_data_version > hotbar_data_version(&other_root)
    {
        other_root.insert("DataVersion", writer_data_version);
    }

    state.insert(family_key, current);
    state.insert(other_key, other_root);
    state.insert("Revisions", revisions);
    state.insert(family_versions_key, family_versions);
    state.insert(other_versions_key, other_versions);
    changed_any
}

fn convert_hotbar_slot(
    value: NbtTag,
    from: HotbarFamily,
    to: HotbarFamily,
) -> Option<NbtTag> {
    if from == to {
        return Some(value);
    }
    let NbtTag::Compound(item) = value else {
        return None;
    };
    if item.is_empty() {
        return Some(NbtCompound::new().into());
    }
    let id = item.get::<_, &str>("id").ok()?.to_string();

    match (from, to) {
        (HotbarFamily::Legacy, HotbarFamily::Components) => {
            if item.contains_key("tag") {
                return None;
            }
            let count = item
                .get::<_, i8>("Count")
                .map(i32::from)
                .or_else(|_| item.get::<_, i32>("Count"))
                .unwrap_or(1);
            let mut converted = NbtCompound::new();
            converted.insert("id", id);
            converted.insert("count", count);
            Some(converted.into())
        }
        (HotbarFamily::Components, HotbarFamily::Legacy) => {
            if item
                .get::<_, &NbtCompound>("components")
                .is_ok_and(|components| !components.is_empty())
            {
                return None;
            }
            let count = item.get::<_, i32>("count").unwrap_or(1);
            let count = i8::try_from(count).ok()?;
            let mut converted = NbtCompound::new();
            converted.insert("id", id);
            converted.insert("Count", count);
            Some(converted.into())
        }
        _ => Some(item.into()),
    }
}

fn hotbar_slot(root: &NbtCompound, slot: usize) -> Option<NbtTag> {
    let toolbar = (slot / 9).to_string();
    let position = slot % 9;
    root.get::<_, &NbtList>(&toolbar)
        .ok()
        .and_then(|list| list.as_ref().get(position).cloned())
}

fn set_hotbar_slot(root: &mut NbtCompound, slot: usize, value: NbtTag) {
    let toolbar = (slot / 9).to_string();
    let position = slot % 9;
    let mut list = root
        .get::<_, &NbtList>(&toolbar)
        .ok()
        .cloned()
        .unwrap_or_default();
    while list.len() < 9 {
        list.push(NbtCompound::new());
    }
    list[position] = value;
    root.insert(toolbar, list);
}

fn set_hotbar_slot_optional(
    root: &mut NbtCompound,
    slot: usize,
    value: Option<NbtTag>,
) {
    set_hotbar_slot(
        root,
        slot,
        value.unwrap_or_else(|| NbtCompound::new().into()),
    );
}

fn hotbar_data_version(root: &NbtCompound) -> i32 {
    root.get::<_, i32>("DataVersion").unwrap_or(0)
}

fn family_data_versions_key(family: HotbarFamily) -> &'static str {
    match family {
        HotbarFamily::Legacy => "LegacyDataVersions",
        HotbarFamily::Components => "ComponentsDataVersions",
    }
}

pub(super) async fn regenerate_hotbars(state: &State) -> crate::Result<()> {
    let instances = crate::state::list_instances(&state.pool).await?;
    for metadata in instances {
        if option_effective(&metadata, SyncedOption::CreativeHotbars, state)
            .await?
        {
            write_hotbar_projection(&metadata, state).await?;
        }
    }
    Ok(())
}

async fn write_hotbar_projection(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let sync_state = read_hotbar_state(state).await?;
    let family = hotbar_family(metadata, state).await?;
    let root = hotbar_family_root(&sync_state.nbt, family);
    let bytes = nbt_to_bytes(&root)?;
    let expected = sha1_bytes(&bytes);
    let generated = generated_hotbar_path(state, family, &metadata.instance.id);
    begin_checkpoint(
        &metadata.instance.id,
        SyncedOption::CreativeHotbars,
        family.as_str(),
        &expected,
        Some(&bytes),
        sync_state.revision,
        state,
    )
    .await?;
    if let Some(parent) = generated.parent() {
        io::create_dir_all(parent).await?;
    }
    io::write(&generated, &bytes).await?;
    let local = instance_dir(metadata, state).join(HOTBAR_FILE);
    let mode = ensure_link(&generated, &local).await?;
    finish_checkpoint(
        &metadata.instance.id,
        SyncedOption::CreativeHotbars,
        family.as_str(),
        mode,
        state,
    )
    .await?;
    remove_other_hotbar_checkpoint(&metadata.instance.id, family, state).await
}

async fn remove_other_hotbar_checkpoint(
    instance_id: &str,
    family: HotbarFamily,
    state: &State,
) -> crate::Result<()> {
    let variant = family.as_str();
    sqlx::query!(
        "
		DELETE FROM instance_sync_checkpoints
		WHERE instance_id = ?
			AND feature = 'creative_hotbars'
			AND variant != ?
		",
        instance_id,
        variant,
    )
    .execute(&state.pool)
    .await?;
    Ok(())
}

pub(super) fn empty_hotbar_root() -> NbtCompound {
    let mut root = NbtCompound::new();
    for toolbar in 0..9 {
        let mut items = NbtList::new();
        for _ in 0..9 {
            items.push(NbtCompound::new());
        }
        root.insert(toolbar.to_string(), items);
    }
    root
}

pub(super) async fn read_hotbar_state(
    state: &State,
) -> crate::Result<HotbarState> {
    let row = sqlx::query!(
        r#"
		SELECT schema_version AS "schema_version!: i64",
			revision AS "revision!: i64", nbt
		FROM synced_hotbar_state
		WHERE singleton = 1
        "#,
    )
    .fetch_optional(&state.pool)
    .await?;
    if let Some(row) = row {
        return Ok(HotbarState {
            schema_version: row.schema_version,
            revision: row.revision,
            nbt: nbt_from_bytes(row.nbt)?,
        });
    }
    Ok(HotbarState {
        schema_version: HOTBAR_SCHEMA_VERSION,
        revision: 0,
        nbt: NbtCompound::new(),
    })
}

pub(super) async fn hotbar_state_exists(state: &State) -> crate::Result<bool> {
    Ok(sqlx::query_scalar!(
        r#"
		SELECT EXISTS(
			SELECT 1 FROM synced_hotbar_state WHERE singleton = 1
		) AS "exists!: bool"
		"#,
    )
    .fetch_one(&state.pool)
    .await?)
}

pub(super) fn hotbar_family_root(
    state: &NbtCompound,
    family: HotbarFamily,
) -> NbtCompound {
    let legacy = state
        .get::<_, &NbtCompound>("Legacy")
        .ok()
        .cloned()
        .unwrap_or_else(empty_hotbar_root);
    match family {
        HotbarFamily::Legacy => legacy,
        HotbarFamily::Components => state
            .get::<_, &NbtCompound>("Components")
            .ok()
            .cloned()
            .unwrap_or(legacy),
    }
}

pub(super) fn increment_hotbar_revision(state: &mut HotbarState) {
    state.schema_version = HOTBAR_SCHEMA_VERSION;
    state.revision = state.revision.saturating_add(1);
}

pub(super) async fn write_hotbar_state(
    state: &State,
    hotbar_state: &HotbarState,
) -> crate::Result<()> {
    let bytes = nbt_to_bytes(&hotbar_state.nbt)?;
    sqlx::query!(
        "
		INSERT INTO synced_hotbar_state
			(singleton, schema_version, revision, nbt)
		VALUES (1, ?, ?, ?)
		ON CONFLICT(singleton) DO UPDATE SET
			schema_version = excluded.schema_version,
			revision = excluded.revision,
			nbt = excluded.nbt
		",
        hotbar_state.schema_version,
        hotbar_state.revision,
        bytes,
    )
    .execute(&state.pool)
    .await?;
    Ok(())
}

fn family_state_key(family: HotbarFamily) -> &'static str {
    match family {
        HotbarFamily::Legacy => "Legacy",
        HotbarFamily::Components => "Components",
    }
}

pub(super) fn generated_hotbar_path(
    state: &State,
    family: HotbarFamily,
    instance_id: &str,
) -> PathBuf {
    synced_options_path(state)
        .join("hotbars/generated")
        .join(family.as_str())
        .join(safe_instance_id(instance_id))
        .join(HOTBAR_FILE)
}

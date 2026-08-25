use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::{
    InstanceInstallStage, InstanceLink, InstanceMetadata, SyncedOption,
};
use crate::util::io;
use crate::{ErrorKind, State};
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use serde::{Deserialize, Serialize};
use sha1_smol::Sha1;
use std::io::Cursor;
use std::path::{Path, PathBuf};

const COMMAND_HISTORY_FILE: &str = "command_history.txt";
const HOTBAR_FILE: &str = "hotbar.nbt";
const COMMAND_HISTORY_LIMIT: usize = 50;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct GlobalSyncedOptions {
    pub command_history: bool,
    pub multiplayer_servers: bool,
    pub creative_hotbars: bool,
    pub screenshots: bool,
}

impl GlobalSyncedOptions {
    pub fn get(self, option: SyncedOption) -> bool {
        match option {
            SyncedOption::CommandHistory => self.command_history,
            SyncedOption::MultiplayerServers => self.multiplayer_servers,
            SyncedOption::CreativeHotbars => self.creative_hotbars,
            SyncedOption::Screenshots => self.screenshots,
        }
    }

    fn set(&mut self, option: SyncedOption, enabled: bool) {
        match option {
            SyncedOption::CommandHistory => self.command_history = enabled,
            SyncedOption::MultiplayerServers => {
                self.multiplayer_servers = enabled
            }
            SyncedOption::CreativeHotbars => self.creative_hotbars = enabled,
            SyncedOption::Screenshots => self.screenshots = enabled,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncedOptionCapability {
    pub option: SyncedOption,
    pub supported: bool,
    pub disabled_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncedOptionsOverview {
    pub global_options: GlobalSyncedOptions,
    pub capabilities: Vec<SyncedOptionCapability>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HotbarFamily {
    Legacy,
    Components,
}

impl HotbarFamily {
    fn as_str(self) -> &'static str {
        match self {
            Self::Legacy => "legacy",
            Self::Components => "components",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum LinkMode {
    Symbolic,
    #[cfg(windows)]
    Hard,
    #[cfg(windows)]
    Copy,
}

impl LinkMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Symbolic => "symbolic",
            #[cfg(windows)]
            Self::Hard => "hard",
            #[cfg(windows)]
            Self::Copy => "copy",
        }
    }
}

pub async fn get_global_options() -> crate::Result<GlobalSyncedOptions> {
    let state = State::get().await?;
    get_global_options_with_state(&state).await
}

pub async fn get_synced_options_folder() -> crate::Result<PathBuf> {
    let state = State::get().await?;
    create_synced_directories(&state).await?;
    Ok(synced_options_path(&state))
}

async fn get_global_options_with_state(
    state: &State,
) -> crate::Result<GlobalSyncedOptions> {
    let mut options = GlobalSyncedOptions::default();
	let rows = sqlx::query!(
		r#"
		SELECT option, enabled AS "enabled!: bool"
		FROM global_synced_options_overrides
		"#,
	)
	.fetch_all(&state.pool)
	.await?;

	for row in rows {
		if let Some(option) = option_from_str(&row.option) {
			options.set(option, row.enabled);
        }
    }

    Ok(options)
}

pub async fn get_overview(
    instance_id: &str,
) -> crate::Result<SyncedOptionsOverview> {
    let state = State::get().await?;
    let global_options = get_global_options_with_state(&state).await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    let mut capabilities = Vec::with_capacity(SyncedOption::ALL.len());

    for option in SyncedOption::ALL {
        capabilities.push(
            capability(&metadata, option, global_options.get(option), &state)
                .await,
        );
    }

    Ok(SyncedOptionsOverview {
        global_options,
        capabilities,
    })
}

pub async fn get_capabilities(
    instance_id: &str,
) -> crate::Result<Vec<SyncedOptionCapability>> {
    Ok(get_overview(instance_id).await?.capabilities)
}

async fn capability(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    global_enabled: bool,
    state: &State,
) -> SyncedOptionCapability {
    let version_reason = version_disabled_reason(metadata, option, state).await;
    let linked_reason = (option == SyncedOption::MultiplayerServers
		&& is_linked_server_project(&metadata.link))
	.then(|| {
		"Multiplayer server syncing is unavailable for linked server-project instances."
			.to_string()
	});
    let global_reason = (!global_enabled).then(|| {
        "This option is disabled in the app's synced options settings."
            .to_string()
    });
    let disabled_reason = version_reason.or(linked_reason).or(global_reason);

    SyncedOptionCapability {
        option,
        supported: disabled_reason.is_none(),
        disabled_reason,
    }
}

async fn version_disabled_reason(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    state: &State,
) -> Option<String> {
    if option == SyncedOption::Screenshots {
        return None;
    }

    let game_version = &metadata.applied_content_set.game_version;
    let Ok((manifest, version_index)) =
        crate::launcher::resolve_minecraft_manifest(game_version, state).await
    else {
        return Some(
			"This instance’s Minecraft version could not be verified, so syncing is unavailable."
				.to_string(),
		);
    };
    let cutoff_id = match option {
        // Mojang's manifest does not include Beta 1.8 Pre-release as a
        // separate entry, so b1.8 is the first resolvable version at that
        // boundary.
        SyncedOption::MultiplayerServers => "b1.8",
        SyncedOption::CreativeHotbars => "1.12",
        SyncedOption::CommandHistory => "1.20.2",
        SyncedOption::Screenshots => return None,
    };
    let Some(cutoff) =
        manifest.versions.iter().find(|item| item.id == cutoff_id)
    else {
        return Some(
			"This instance’s Minecraft version could not be verified, so syncing is unavailable."
				.to_string(),
		);
    };

    let version = &manifest.versions[version_index];
    let release_only_option = matches!(
        option,
        SyncedOption::CreativeHotbars | SyncedOption::CommandHistory
    );
    let is_supported_release =
        matches!(&version.type_, daedalus::minecraft::VersionType::Release);

    if version.release_time >= cutoff.release_time
        && (!release_only_option || is_supported_release)
    {
        return None;
    }

    Some(
		match option {
			SyncedOption::MultiplayerServers => {
				"Multiplayer server syncing requires Minecraft Beta 1.8 Pre-release or newer."
			}
			SyncedOption::CreativeHotbars => {
				"Saved creative hotbars require Minecraft 1.12 or newer."
			}
			SyncedOption::CommandHistory => {
				"Command history syncing requires Minecraft 1.20.2 or newer."
			}
			SyncedOption::Screenshots => unreachable!(),
		}
		.to_string(),
	)
}

async fn hotbar_family(
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

pub async fn set_global_option(
    option: SyncedOption,
    enabled: bool,
    base_instance_id: Option<&str>,
) -> crate::Result<GlobalSyncedOptions> {
    let state = State::get().await?;
    if enabled && option != SyncedOption::Screenshots {
        let base_instance_id = base_instance_id.ok_or_else(|| {
            ErrorKind::InputError(
                "Choose a base instance before enabling a synced option."
                    .to_string(),
            )
        })?;
        let metadata =
            crate::state::get_instance(base_instance_id, &state.pool)
                .await?
                .ok_or_else(|| {
                    ErrorKind::InputError("Unknown instance".to_string())
                })?;
        let base_capability = capability(&metadata, option, true, &state).await;
        if !base_capability.supported {
            return Err(ErrorKind::InputError(
                base_capability.disabled_reason.unwrap_or_default(),
            )
            .into());
        }
        seed_from_instance(&metadata, option, &state).await?;
    }

	let option_name = option.as_str();
	sqlx::query!(
		"
		INSERT INTO global_synced_options_overrides (option, enabled)
		VALUES (?, ?)
		ON CONFLICT(option) DO UPDATE SET enabled = excluded.enabled
		",
		option_name,
		enabled,
	)
	.execute(&state.pool)
    .await?;

    let instances = crate::state::list_instances(&state.pool).await?;
    for metadata in instances {
        let supported = enabled
            && capability(&metadata, option, true, &state).await.supported;
        instance_rows::set_instance_synced_option(
            &metadata.instance.id,
            option,
            supported,
            &state.pool,
        )
        .await?;
        if supported {
            ensure_option(&metadata, option, &state).await?;
        } else {
            detach_option(&metadata, option, &state).await?;
        }
    }

    get_global_options_with_state(&state).await
}

pub async fn set_instance_option(
    instance_id: &str,
    option: SyncedOption,
    enabled: bool,
) -> crate::Result<InstanceMetadata> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    let global = get_global_options_with_state(&state).await?;
    if enabled {
        let eligibility =
            capability(&metadata, option, global.get(option), &state).await;
        if !eligibility.supported {
            return Err(ErrorKind::InputError(
                eligibility.disabled_reason.unwrap_or_default(),
            )
            .into());
        }
        if !canonical_exists(option, &state) {
            seed_from_instance(&metadata, option, &state).await?;
        }
    }

    instance_rows::set_instance_synced_option(
        instance_id,
        option,
        enabled,
        &state.pool,
    )
    .await?;
    if enabled {
        ensure_option(&metadata, option, &state).await?;
    } else {
        detach_option(&metadata, option, &state).await?;
    }

    crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            ErrorKind::InputError("Unknown instance".to_string()).into()
        })
}

pub async fn reconcile_all() -> crate::Result<()> {
    let state = State::get().await?;
    create_synced_directories(&state).await?;
    let instances = crate::state::list_instances(&state.pool).await?;
    for metadata in instances {
        if let Err(error) =
            reconcile_instance_with_state(&metadata, &state).await
        {
            tracing::warn!(
                "Failed to reconcile synced options for {}: {error}",
                metadata.instance.id
            );
        }
    }
    Ok(())
}

pub async fn reconcile_instance(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    reconcile_instance_with_state(&metadata, &state).await
}

pub(crate) async fn prepare_instance_update(
    instance_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    for option in [
        SyncedOption::CommandHistory,
        SyncedOption::CreativeHotbars,
        SyncedOption::MultiplayerServers,
    ] {
        if instance_option_enabled(&metadata, option) {
            detach_option(&metadata, option, &state).await?;
        }
    }
    Ok(())
}

async fn reconcile_instance_with_state(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    if sync_files_are_protected(metadata)
        || instance_is_running(metadata, state)
    {
        return Ok(());
    }
    let global = get_global_options_with_state(state).await?;
    for option in SyncedOption::ALL {
        let participating = instance_option_enabled(metadata, option);
        let eligible = capability(metadata, option, global.get(option), state)
            .await
            .supported;
        if participating && eligible {
            match option {
                SyncedOption::CommandHistory => {
                    reconcile_command_history(metadata, state).await?
                }
                SyncedOption::CreativeHotbars => {
                    reconcile_hotbar(metadata, state).await?
                }
                SyncedOption::MultiplayerServers => {
                    super::synced_servers::reconcile_servers(metadata, state)
                        .await?
                }
                SyncedOption::Screenshots => {}
            }
        } else if participating {
            instance_rows::set_instance_synced_option(
                &metadata.instance.id,
                option,
                false,
                &state.pool,
            )
            .await?;
            detach_option(metadata, option, state).await?;
        }
    }
    Ok(())
}

pub async fn reconcile_changed_file(
    instance_id: &str,
    file_name: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    if sync_files_are_protected(&metadata) {
        return Ok(());
    }
    match file_name {
        COMMAND_HISTORY_FILE => {
            reconcile_command_history(&metadata, &state).await
        }
        HOTBAR_FILE => reconcile_hotbar(&metadata, &state).await,
        "servers.dat" => {
            super::synced_servers::reconcile_servers(&metadata, &state).await
        }
        _ => Ok(()),
    }
}

pub async fn get_command_history() -> crate::Result<String> {
    let state = State::get().await?;
    let path = command_history_path(&state);
    if !path.exists() {
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&io::read(path).await?).into_owned())
}

pub async fn set_command_history(contents: &str) -> crate::Result<String> {
    let state = State::get().await?;
    create_synced_directories(&state).await?;
    let normalized = normalize_command_history(contents);
    io::write(command_history_path(&state), normalized.as_bytes()).await?;
    refresh_command_history_links(&state).await?;
    Ok(normalized)
}

pub fn synced_options_path(state: &State) -> PathBuf {
    state.directories.synced_options_dir()
}

async fn create_synced_directories(state: &State) -> crate::Result<()> {
    for path in [
        synced_options_path(state),
        synced_options_path(state).join("hotbars/generated/legacy"),
        synced_options_path(state).join("hotbars/generated/components"),
        synced_options_path(state).join("servers/generated"),
    ] {
        io::create_dir_all(path).await?;
    }
    Ok(())
}

async fn seed_from_instance(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    state: &State,
) -> crate::Result<()> {
    create_synced_directories(state).await?;
    let instance_dir = instance_dir(metadata, state);
    match option {
        SyncedOption::CommandHistory => {
            let path = instance_dir.join(COMMAND_HISTORY_FILE);
            let contents = if path.exists() {
                String::from_utf8_lossy(&io::read(path).await?).into_owned()
            } else {
                String::new()
            };
            io::write(
                command_history_path(state),
                normalize_command_history(&contents),
            )
            .await?;
        }
        SyncedOption::CreativeHotbars => {
            let path = instance_dir.join(HOTBAR_FILE);
            let family = hotbar_family(metadata, state).await?;
            let root = if path.exists() {
                read_nbt_file(&path).await?
            } else {
                empty_hotbar_root()
            };
            let mut sync_state = read_hotbar_state(state).await?;
            merge_hotbar_family(&mut sync_state, family, root);
            write_hotbar_state(state, &sync_state).await?;
            regenerate_hotbars(state).await?;
        }
        SyncedOption::MultiplayerServers => {
            super::synced_servers::seed_servers(metadata, state).await?;
        }
        SyncedOption::Screenshots => {}
    }
    Ok(())
}

async fn ensure_option(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    state: &State,
) -> crate::Result<()> {
    match option {
        SyncedOption::CommandHistory => {
            ensure_command_history(metadata, state).await
        }
        SyncedOption::CreativeHotbars => ensure_hotbar(metadata, state).await,
        SyncedOption::MultiplayerServers => {
            super::synced_servers::ensure_servers(metadata, state).await
        }
        SyncedOption::Screenshots => Ok(()),
    }
}

async fn detach_option(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    state: &State,
) -> crate::Result<()> {
    let instance_dir = instance_dir(metadata, state);
    match option {
        SyncedOption::CommandHistory => {
            detach_link(
                &command_history_path(state),
                &instance_dir.join(COMMAND_HISTORY_FILE),
            )
            .await
        }
        SyncedOption::CreativeHotbars => {
            let family = hotbar_family(metadata, state).await.ok();
            let source = family.map(|family| {
                generated_hotbar_path(state, family, &metadata.instance.id)
            });
            if let Some(source) = source {
                detach_link(&source, &instance_dir.join(HOTBAR_FILE)).await?;
            }
            Ok(())
        }
        SyncedOption::MultiplayerServers => {
            super::synced_servers::detach_servers(metadata, state).await
        }
        SyncedOption::Screenshots => Ok(()),
    }
}

fn canonical_exists(option: SyncedOption, state: &State) -> bool {
    match option {
        SyncedOption::CommandHistory => command_history_path(state).exists(),
        SyncedOption::CreativeHotbars => hotbar_state_path(state).exists(),
        SyncedOption::MultiplayerServers => synced_options_path(state)
            .join("servers/canonical.nbt")
            .exists(),
        SyncedOption::Screenshots => true,
    }
}

async fn ensure_command_history(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    create_synced_directories(state).await?;
    let canonical = command_history_path(state);
    if !canonical.exists() {
        let local = instance_dir(metadata, state).join(COMMAND_HISTORY_FILE);
        let contents = if local.exists() {
            String::from_utf8_lossy(&io::read(&local).await?).into_owned()
        } else {
            String::new()
        };
        io::write(&canonical, normalize_command_history(&contents)).await?;
    }
    let target = instance_dir(metadata, state).join(COMMAND_HISTORY_FILE);
    let mode = ensure_link(&canonical, &target).await?;
    record_materialization(
        &metadata.instance.id,
        SyncedOption::CommandHistory,
        "",
        Some(&sha1_file(&canonical).await?),
        mode,
        state,
    )
    .await
}

async fn reconcile_command_history(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    if !option_effective(metadata, SyncedOption::CommandHistory, state).await? {
        return Ok(());
    }
    let local = instance_dir(metadata, state).join(COMMAND_HISTORY_FILE);
    if !local.exists() {
        return ensure_command_history(metadata, state).await;
    }
    let symlink = tokio::fs::symlink_metadata(&local)
        .await
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false);
    let expected = materialization_hash(
        &metadata.instance.id,
        SyncedOption::CommandHistory,
        "",
        state,
    )
    .await?;
    let actual = sha1_file(&local).await?;
    if !symlink && expected.as_deref() != Some(actual.as_str()) {
        let contents =
            String::from_utf8_lossy(&io::read(&local).await?).into_owned();
        io::write(
            command_history_path(state),
            normalize_command_history(&contents),
        )
        .await?;
        refresh_command_history_links(state).await?;
    } else {
        ensure_command_history(metadata, state).await?;
    }
    Ok(())
}

async fn refresh_command_history_links(state: &State) -> crate::Result<()> {
    let instances = crate::state::list_instances(&state.pool).await?;
    for metadata in instances {
        if option_effective(&metadata, SyncedOption::CommandHistory, state)
            .await?
        {
            ensure_command_history(&metadata, state).await?;
        }
    }
    Ok(())
}

fn normalize_command_history(contents: &str) -> String {
    let lines = contents.lines().collect::<Vec<_>>();
    let start = lines.len().saturating_sub(COMMAND_HISTORY_LIMIT);
    let mut normalized = lines[start..].join("\n");
    if !normalized.is_empty() {
        normalized.push('\n');
    }
    normalized
}

async fn ensure_hotbar(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    create_synced_directories(state).await?;
    if !hotbar_state_path(state).exists() {
        seed_from_instance(metadata, SyncedOption::CreativeHotbars, state)
            .await?;
    }
    materialize_hotbars_for_instance(metadata, state).await
}

async fn reconcile_hotbar(
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
    let expected = materialization_hash(
        &metadata.instance.id,
        SyncedOption::CreativeHotbars,
        family.as_str(),
        state,
    )
    .await?;
    let actual = sha1_file(&local).await?;
    if expected.as_deref() == Some(actual.as_str()) {
        return Ok(());
    }

    let changed = read_nbt_file(&local).await?;
    let mut sync_state = read_hotbar_state(state).await?;
    merge_hotbar_family(&mut sync_state, family, changed);
    write_hotbar_state(state, &sync_state).await?;
    regenerate_hotbars(state).await
}

fn merge_hotbar_family(
    state: &mut NbtCompound,
    family: HotbarFamily,
    changed: NbtCompound,
) {
    let family_key = family_state_key(family);
    let previous = state
        .get::<_, &NbtCompound>(family_key)
        .ok()
        .cloned()
        .unwrap_or_else(empty_hotbar_root);
    let other_family = match family {
        HotbarFamily::Legacy => HotbarFamily::Components,
        HotbarFamily::Components => HotbarFamily::Legacy,
    };
    let other_key = family_state_key(other_family);
    let mut other = state.get::<_, &NbtCompound>(other_key).ok().cloned();

    let seed_components_with_legacy =
        other.is_none() && family == HotbarFamily::Legacy;
    if seed_components_with_legacy {
        other = Some(changed.clone());
    }
    let mut other_root = other.unwrap_or_else(empty_hotbar_root);
    let mut revisions = state
        .get::<_, &NbtCompound>("Revisions")
        .ok()
        .cloned()
        .unwrap_or_default();

    for slot in 0..81 {
        let old_slot = hotbar_slot(&previous, slot);
        let new_slot = hotbar_slot(&changed, slot);
        if old_slot == new_slot {
            continue;
        }
        let revision = revisions
            .get::<_, i64>(&slot.to_string())
            .unwrap_or(0)
            .saturating_add(1);
        revisions.insert(slot.to_string(), revision);
        if let Some(slot_value) = (!seed_components_with_legacy)
            .then_some(new_slot)
            .flatten()
            .and_then(|value| convert_hotbar_slot(value, family, other_family))
        {
            set_hotbar_slot(&mut other_root, slot, slot_value);
        }
    }

    state.insert(family_key, changed);
    state.insert(other_key, other_root);
    state.insert("Revisions", revisions);
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

async fn regenerate_hotbars(state: &State) -> crate::Result<()> {
    let instances = crate::state::list_instances(&state.pool).await?;
    for metadata in instances {
        if option_effective(&metadata, SyncedOption::CreativeHotbars, state)
            .await?
        {
            materialize_hotbars_for_instance(&metadata, state).await?;
        }
    }
    Ok(())
}

async fn materialize_hotbars_for_instance(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let sync_state = read_hotbar_state(state).await?;
    let legacy = sync_state
        .get::<_, &NbtCompound>("Legacy")
        .ok()
        .cloned()
        .unwrap_or_else(empty_hotbar_root);
    let components = sync_state
        .get::<_, &NbtCompound>("Components")
        .ok()
        .cloned()
        .unwrap_or_else(|| legacy.clone());

    for (family, root) in [
        (HotbarFamily::Legacy, legacy),
        (HotbarFamily::Components, components),
    ] {
        let path = generated_hotbar_path(state, family, &metadata.instance.id);
        if let Some(parent) = path.parent() {
            io::create_dir_all(parent).await?;
        }
        write_nbt_file(&path, &root).await?;
    }

    let family = hotbar_family(metadata, state).await?;
    let generated = generated_hotbar_path(state, family, &metadata.instance.id);
    let local = instance_dir(metadata, state).join(HOTBAR_FILE);
    let mode = ensure_link(&generated, &local).await?;
    record_materialization(
        &metadata.instance.id,
        SyncedOption::CreativeHotbars,
        family.as_str(),
        Some(&sha1_file(&generated).await?),
        mode,
        state,
    )
    .await
}

fn empty_hotbar_root() -> NbtCompound {
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

async fn read_hotbar_state(state: &State) -> crate::Result<NbtCompound> {
    let path = hotbar_state_path(state);
    if !path.exists() {
        let mut root = NbtCompound::new();
        root.insert("Version", 1_i32);
        return Ok(root);
    }
    read_nbt_file(&path).await
}

async fn write_hotbar_state(
    state: &State,
    root: &NbtCompound,
) -> crate::Result<()> {
    write_nbt_file(&hotbar_state_path(state), root).await
}

fn family_state_key(family: HotbarFamily) -> &'static str {
    match family {
        HotbarFamily::Legacy => "Legacy",
        HotbarFamily::Components => "Components",
    }
}

fn command_history_path(state: &State) -> PathBuf {
    synced_options_path(state).join(COMMAND_HISTORY_FILE)
}

fn hotbar_state_path(state: &State) -> PathBuf {
    synced_options_path(state).join("hotbars/state.nbt")
}

fn generated_hotbar_path(
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

pub(super) fn safe_instance_id(instance_id: &str) -> String {
    instance_id.replace([':', '/', '\\'], "_")
}

pub(super) fn instance_dir(
    metadata: &InstanceMetadata,
    state: &State,
) -> PathBuf {
    state
        .directories
        .instances_dir()
        .join(&metadata.instance.path)
}

async fn option_effective(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    state: &State,
) -> crate::Result<bool> {
    if sync_files_are_protected(metadata)
        || instance_is_running(metadata, state)
    {
        return Ok(false);
    }
    let global = get_global_options_with_state(state).await?;
    Ok(instance_option_enabled(metadata, option)
        && capability(metadata, option, global.get(option), state)
            .await
            .supported)
}

pub(super) fn sync_files_are_protected(metadata: &InstanceMetadata) -> bool {
    matches!(
        metadata.instance.install_stage,
        InstanceInstallStage::MinecraftInstalling
            | InstanceInstallStage::PackInstalling
    )
}

pub(super) fn instance_is_running(
    metadata: &InstanceMetadata,
    state: &State,
) -> bool {
    state
        .process_manager
        .get_all()
        .iter()
        .any(|process| process.instance_id == metadata.instance.id)
}

pub(super) fn instance_option_enabled(
    metadata: &InstanceMetadata,
    option: SyncedOption,
) -> bool {
    match option {
        SyncedOption::CommandHistory => metadata.synced_options.command_history,
        SyncedOption::MultiplayerServers => {
            metadata.synced_options.multiplayer_servers
        }
        SyncedOption::CreativeHotbars => {
            metadata.synced_options.creative_hotbars
        }
        SyncedOption::Screenshots => metadata.synced_options.screenshots,
    }
}

fn is_linked_server_project(link: &InstanceLink) -> bool {
    matches!(
        link,
        InstanceLink::ServerProject { .. }
            | InstanceLink::ServerProjectModpack { .. }
            | InstanceLink::ModrinthHosting { .. }
    )
}

fn option_from_str(value: &str) -> Option<SyncedOption> {
    match value {
        "command_history" => Some(SyncedOption::CommandHistory),
        "multiplayer_servers" => Some(SyncedOption::MultiplayerServers),
        "creative_hotbars" => Some(SyncedOption::CreativeHotbars),
        "screenshots" => Some(SyncedOption::Screenshots),
        _ => None,
    }
}

pub(super) async fn ensure_link(
    source: &Path,
    target: &Path,
) -> crate::Result<LinkMode> {
    if let Some(parent) = target.parent() {
        io::create_dir_all(parent).await?;
    }
    if tokio::fs::symlink_metadata(target)
        .await
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
        && tokio::fs::read_link(target)
            .await
            .is_ok_and(|current| current == source)
    {
        return Ok(LinkMode::Symbolic);
    }
    if tokio::fs::symlink_metadata(target).await.is_ok() {
        io::remove_file(target).await?;
    }

    #[cfg(unix)]
    {
        tokio::fs::symlink(source, target).await?;
        Ok(LinkMode::Symbolic)
    }
    #[cfg(windows)]
    {
        if tokio::fs::symlink_file(source, target).await.is_ok() {
            return Ok(LinkMode::Symbolic);
        }
        if tokio::fs::hard_link(source, target).await.is_ok() {
            return Ok(LinkMode::Hard);
        }
        io::copy(source, target).await?;
        Ok(LinkMode::Copy)
    }
}

pub(super) async fn detach_link(
    source: &Path,
    target: &Path,
) -> crate::Result<()> {
    let contents = if source.exists() {
        Some(io::read(source).await?)
    } else if target.exists() {
        Some(io::read(target).await?)
    } else {
        None
    };
    if tokio::fs::symlink_metadata(target).await.is_ok() {
        io::remove_file(target).await?;
    }
    if let Some(contents) = contents {
        io::write(target, contents).await?;
    }
    Ok(())
}

pub(super) async fn record_materialization(
    instance_id: &str,
    option: SyncedOption,
    family: &str,
    expected_sha1: Option<&str>,
    mode: LinkMode,
    state: &State,
) -> crate::Result<()> {
	let option_name = option.as_str();
	let link_mode = mode.as_str();
	sqlx::query!(
		"
		INSERT INTO synced_option_materializations
			(instance_id, option, family, expected_sha1, link_mode)
		VALUES (?, ?, ?, ?, ?)
		ON CONFLICT(instance_id, option, family) DO UPDATE SET
			expected_sha1 = excluded.expected_sha1,
			link_mode = excluded.link_mode
		",
		instance_id,
		option_name,
		family,
		expected_sha1,
		link_mode,
	)
	.execute(&state.pool)
    .await?;
    Ok(())
}

async fn materialization_hash(
    instance_id: &str,
    option: SyncedOption,
    family: &str,
    state: &State,
) -> crate::Result<Option<String>> {
	let option_name = option.as_str();
	Ok(sqlx::query_scalar!(
		"
		SELECT expected_sha1
		FROM synced_option_materializations
		WHERE instance_id = ? AND option = ? AND family = ?
		",
		instance_id,
		option_name,
		family,
	)
	.fetch_optional(&state.pool)
	.await?
	.flatten())
}

pub(super) async fn sha1_file(path: &Path) -> crate::Result<String> {
    Ok(Sha1::from(io::read(path).await?).digest().to_string())
}

pub(super) async fn read_nbt_file(path: &Path) -> crate::Result<NbtCompound> {
    let bytes = io::read(path).await?;
    let (root, _) = quartz_nbt::io::read_nbt(
        &mut Cursor::new(bytes),
        quartz_nbt::io::Flavor::Uncompressed,
    )?;
    Ok(root)
}

pub(super) async fn write_nbt_file(
    path: &Path,
    root: &NbtCompound,
) -> crate::Result<()> {
    if let Some(parent) = path.parent() {
        io::create_dir_all(parent).await?;
    }
    let mut bytes = Vec::new();
    quartz_nbt::io::write_nbt(
        &mut bytes,
        None,
        root,
        quartz_nbt::io::Flavor::Uncompressed,
    )?;
    io::write(path, bytes).await?;
    Ok(())
}

pub(super) fn nbt_to_bytes(root: &NbtCompound) -> crate::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    quartz_nbt::io::write_nbt(
        &mut bytes,
        None,
        root,
        quartz_nbt::io::Flavor::Uncompressed,
    )?;
    Ok(bytes)
}

pub(super) fn nbt_from_bytes(bytes: Vec<u8>) -> crate::Result<NbtCompound> {
    let (root, _) = quartz_nbt::io::read_nbt(
        &mut Cursor::new(bytes),
        quartz_nbt::io::Flavor::Uncompressed,
    )?;
    Ok(root)
}

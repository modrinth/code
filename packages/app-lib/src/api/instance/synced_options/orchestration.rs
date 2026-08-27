use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::{InstanceLink, InstanceMetadata, SyncedOption};
use crate::util::io;
use crate::{ErrorKind, State};
use quartz_nbt::NbtCompound;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::synced_servers;
use super::command_history::{
    command_history_path, ensure_command_history,
    merge_command_history_from_instance, normalize_command_history,
    reconcile_command_history,
};
use super::files::{
    detach_link, instance_dir, instance_is_running, instance_option_enabled,
    read_nbt_file, safe_instance_id, sync_files_are_protected,
};
use super::hotbars::{
    HOTBAR_SCHEMA_VERSION, HotbarState, backup_shared_hotbars,
    empty_hotbar_root, ensure_hotbar, generated_hotbar_path, hotbar_family,
    hotbar_state_exists, increment_hotbar_revision,
    instance_hotbars_differ_from_synced, merge_hotbar_family,
    read_hotbar_state, reconcile_hotbar, regenerate_hotbars,
    write_hotbar_state,
};
use super::{COMMAND_HISTORY_FILE, HOTBAR_FILE};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncedOptionJoinAction {
    SeedShared,
    Attach,
    Merge,
    RequiresResolution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct SyncedOptionJoinPreview {
    pub action: SyncedOptionJoinAction,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncedOptionJoinResolution {
    UseSynced,
    UseInstance,
}

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

enum CapabilityStatus {
    Supported,
    Unsupported(String),
    Indeterminate(String),
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
		SELECT feature, globally_enabled AS "globally_enabled!: bool"
		FROM sync_feature_settings
		"#,
    )
    .fetch_all(&state.pool)
    .await?;

    for row in rows {
        if let Some(option) = option_from_str(&row.feature) {
            options.set(option, row.globally_enabled);
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
    let status =
        capability_status(metadata, option, global_enabled, state).await;
    let (supported, disabled_reason) = match status {
        CapabilityStatus::Supported => (true, None),
        CapabilityStatus::Unsupported(reason)
        | CapabilityStatus::Indeterminate(reason) => (false, Some(reason)),
    };

    SyncedOptionCapability {
        option,
        supported,
        disabled_reason,
    }
}

async fn capability_status(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    global_enabled: bool,
    state: &State,
) -> CapabilityStatus {
    if !global_enabled {
        return CapabilityStatus::Unsupported(
            "This option is disabled in the app's synced options settings."
                .to_string(),
        );
    }
    if option == SyncedOption::MultiplayerServers
        && is_linked_server_project(&metadata.link)
    {
        return CapabilityStatus::Unsupported(
			"Multiplayer server syncing is unavailable for linked server-project instances."
				.to_string(),
		);
    }
    match version_capability(metadata, option, state).await {
        CapabilityStatus::Supported => CapabilityStatus::Supported,
        status => status,
    }
}

pub(in crate::api::instance) async fn instance_option_supported(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    global_enabled: bool,
    state: &State,
) -> bool {
    matches!(
        capability_status(metadata, option, global_enabled, state).await,
        CapabilityStatus::Supported
    )
}

async fn version_capability(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    state: &State,
) -> CapabilityStatus {
    if option == SyncedOption::Screenshots {
        return CapabilityStatus::Supported;
    }

    let game_version = &metadata.applied_content_set.game_version;
    let Ok((manifest, version_index)) =
        crate::launcher::resolve_minecraft_manifest(game_version, state).await
    else {
        return CapabilityStatus::Indeterminate(
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
        SyncedOption::Screenshots => return CapabilityStatus::Supported,
    };
    let Some(cutoff) =
        manifest.versions.iter().find(|item| item.id == cutoff_id)
    else {
        return CapabilityStatus::Indeterminate(
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
        return CapabilityStatus::Supported;
    }

    CapabilityStatus::Unsupported(
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

pub async fn set_global_option(
    option: SyncedOption,
    enabled: bool,
) -> crate::Result<GlobalSyncedOptions> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let reset_participation =
        enabled && !canonical_exists(option, &state).await?;

    let option_name = option.as_str();
    sqlx::query!(
        "
		INSERT INTO sync_feature_settings
			(feature, globally_enabled, new_instance_default)
		VALUES (?, ?, 1)
		ON CONFLICT(feature) DO UPDATE SET
			globally_enabled = excluded.globally_enabled
		",
        option_name,
        enabled,
    )
    .execute(&state.pool)
    .await?;

    let instances = crate::state::list_instances(&state.pool).await?;
    if reset_participation {
        for metadata in instances {
            if instance_option_enabled(&metadata, option) {
                instance_rows::set_instance_sync_preference(
                    &metadata.instance.id,
                    option,
                    false,
                    &state.pool,
                )
                .await?;
            }
            if !sync_files_are_protected(&metadata)
                && !instance_is_running(&metadata, &state).await?
            {
                detach_option(&metadata, option, &state).await?;
            }
        }
        return get_global_options_with_state(&state).await;
    }
    for metadata in instances {
        if sync_files_are_protected(&metadata)
            || instance_is_running(&metadata, &state).await?
        {
            continue;
        }
        if !instance_option_enabled(&metadata, option) {
            detach_option(&metadata, option, &state).await?;
            continue;
        }
        match capability_status(&metadata, option, enabled, &state).await {
            CapabilityStatus::Supported => {
                reconcile_option(&metadata, option, &state).await?
            }
            CapabilityStatus::Unsupported(_) => {
                detach_option(&metadata, option, &state).await?
            }
            CapabilityStatus::Indeterminate(_) => {}
        }
    }

    get_global_options_with_state(&state).await
}

pub async fn set_instance_option(
    instance_id: &str,
    option: SyncedOption,
    enabled: bool,
    resolution: Option<SyncedOptionJoinResolution>,
) -> crate::Result<InstanceMetadata> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    if instance_option_enabled(&metadata, option) == enabled {
        return Ok(metadata);
    }
    let global = get_global_options_with_state(&state).await?;
    let can_reconcile = !sync_files_are_protected(&metadata)
        && !instance_is_running(&metadata, &state).await?;
    if enabled {
        let eligibility =
            capability(&metadata, option, global.get(option), &state).await;
        if !eligibility.supported {
            return Err(ErrorKind::InputError(
                eligibility.disabled_reason.unwrap_or_default(),
            )
            .into());
        }
        if option != SyncedOption::Screenshots && !can_reconcile {
            return Err(ErrorKind::InputError(
                "Close the instance before including it in file syncing."
                    .to_string(),
            )
            .into());
        }

        let action =
            instance_option_join_action(&metadata, option, &state).await?;
        if action == SyncedOptionJoinAction::RequiresResolution
            && resolution.is_none()
        {
            return Err(ErrorKind::InputError(
                "Choose whether to use the synced hotbars or this instance's hotbars."
                    .to_string(),
            )
            .into());
        }
        if option != SyncedOption::Screenshots {
            backup_instance_option_file(&metadata, option, &state).await?;
        }
        match action {
            SyncedOptionJoinAction::SeedShared => {
                seed_from_instance(&metadata, option, &state).await?;
            }
            SyncedOptionJoinAction::Attach => {}
            SyncedOptionJoinAction::Merge => match option {
                SyncedOption::CommandHistory => {
                    merge_command_history_from_instance(&metadata, &state)
                        .await?;
                }
                SyncedOption::MultiplayerServers => {
                    synced_servers::merge_servers_from_instance(
                        &metadata, &state,
                    )
                    .await?;
                }
                SyncedOption::CreativeHotbars | SyncedOption::Screenshots => {
                    unreachable!()
                }
            },
            SyncedOptionJoinAction::RequiresResolution => match resolution {
                Some(SyncedOptionJoinResolution::UseSynced) => {}
                Some(SyncedOptionJoinResolution::UseInstance) => {
                    backup_shared_hotbars(&state).await?;
                    seed_from_instance(&metadata, option, &state).await?;
                }
                None => unreachable!(),
            },
        }
    }

    instance_rows::set_instance_sync_preference(
        instance_id,
        option,
        enabled,
        &state.pool,
    )
    .await?;
    if can_reconcile {
        if enabled {
            ensure_option(&metadata, option, &state).await?;
        } else {
            detach_option(&metadata, option, &state).await?;
        }
    }

    crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            ErrorKind::InputError("Unknown instance".to_string()).into()
        })
}

pub async fn get_instance_option_join_preview(
    instance_id: &str,
    option: SyncedOption,
) -> crate::Result<SyncedOptionJoinPreview> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    let global = get_global_options_with_state(&state).await?;
    let eligibility =
        capability(&metadata, option, global.get(option), &state).await;
    if !eligibility.supported {
        return Err(ErrorKind::InputError(
            eligibility.disabled_reason.unwrap_or_default(),
        )
        .into());
    }
    if option != SyncedOption::Screenshots
        && (sync_files_are_protected(&metadata)
            || instance_is_running(&metadata, &state).await?)
    {
        return Err(ErrorKind::InputError(
            "Close the instance before including it in file syncing."
                .to_string(),
        )
        .into());
    }

    Ok(SyncedOptionJoinPreview {
        action: instance_option_join_action(&metadata, option, &state).await?,
    })
}

async fn instance_option_join_action(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    state: &State,
) -> crate::Result<SyncedOptionJoinAction> {
    if !canonical_exists(option, state).await? {
        return Ok(SyncedOptionJoinAction::SeedShared);
    }
    Ok(match option {
        SyncedOption::CommandHistory | SyncedOption::MultiplayerServers => {
            SyncedOptionJoinAction::Merge
        }
        SyncedOption::CreativeHotbars => {
            if instance_hotbars_differ_from_synced(metadata, state).await? {
                SyncedOptionJoinAction::RequiresResolution
            } else {
                SyncedOptionJoinAction::Attach
            }
        }
        SyncedOption::Screenshots => SyncedOptionJoinAction::Attach,
    })
}

pub async fn reconcile_all() -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
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

pub(crate) async fn monitor_persisted_processes() -> crate::Result<()> {
    let state = State::get().await?;
    let instance_ids =
        sqlx::query_scalar!("SELECT DISTINCT instance_id FROM processes",)
            .fetch_all(&state.pool)
            .await?;
    for instance_id in instance_ids {
        tokio::spawn(async move {
            loop {
                let Ok(state) = State::get().await else {
                    break;
                };
                let Ok(Some(metadata)) =
                    crate::state::get_instance(&instance_id, &state.pool).await
                else {
                    break;
                };
                match instance_is_running(&metadata, &state).await {
                    Ok(true) => {
                        tokio::time::sleep(std::time::Duration::from_secs(5))
                            .await;
                    }
                    Ok(false) => {
                        if let Err(error) =
                            reconcile_instance(&instance_id).await
                        {
                            tracing::warn!(
                                "Failed to reconcile synced options after a persisted process exited for {instance_id}: {error}"
                            );
                        }
                        break;
                    }
                    Err(error) => {
                        tracing::warn!(
                            "Failed to inspect the persisted process for {instance_id}: {error}"
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(5))
                            .await;
                    }
                }
            }
        });
    }
    Ok(())
}

pub async fn reconcile_instance(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    reconcile_instance_with_state(&metadata, &state).await
}

pub(crate) async fn prepare_instance_update(
    instance_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
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
        || instance_is_running(metadata, state).await?
    {
        return Ok(());
    }
    let global = get_global_options_with_state(state).await?;
    for option in SyncedOption::ALL {
        if !instance_option_enabled(metadata, option) {
            detach_option(metadata, option, state).await?;
            continue;
        }
        match capability_status(metadata, option, global.get(option), state)
            .await
        {
            CapabilityStatus::Supported => {
                reconcile_option(metadata, option, state).await?
            }
            CapabilityStatus::Unsupported(_) => {
                detach_option(metadata, option, state).await?
            }
            CapabilityStatus::Indeterminate(_) => {}
        }
    }
    Ok(())
}

async fn reconcile_option(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    state: &State,
) -> crate::Result<()> {
    match option {
        SyncedOption::CommandHistory => {
            reconcile_command_history(metadata, state).await
        }
        SyncedOption::CreativeHotbars => {
            reconcile_hotbar(metadata, state).await
        }
        SyncedOption::MultiplayerServers => {
            synced_servers::reconcile_servers(metadata, state).await
        }
        SyncedOption::Screenshots => Ok(()),
    }
}

pub async fn reconcile_changed_file(
    instance_id: &str,
    file_name: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
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
            synced_servers::reconcile_servers(&metadata, &state).await
        }
        _ => Ok(()),
    }
}

pub fn synced_options_path(state: &State) -> PathBuf {
    state.directories.synced_options_dir()
}

pub(crate) async fn remove_generated_instance_files(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    let instance_id = safe_instance_id(instance_id);
    for path in [
        synced_options_path(state)
            .join("hotbars/generated/legacy")
            .join(&instance_id),
        synced_options_path(state)
            .join("hotbars/generated/components")
            .join(&instance_id),
        synced_options_path(state)
            .join("servers/generated")
            .join(&instance_id),
    ] {
        if path.exists() {
            io::remove_dir_all(path).await?;
        }
    }
    Ok(())
}

pub(super) async fn create_synced_directories(
    state: &State,
) -> crate::Result<()> {
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

async fn backup_instance_option_file(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    state: &State,
) -> crate::Result<()> {
    let directory = instance_dir(metadata, state);
    let path = match option {
        SyncedOption::CommandHistory => directory.join(COMMAND_HISTORY_FILE),
        SyncedOption::CreativeHotbars => directory.join(HOTBAR_FILE),
        SyncedOption::MultiplayerServers => directory.join("servers.dat"),
        SyncedOption::Screenshots => return Ok(()),
    };
    if !path.exists() {
        return Ok(());
    }
    let Some(file_name) = path.file_name().and_then(|value| value.to_str())
    else {
        return Ok(());
    };
    backup_bytes(
        &metadata.instance.id,
        file_name,
        &io::read(&path).await?,
        state,
    )
    .await
}

pub(super) async fn backup_bytes(
    owner: &str,
    file_name: &str,
    contents: &[u8],
    state: &State,
) -> crate::Result<()> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let directory = synced_options_path(state)
        .join("backups")
        .join(safe_instance_id(owner))
        .join(timestamp.to_string());
    io::create_dir_all(&directory).await?;
    io::write(directory.join(file_name), contents).await?;
    Ok(())
}

pub(super) async fn seed_from_instance(
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
            let previous_state = read_hotbar_state(state).await?;
            let mut sync_state = HotbarState {
                schema_version: HOTBAR_SCHEMA_VERSION,
                revision: previous_state.revision,
                nbt: NbtCompound::new(),
            };
            let merge_base = empty_hotbar_root();
            if merge_hotbar_family(
                &mut sync_state.nbt,
                family,
                &merge_base,
                &root,
            ) {
                increment_hotbar_revision(&mut sync_state);
            }
            write_hotbar_state(state, &sync_state).await?;
            regenerate_hotbars(state).await?;
        }
        SyncedOption::MultiplayerServers => {
            synced_servers::seed_servers(metadata, state).await?;
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
            synced_servers::ensure_servers(metadata, state).await
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
            let target = instance_dir.join(HOTBAR_FILE);
            let family = hotbar_family(metadata, state).await.ok();
            let source = family
                .map(|family| {
                    generated_hotbar_path(state, family, &metadata.instance.id)
                })
                .unwrap_or_else(|| target.clone());
            detach_link(&source, &target).await
        }
        SyncedOption::MultiplayerServers => {
            synced_servers::detach_servers(metadata, state).await
        }
        SyncedOption::Screenshots => Ok(()),
    }
}

async fn canonical_exists(
    option: SyncedOption,
    state: &State,
) -> crate::Result<bool> {
    Ok(match option {
        SyncedOption::CommandHistory => command_history_path(state).exists(),
        SyncedOption::CreativeHotbars => hotbar_state_exists(state).await?,
        SyncedOption::MultiplayerServers => {
            synced_servers::canonical_exists(state).await?
        }
        SyncedOption::Screenshots => true,
    })
}

pub(super) async fn option_effective(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    state: &State,
) -> crate::Result<bool> {
    if sync_files_are_protected(metadata)
        || instance_is_running(metadata, state).await?
    {
        return Ok(false);
    }
    let global = get_global_options_with_state(state).await?;
    Ok(instance_option_enabled(metadata, option)
        && capability(metadata, option, global.get(option), state)
            .await
            .supported)
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

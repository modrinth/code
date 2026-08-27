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
use std::time::{SystemTime, UNIX_EPOCH};

const COMMAND_HISTORY_FILE: &str = "command_history.txt";
const HOTBAR_FILE: &str = "hotbar.nbt";
const COMMAND_HISTORY_LIMIT: usize = 50;
const COMPONENTS_DATA_VERSION_FLOOR: i32 = 3837;
const HOTBAR_SCHEMA_VERSION: i64 = 2;

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

pub(super) struct SyncCheckpoint {
    pub expected_sha1: String,
    pub merge_base: Option<Vec<u8>>,
    pub source_revision: i64,
    pub status: CheckpointStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CheckpointStatus {
    Pending,
    Ready,
}

impl CheckpointStatus {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "ready" => Some(Self::Ready),
            _ => None,
        }
    }
}

struct HotbarState {
    schema_version: i64,
    revision: i64,
    nbt: NbtCompound,
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

    fn other(self) -> Self {
        match self {
            Self::Legacy => Self::Components,
            Self::Components => Self::Legacy,
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

pub(super) async fn instance_option_supported(
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
                    super::synced_servers::merge_servers_from_instance(
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
            super::synced_servers::reconcile_servers(metadata, state).await
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
            super::synced_servers::reconcile_servers(&metadata, &state).await
        }
        _ => Ok(()),
    }
}

pub async fn get_command_history() -> crate::Result<String> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let path = command_history_path(&state);
    if !path.exists() {
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&io::read(path).await?).into_owned())
}

pub async fn set_command_history(contents: &str) -> crate::Result<String> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    create_synced_directories(&state).await?;
    let normalized = normalize_command_history(contents);
    io::write(command_history_path(&state), normalized.as_bytes()).await?;
    refresh_command_history_links(&state).await?;
    Ok(normalized)
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

async fn backup_shared_hotbars(state: &State) -> crate::Result<()> {
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

async fn backup_bytes(
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
            super::synced_servers::detach_servers(metadata, state).await
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
            super::synced_servers::canonical_exists(state).await?
        }
        SyncedOption::Screenshots => true,
    })
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
    let canonical_bytes = io::read(&canonical).await?;
    let expected = sha1_bytes(&canonical_bytes);
    begin_checkpoint(
        &metadata.instance.id,
        SyncedOption::CommandHistory,
        "default",
        &expected,
        None,
        0,
        state,
    )
    .await?;
    let mode = ensure_link(&canonical, &target).await?;
    finish_checkpoint(
        &metadata.instance.id,
        SyncedOption::CommandHistory,
        "default",
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
    let current_checkpoint = checkpoint(
        &metadata.instance.id,
        SyncedOption::CommandHistory,
        "default",
        state,
    )
    .await?;
    if current_checkpoint
        .as_ref()
        .is_some_and(|value| value.status == CheckpointStatus::Pending)
    {
        return ensure_command_history(metadata, state).await;
    }
    let actual = sha1_file(&local).await?;
    let expected = current_checkpoint.map(|value| value.expected_sha1);
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

async fn merge_command_history_from_instance(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let canonical_path = command_history_path(state);
    let canonical = if canonical_path.exists() {
        String::from_utf8_lossy(&io::read(&canonical_path).await?).into_owned()
    } else {
        String::new()
    };
    let local_path = instance_dir(metadata, state).join(COMMAND_HISTORY_FILE);
    let local = if local_path.exists() {
        String::from_utf8_lossy(&io::read(&local_path).await?).into_owned()
    } else {
        String::new()
    };
    let canonical_lines = canonical.lines().collect::<Vec<_>>();
    let available = COMMAND_HISTORY_LIMIT.saturating_sub(canonical_lines.len());
    let mut seen = canonical_lines
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    let mut imported = Vec::new();
    for line in local.lines().rev() {
        if imported.len() == available {
            break;
        }
        if seen.insert(line) {
            imported.push(line);
        }
    }
    imported.reverse();
    imported.extend(canonical_lines);
    let merged = normalize_command_history(&imported.join("\n"));
    if merged != normalize_command_history(&canonical) {
        io::write(&canonical_path, merged).await?;
        refresh_command_history_links(state).await?;
    }
    Ok(())
}

async fn ensure_hotbar(
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

async fn instance_hotbars_differ_from_synced(
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

fn merge_hotbar_family(
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

async fn regenerate_hotbars(state: &State) -> crate::Result<()> {
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

async fn read_hotbar_state(state: &State) -> crate::Result<HotbarState> {
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

async fn hotbar_state_exists(state: &State) -> crate::Result<bool> {
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

fn hotbar_family_root(
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

fn increment_hotbar_revision(state: &mut HotbarState) {
    state.schema_version = HOTBAR_SCHEMA_VERSION;
    state.revision = state.revision.saturating_add(1);
}

async fn write_hotbar_state(
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

fn command_history_path(state: &State) -> PathBuf {
    synced_options_path(state).join(COMMAND_HISTORY_FILE)
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

pub(super) fn sync_files_are_protected(metadata: &InstanceMetadata) -> bool {
    matches!(
        metadata.instance.install_stage,
        InstanceInstallStage::MinecraftInstalling
            | InstanceInstallStage::PackInstalling
    )
}

pub(super) async fn instance_is_running(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<bool> {
    crate::state::instance_has_running_process(&metadata.instance.id, state)
        .await
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
    let target_metadata = tokio::fs::symlink_metadata(target).await.ok();
    let contents = if target_metadata.is_some() && target.exists() {
        Some(io::read(target).await?)
    } else if source.exists() {
        Some(io::read(source).await?)
    } else {
        None
    };
    if target_metadata.is_some() {
        io::remove_file(target).await?;
    }
    if let Some(contents) = contents {
        io::write(target, contents).await?;
    }
    Ok(())
}

pub(super) async fn begin_checkpoint(
    instance_id: &str,
    option: SyncedOption,
    variant: &str,
    expected_sha1: &str,
    merge_base: Option<&[u8]>,
    source_revision: i64,
    state: &State,
) -> crate::Result<()> {
    let option_name = option.as_str();
    sqlx::query!(
        "
		INSERT INTO instance_sync_checkpoints
			(instance_id, feature, variant, expected_sha1, merge_base,
			 source_revision, status, link_mode)
		VALUES (?, ?, ?, ?, ?, ?, 'pending', NULL)
		ON CONFLICT(instance_id, feature, variant) DO UPDATE SET
			expected_sha1 = excluded.expected_sha1,
			merge_base = excluded.merge_base,
			source_revision = excluded.source_revision,
			status = 'pending',
			link_mode = NULL
		",
        instance_id,
        option_name,
        variant,
        expected_sha1,
        merge_base,
        source_revision,
    )
    .execute(&state.pool)
    .await?;
    Ok(())
}

pub(super) async fn finish_checkpoint(
    instance_id: &str,
    option: SyncedOption,
    variant: &str,
    mode: LinkMode,
    state: &State,
) -> crate::Result<()> {
    let option_name = option.as_str();
    let link_mode = mode.as_str();
    sqlx::query!(
        "
		UPDATE instance_sync_checkpoints
		SET status = 'ready', link_mode = ?
		WHERE instance_id = ? AND feature = ? AND variant = ?
		",
        link_mode,
        instance_id,
        option_name,
        variant,
    )
    .execute(&state.pool)
    .await?;
    Ok(())
}

pub(super) async fn checkpoint(
    instance_id: &str,
    option: SyncedOption,
    variant: &str,
    state: &State,
) -> crate::Result<Option<SyncCheckpoint>> {
    let option_name = option.as_str();
    let row = sqlx::query!(
        r#"
		SELECT expected_sha1, merge_base,
			source_revision AS "source_revision!: i64", status
		FROM instance_sync_checkpoints
		WHERE instance_id = ? AND feature = ? AND variant = ?
		"#,
        instance_id,
        option_name,
        variant,
    )
    .fetch_optional(&state.pool)
    .await?;
    row.map(|row| {
        Ok(SyncCheckpoint {
            expected_sha1: row.expected_sha1,
            merge_base: row.merge_base,
            source_revision: row.source_revision,
            status: CheckpointStatus::from_str(&row.status).ok_or_else(
                || {
                    ErrorKind::InputError(format!(
                        "Unknown sync checkpoint status {}",
                        row.status
                    ))
                },
            )?,
        })
    })
    .transpose()
}

pub(super) async fn sha1_file(path: &Path) -> crate::Result<String> {
    Ok(Sha1::from(io::read(path).await?).digest().to_string())
}

pub(super) fn sha1_bytes(bytes: &[u8]) -> String {
    Sha1::from(bytes).digest().to_string()
}

pub(super) async fn read_nbt_file(path: &Path) -> crate::Result<NbtCompound> {
    let bytes = io::read(path).await?;
    let (root, _) = quartz_nbt::io::read_nbt(
        &mut Cursor::new(bytes),
        quartz_nbt::io::Flavor::Uncompressed,
    )?;
    Ok(root)
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

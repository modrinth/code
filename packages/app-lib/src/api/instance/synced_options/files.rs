use crate::state::{InstanceInstallStage, InstanceMetadata, SyncedOption};
use crate::util::io;
use crate::{ErrorKind, State};
use quartz_nbt::NbtCompound;
use sha1_smol::Sha1;
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub(in crate::api::instance) struct SyncCheckpoint {
    pub expected_sha1: String,
    pub merge_base: Option<Vec<u8>>,
    pub source_revision: i64,
    pub status: CheckpointStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::api::instance) enum CheckpointStatus {
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

#[derive(Clone, Copy, Debug)]
pub(in crate::api::instance) enum LinkMode {
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

pub(in crate::api::instance) fn safe_instance_id(instance_id: &str) -> String {
    instance_id.replace([':', '/', '\\'], "_")
}

pub(in crate::api::instance) fn instance_dir(
    metadata: &InstanceMetadata,
    state: &State,
) -> PathBuf {
    state
        .directories
        .instances_dir()
        .join(&metadata.instance.path)
}

pub(in crate::api::instance) fn sync_files_are_protected(
    metadata: &InstanceMetadata,
) -> bool {
    matches!(
        metadata.instance.install_stage,
        InstanceInstallStage::MinecraftInstalling
            | InstanceInstallStage::PackInstalling
    )
}

pub(in crate::api::instance) async fn instance_is_running(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<bool> {
    crate::state::instance_has_running_process(&metadata.instance.id, state)
        .await
}

pub(in crate::api::instance) fn instance_option_enabled(
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

pub(in crate::api::instance) async fn ensure_link(
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

pub(in crate::api::instance) async fn detach_link(
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

pub(in crate::api::instance) async fn begin_checkpoint(
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

pub(in crate::api::instance) async fn finish_checkpoint(
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

pub(in crate::api::instance) async fn checkpoint(
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

pub(in crate::api::instance) async fn sha1_file(
    path: &Path,
) -> crate::Result<String> {
    Ok(Sha1::from(io::read(path).await?).digest().to_string())
}

pub(in crate::api::instance) fn sha1_bytes(bytes: &[u8]) -> String {
    Sha1::from(bytes).digest().to_string()
}

pub(in crate::api::instance) async fn read_nbt_file(
    path: &Path,
) -> crate::Result<NbtCompound> {
    let bytes = io::read(path).await?;
    let (root, _) = quartz_nbt::io::read_nbt(
        &mut Cursor::new(bytes),
        quartz_nbt::io::Flavor::Uncompressed,
    )?;
    Ok(root)
}

pub(in crate::api::instance) fn nbt_to_bytes(
    root: &NbtCompound,
) -> crate::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    quartz_nbt::io::write_nbt(
        &mut bytes,
        None,
        root,
        quartz_nbt::io::Flavor::Uncompressed,
    )?;
    Ok(bytes)
}

pub(in crate::api::instance) fn nbt_from_bytes(
    bytes: Vec<u8>,
) -> crate::Result<NbtCompound> {
    let (root, _) = quartz_nbt::io::read_nbt(
        &mut Cursor::new(bytes),
        quartz_nbt::io::Flavor::Uncompressed,
    )?;
    Ok(root)
}

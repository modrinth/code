use crate::state::content_store::model::InstancePathContent;
use crate::state::content_store::{FileStorageKind, FileStoragePolicy, input};
use crate::util::content_hash::{
    FileHashes, copy_and_hash, hash_file, temporary_file,
};
use crate::util::fetch;
use std::fs::Metadata;
use std::path::Path;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

pub(crate) async fn symlink_metadata_if_exists(
    path: &Path,
) -> std::io::Result<Option<Metadata>> {
    match fs::symlink_metadata(path).await {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub(crate) async fn sync_directory(path: &Path) -> crate::Result<()> {
    #[cfg(unix)]
    File::open(path).await?.sync_all().await?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

pub(crate) async fn writable_copy(
    source: &Path,
    destination: &Path,
) -> crate::Result<()> {
    fs::copy(source, destination).await?;
    make_writable(destination).await
}

pub(crate) async fn try_reflink(
    source: &Path,
    destination: &Path,
) -> crate::Result<bool> {
    let source = source.to_path_buf();
    let target = destination.to_path_buf();
    let result = tokio::task::spawn_blocking(move || {
        reflink_copy::reflink(source, target)
    })
    .await?;
    match result {
        Ok(()) => {
            make_writable(destination).await?;
            Ok(true)
        }
        Err(error) => {
            // Windows can wrap filesystem errors in an HRESULT. Unwrap it before
            // deciding whether another storage method can work.
            let error = match error.raw_os_error().map(|code| code as u32) {
                Some(code)
                    if cfg!(windows) && code & 0xffff0000 == 0x80070000 =>
                {
                    std::io::Error::from_raw_os_error((code & 0xffff) as i32)
                }
                _ => error,
            };
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                return Err(error.into());
            }
            if symlink_metadata_if_exists(destination).await?.is_some() {
                remove_instance_file(destination).await?;
            }
            if matches!(
                error.kind(),
                std::io::ErrorKind::Unsupported
                    | std::io::ErrorKind::CrossesDevices
                    | std::io::ErrorKind::InvalidInput
                    | std::io::ErrorKind::PermissionDenied
            ) || cfg!(unix) && error.raw_os_error() == Some(25)
                || cfg!(windows) && matches!(error.raw_os_error(), Some(1 | 50))
            {
                Ok(false)
            } else {
                Err(error.into())
            }
        }
    }
}

async fn make_writable(destination: &Path) -> crate::Result<()> {
    let mut permissions = fs::metadata(destination).await?.permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(permissions.mode() | 0o200);
    }
    #[cfg(not(unix))]
    permissions.set_readonly(false);
    fs::set_permissions(destination, permissions).await?;
    Ok(())
}
pub(in crate::state::content_store) async fn move_instance_file(
    source: &Path,
    target: &Path,
) -> crate::Result<()> {
    let metadata = fs::symlink_metadata(source).await?;
    if metadata.is_dir() {
        return Err(input("Refusing to move a directory as content"));
    }
    let parent = target
        .parent()
        .ok_or_else(|| input("Content destination has no parent"))?;
    fs::create_dir_all(parent).await?;
    if symlink_metadata_if_exists(target).await?.is_some() {
        return Err(input("Content destination already exists"));
    }
    fs::rename(source, target).await?;
    if let Some(source_parent) = source.parent() {
        sync_directory(source_parent).await?;
    }
    if source.parent() != Some(parent) {
        sync_directory(parent).await?;
    }
    Ok(())
}

pub(crate) fn link_unavailable(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::Unsupported
    ) || cfg!(windows) && matches!(error.raw_os_error(), Some(1 | 50 | 1314))
}

pub(in crate::state::content_store) async fn try_hardlink(
    source: &Path,
    target: &Path,
) -> crate::Result<bool> {
    match fs::hard_link(source, target).await {
        Ok(()) => Ok(true),
        Err(error)
            if link_unavailable(&error)
                || error.kind() == std::io::ErrorKind::CrossesDevices
                || cfg!(unix) && error.raw_os_error() == Some(31)
                || cfg!(windows) && error.raw_os_error() == Some(1142) =>
        {
            Ok(false)
        }
        Err(error) => Err(error.into()),
    }
}

pub(crate) async fn remove_instance_file(path: &Path) -> crate::Result<()> {
    let metadata = fs::symlink_metadata(path).await?;
    if metadata.is_dir() {
        return Err(input("Refusing to remove a directory as content"));
    }
    #[cfg(windows)]
    if metadata.is_file() && metadata.permissions().readonly() {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || -> std::io::Result<()> {
            use std::os::windows::fs::OpenOptionsExt;
            use windows::Win32::Storage::FileSystem::{
                FILE_READ_ATTRIBUTES, FILE_WRITE_ATTRIBUTES,
            };
            let file = std::fs::OpenOptions::new()
                .access_mode(FILE_READ_ATTRIBUTES.0 | FILE_WRITE_ATTRIBUTES.0)
                .open(&path)?;
            let original = file.metadata()?.permissions();
            let mut writable = original.clone();
            writable.set_readonly(false);
            file.set_permissions(writable)?;
            let removed = std::fs::remove_file(path);
            file.set_permissions(original)?;
            removed
        })
        .await??;
    } else {
        fs::remove_file(path).await?;
    }
    #[cfg(not(windows))]
    fs::remove_file(path).await?;
    if let Some(parent) = path.parent() {
        sync_directory(parent).await?;
    }
    Ok(())
}

/// Returns `None` when sharing is unavailable, without creating a full copy.
/// Background migration uses this to avoid duplicating existing files just to adopt them.
pub(crate) async fn try_shared_file(
    source: &Path,
    destination: &Path,
    policy: FileStoragePolicy,
) -> crate::Result<Option<FileStorageKind>> {
    if try_reflink(source, destination).await? {
        return Ok(Some(FileStorageKind::Reflink));
    }
    if matches!(policy, FileStoragePolicy::Shared)
        && try_hardlink(source, destination).await?
    {
        return Ok(Some(FileStorageKind::Hardlink));
    }
    Ok(None)
}

pub(crate) async fn create_content_file(
    source: &Path,
    destination: &Path,
    policy: FileStoragePolicy,
) -> crate::Result<FileStorageKind> {
    if let Some(kind) = try_shared_file(source, destination, policy).await? {
        return Ok(kind);
    }
    writable_copy(source, destination).await?;
    Ok(FileStorageKind::Copy)
}

/// Rejects linked parent directories because they can redirect an otherwise valid
/// relative path outside the instance or store.
pub(crate) async fn validate_parent_directories(
    root: &Path,
    path: &Path,
) -> crate::Result<()> {
    let parent = path.parent().ok_or_else(|| input("Invalid content path"))?;
    let mut current = root.to_path_buf();
    for component in parent.strip_prefix(root)?.components() {
        current.push(component);
        if let Some(metadata) = symlink_metadata_if_exists(&current).await?
            && !metadata.is_dir()
        {
            return Err(input(
                "Managed content directories must not be symbolic links or files",
            ));
        }
    }
    Ok(())
}

pub(in crate::state::content_store) fn require_staging_space(
    staging: &Path,
    size: u64,
) -> crate::Result<()> {
    if fs4::available_space(staging)? < size.saturating_add(64 * 1024 * 1024) {
        return Err(std::io::Error::new(
			std::io::ErrorKind::StorageFull,
			"There is not enough space to import this file into the shared content store",
		).into());
    }
    Ok(())
}

pub(in crate::state::content_store) async fn stage_file(
    staging: &Path,
    source: &Path,
) -> crate::Result<fetch::StagedDownload> {
    let mut input_file = File::open(source).await?;
    let before = input_file.metadata().await?;
    if !before.is_file() {
        return Err(input("Only regular content files can be stored"));
    }
    require_staging_space(staging, before.len())?;
    let (mut output, temporary) = temporary_file(Some(staging)).await?;
    let FileHashes { sha512, size } =
        copy_and_hash(&mut input_file, &mut output, &|_| {}).await?;
    output.sync_all().await?;
    drop(output);
    let after = input_file.metadata().await?;
    if before.len() != after.len()
        || before.modified()? != after.modified()?
        || size != after.len()
    {
        return Err(input(
            "Content changed while it was being imported; try again after closing the instance",
        ));
    }
    Ok(fetch::StagedDownload {
        path: temporary,
        size,
        sha512,
    })
}

pub(crate) async fn create_instance_file(
    source: &Path,
    target: &Path,
    policy: FileStoragePolicy,
) -> crate::Result<FileStorageKind> {
    let parent = target
        .parent()
        .ok_or_else(|| input("Content destination has no parent"))?;
    fs::create_dir_all(parent).await?;
    if symlink_metadata_if_exists(target).await?.is_some() {
        return Err(input("Content destination already exists"));
    }
    let temporary =
        parent.join(format!(".modrinth-{}.tmp", uuid::Uuid::new_v4()));
    let result = async {
        let storage_kind =
            create_content_file(source, &temporary, policy).await?;
        if matches!(
            storage_kind,
            FileStorageKind::Copy | FileStorageKind::Reflink
        ) {
            fs::File::options()
                .write(true)
                .open(&temporary)
                .await?
                .sync_all()
                .await?;
        }
        fs::rename(&temporary, target).await?;
        sync_directory(parent).await?;
        Ok(storage_kind)
    }
    .await;
    if result.is_err()
        && symlink_metadata_if_exists(&temporary).await?.is_some()
    {
        let _ = remove_instance_file(&temporary).await;
    }
    result
}

pub(in crate::state::content_store) async fn inspect_instance_path(
    path: &Path,
) -> crate::Result<InstancePathContent> {
    Ok(match symlink_metadata_if_exists(path).await? {
        None => InstancePathContent::Missing,
        Some(metadata) if metadata.is_file() => {
            InstancePathContent::File(hash_file(path).await?.sha512)
        }
        Some(_) => InstancePathContent::Conflict,
    })
}

pub(crate) async fn can_share_content(
    staging: &Path,
    source: &Path,
) -> crate::Result<bool> {
    let probe =
        staging.join(format!(".modrinth-share-{}.tmp", uuid::Uuid::new_v4()));
    let result =
        try_shared_file(source, &probe, FileStoragePolicy::Shared).await;
    if symlink_metadata_if_exists(&probe).await?.is_some() {
        remove_instance_file(&probe).await?;
    }
    Ok(result?.is_some())
}

pub(in crate::state::content_store) async fn quarantine_file(
    root: &Path,
    path: &std::path::Path,
    sha512: &str,
) -> crate::Result<()> {
    if symlink_metadata_if_exists(path).await?.is_some() {
        let quarantine = root.join("quarantine");
        fs::create_dir_all(&quarantine).await?;
        fs::rename(
            path,
            quarantine.join(format!("{sha512}-{}", uuid::Uuid::new_v4())),
        )
        .await?;
    }
    Ok(())
}

pub(crate) async fn remove_abandoned_staging(
    staging: &Path,
) -> crate::Result<()> {
    let mut entries = fs::read_dir(staging).await?;
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(".tmp")
            && entry.file_type().await?.is_file()
        {
            fs::remove_file(entry.path()).await?;
        }
    }
    Ok(())
}

pub(in crate::state::content_store) async fn publish_staged_file(
    root: &Path,
    destination: &Path,
    temporary: tempfile::TempPath,
    hash: &str,
) -> crate::Result<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| input("Invalid store path"))?;
    validate_parent_directories(root, destination).await?;
    fs::create_dir_all(parent).await?;
    validate_parent_directories(root, destination).await?;
    if let Some(metadata) = symlink_metadata_if_exists(destination).await? {
        if !metadata.is_file() || hash_file(destination).await?.sha512 != hash {
            return Err(input(format!(
                "Content object {hash} needs repair before it can be replaced"
            )));
        }
    } else {
        let destination_copy = destination.to_path_buf();
        tokio::task::spawn_blocking(move || {
            temporary
                .persist_noclobber(destination_copy)
                .map_err(|error| error.error)
        })
        .await??;
    }
    let mut permissions = fs::metadata(&destination).await?.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&destination, permissions).await?;
    sync_directory(parent).await?;
    Ok(())
}

pub(in crate::state::content_store) async fn modified_at_ns(
    path: &Path,
) -> crate::Result<i64> {
    Ok(crate::state::file_modified_at_ns(&fs::metadata(path).await?)? as i64)
}

pub(in crate::state::content_store) async fn write_staged_bytes(
    file: &mut File,
    bytes: &[u8],
) -> crate::Result<()> {
    file.write_all(bytes).await?;
    file.sync_all().await?;
    Ok(())
}

pub(in crate::state::content_store) async fn quarantined_files(
    root: &Path,
    sha512: &str,
) -> crate::Result<Vec<std::path::PathBuf>> {
    let mut entries = match fs::read_dir(root.join("quarantine")).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(error) => return Err(error.into()),
    };
    let prefix = format!("{sha512}-");
    let mut quarantined = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_name().to_string_lossy().starts_with(&prefix)
            && entry.file_type().await?.is_file()
        {
            quarantined.push(entry.path());
        }
    }
    Ok(quarantined)
}

pub(in crate::state::content_store) async fn matches_any_file(
    path: &Path,
    originals: &[std::path::PathBuf],
) -> crate::Result<bool> {
    let candidate = path.to_path_buf();
    let originals = originals.to_vec();
    Ok(tokio::task::spawn_blocking(move || {
        for original in originals {
            if same_file::is_same_file(&candidate, original)? {
                return Ok::<_, std::io::Error>(true);
            }
        }
        Ok(false)
    })
    .await??)
}

pub(in crate::state::content_store) async fn is_regular_file(
    path: &Path,
    follow_links: bool,
) -> crate::Result<bool> {
    let metadata = if follow_links {
        fs::metadata(path).await?
    } else {
        fs::symlink_metadata(path).await?
    };
    Ok(metadata.is_file())
}

pub(in crate::state::content_store) async fn remove_unused_file(
    path: &Path,
) -> crate::Result<()> {
    if let Some(metadata) = symlink_metadata_if_exists(path).await? {
        if !metadata.is_file() {
            return Err(input(
                "An unreferenced store object was replaced by an unexpected filesystem entry",
            ));
        }
        remove_instance_file(path).await?;
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir(parent).await;
        }
    }
    Ok(())
}

pub(in crate::state::content_store) async fn path_exists(
    path: &Path,
) -> crate::Result<bool> {
    Ok(fs::try_exists(path).await?)
}

pub(in crate::state::content_store) async fn canonical_path(
    path: &Path,
) -> crate::Result<Option<std::path::PathBuf>> {
    match fs::canonicalize(path).await {
        Ok(path) => Ok(Some(path)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

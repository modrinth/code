use super::{FileHashes, FileStorageKind, FileStoragePolicy, input};
use sha2::{Digest, Sha512};
use std::path::{Component, Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
pub(crate) fn validate_digest(hash: &str, length: usize) -> crate::Result<()> {
    if hash.len() != length
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(input("Invalid content hash"));
    }
    Ok(())
}

pub(crate) fn validate_relative(path: &str) -> crate::Result<()> {
    if path.is_empty()
        || path.contains('\\')
        || path.contains(':')
        || path.split('/').any(|part| {
            part.is_empty()
                || part == "."
                || part == ".."
                || part.ends_with('.')
                || part.ends_with(' ')
        })
        || Path::new(path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(input("Invalid instance-relative path"));
    }
    Ok(())
}

pub(crate) fn is_managed_content_path(path: &str) -> bool {
    if validate_relative(path).is_err() {
        return false;
    }
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() != 2 {
        return false;
    }
    let extension = Path::new(parts[1].trim_end_matches(".disabled"))
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    match parts[0] {
        "mods" => extension.eq_ignore_ascii_case("jar"),
        "resourcepacks" | "shaderpacks" | "datapacks" => {
            extension.eq_ignore_ascii_case("zip")
        }
        _ => false,
    }
}

pub(crate) async fn hash_file(path: &Path) -> crate::Result<FileHashes> {
    hash_file_with_progress(path, &|_| {}).await
}

pub(super) async fn hash_file_with_progress(
    path: &Path,
    on_read: &(dyn Fn(u64) + Send + Sync),
) -> crate::Result<FileHashes> {
    let mut file = File::open(path).await?;
    let mut sha512 = Sha512::new();
    let mut sha1 = sha1_smol::Sha1::new();
    let mut size = 0;
    let mut buffer = vec![0; 256 * 1024];
    loop {
        let count = file.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        sha512.update(&buffer[..count]);
        sha1.update(&buffer[..count]);
        size += count as u64;
        on_read(count as u64);
    }
    Ok(FileHashes {
        sha512: format!("{:x}", sha512.finalize()),
        sha1: sha1.hexdigest(),
        size,
    })
}

pub(crate) async fn sync_directory(path: &Path) -> crate::Result<()> {
    #[cfg(unix)]
    File::open(path).await?.sync_all().await?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

pub(crate) fn relative_link(source: &Path, parent: &Path) -> PathBuf {
    let source_parts = source.components().collect::<Vec<_>>();
    let parent_parts = parent.components().collect::<Vec<_>>();
    let common = source_parts
        .iter()
        .zip(&parent_parts)
        .take_while(|(a, b)| a == b)
        .count();
    if common == 0 {
        return source.to_path_buf();
    }
    let mut relative = PathBuf::new();
    for _ in common..parent_parts.len() {
        relative.push("..");
    }
    for part in &source_parts[common..] {
        relative.push(part.as_os_str());
    }
    relative
}

pub(crate) fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            part => normalized.push(part.as_os_str()),
        }
    }
    normalized
}

pub(crate) async fn writable_copy(
    source: &Path,
    destination: &Path,
) -> crate::Result<()> {
    fs::copy(source, destination).await?;
    make_writable(destination).await
}

/// Attempts a copy-on-write clone, returning false when cloning is unavailable.
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
            // Windows bindings can return HRESULT-wrapped Win32 errors.
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
            match fs::symlink_metadata(destination).await {
                Ok(_) => remove_instance_file(destination).await?,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
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
pub(super) async fn move_instance_file(
    source: &Path,
    target: &Path,
) -> crate::Result<()> {
    let metadata = fs::symlink_metadata(source).await?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        return Err(input("Refusing to move a directory as content"));
    }
    let parent = target
        .parent()
        .ok_or_else(|| input("Content destination has no parent"))?;
    fs::create_dir_all(parent).await?;
    if fs::symlink_metadata(target).await.is_ok() {
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

pub(super) async fn try_hardlink(
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
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
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

/// Attempts filesystem sharing without allocating a full fallback copy.
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

/// Validates each parent without following directory links outside the managed root.
pub(crate) async fn validate_parent_directories(
    root: &Path,
    path: &Path,
) -> crate::Result<()> {
    let parent = path.parent().ok_or_else(|| input("Invalid content path"))?;
    let mut current = root.to_path_buf();
    for component in parent.strip_prefix(root)?.components() {
        current.push(component);
        match fs::symlink_metadata(&current).await {
            Ok(metadata) if !metadata.is_dir() => {
                return Err(input(
                    "Managed content directories must not be symbolic links or files",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

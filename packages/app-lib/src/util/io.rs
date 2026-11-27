// IO error
// A wrapper around the tokio IO functions that adds the path to the error message, instead of the uninformative std::io::Error.

use anyhow::{Context, Result};
use std::{
    io::{ErrorKind, Read, Write},
    path::Path,
};
use tempfile::NamedTempFile;
use tokio::task::spawn_blocking;

#[derive(Debug, thiserror::Error)]
pub enum IOError {
    #[error("{source}, path: {path}")]
    IOPathError {
        #[source]
        source: std::io::Error,
        path: String,
    },
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

impl IOError {
    pub fn from(source: std::io::Error) -> Self {
        Self::IOError(source)
    }
    pub fn with_path(
        source: std::io::Error,
        path: impl AsRef<std::path::Path>,
    ) -> Self {
        let path = path.as_ref();

        Self::IOPathError {
            source,
            path: path.to_string_lossy().to_string(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        match self {
            IOError::IOPathError { source, .. } => source.kind(),
            IOError::IOError(source) => source.kind(),
        }
    }
}

pub fn canonicalize(
    path: impl AsRef<std::path::Path>,
) -> Result<std::path::PathBuf, IOError> {
    let path = path.as_ref();
    dunce::canonicalize(path).map_err(|e| IOError::IOPathError {
        source: e,
        path: path.to_string_lossy().to_string(),
    })
}

pub async fn read_dir(
    path: impl AsRef<std::path::Path>,
) -> Result<tokio::fs::ReadDir, IOError> {
    let path = path.as_ref();
    tokio::fs::read_dir(path)
        .await
        .map_err(|e| IOError::IOPathError {
            source: e,
            path: path.to_string_lossy().to_string(),
        })
}

pub async fn create_dir(
    path: impl AsRef<std::path::Path>,
) -> Result<(), IOError> {
    let path = path.as_ref();
    tokio::fs::create_dir(path)
        .await
        .map_err(|e| IOError::IOPathError {
            source: e,
            path: path.to_string_lossy().to_string(),
        })
}

pub async fn create_dir_all(
    path: impl AsRef<std::path::Path>,
) -> Result<(), IOError> {
    let path = path.as_ref();
    tokio::fs::create_dir_all(path)
        .await
        .map_err(|e| IOError::IOPathError {
            source: e,
            path: path.to_string_lossy().to_string(),
        })
}

pub async fn remove_dir_all(
    path: impl AsRef<std::path::Path>,
) -> Result<(), IOError> {
    let path = path.as_ref();
    tokio::fs::remove_dir_all(path)
        .await
        .map_err(|e| IOError::IOPathError {
            source: e,
            path: path.to_string_lossy().to_string(),
        })
}

/// Reads a text file to a string, automatically detecting its encoding and
/// substituting any invalid characters with the Unicode replacement character.
///
/// This function is best suited for reading Minecraft instance files, whose
/// encoding may vary depending on the platform, launchers, client versions
/// (older Minecraft versions tended to rely on the system's default codepage
/// more on Windows platforms), and mods used, while not being highly sensitive
/// to occasional occurrences of mojibake or character replacements.
pub async fn read_any_encoding_to_string(
    path: impl AsRef<std::path::Path>,
) -> Result<(String, &'static encoding_rs::Encoding), IOError> {
    let path = path.as_ref();
    let file_bytes =
        tokio::fs::read(path)
            .await
            .map_err(|e| IOError::IOPathError {
                source: e,
                path: path.to_string_lossy().to_string(),
            })?;

    let file_encoding = {
        let mut encoding_detector = chardetng::EncodingDetector::new(
            chardetng::Iso2022JpDetection::Allow,
        );
        encoding_detector.feed(&file_bytes, true);
        encoding_detector.guess(None, chardetng::Utf8Detection::Allow)
    };

    let (file_string, actual_file_encoding, _) =
        file_encoding.decode(&file_bytes);
    Ok((file_string.to_string(), actual_file_encoding))
}

pub async fn read(
    path: impl AsRef<std::path::Path>,
) -> Result<Vec<u8>, IOError> {
    let path = path.as_ref();
    tokio::fs::read(path)
        .await
        .map_err(|e| IOError::IOPathError {
            source: e,
            path: path.to_string_lossy().to_string(),
        })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenMode {
    Read,
    /// Empties the file, and opens it for appending.
    Truncate,
}

/// Opens a file in a folder which can be modified by the game, such as an instance.
///
/// The game could have replaced the file, or the folders that it's in, with a link to
/// something that it can't access itself. To prevent this, nothing in `path` is allowed
/// to be a link. `folder` can be a link, so the game must not be able to replace it.
///
/// Folders in `path` are created if necessary, unless the file is opened for reading.
#[cfg(unix)]
pub fn open_no_follow(
    folder: &Path,
    path: &Path,
    mode: OpenMode,
) -> std::io::Result<std::fs::File> {
    use rustix::{
        fs::{Mode, OFlags, mkdirat, openat},
        io::Errno,
    };
    use std::{fs::File, os::fd::OwnedFd, path::Component};

    // Without NONBLOCK, opening a pipe would wait for the other end
    const COMMON_FLAGS: OFlags = OFlags::NOFOLLOW
        .union(OFlags::NONBLOCK)
        .union(OFlags::CLOEXEC);
    const FOLDER_FLAGS: OFlags =
        COMMON_FLAGS.union(OFlags::RDONLY).union(OFlags::DIRECTORY);

    let invalid_path =
        || std::io::Error::new(ErrorKind::InvalidInput, "invalid path");
    let names = path
        .components()
        .map(|component| match component {
            Component::Normal(name) => Ok(name),
            _ => Err(invalid_path()),
        })
        .collect::<std::io::Result<Vec<_>>>()?;
    let (file_name, folder_names) =
        names.split_last().ok_or_else(invalid_path)?;

    let (flags, create_mode) = match mode {
        OpenMode::Read => (OFlags::RDONLY, Mode::empty()),
        OpenMode::Truncate => (
            OFlags::WRONLY | OFlags::APPEND | OFlags::CREATE,
            Mode::from_raw_mode(0o666),
        ),
    };
    let flags = COMMON_FLAGS | flags;

    let mut folder: OwnedFd = File::open(folder)?.into();
    let file = if let Some(file) =
        try_openat2(&folder, path, flags, create_mode)?
    {
        file
    } else {
        // Open each folder in order, creating it if it doesn't exist
        for name in folder_names {
            let mut child = openat(&folder, *name, FOLDER_FLAGS, Mode::empty());
            if mode != OpenMode::Read && matches!(child, Err(Errno::NOENT)) {
                match mkdirat(&folder, *name, Mode::from_raw_mode(0o777)) {
                    Ok(()) | Err(Errno::EXIST) => {}
                    Err(err) => return Err(err.into()),
                }
                child = openat(&folder, *name, FOLDER_FLAGS, Mode::empty());
            }
            folder = child?;
        }
        openat(&folder, *file_name, flags, create_mode)?
    };

    let file = File::from(file);
    if !file.metadata()?.is_file() {
        return Err(std::io::Error::other("not a regular file"));
    }
    if mode == OpenMode::Truncate {
        file.set_len(0)?;
    }
    Ok(file)
}

/// Returns `None` if the path can't be opened this way, or if one of the folders that
/// it's in may need to be created.
#[cfg(target_os = "linux")]
fn try_openat2(
    folder: &std::os::fd::OwnedFd,
    path: &Path,
    flags: rustix::fs::OFlags,
    create_mode: rustix::fs::Mode,
) -> std::io::Result<Option<std::os::fd::OwnedFd>> {
    use rustix::{
        fs::{OFlags, ResolveFlags, openat2},
        io::Errno,
    };
    use std::sync::atomic::{AtomicBool, Ordering};

    static SUPPORTS_OPENAT2: AtomicBool = AtomicBool::new(true);
    if !SUPPORTS_OPENAT2.load(Ordering::Relaxed) {
        return Ok(None);
    }

    // Retry up to 3 times if receiving EAGAIN
    for _ in 0..3 {
        let result = openat2(
            folder,
            path,
            flags,
            create_mode,
            ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS,
        );
        match result {
            Ok(file) => return Ok(Some(file)),
            Err(Errno::AGAIN) => {}
            Err(Errno::NOENT) if flags.contains(OFlags::CREATE) => {
                return Ok(None);
            }
            Err(Errno::NOSYS | Errno::PERM) => {
                SUPPORTS_OPENAT2.store(false, Ordering::Relaxed);
                return Ok(None);
            }
            Err(err) => return Err(err.into()),
        }
    }
    Ok(None)
}

#[cfg(all(unix, not(target_os = "linux")))]
fn try_openat2(
    _folder: &std::os::fd::OwnedFd,
    _path: &Path,
    _flags: rustix::fs::OFlags,
    _create_mode: rustix::fs::Mode,
) -> std::io::Result<Option<std::os::fd::OwnedFd>> {
    Ok(None)
}

/// The sandbox can't create links on Windows, other than to folders that it can
/// already write to, so we don't need any special logic here.
#[cfg(not(unix))]
pub fn open_no_follow(
    folder: &Path,
    path: &Path,
    mode: OpenMode,
) -> std::io::Result<std::fs::File> {
    let path = folder.join(path);
    match mode {
        OpenMode::Read => std::fs::File::open(path),
        OpenMode::Truncate => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            // A file opened for appending can't be truncated on Windows
            std::fs::File::create(&path)?;
            std::fs::OpenOptions::new().append(true).open(path)
        }
    }
}

/// Reads a file in a folder which can be modified by the game, see [`open_no_follow`].
pub async fn read_no_follow(
    folder: &Path,
    path: &Path,
) -> Result<Vec<u8>, IOError> {
    let full_path = folder.join(path);
    let folder = folder.to_owned();
    let path = path.to_owned();
    spawn_blocking(move || {
        let mut contents = Vec::new();
        open_no_follow(&folder, &path, OpenMode::Read)?
            .read_to_end(&mut contents)?;
        std::io::Result::Ok(contents)
    })
    .await
    .map_err(|_| std::io::Error::other("background task failed"))?
    .map_err(|e| IOError::with_path(e, full_path))
}

pub async fn write(
    path: impl AsRef<std::path::Path>,
    data: impl AsRef<[u8]>,
) -> Result<(), IOError> {
    let path = path.as_ref().to_owned();
    let data = data.as_ref().to_owned();
    spawn_blocking(move || {
        let cloned_path = path.clone();
        sync_write(data, path).map_err(|e| IOError::IOPathError {
            source: e,
            path: cloned_path.to_string_lossy().to_string(),
        })
    })
    .await
    .map_err(|_| std::io::Error::other("background task failed"))??;

    Ok(())
}

fn sync_write(
    data: impl AsRef<[u8]>,
    path: impl AsRef<Path>,
) -> Result<(), std::io::Error> {
    let mut tempfile =
        NamedTempFile::new_in(path.as_ref().parent().ok_or_else(|| {
            std::io::Error::other(
                "could not get parent directory for temporary file",
            )
        })?)?;
    tempfile.write_all(data.as_ref())?;
    let tmp_path = tempfile.into_temp_path();
    let path = path.as_ref();
    tmp_path.persist(path)?;
    std::io::Result::Ok(())
}

pub fn is_same_disk(old_dir: &Path, new_dir: &Path) -> Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        // we need to use `symlink_metadata` instead of `metadata`, because
        // if this file is a symlink, we need to query the symlink file itself,
        // rather than the target.
        // downloaded JREs use symlinks to point to certain stuff like LICENSE
        // files.
        // this fixes moving JRE dirs.

        let old_meta = std::fs::symlink_metadata(old_dir)
            .with_context(|| format!("getting meta of old dir {old_dir:?}"))?;
        let new_meta = std::fs::symlink_metadata(new_dir)
            .with_context(|| format!("getting meta of new dir {new_dir:?}"))?;

        Ok(old_meta.dev() == new_meta.dev())
    }

    #[cfg(windows)]
    {
        let old_dir = canonicalize(old_dir)
            .with_context(|| format!("canonicalizing {old_dir:?}"))?;
        let new_dir = canonicalize(new_dir)
            .with_context(|| format!("canonicalizing {new_dir:?}"))?;

        let old_component = old_dir.components().next();
        let new_component = new_dir.components().next();

        match (old_component, new_component) {
            (
                Some(std::path::Component::Prefix(old)),
                Some(std::path::Component::Prefix(new)),
            ) => Ok(old.as_os_str() == new.as_os_str()),
            _ => Ok(false),
        }
    }
}

pub async fn rename_or_move(
    from: impl AsRef<std::path::Path>,
    to: impl AsRef<std::path::Path>,
) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    let to_parent = to
        .parent()
        .with_context(|| format!("getting parent of `to` dir {to:?}"))?;
    let same_disk = is_same_disk(from, to_parent).with_context(|| {
        format!("checking if `to_parent` ({to_parent:?}) and `from` ({from:?}) are on the same disk")
    })?;

    if same_disk {
        tokio::fs::rename(from, to)
            .await
            .map_err(|e| IOError::IOPathError {
                source: e,
                path: from.to_string_lossy().to_string(),
            })
            .with_context(|| format!("moving {from:?} to {to:?} on same disk"))
    } else {
        move_recursive(from, to).await.with_context(|| {
            format!("moving {from:?} to {to:?} on different disks")
        })
    }
}

#[async_recursion::async_recursion]
async fn move_recursive(from: &Path, to: &Path) -> Result<()> {
    if from.is_file() {
        copy(from, to)
            .await
            .with_context(|| format!("copying {from:?} to {to:?}"))?;
        remove_file(from).await.with_context(|| {
            format!("removing {from:?} after copying to {to:?}")
        })?;
        return Ok(());
    }

    create_dir(to)
        .await
        .with_context(|| format!("creating dir for {to:?}"))?;

    let mut dir = read_dir(from)
        .await
        .with_context(|| format!("reading dir {from:?}"))?;
    while let Some(entry) = dir
        .next_entry()
        .await
        .with_context(|| format!("reading dir entry in {from:?}"))?
    {
        let new_path = to.join(entry.file_name());
        move_recursive(&entry.path(), &new_path)
            .await
            .with_context(|| {
                format!("moving {:?} to {new_path:?}", entry.path())
            })?;
    }

    Ok(())
}

pub async fn copy(
    from: impl AsRef<std::path::Path>,
    to: impl AsRef<std::path::Path>,
) -> Result<u64, IOError> {
    let from: &Path = from.as_ref();
    let to = to.as_ref();
    tokio::fs::copy(from, to)
        .await
        .map_err(|e| IOError::IOPathError {
            source: e,
            path: from.to_string_lossy().to_string(),
        })
}

pub async fn remove_file(
    path: impl AsRef<std::path::Path>,
) -> Result<(), IOError> {
    let path = path.as_ref();
    tokio::fs::remove_file(path)
        .await
        .map_err(|e| IOError::IOPathError {
            source: e,
            path: path.to_string_lossy().to_string(),
        })
}

pub async fn remove_dir(
    path: impl AsRef<std::path::Path>,
) -> Result<(), IOError> {
    let path = path.as_ref();
    tokio::fs::remove_dir(path)
        .await
        .map_err(|e| IOError::IOPathError {
            source: e,
            path: path.to_string_lossy().to_string(),
        })
}

pub async fn metadata(
    path: impl AsRef<std::path::Path>,
) -> Result<std::fs::Metadata, IOError> {
    let path = path.as_ref();
    tokio::fs::metadata(path)
        .await
        .map_err(|e| IOError::IOPathError {
            source: e,
            path: path.to_string_lossy().to_string(),
        })
}

/// Gets a resource file from the executable. Returns `theseus::Result<(TempDir, PathBuf)>`.
#[macro_export]
macro_rules! get_resource_file {
    (directory: $relative_dir:expr, file: $file_name:expr) => {
        'get_resource_file: {
            let dir = match tempfile::tempdir() {
                Ok(dir) => dir,
                Err(e) => {
                    break 'get_resource_file $crate::Result::Err(
                        $crate::util::io::IOError::from(e).into(),
                    );
                }
            };
            let path = dir.path().join($file_name);
            if let Err(e) = $crate::util::io::write(
                &path,
                include_bytes!(concat!($relative_dir, "/", $file_name)),
            )
            .await
            {
                break 'get_resource_file $crate::Result::Err(e.into());
            }
            let path = match $crate::util::io::canonicalize(path) {
                Ok(path) => path,
                Err(e) => {
                    break 'get_resource_file $crate::Result::Err(e.into());
                }
            };
            $crate::Result::Ok((dir, path))
        }
    };

    ($relative_dir:literal / $file_name:literal) => {
        get_resource_file!(directory: $relative_dir, file: $file_name)
    };

    (env $dir_env_name:literal / $file_name:literal) => {
        get_resource_file!(directory: env!($dir_env_name), file: $file_name)
    };
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = UNITS[0];
    for next_unit in UNITS.iter().skip(1) {
        if value < 1024.0 {
            break;
        }
        value /= 1024.0;
        unit = next_unit;
    }

    if unit == "B" {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {unit}")
    }
}

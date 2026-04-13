// IO error
// A wrapper around the tokio IO functions that adds the path to the error message, instead of the uninformative std::io::Error.

use eyre::{Context, ContextCompat, Result, eyre};
use std::{
    io::{ErrorKind, Write},
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
        let mut encoding_detector = chardetng::EncodingDetector::new();
        encoding_detector.feed(&file_bytes, true);
        encoding_detector.guess(None, true)
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

        use eyre::eyre;

        // we need to use `symlink_metadata` instead of `metadata`, because
        // if this file is a symlink, we need to query the symlink file itself,
        // rather than the target.
        // downloaded JREs use symlinks to point to certain stuff like LICENSE
        // files.
        // this fixes moving JRE dirs.

        let old_meta = std::fs::symlink_metadata(old_dir)
            .wrap_err_with(|| eyre!("getting meta of old dir {old_dir:?}"))?;
        let new_meta = std::fs::symlink_metadata(new_dir)
            .wrap_err_with(|| eyre!("getting meta of new dir {new_dir:?}"))?;

        Ok(old_meta.dev() == new_meta.dev())
    }

    #[cfg(windows)]
    {
        let old_dir = canonicalize(old_dir)
            .wrap_err_with(|| eyre!("canonicalizing {old_dir:?}"))?;
        let new_dir = canonicalize(new_dir)
            .wrap_err_with(|| eyre!("canonicalizing {new_dir:?}"))?;

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
        .wrap_err_with(|| eyre!("getting parent of `to` dir {to:?}"))?;
    let same_disk = is_same_disk(from, to_parent).wrap_err_with(|| {
        eyre!("checking if `to_parent` ({to_parent:?}) and `from` ({from:?}) are on the same disk")
    })?;

    if same_disk {
        tokio::fs::rename(from, to)
            .await
            .map_err(|e| IOError::IOPathError {
                source: e,
                path: from.to_string_lossy().to_string(),
            })
            .wrap_err_with(|| eyre!("moving {from:?} to {to:?} on same disk"))
    } else {
        move_recursive(from, to).await.with_context(|| {
            eyre!("moving {from:?} to {to:?} on different disks")
        })
    }
}

#[async_recursion::async_recursion]
async fn move_recursive(from: &Path, to: &Path) -> Result<()> {
    if from.is_file() {
        copy(from, to)
            .await
            .wrap_err_with(|| eyre!("copying {from:?} to {to:?}"))?;
        remove_file(from).await.wrap_err_with(|| {
            eyre!("removing {from:?} after copying to {to:?}")
        })?;
        return Ok(());
    }

    create_dir(to)
        .await
        .wrap_err_with(|| eyre!("creating dir for {to:?}"))?;

    let mut dir = read_dir(from)
        .await
        .wrap_err_with(|| eyre!("reading dir {from:?}"))?;
    while let Some(entry) = dir
        .next_entry()
        .await
        .wrap_err_with(|| eyre!("reading dir entry in {from:?}"))?
    {
        let new_path = to.join(entry.file_name());
        move_recursive(&entry.path(), &new_path)
            .await
            .with_context(|| {
                eyre!("moving {:?} to {new_path:?}", entry.path())
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

pub async fn open_file(
    path: impl AsRef<std::path::Path>,
) -> Result<tokio::fs::File, IOError> {
    let path = path.as_ref();
    tokio::fs::File::open(path)
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

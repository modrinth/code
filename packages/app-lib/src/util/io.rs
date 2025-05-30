// IO error
// A wrapper around the tokio IO functions that adds the path to the error message, instead of the uninformative std::io::Error.

use std::{io::Write, path::Path};

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
}

// dunce canonicalize
pub fn canonicalize(
    path: impl AsRef<std::path::Path>,
) -> Result<std::path::PathBuf, IOError> {
    let path = path.as_ref();
    dunce::canonicalize(path).map_err(|e| IOError::IOPathError {
        source: e,
        path: path.to_string_lossy().to_string(),
    })
}

// read_dir
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

// create_dir
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

// create_dir_all
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

// remove_dir_all
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

// read_to_string
pub async fn read_to_string(
    path: impl AsRef<std::path::Path>,
) -> Result<String, IOError> {
    let path = path.as_ref();
    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| IOError::IOPathError {
            source: e,
            path: path.to_string_lossy().to_string(),
        })
}

// read
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

// write
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

pub fn is_same_disk(old_dir: &Path, new_dir: &Path) -> Result<bool, IOError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(old_dir.metadata()?.dev() == new_dir.metadata()?.dev())
    }

    #[cfg(windows)]
    {
        let old_dir = canonicalize(old_dir)?;
        let new_dir = canonicalize(new_dir)?;

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

// rename
pub async fn rename_or_move(
    from: impl AsRef<std::path::Path>,
    to: impl AsRef<std::path::Path>,
) -> Result<(), IOError> {
    let from = from.as_ref();
    let to = to.as_ref();

    if to
        .parent()
        .map_or(Ok(false), |to_dir| is_same_disk(from, to_dir))?
    {
        tokio::fs::rename(from, to)
            .await
            .map_err(|e| IOError::IOPathError {
                source: e,
                path: from.to_string_lossy().to_string(),
            })
    } else {
        move_recursive(from, to).await
    }
}

#[async_recursion::async_recursion]
async fn move_recursive(from: &Path, to: &Path) -> Result<(), IOError> {
    if from.is_file() {
        copy(from, to).await?;
        remove_file(from).await?;
        return Ok(());
    }

    create_dir(to).await?;

    let mut dir = read_dir(from).await?;
    while let Some(entry) = dir.next_entry().await? {
        let new_path = to.join(entry.file_name());
        move_recursive(&entry.path(), &new_path).await?;
    }

    Ok(())
}

// copy
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

// remove file
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

// open file
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

// remove dir
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

// metadata
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

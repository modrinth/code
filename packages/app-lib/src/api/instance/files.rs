use crate::state::content_store;
use crate::state::content_store::{
    ReadableContent, content_file_path, validate_relative,
};
use crate::state::instances::adapters::sqlite::{content_rows, instance_rows};
use crate::{ErrorKind, State};
use anyhow::bail;
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use tokio::fs;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceFileItem {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub path: String,
    pub modified: u64,
    pub created: u64,
    pub size: Option<u64>,
    pub count: Option<usize>,
    pub read_only: bool,
}

fn normalized(path: &str) -> &str {
    path.trim_start_matches('/')
}

async fn resolve(
    state: &State,
    instance_id: &str,
    path: &str,
    writing: bool,
) -> crate::Result<ReadableContent> {
    let path = normalized(path);
    let instance = instance_rows::get_instance_by_id(instance_id, &state.pool)
        .await?
        .ok_or(ErrorKind::InputError("unknown instance".to_string()))?;
    let base = state
        .content_store
        .instance_path(&instance.path, "")
        .await?;
    if path.is_empty() {
        if writing {
            bail!(ErrorKind::InputError(
                "the instance directory cannot be changed in the Files tab"
                    .to_string(),
            ));
        }
        if fs::symlink_metadata(&base).await?.file_type().is_symlink() {
            bail!(ErrorKind::InputError(
                "the instance directory must not be a symbolic link"
                    .to_string(),
            ));
        }
        return Ok(ReadableContent::Local(base));
    }
    validate_relative(path)?;
    if writing {
        ManagedPaths::load(state, instance_id)
            .await?
            .ensure_writable(path)?;
    }
    Ok(ReadableContent::Local(
        local_destination(state, &instance.path, path).await?,
    ))
}

async fn local_destination(
    state: &State,
    instance_path: &str,
    path: &str,
) -> crate::Result<PathBuf> {
    let destination = state
        .content_store
        .instance_path(instance_path, path)
        .await?;
    if let Ok(metadata) = fs::symlink_metadata(&destination).await
        && metadata.file_type().is_symlink()
    {
        bail!(ErrorKind::InputError(
            "Files cannot access a symbolic link or its target".to_string()
        ));
    }
    Ok(destination)
}

/// Lowercased paths of an instance's managed content, and every directory
/// containing it, which the Files tab must not write.
struct ManagedPaths {
    files: HashSet<String>,
    directories: HashSet<String>,
}

impl ManagedPaths {
    async fn load(state: &State, instance_id: &str) -> crate::Result<Self> {
        let bindings =
            content_store::instance_storage(&state.pool, instance_id).await?;
        let bound = bindings
            .iter()
            .map(|binding| binding.file_id.as_str())
            .collect::<HashSet<_>>();
        let mut managed = Self {
            files: HashSet::new(),
            directories: HashSet::new(),
        };
        for file in
            content_rows::get_instance_files(instance_id, &state.pool).await?
        {
            if !bound.contains(file.id.as_str()) {
                continue;
            }
            let relative_path = file.relative_path.to_lowercase();
            for (index, _) in relative_path.match_indices('/') {
                managed
                    .directories
                    .insert(relative_path[..index].to_string());
            }
            managed
                .files
                .insert(content_file_path(&file).to_lowercase());
            managed.files.insert(relative_path);
        }
        Ok(managed)
    }

    fn ensure_writable(&self, path: &str) -> crate::Result<()> {
        if path
            .split('/')
            .next()
            .is_some_and(|part| part.eq_ignore_ascii_case("mods"))
        {
            bail!(ErrorKind::InputError(
                "the mods folder is read-only in Files; manage mods from the Content tab"
                    .to_string(),
            ));
        }
        let path = path.to_lowercase();
        if self.files.contains(&path) || self.directories.contains(&path) {
            bail!(ErrorKind::InputError(
                "managed content is read-only in Files; use the Content tab"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

/// Writes many files into one instance, checking each against managed content
/// loaded once rather than per file.
pub(crate) struct InstanceFileWriter {
    instance_id: String,
    instance_path: String,
    managed: ManagedPaths,
}

impl InstanceFileWriter {
    pub(crate) async fn new(
        state: &State,
        instance_id: &str,
    ) -> crate::Result<Self> {
        let instance =
            instance_rows::get_instance_by_id(instance_id, &state.pool)
                .await?
                .ok_or(ErrorKind::InputError("unknown instance".to_string()))?;
        Ok(Self {
            instance_id: instance_id.to_string(),
            instance_path: instance.path,
            managed: ManagedPaths::load(state, instance_id).await?,
        })
    }

    pub(crate) async fn destination(
        &self,
        state: &State,
        path: &str,
    ) -> crate::Result<PathBuf> {
        let path = normalized(path);
        if path.is_empty() {
            bail!(ErrorKind::InputError(
                "the instance directory cannot be changed in the Files tab"
                    .to_string(),
            ));
        }
        validate_relative(path)?;
        self.managed.ensure_writable(path)?;
        local_destination(state, &self.instance_path, path).await
    }

    /// Streams `reader` into an instance file, replacing any file already
    /// there. The contents are only kept when exactly `expected_size` bytes
    /// are read, and the instance is only locked to move the finished file
    /// into place.
    pub(crate) async fn write_from_reader(
        &self,
        state: &State,
        path: &str,
        reader: impl tokio::io::AsyncRead + Unpin,
        expected_size: u64,
    ) -> crate::Result<()> {
        use tokio::io::AsyncReadExt;

        let destination = self.destination(state, path).await?;
        let parent = destination.parent().ok_or(ErrorKind::InputError(
            "invalid file destination".to_string(),
        ))?;
        fs::create_dir_all(parent).await?;
        let parent = parent.to_path_buf();
        let temporary = tokio::task::spawn_blocking(move || {
            tempfile::NamedTempFile::new_in(parent)
                .map(|file| file.into_temp_path())
        })
        .await??;
        let mut file = fs::File::options().write(true).open(&temporary).await?;
        let written = tokio::io::copy(
            &mut reader.take(expected_size.saturating_add(1)),
            &mut file,
        )
        .await?;
        if written != expected_size {
            bail!("{path} has an unexpected size");
        }
        file.sync_all().await?;
        drop(file);

        let _instance =
            state.instance_locks.lock_content(&self.instance_id).await;
        let _files = state.content_store.files_lock.lock().await;
        self.destination(state, path).await?;
        tokio::task::spawn_blocking(move || {
            temporary.persist(destination).map_err(|error| error.error)
        })
        .await??;
        Ok(())
    }
}

pub async fn list_instance_files(
    instance_id: &str,
    path: &str,
) -> crate::Result<Vec<InstanceFileItem>> {
    let state = State::get().await?;
    let directory = resolve(&state, instance_id, path, false).await?;
    let mut entries = fs::read_dir(directory.path()).await?;
    let mut output = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name().to_string_lossy().into_owned();

        // exclude macOS .DS_Store files
        if name == ".DS_Store" {
            continue;
        }

        let relative = if normalized(path).is_empty() {
            name.clone()
        } else {
            format!("{}/{name}", normalized(path))
        };
        let file_type = entry.file_type().await?;
        let resolved = resolve(&state, instance_id, &relative, false).await;
        let metadata = match &resolved {
            Ok(content) => fs::metadata(content.path()).await.ok(),
            Err(_) => None,
        };
        let read_only =
            resolve(&state, instance_id, &relative, true).await.is_err();
        let count = if file_type.is_dir() && !file_type.is_symlink() {
            let mut children = fs::read_dir(entry.path()).await?;
            let mut count = 0;
            while children.next_entry().await?.is_some() {
                count += 1;
            }
            Some(count)
        } else {
            None
        };
        output.push(InstanceFileItem {
            name,
            path: relative,
            kind: if file_type.is_dir() {
                "directory"
            } else if resolved.is_err() && file_type.is_symlink() {
                "symlink"
            } else {
                "file"
            }
            .to_string(),
            modified: metadata
                .as_ref()
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |time| time.as_secs()),
            created: metadata
                .as_ref()
                .and_then(|metadata| metadata.created().ok())
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |time| time.as_secs()),
            size: metadata
                .as_ref()
                .filter(|metadata| metadata.is_file())
                .map(|metadata| metadata.len()),
            count,
            read_only,
        });
    }
    Ok(output)
}

pub async fn read_instance_file(
    instance_id: &str,
    path: &str,
) -> crate::Result<Vec<u8>> {
    let state = State::get().await?;
    let content = resolve(&state, instance_id, path, false).await?;
    Ok(fs::read(content.path()).await?)
}

pub async fn validate_instance_file_write(
    instance_id: &str,
    path: &str,
) -> crate::Result<PathBuf> {
    let state = State::get().await?;
    instance_file_write_path(&state, instance_id, path).await
}

async fn instance_file_write_path(
    state: &State,
    instance_id: &str,
    path: &str,
) -> crate::Result<PathBuf> {
    Ok(resolve(state, instance_id, path, true)
        .await?
        .path()
        .to_path_buf())
}

pub async fn write_instance_file(
    instance_id: &str,
    path: &str,
    bytes: &[u8],
    create_only: bool,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _instance = state.instance_locks.lock_content(instance_id).await;
    let _files = state.content_store.files_lock.lock().await;
    let destination = resolve(&state, instance_id, path, true)
        .await?
        .path()
        .to_path_buf();
    if create_only && fs::symlink_metadata(&destination).await.is_ok() {
        bail!(ErrorKind::InputError(
            "a file already exists at this path".to_string()
        ));
    }
    let parent = destination.parent().ok_or(ErrorKind::InputError(
        "invalid file destination".to_string(),
    ))?;
    fs::create_dir_all(parent).await?;
    let parent = parent.to_path_buf();
    let temporary = tokio::task::spawn_blocking(move || {
        tempfile::NamedTempFile::new_in(parent)
            .map(|file| file.into_temp_path())
    })
    .await??;
    fs::write(&temporary, bytes).await?;
    fs::File::options()
        .write(true)
        .open(&temporary)
        .await?
        .sync_all()
        .await?;
    resolve(&state, instance_id, path, true).await?;
    tokio::task::spawn_blocking(move || {
        if create_only {
            temporary.persist_noclobber(destination)
        } else {
            temporary.persist(destination)
        }
        .map_err(|error| error.error)
    })
    .await??;
    Ok(())
}

pub async fn create_instance_directory(
    instance_id: &str,
    path: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _instance = state.instance_locks.lock_content(instance_id).await;
    let _files = state.content_store.files_lock.lock().await;
    let destination = resolve(&state, instance_id, path, true).await?;
    fs::create_dir(destination.path()).await?;
    Ok(())
}

pub async fn rename_instance_file(
    instance_id: &str,
    source: &str,
    destination: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _instance = state.instance_locks.lock_content(instance_id).await;
    let _files = state.content_store.files_lock.lock().await;
    let source = resolve(&state, instance_id, source, true).await?;
    let destination = resolve(&state, instance_id, destination, true).await?;
    if fs::symlink_metadata(destination.path()).await.is_ok() {
        bail!(ErrorKind::InputError(
            "the destination already exists".to_string()
        ));
    }
    fs::rename(source.path(), destination.path()).await?;
    Ok(())
}

pub async fn delete_instance_file(
    instance_id: &str,
    path: &str,
    recursive: bool,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _instance = state.instance_locks.lock_content(instance_id).await;
    let _files = state.content_store.files_lock.lock().await;
    let path = resolve(&state, instance_id, path, true).await?;
    if fs::symlink_metadata(path.path()).await?.is_dir() {
        if recursive {
            fs::remove_dir_all(path.path()).await?;
        } else {
            fs::remove_dir(path.path()).await?;
        }
    } else {
        fs::remove_file(path.path()).await?;
    }
    Ok(())
}

pub async fn save_instance_file_as(
    instance_id: &str,
    source: &str,
    destination: &Path,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _files = state.content_store.files_lock.lock().await;
    let parent = destination.parent().ok_or(ErrorKind::InputError(
        "invalid save destination".to_string(),
    ))?;
    let canonical_parent = fs::canonicalize(parent).await?;
    let store = fs::canonicalize(state.directories.store_dir()).await?;
    let profiles = fs::canonicalize(state.directories.instances_dir()).await?;
    if canonical_parent.starts_with(&store)
        || canonical_parent.starts_with(&profiles)
    {
        bail!(ErrorKind::InputError(
            "save a copy outside the store and instance directories"
                .to_string(),
        ));
    }
    if fs::symlink_metadata(destination)
        .await
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        bail!(ErrorKind::InputError(
            "cannot save over a symbolic link".to_string()
        ));
    }
    let content = resolve(&state, instance_id, source, false).await?;
    fs::copy(content.path(), destination).await?;
    Ok(())
}

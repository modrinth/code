use crate::State;
use crate::state::content_store::{
    FileContent, ReadableContent, catalog, content_file_path, input,
    validate_relative,
};
use crate::state::instances::adapters::sqlite::{content_rows, instance_rows};
use serde::Serialize;
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
        .ok_or_else(|| input("Unknown instance"))?;
    let base = state.directories.instances_dir().join(&instance.path);
    if path.is_empty() {
        if writing {
            return Err(input(
                "The instance directory cannot be changed in the Files tab",
            ));
        }
        if fs::symlink_metadata(&base).await?.file_type().is_symlink() {
            return Err(input(
                "The instance directory must not be a symbolic link",
            ));
        }
        return Ok(ReadableContent::Local(base));
    }
    validate_relative(path)?;
    if writing {
        if path
            .split('/')
            .next()
            .is_some_and(|part| part.eq_ignore_ascii_case("mods"))
        {
            return Err(input(
                "The mods folder is read-only in Files. Manage mods from the Content tab",
            ));
        }
        let prefix = format!("{}/", path.to_lowercase());
        let bindings =
            catalog::instance_storage(&state.pool, instance_id).await?;
        let files =
            content_rows::get_instance_files(instance_id, &state.pool).await?;
        if files.iter().any(|file| {
            bindings.iter().any(|binding| binding.file_id == file.id)
                && (file.relative_path.eq_ignore_ascii_case(path)
                    || content_file_path(file).eq_ignore_ascii_case(path)
                    || file.relative_path.to_lowercase().starts_with(&prefix))
        }) {
            return Err(input(
                "Managed content is read-only in Files. Use the Content tab",
            ));
        }
    }
    let destination = state
        .content_store
        .instance_path(&instance.path, path)
        .await?;
    if let Ok(metadata) = fs::symlink_metadata(&destination).await
        && metadata.file_type().is_symlink()
    {
        if writing {
            return Err(input(
                "Files cannot modify a symbolic link or its target",
            ));
        }
        let file = content_rows::get_instance_file_by_relative_path(
            instance_id,
            path.trim_end_matches(".disabled"),
            &state.pool,
        )
        .await?
        .ok_or_else(|| input("Files cannot read an unmanaged symbolic link"))?;
        if content_file_path(&file) != path {
            return Err(input(
                "The content link is not at its registered path",
            ));
        }
        let FileContent::Stored { stored_file, .. } =
            state.content_store.file_content(&file).await?
        else {
            return Err(input("Managed content needs repair"));
        };
        if !state
            .content_store
            .instance_file_matches(&destination, &stored_file.metadata.sha512)
            .await?
        {
            return Err(input(
                "The content link points to an unexpected target",
            ));
        }
        return Ok(ReadableContent::Stored(stored_file));
    }
    Ok(ReadableContent::Local(destination))
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
    Ok(resolve(&state, instance_id, path, true)
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
    let _instance = state.lock_instance_content(instance_id).await;
    let _files = state.content_store.files_lock.lock().await;
    let destination = resolve(&state, instance_id, path, true)
        .await?
        .path()
        .to_path_buf();
    if create_only && fs::symlink_metadata(&destination).await.is_ok() {
        return Err(input("A file already exists at this path"));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| input("Invalid file destination"))?;
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
    let _instance = state.lock_instance_content(instance_id).await;
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
    let _instance = state.lock_instance_content(instance_id).await;
    let _files = state.content_store.files_lock.lock().await;
    let source = resolve(&state, instance_id, source, true).await?;
    let destination = resolve(&state, instance_id, destination, true).await?;
    if fs::symlink_metadata(destination.path()).await.is_ok() {
        return Err(input("The destination already exists"));
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
    let _instance = state.lock_instance_content(instance_id).await;
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
    let parent = destination
        .parent()
        .ok_or_else(|| input("Invalid save destination"))?;
    let canonical_parent = fs::canonicalize(parent).await?;
    let store = fs::canonicalize(state.directories.store_dir()).await?;
    let profiles = fs::canonicalize(state.directories.instances_dir()).await?;
    if canonical_parent.starts_with(&store)
        || canonical_parent.starts_with(&profiles)
    {
        return Err(input(
            "Save a copy outside the store and instance directories",
        ));
    }
    if fs::symlink_metadata(destination)
        .await
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err(input("Cannot save over a symbolic link"));
    }
    let content = resolve(&state, instance_id, source, false).await?;
    fs::copy(content.path(), destination).await?;
    Ok(())
}

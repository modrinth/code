use crate::state::instances::{
    ContentRequirement, ContentSourceKind, Instance, InstanceFile,
    adapters::sqlite::{content_rows, instance_rows},
};
use crate::state::{
    CachedEntry, KnownModrinthFile, ProjectType, State, cache_file_hash,
};
use crate::util::fetch::{self, DownloadMeta, DownloadReason};
use crate::util::io;
use bytes::Bytes;
use std::path::{Path, PathBuf};

pub(crate) struct ContentScope {
    pub instance: Instance,
    pub content_set_id: String,
}

pub(crate) struct InstalledContentFile {
    pub relative_path: String,
    pub project_id: Option<String>,
    pub enabled: bool,
}

pub(crate) async fn resolve_content_scope(
    instance_id: &str,
    content_set_id: Option<&str>,
    state: &State,
) -> crate::Result<ContentScope> {
    let instance = instance_rows::get_instance_by_id(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let content_set_id = match content_set_id {
        Some(id) => id.to_string(),
        None => instance.applied_content_set_id.clone().ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "Instance {} has no applied content set",
                instance.id
            ))
        })?,
    };

    Ok(ContentScope {
        instance,
        content_set_id,
    })
}

pub(crate) async fn add_project_from_version(
    instance_id: &str,
    version_id: &str,
    reason: DownloadReason,
    dependent_on_version_id: Option<String>,
    source_kind: ContentSourceKind,
    state: &State,
) -> crate::Result<String> {
    let scope = resolve_content_scope(instance_id, None, state).await?;
    let content_set =
        content_rows::get_content_set(&scope.content_set_id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Unknown content set {}",
                    scope.content_set_id
                ))
            })?;
    let version = CachedEntry::get_version(
        version_id,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Unable to install version id {version_id}. Not found."
        ))
    })?;
    let file = version
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "No files for input version present!".to_string(),
            )
        })?;
    let download_meta = DownloadMeta {
        reason,
        game_version: content_set.game_version,
        loader: content_set.loader.as_str().to_string(),
        dependent_on: dependent_on_version_id,
    };
    let bytes = fetch::fetch(
        &file.url,
        file.hashes.get("sha1").map(|hash| hash.as_str()),
        Some(&download_meta),
        None,
        &state.fetch_semaphore,
        &state.pool,
    )
    .await?;
    let project_type = ProjectType::get_from_loaders(version.loaders.clone())
        .ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Unable to infer project type for version {version_id}"
        ))
    })?;

    add_project_bytes(
        &scope.instance.id,
        &file.filename,
        bytes,
        file.hashes.get("sha1").map(|hash| hash.as_str()),
        Some(project_type),
        source_kind,
        Some(version.project_id.as_str()),
        Some(version.id.as_str()),
        state,
    )
    .await
}

pub(crate) async fn add_project_from_path(
    instance_id: &str,
    path: &Path,
    project_type: Option<ProjectType>,
    state: &State,
) -> crate::Result<String> {
    let file = io::read(path).await?;
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    add_project_bytes(
        instance_id,
        &file_name,
        Bytes::from(file),
        None,
        project_type,
        ContentSourceKind::Local,
        None,
        None,
        state,
    )
    .await
}

pub(crate) async fn add_project_bytes(
    instance_id: &str,
    file_name: &str,
    bytes: Bytes,
    hash: Option<&str>,
    project_type: Option<ProjectType>,
    source_kind: ContentSourceKind,
    project_id: Option<&str>,
    version_id: Option<&str>,
    state: &State,
) -> crate::Result<String> {
    let scope = resolve_content_scope(instance_id, None, state).await?;
    let project_type = match project_type {
        Some(project_type) => project_type,
        None => infer_project_type(&bytes)?,
    };
    let relative_path = format!("{}/{}", project_type.get_folder(), file_name);
    let full_path =
        instance_full_path(state, &scope.instance).join(&relative_path);
    let sha1 = match hash {
        Some(hash) => hash.to_string(),
        None => fetch::sha1_async(bytes.clone()).await?,
    };

    cache_file_hash(
        bytes.clone(),
        &scope.instance.id,
        &relative_path,
        Some(&sha1),
        Some(project_type),
        project_id.zip(version_id).map(|(project_id, version_id)| {
            KnownModrinthFile {
                project_id,
                version_id,
            }
        }),
        &state.pool,
    )
    .await?;
    fetch::write(&full_path, &bytes, &state.io_semaphore).await?;

    let file = content_rows::upsert_instance_file_from_parts(
        content_rows::UpsertInstanceFile {
            instance_id: &scope.instance.id,
            relative_path: &relative_path,
            file_name,
            enabled: !relative_path.ends_with(".disabled"),
            sha1: &sha1,
            size: bytes.len() as u64,
            missing: false,
        },
        &state.pool,
    )
    .await?;
    upsert_entry_for_file(
        &scope,
        &file,
        project_type,
        project_id,
        version_id,
        source_kind,
        &state.pool,
    )
    .await?;

    Ok(relative_path)
}

pub(crate) async fn record_project_file(
    instance_id: &str,
    relative_path: &str,
    sha1: &str,
    size: u64,
    project_type: ProjectType,
    source_kind: ContentSourceKind,
    project_id: Option<&str>,
    version_id: Option<&str>,
    state: &State,
) -> crate::Result<()> {
    let scope = resolve_content_scope(instance_id, None, state).await?;
    let file_name = Path::new(relative_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let file = content_rows::upsert_instance_file_from_parts(
        content_rows::UpsertInstanceFile {
            instance_id: &scope.instance.id,
            relative_path,
            file_name: &file_name,
            enabled: !relative_path.ends_with(".disabled"),
            sha1,
            size,
            missing: false,
        },
        &state.pool,
    )
    .await?;
    upsert_entry_for_file(
        &scope,
        &file,
        project_type,
        project_id,
        version_id,
        source_kind,
        &state.pool,
    )
    .await
}

pub(crate) async fn toggle_disable_project(
    instance_id: &str,
    project_path: &str,
    state: &State,
) -> crate::Result<String> {
    let scope = resolve_content_scope(instance_id, None, state).await?;
    let base = instance_full_path(state, &scope.instance);
    let trimmed = project_path.trim_end_matches(".disabled");
    let current_path = if base.join(project_path).exists() {
        project_path.to_string()
    } else if base.join(format!("{trimmed}.disabled")).exists() {
        format!("{trimmed}.disabled")
    } else if base.join(trimmed).exists() {
        trimmed.to_string()
    } else {
        return Err(crate::ErrorKind::FSError(format!(
            "Could not find project file for '{project_path}' in instance"
        ))
        .into());
    };
    let new_path = if current_path.ends_with(".disabled") {
        current_path.trim_end_matches(".disabled").to_string()
    } else {
        format!("{current_path}.disabled")
    };

    io::rename_or_move(&base.join(&current_path), &base.join(&new_path))
        .await?;

    let file_name = Path::new(&new_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let enabled = !new_path.ends_with(".disabled");
    let file = match content_rows::rename_instance_file(
        &scope.instance.id,
        &current_path,
        &new_path,
        &file_name,
        enabled,
        &state.pool,
    )
    .await?
    {
        Some(file) => file,
        None => index_existing_file(&scope, &new_path, state).await?,
    };
    content_rows::set_content_entry_enabled_for_file(
        &scope.content_set_id,
        &file.id,
        enabled,
        &state.pool,
    )
    .await?;

    Ok(new_path)
}

pub(crate) async fn remove_project(
    instance_id: &str,
    project_path: &str,
    state: &State,
) -> crate::Result<()> {
    let scope = resolve_content_scope(instance_id, None, state).await?;
    let base = instance_full_path(state, &scope.instance);
    let file = content_rows::get_instance_file_by_relative_path(
        &scope.instance.id,
        project_path,
        &state.pool,
    )
    .await?;

    io::remove_file(base.join(project_path)).await?;

    if let Some(file) = file {
        content_rows::remove_content_entries_for_file(
            &scope.content_set_id,
            &file.id,
            &state.pool,
        )
        .await?;
        content_rows::remove_instance_file_by_relative_path(
            &scope.instance.id,
            project_path,
            &state.pool,
        )
        .await?;
    }

    Ok(())
}

pub(crate) async fn list_project_files(
    instance_id: &str,
    state: &State,
) -> crate::Result<Vec<InstalledContentFile>> {
    let scope = resolve_content_scope(instance_id, None, state).await?;
    let entries =
        content_rows::get_content_entries(&scope.content_set_id, &state.pool)
            .await?;
    let files =
        content_rows::get_instance_files(&scope.instance.id, &state.pool)
            .await?
            .into_iter()
            .map(|file| (file.id.clone(), file))
            .collect::<std::collections::HashMap<_, _>>();

    Ok(entries
        .into_iter()
        .filter_map(|entry| {
            let file = files.get(entry.file_id.as_ref()?)?;
            Some(InstalledContentFile {
                relative_path: file.relative_path.clone(),
                project_id: entry.project_id,
                enabled: entry.enabled && file.enabled,
            })
        })
        .collect())
}

pub(crate) fn instance_full_path(
    state: &State,
    instance: &Instance,
) -> PathBuf {
    state.directories.instances_dir().join(&instance.path)
}

async fn index_existing_file(
    scope: &ContentScope,
    relative_path: &str,
    state: &State,
) -> crate::Result<InstanceFile> {
    let full_path =
        instance_full_path(state, &scope.instance).join(relative_path);
    let (size, sha1) = fetch::sha1_file_async(&full_path).await?;
    let file_name = Path::new(relative_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let project_type = ProjectType::get_from_parent_folder(relative_path)
        .ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "Unable to infer project type from {relative_path}"
            ))
        })?;

    let file = content_rows::upsert_instance_file_from_parts(
        content_rows::UpsertInstanceFile {
            instance_id: &scope.instance.id,
            relative_path,
            file_name: &file_name,
            enabled: !relative_path.ends_with(".disabled"),
            sha1: &sha1,
            size,
            missing: false,
        },
        &state.pool,
    )
    .await?;
    upsert_entry_for_file(
        scope,
        &file,
        project_type,
        None,
        None,
        ContentSourceKind::Local,
        &state.pool,
    )
    .await?;

    Ok(file)
}

async fn upsert_entry_for_file(
    scope: &ContentScope,
    file: &InstanceFile,
    project_type: ProjectType,
    project_id: Option<&str>,
    version_id: Option<&str>,
    source_kind: ContentSourceKind,
    pool: &sqlx::SqlitePool,
) -> crate::Result<()> {
    content_rows::upsert_content_entry_from_parts(
        content_rows::UpsertContentEntry {
            instance_id: &scope.instance.id,
            content_set_id: &scope.content_set_id,
            file_id: Some(&file.id),
            project_type,
            project_id,
            version_id,
            source_kind,
            server_requirement: ContentRequirement::Required,
            client_requirement: ContentRequirement::Required,
            enabled: file.enabled,
        },
        pool,
    )
    .await?;

    Ok(())
}

fn infer_project_type(bytes: &Bytes) -> crate::Result<ProjectType> {
    let cursor = std::io::Cursor::new(&**bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|_| {
        crate::ErrorKind::InputError(
            "Unable to infer project type for input file".to_string(),
        )
    })?;

    if archive.by_name("fabric.mod.json").is_ok()
        || archive.by_name("quilt.mod.json").is_ok()
        || archive.by_name("META-INF/neoforge.mods.toml").is_ok()
        || archive.by_name("META-INF/mods.toml").is_ok()
        || archive.by_name("mcmod.info").is_ok()
    {
        Ok(ProjectType::Mod)
    } else if archive.by_name("pack.mcmeta").is_ok() {
        if archive.file_names().any(|name| name.starts_with("data/")) {
            Ok(ProjectType::DataPack)
        } else {
            Ok(ProjectType::ResourcePack)
        }
    } else if archive
        .file_names()
        .any(|name| name.starts_with("shaders/"))
    {
        Ok(ProjectType::ShaderPack)
    } else {
        Err(crate::ErrorKind::InputError(
            "Unable to infer project type for input file".to_string(),
        )
        .into())
    }
}

use crate::state::content_store::adapters::sqlite as catalog;
use crate::state::content_store::adapters::{downloads, filesystem};
use crate::state::content_store::{
    FileStorageKind, InstanceFileStatus, content_file_path, input,
};
use crate::state::dirs::move_app_directory::{
    relocate_tree, remove_migrated_tree,
};
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::{Instance, InstanceFile, InstanceInstallStage, State};
use futures::{StreamExt, stream};
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;

const COPY_CONCURRENCY: usize = 4;

struct PreparedCopy {
    temporary: tempfile::TempPath,
    source: std::fs::File,
    metadata: std::fs::Metadata,
    hashes: crate::util::content_hash::FileHashes,
}

async fn prepare_copy(path: &Path) -> crate::Result<PreparedCopy> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || -> crate::Result<PreparedCopy> {
        let parent =
            path.parent().ok_or_else(|| input("Invalid content path"))?;
        let mut source = std::fs::File::open(&path)?;
        let metadata = source.metadata()?;
        if !metadata.is_file()
            || std::fs::symlink_metadata(&path)?.file_type().is_symlink()
        {
            return Err(input("Managed content is not a regular file"));
        }
        filesystem::require_staging_space(parent, metadata.len())?;
        let (mut output, temporary) =
            tempfile::NamedTempFile::new_in(parent)?.into_parts();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            output.set_permissions(std::fs::Permissions::from_mode(
                metadata.permissions().mode() | 0o200,
            ))?;
        }
        let mut hasher = crate::util::content_hash::ContentHasher::default();
        let mut buffer = vec![0; 256 * 1024];
        let mut size = 0;
        loop {
            let read = source.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            output.write_all(&buffer[..read])?;
            hasher.update(&buffer[..read]);
            size += read as u64;
        }
        output.sync_all()?;
        Ok(PreparedCopy {
            temporary,
            source,
            metadata,
            hashes: hasher.finish(size),
        })
    })
    .await?
}

fn source_unchanged(
    prepared: &PreparedCopy,
    path: &Path,
) -> crate::Result<bool> {
    let current = std::fs::symlink_metadata(path)?;
    if !current.is_file()
        || current.len() != prepared.metadata.len()
        || current.modified()? != prepared.metadata.modified()?
    {
        return Ok(false);
    }
    Ok(same_file::Handle::from_file(prepared.source.try_clone()?)?
        == same_file::Handle::from_path(path)?)
}

pub(crate) async fn migrate(state: &State) -> crate::Result<()> {
    let store = &state.content_store;
    if catalog::setting(&state.pool, "game_locales_layout_version")
        .await?
        .as_deref()
        != Some("1")
    {
        let source = state.directories.metadata_dir().join("game-locales");
        let destination = state.directories.caches_dir().join("game-locales");
        relocate_tree(&source, &destination).await?;
        if filesystem::path_exists(&source).await?
            && filesystem::path_exists(&destination).await?
        {
            remove_migrated_tree(&source).await?;
        }
        catalog::set_setting(&state.pool, "game_locales_layout_version", "1")
            .await?;
    }
    catalog::set_setting(&state.pool, "store_layout_version", "1").await?;
    for owner in catalog::retained_owners(&state.pool, "rollback").await? {
        if uuid::Uuid::parse_str(&owner).is_ok()
            && !filesystem::path_exists(
                &state.directories.install_backups_dir().join(&owner),
            )
            .await?
        {
            store.release("rollback", &owner).await?;
        }
    }
    store.remove_abandoned_staging().await?;
    store.recover_unregistered_files().await?;
    Ok(())
}

pub(crate) async fn migrate_instance_copies(
    state: &State,
) -> crate::Result<()> {
    let store = &state.content_store;
    for instance in
        crate::state::instances::load_instance_rows(&state.pool).await?
    {
        let files = store.instance_files_with_storage(&instance).await;
        let files = match files {
            Ok(files) => files,
            Err(error) => {
                tracing::warn!(
                    instance_id = %instance.id,
                    "Content copy migration deferred: {error}",
                );
                continue;
            }
        };
        let pending: Vec<_> = files
            .into_iter()
            .filter(|(_, binding)| {
                binding.storage_kind != FileStorageKind::Copy
            })
            .collect();
        if pending.is_empty() {
            continue;
        }
        let started = Instant::now();
        let total = pending.len();
        tracing::info!(instance_id = %instance.id, total, concurrency = COPY_CONCURRENCY, "Content copy migration started");
        let mut completed = 0usize;
        let mut deferred = 0usize;
        let mut migrations = stream::iter(pending)
            .map(|(file, binding)| {
                let instance = &instance;
                async move {
                    let result = migrate_file_copy(
                        state,
                        instance,
                        &file,
                        &binding.blob_sha512,
                    )
                    .await;
                    (file.id, result)
                }
            })
            .buffer_unordered(COPY_CONCURRENCY);
        while let Some((file_id, result)) = migrations.next().await {
            match result {
                Ok(true) => completed += 1,
                Ok(false) => deferred += 1,
                Err(error) => {
                    deferred += 1;
                    tracing::warn!(instance_id = %instance.id, %file_id, "Content copy migration deferred: {error}");
                }
            }
            tracing::info!(instance_id = %instance.id, completed, deferred, total,
				elapsed_ms = started.elapsed().as_millis() as u64, "Content copy migration progress");
        }
        tracing::info!(instance_id = %instance.id, completed, deferred, total,
			elapsed_ms = started.elapsed().as_millis() as u64, "Content copy migration pass finished");
    }
    Ok(())
}

async fn migrate_file_copy(
    state: &State,
    instance: &Instance,
    file: &InstanceFile,
    expected_sha512: &str,
) -> crate::Result<bool> {
    let store = &state.content_store;
    let started = Instant::now();
    if instance.install_stage != InstanceInstallStage::Installed
        || crate::state::instance_has_running_process(&instance.id, state)
            .await?
        || instance_rows::is_instance_quarantined(&instance.id, &state.pool)
            .await?
    {
        return Ok(false);
    }
    let original_path = store
        .instance_path(&instance.path, &content_file_path(file))
        .await?;
    let prepared = prepare_copy(&original_path).await?;
    let copy_ms = started.elapsed().as_millis() as u64;
    let waiting = Instant::now();
    let _turn = store.legacy_migration_priority.read().await;
    let _instance = state.lock_instance_content(&instance.id).await;
    let _files = store.files_lock.lock().await;
    let lock_wait_ms = waiting.elapsed().as_millis() as u64;
    let Some(instance) =
        instance_rows::get_instance_by_id(&instance.id, &state.pool).await?
    else {
        return Ok(false);
    };
    if instance.install_stage != InstanceInstallStage::Installed
        || crate::state::instance_has_running_process(&instance.id, state)
            .await?
        || instance_rows::is_instance_quarantined(&instance.id, &state.pool)
            .await?
    {
        return Ok(false);
    }
    let Some(file) =
        crate::state::instances::get_instance_file_by_relative_path(
            &instance.id,
            &file.relative_path,
            &state.pool,
        )
        .await?
        .filter(|current| current.id == file.id)
    else {
        return Ok(false);
    };
    let Some(binding) = catalog::file_storage(&state.pool, &file.id).await?
    else {
        return Ok(false);
    };
    if binding.storage_kind == FileStorageKind::Copy {
        return Ok(false);
    }
    let path = store
        .instance_path(&instance.path, &content_file_path(&file))
        .await?;
    if path != original_path
        || binding.blob_sha512 != expected_sha512
        || !source_unchanged(&prepared, &path)?
    {
        tracing::debug!(file_id = %file.id, "Content copy migration source changed; retrying later");
        return Ok(false);
    }
    if prepared.hashes.sha512 != binding.blob_sha512 {
        if binding.storage_kind != FileStorageKind::Hardlink {
            return Err(input(
                "Changed instance content must be re-imported before migration",
            ));
        }
        if store.any_instance_running(state).await? {
            return Err(input(
                "Stop Minecraft instances before repairing shared content",
            ));
        }
        let stored = catalog::find_file(&state.pool, &binding.blob_sha512)
            .await?
            .ok_or_else(|| {
                input("Managed content has no stored file record")
            })?;
        if store.is_healthy(&stored, true).await? {
            if let Some(healthy) =
                store.lookup(Some(&stored.sha512), None).await?
            {
                store.restore_quarantined_hardlinks(&healthy).await?;
            }
        } else {
            let mut sources = catalog::file_sources(&stored)?;
            if sources.is_empty() {
                sources = downloads::repair_sources(
                    &state.pool,
                    &stored.sha512,
                    state,
                )
                .await;
            }
            if sources.is_empty() {
                return Err(input(
                    "Damaged shared content must be re-imported before migration",
                ));
            }
            store.repair_stored_file(&stored, &sources, state).await?;
        }
        if store
            .check_instance_file(&instance, &file, &binding)
            .await?
            != InstanceFileStatus::Healthy
        {
            return Err(input(
                "Damaged instance content could not be repaired before migration",
            ));
        }
        return Ok(false);
    }
    let parent = path.parent().ok_or_else(|| input("Invalid content path"))?;
    let bytes = prepared.hashes.size;
    let temporary = prepared.temporary;
    let publishing = Instant::now();
    let destination = path.clone();
    tokio::task::spawn_blocking(move || {
        replace_file_copy(temporary, &destination)
    })
    .await??;
    filesystem::sync_directory(parent).await?;
    let replace_ms = publishing.elapsed().as_millis() as u64;
    let committing = Instant::now();
    let mut tx = state.pool.begin().await?;
    catalog::set_file_storage(
        &mut tx,
        &file.id,
        &binding.blob_sha512,
        FileStorageKind::Copy,
    )
    .await?;
    tx.commit().await?;
    tracing::info!(instance_id = %instance.id, file_id = %file.id, bytes, copy_ms,
		lock_wait_ms, replace_ms, commit_ms = committing.elapsed().as_millis() as u64,
		elapsed_ms = started.elapsed().as_millis() as u64, "Content copy migration file completed");
    Ok(true)
}

fn replace_file_copy(
    temporary: tempfile::TempPath,
    destination: &std::path::Path,
) -> std::io::Result<()> {
    let mut temporary = tempfile::TempPath::from_path(
        temporary.keep().map_err(|error| error.error)?,
    );
    #[cfg(windows)]
    let original = {
        use std::os::windows::fs::OpenOptionsExt;
        use windows::Win32::Storage::FileSystem::{
            FILE_READ_ATTRIBUTES, FILE_WRITE_ATTRIBUTES,
        };
        let file = std::fs::OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES.0 | FILE_WRITE_ATTRIBUTES.0)
            .open(destination)?;
        let permissions = file.metadata()?.permissions();
        let mut writable = permissions.clone();
        writable.set_readonly(false);
        file.set_permissions(writable)?;
        (file, permissions)
    };
    let result = std::fs::rename(&temporary, destination);
    if result.is_ok() {
        temporary.disable_cleanup(true);
    }
    #[cfg(windows)]
    original.0.set_permissions(original.1)?;
    result
}

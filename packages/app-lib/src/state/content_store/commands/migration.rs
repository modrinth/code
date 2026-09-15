use crate::state::content_store::adapters::sqlite as catalog;
use crate::state::content_store::adapters::{downloads, filesystem};
use crate::state::content_store::{
    FileStorageKind, InstanceFileStatus, content_file_path, hash_file, input,
};
use crate::state::dirs::move_app_directory::{
    relocate_tree, remove_migrated_tree,
};
use crate::state::{Instance, InstanceFile, State};

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
    if let Err(error) = migrate_instance_copies(state).await {
        tracing::warn!("Content copy migration deferred: {error}");
    }
    Ok(())
}

async fn migrate_instance_copies(state: &State) -> crate::Result<()> {
    let store = &state.content_store;
    for instance in
        crate::state::instances::load_instance_rows(&state.pool).await?
    {
        let _instance = state.lock_instance_content(&instance.id).await;
        let _files = store.files_lock.lock().await;
        let files = async {
            if crate::state::instance_has_running_process(&instance.id, state)
                .await?
            {
                return Ok(Vec::new());
            }
            store.instance_files_with_storage(&instance).await
        }
        .await;
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
        for (file, _) in files {
            if let Err(error) = migrate_file_copy(state, &instance, &file).await
            {
                tracing::warn!(
                    instance_id = %instance.id,
                    file_id = %file.id,
                    "Content copy migration deferred: {error}",
                );
            }
        }
    }
    Ok(())
}

async fn migrate_file_copy(
    state: &State,
    instance: &Instance,
    file: &InstanceFile,
) -> crate::Result<()> {
    let store = &state.content_store;
    let Some(binding) = catalog::file_storage(&state.pool, &file.id).await?
    else {
        return Ok(());
    };
    if binding.storage_kind == FileStorageKind::Copy {
        return Ok(());
    }
    let path = store
        .instance_path(&instance.path, &content_file_path(file))
        .await?;
    let Some(metadata) = filesystem::symlink_metadata_if_exists(&path).await?
    else {
        return Ok(());
    };
    if !metadata.is_file() {
        return Err(input("Managed content is not a regular file"));
    }
    if hash_file(&path).await?.sha512 != binding.blob_sha512 {
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
        if store.check_instance_file(instance, file, &binding).await?
            != InstanceFileStatus::Healthy
        {
            return Err(input(
                "Damaged instance content could not be repaired before migration",
            ));
        }
        return Ok(());
    }
    let parent = path.parent().ok_or_else(|| input("Invalid content path"))?;
    filesystem::require_staging_space(parent, metadata.len())?;
    let (output, temporary) =
        crate::util::content_hash::temporary_file(Some(parent)).await?;
    drop(output);
    filesystem::writable_copy(&path, &temporary).await?;
    if hash_file(&temporary).await?.sha512 != binding.blob_sha512 {
        return Err(input("Content changed while it was being migrated"));
    }
    tokio::fs::File::options()
        .write(true)
        .open(&temporary)
        .await?
        .sync_all()
        .await?;
    let destination = path.clone();
    tokio::task::spawn_blocking(move || -> std::io::Result<()> {
        #[cfg(windows)]
        let original = {
            use std::os::windows::fs::OpenOptionsExt;
            use windows::Win32::Storage::FileSystem::{
                FILE_READ_ATTRIBUTES, FILE_WRITE_ATTRIBUTES,
            };
            let file = std::fs::OpenOptions::new()
                .access_mode(FILE_READ_ATTRIBUTES.0 | FILE_WRITE_ATTRIBUTES.0)
                .open(&destination)?;
            let permissions = file.metadata()?.permissions();
            let mut writable = permissions.clone();
            writable.set_readonly(false);
            file.set_permissions(writable)?;
            (file, permissions)
        };
        let result =
            temporary.persist(destination).map_err(|error| error.error);
        #[cfg(windows)]
        original.0.set_permissions(original.1)?;
        result
    })
    .await??;
    filesystem::sync_directory(parent).await?;
    let mut tx = state.pool.begin().await?;
    catalog::set_file_storage(
        &mut tx,
        &file.id,
        &binding.blob_sha512,
        FileStorageKind::Copy,
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

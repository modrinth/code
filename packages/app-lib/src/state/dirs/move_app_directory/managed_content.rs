use crate::state::content_store;
use crate::state::content_store::{
    FileStorageKind, content_file_path, hash_file, input, sync_directory,
    validate_relative,
};
use crate::state::content_store::{
    remove_instance_file, try_shared_file, validate_parent_directories,
    writable_copy,
};
use crate::state::instances::adapters::sqlite::{content_rows, instance_rows};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tokio::fs;

const CHECKPOINT_KEY: &str = "store_directory_move_storage";

pub(super) struct ManagedContentMove {
    pub source: PathBuf,
    destination: PathBuf,
    stored_path: PathBuf,
    binding: MovedFileBinding,
}

#[derive(Serialize, Deserialize)]
struct MovedFileBinding {
    file_id: String,
    sha512: String,
    storage_kind: FileStorageKind,
}

pub(super) async fn prepare(
    pool: &SqlitePool,
    from: &Path,
    to: &Path,
) -> crate::Result<Vec<ManagedContentMove>> {
    let mut moves = Vec::new();
    for instance in instance_rows::list_instances(pool).await? {
        validate_relative(&instance.path)?;
        let files =
            content_rows::get_instance_files(&instance.id, pool).await?;
        for binding in
            content_store::instance_storage(pool, &instance.id).await?
        {
            let file = files
                .iter()
                .find(|file| file.id == binding.file_id)
                .ok_or_else(|| {
                    input("Managed content has no instance file record")
                })?;
            let relative_path = content_file_path(file);
            validate_relative(&relative_path)?;
            let Some(stored) = content_store::find_file(
                pool,
                &binding.blob_sha512,
            )
            .await? else {
                return Err(input("Managed content has no stored file record"));
            };
            validate_relative(&stored.relative_path)?;
            moves.push(ManagedContentMove {
                source: from
                    .join("profiles")
                    .join(&instance.path)
                    .join(&relative_path),
                destination: to
                    .join("profiles")
                    .join(&instance.path)
                    .join(relative_path),
                stored_path: to
                    .join("store/content")
                    .join(stored.relative_path),
                binding: MovedFileBinding {
                    file_id: binding.file_id,
                    sha512: binding.blob_sha512,
                    storage_kind: binding.storage_kind,
                },
            });
        }
    }
    Ok(moves)
}

pub(super) async fn copy_and_checkpoint(
    pool: &SqlitePool,
    files: Vec<ManagedContentMove>,
    from: &Path,
    to: &Path,
) -> crate::Result<()> {
    let mut bindings = Vec::new();
    for mut file in files {
        validate_parent_directories(from, &file.source).await?;
        validate_parent_directories(to, &file.destination).await?;
        validate_parent_directories(to, &file.stored_path).await?;
        let metadata = match fs::symlink_metadata(&file.source).await {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        if !metadata.is_file() {
            return Err(input(
                "Managed content was replaced by a directory or symbolic link",
            ));
        }
        let hashes = hash_file(&file.source).await?;
        let parent = file
            .destination
            .parent()
            .ok_or_else(|| input("Invalid content destination"))?;
        fs::create_dir_all(parent).await?;
        let temporary =
            parent.join(format!(".modrinth-move-{}.tmp", uuid::Uuid::new_v4()));
        let result = async {
            let stored_matches = hashes.sha512 == file.binding.sha512
                && fs::symlink_metadata(&file.stored_path)
                    .await
                    .is_ok_and(|metadata| metadata.is_file())
                && hash_file(&file.stored_path).await? == hashes;
            let shared_kind = if stored_matches {
                try_shared_file(
                    &file.stored_path,
                    &temporary,
                    file.binding.storage_kind.restore_policy(),
                )
                .await?
            } else {
                None
            };
            let storage_kind = if let Some(kind) = shared_kind {
                kind
            } else {
                super::ensure_move_space(parent, hashes.size)?;
                writable_copy(&file.source, &temporary).await?;
                FileStorageKind::Copy
            };
            if hash_file(&temporary).await? != hashes
                || hash_file(&file.source).await? != hashes
            {
                return Err(input(
                    "Content changed during the app-directory move",
                ));
            }
            if storage_kind != FileStorageKind::Hardlink {
                fs::File::options()
                    .write(true)
                    .open(&temporary)
                    .await?
                    .sync_all()
                    .await?;
            }
            match fs::symlink_metadata(&file.destination).await {
                Ok(metadata) => {
                    if !metadata.is_file()
                        || hash_file(&file.destination).await? != hashes
                    {
                        return Err(input(
                            "The content destination contains different data",
                        ));
                    }
                    remove_instance_file(&file.destination).await?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            fs::rename(&temporary, &file.destination).await?;
            sync_directory(parent).await?;
            Ok(storage_kind)
        }
        .await;
        if result.is_err() && fs::symlink_metadata(&temporary).await.is_ok() {
            let _ = remove_instance_file(&temporary).await;
        }
        file.binding.storage_kind = result?;
        bindings.push(file.binding);
    }
    content_store::set_setting(
        pool,
        CHECKPOINT_KEY,
        &serde_json::to_string(&bindings)?,
    )
    .await
}

/// Records how files are stored on the destination filesystem after the app switches to it.
/// Until then, the records must describe the original files so cancelling the move leaves
/// storage accounting and recovery consistent with the directory the app still uses.
pub(super) async fn commit(pool: &SqlitePool) -> crate::Result<()> {
    let Some(checkpoint) = content_store::setting(pool, CHECKPOINT_KEY)
        .await?
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    let bindings: Vec<MovedFileBinding> = serde_json::from_str(&checkpoint)?;
    let mut tx = pool.begin().await?;
    for binding in bindings {
        content_store::set_file_storage(
            &mut tx,
            &binding.file_id,
            &binding.sha512,
            binding.storage_kind,
        )
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub(super) async fn clear_checkpoint(pool: &SqlitePool) -> crate::Result<()> {
    content_store::set_setting(pool, CHECKPOINT_KEY, "").await
}

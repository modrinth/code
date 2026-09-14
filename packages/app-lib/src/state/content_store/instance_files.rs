use super::file_io::{
    create_content_file, remove_instance_file, try_shared_file,
};
use super::{
    ContentStore, FileContent, FileStorageKind, FileStoragePolicy,
    StoredFileHandle, catalog, hash_file, input, sync_directory,
};
use crate::state::instances::adapters::sqlite::content_rows;
use crate::state::{Instance, InstanceFile};
use std::path::Path;
use tokio::fs;

pub(crate) fn content_file_path(file: &InstanceFile) -> String {
    file_path_on_disk(&file.relative_path, file.enabled)
}

pub(crate) fn file_path_on_disk(relative_path: &str, enabled: bool) -> String {
    let canonical = relative_path.trim_end_matches(".disabled");
    if enabled {
        canonical.to_string()
    } else {
        format!("{canonical}.disabled")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InstanceFileStatus {
    Healthy,
    Missing,
    Conflict,
}

impl ContentStore {
    pub(crate) async fn can_share_content(
        &self,
        source: &Path,
    ) -> crate::Result<bool> {
        let probe = self
            .staging
            .join(format!(".modrinth-share-{}.tmp", uuid::Uuid::new_v4()));
        let result =
            try_shared_file(source, &probe, FileStoragePolicy::Shared).await;
        if fs::symlink_metadata(&probe).await.is_ok() {
            remove_instance_file(&probe).await?;
        }
        Ok(result?.is_some())
    }

    pub(crate) async fn check_instance_file(
        &self,
        instance: &Instance,
        file: &InstanceFile,
        binding: &super::InstanceFileStorage,
    ) -> crate::Result<InstanceFileStatus> {
        let opposite = self
            .instance_path(
                &instance.path,
                &file_path_on_disk(&file.relative_path, !file.enabled),
            )
            .await?;
        match fs::symlink_metadata(&opposite).await {
            Ok(_) => return Ok(InstanceFileStatus::Conflict),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let path = self
            .instance_path(&instance.path, &content_file_path(file))
            .await?;
        match fs::symlink_metadata(&path).await {
            Ok(_)
                if self
                    .instance_file_matches(&path, &binding.blob_sha512)
                    .await? =>
            {
                Ok(InstanceFileStatus::Healthy)
            }
            Ok(_) => Ok(InstanceFileStatus::Conflict),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(InstanceFileStatus::Missing)
            }
            Err(error) => Err(error.into()),
        }
    }

    pub(crate) async fn validate_instance(
        &self,
        instance: &Instance,
    ) -> crate::Result<()> {
        if catalog::setting(&self.pool, "store_layout_version")
            .await?
            .as_deref()
            != Some("1")
        {
            return Err(input(
                "Finish shared-store migration before launching Minecraft",
            ));
        }
        for file in
            content_rows::get_instance_files(&instance.id, &self.pool).await?
        {
            let Some(binding) =
                catalog::file_storage(&self.pool, &file.id).await?
            else {
                continue;
            };
            if !file.enabled {
                let active_path = self
                    .instance_path(
                        &instance.path,
                        &file_path_on_disk(&file.relative_path, true),
                    )
                    .await?;
                match fs::symlink_metadata(&active_path).await {
                    Ok(_) => {
                        return Err(input(format!(
                            "Disabled content {} has an unexpected active file",
                            file.relative_path
                        )));
                    }
                    Err(error)
                        if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
                continue;
            }
            let content = self.file_content(&file).await?;
            let file_status =
                self.check_instance_file(instance, &file, &binding).await?;
            if !matches!(content, FileContent::Stored { .. })
                || file_status != InstanceFileStatus::Healthy
            {
                return Err(input(format!(
                    "{} needs repair or re-import before launching this instance",
                    file.relative_path
                )));
            }
        }
        Ok(())
    }

    pub(crate) async fn restore_instance_files(
        &self,
        instance: &Instance,
        files: &[InstanceFile],
        bindings: &[super::InstanceFileStorage],
    ) -> crate::Result<()> {
        let mut restored = Vec::new();
        for binding in bindings {
            let file = files
                .iter()
                .find(|file| file.id == binding.file_id)
                .ok_or_else(|| {
                    input("Rollback content reference has no file record")
                })?;
            let stored_file = self
                .lookup(Some(&binding.blob_sha512), None, Some(file.size))
                .await?
                .ok_or_else(|| {
                    input("Rollback content needs repair or re-import")
                })?;
            let path = self
                .instance_path(&instance.path, &content_file_path(file))
                .await?;
            if fs::symlink_metadata(&path).await.is_ok() {
                if !self
                    .instance_file_matches(&path, &stored_file.metadata.sha512)
                    .await?
                {
                    return Err(input(
                        "Rollback destination contains different content",
                    ));
                }
                remove_instance_file(&path).await?;
            }
            let storage_kind = self
                .create_instance_file(
                    &stored_file,
                    &path,
                    binding.storage_kind.restore_policy(),
                )
                .await?;
            restored.push((
                &binding.file_id,
                &binding.blob_sha512,
                storage_kind,
            ));
        }
        let mut tx = self.pool.begin().await?;
        for (id, stored_file, storage_kind) in restored {
            catalog::set_file_storage(&mut tx, id, stored_file, storage_kind)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub(crate) async fn instance_file_matches(
        &self,
        path: &Path,
        sha512: &str,
    ) -> crate::Result<bool> {
        let metadata = match fs::symlink_metadata(path).await {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(false);
            }
            Err(error) => return Err(error.into()),
        };
        Ok(metadata.is_file() && hash_file(path).await?.sha512 == sha512)
    }

    pub(crate) async fn create_instance_file(
        &self,
        stored_file: &StoredFileHandle,
        target: &Path,
        policy: FileStoragePolicy,
    ) -> crate::Result<FileStorageKind> {
        let parent = target
            .parent()
            .ok_or_else(|| input("Content destination has no parent"))?;
        fs::create_dir_all(parent).await?;
        if fs::symlink_metadata(target).await.is_ok() {
            return Err(input("Content destination already exists"));
        }
        let temporary =
            parent.join(format!(".modrinth-{}.tmp", uuid::Uuid::new_v4()));
        let result = async {
            let storage_kind =
                create_content_file(&stored_file.path, &temporary, policy)
                    .await?;
            if matches!(
                storage_kind,
                FileStorageKind::Copy | FileStorageKind::Reflink
            ) {
                fs::File::options()
                    .write(true)
                    .open(&temporary)
                    .await?
                    .sync_all()
                    .await?;
            }
            fs::rename(&temporary, target).await?;
            sync_directory(parent).await?;
            Ok(storage_kind)
        }
        .await;
        if result.is_err() && fs::symlink_metadata(&temporary).await.is_ok() {
            let _ = remove_instance_file(&temporary).await;
        }
        result
    }
}

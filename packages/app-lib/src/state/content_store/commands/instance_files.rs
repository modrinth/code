use crate::state::content_store::adapters::filesystem;
use crate::state::content_store::adapters::filesystem::remove_instance_file;
use crate::state::content_store::adapters::filesystem::symlink_metadata_if_exists;
use crate::state::content_store::adapters::sqlite as catalog;
use crate::state::content_store::model::InstancePathContent;
use crate::state::content_store::{
    ContentStore, FileContent, FileStorageKind, FileStoragePolicy,
    InstanceFileStatus, InstanceFileStorage, StoredFileHandle,
    content_file_path, file_path_on_disk, input,
};
use crate::state::instances as content_rows;
use crate::state::{Instance, InstanceFile};
use std::collections::HashMap;
use std::path::Path;

impl ContentStore {
    pub(crate) async fn check_instance_file(
        &self,
        instance: &Instance,
        file: &InstanceFile,
        binding: &InstanceFileStorage,
    ) -> crate::Result<InstanceFileStatus> {
		self.check_instance_file_inner(instance, file, binding, false)
			.await
	}

	pub(crate) async fn check_instance_file_cached(
		&self,
		instance: &Instance,
		file: &InstanceFile,
		binding: &InstanceFileStorage,
	) -> crate::Result<InstanceFileStatus> {
		self.check_instance_file_inner(instance, file, binding, true)
			.await
	}

	async fn check_instance_file_inner(
		&self,
		instance: &Instance,
		file: &InstanceFile,
		binding: &InstanceFileStorage,
		use_cached_hash: bool,
	) -> crate::Result<InstanceFileStatus> {
        let opposite = self
            .instance_path(
                &instance.path,
                &file_path_on_disk(&file.relative_path, !file.enabled),
            )
            .await?;
        if symlink_metadata_if_exists(&opposite).await?.is_some() {
            return Ok(InstanceFileStatus::Conflict);
        }
        let path = self
            .instance_path(&instance.path, &content_file_path(file))
            .await?;
		if use_cached_hash {
			let content = match symlink_metadata_if_exists(&path).await? {
				None => InstancePathContent::Missing,
				Some(metadata) if metadata.is_file() => {
					InstancePathContent::File(
						self.verified_files.hash_file(&path).await?,
					)
				}
				Some(_) => InstancePathContent::Conflict,
			};
			return Ok(content.status(&binding.blob_sha512));
		}
        self.check_instance_path(&path, &binding.blob_sha512).await
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
        for (file, binding) in
            self.instance_files_with_storage(instance).await?
        {
            if !file.enabled {
                let active_path = self
                    .instance_path(
                        &instance.path,
                        &file_path_on_disk(&file.relative_path, true),
                    )
                    .await?;
                if symlink_metadata_if_exists(&active_path).await?.is_some() {
                    return Err(input(format!(
                        "Disabled content {} has an unexpected active file",
                        file.relative_path
                    )));
                }
                continue;
            }
            let content = self
                .file_content_with_binding(&file, binding.clone())
                .await?;
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
        bindings: &[InstanceFileStorage],
    ) -> crate::Result<()> {
        let files: HashMap<_, _> =
            files.iter().map(|file| (file.id.as_str(), file)).collect();
        let mut restored = Vec::new();
        for binding in bindings {
            let file =
                files.get(binding.file_id.as_str()).ok_or_else(|| {
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
            match self
                .check_instance_path(&path, &stored_file.metadata.sha512)
                .await?
            {
                InstanceFileStatus::Healthy => {
                    remove_instance_file(&path).await?
                }
                InstanceFileStatus::Missing => {}
                InstanceFileStatus::Conflict => {
                    return Err(input(
                        "Rollback destination contains different content",
                    ));
                }
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
        Ok(self.check_instance_path(path, sha512).await?
            == InstanceFileStatus::Healthy)
    }

    pub(in crate::state::content_store) async fn check_instance_path(
        &self,
        path: &Path,
        sha512: &str,
    ) -> crate::Result<InstanceFileStatus> {
        Ok(self.inspect_instance_path(path).await?.status(sha512))
    }

    pub(in crate::state::content_store) async fn instance_files_with_storage(
        &self,
        instance: &Instance,
    ) -> crate::Result<Vec<(InstanceFile, InstanceFileStorage)>> {
        let mut bindings: HashMap<_, _> =
            catalog::instance_storage(&self.pool, &instance.id)
                .await?
                .into_iter()
                .map(|binding| (binding.file_id.clone(), binding))
                .collect();
        Ok(content_rows::get_instance_files(&instance.id, &self.pool)
            .await?
            .into_iter()
            .filter_map(|file| {
                bindings.remove(&file.id).map(|binding| (file, binding))
            })
            .collect())
    }

    pub(in crate::state::content_store) async fn save_repaired_instance_file(
        &self,
        file: &mut InstanceFile,
        sha512: &str,
        storage_kind: FileStorageKind,
    ) -> crate::Result<()> {
        let mut tx = self.pool.begin().await?;
        file.missing = false;
        content_rows::upsert_instance_file(file, &mut tx).await?;
        catalog::set_file_storage(&mut tx, &file.id, sha512, storage_kind)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub(crate) async fn create_instance_file(
        &self,
        stored_file: &StoredFileHandle,
        target: &Path,
        policy: FileStoragePolicy,
    ) -> crate::Result<FileStorageKind> {
        filesystem::create_instance_file(&stored_file.path, target, policy)
            .await
    }
    pub(in crate::state::content_store) async fn inspect_instance_path(
        &self,
        path: &Path,
    ) -> crate::Result<InstancePathContent> {
        filesystem::inspect_instance_path(path).await
    }
    pub(crate) async fn can_share_content(
        &self,
        source: &Path,
    ) -> crate::Result<bool> {
        filesystem::can_share_content(&self.staging, source).await
    }
    pub(in crate::state::content_store) async fn any_instance_running(
        &self,
        state: &crate::State,
    ) -> crate::Result<bool> {
        for instance in
            crate::state::instances::load_instance_rows(&self.pool).await?
        {
            if crate::state::instance_has_running_process(&instance.id, state)
                .await?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

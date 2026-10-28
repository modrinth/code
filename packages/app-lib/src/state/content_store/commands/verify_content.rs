use crate::state::content_store::adapters::downloads;
use crate::state::content_store::adapters::filesystem;
use crate::state::content_store::adapters::filesystem::symlink_metadata_if_exists;
use crate::state::content_store::{
    ContentStore, FileStorageKind, InstanceFileStatus, StoredFileHandle,
    StoredFileMetadata, content_file_path,
};
use crate::state::instances;
use crate::{ErrorKind, State};
use anyhow::bail;

impl ContentStore {
    pub(crate) async fn import_file(
        &self,
        source: &std::path::Path,
        state: &State,
    ) -> crate::Result<StoredFileHandle> {
        let staged = self.stage_file(source).await?;
        let hash = staged.sha512.clone();
        let _files_lock = self.files_lock.lock().await;
        let mut repaired = false;
        if let Some(stored_file) = self.get_file_record(&hash).await?
            && !self.is_healthy(&stored_file.metadata, true).await?
        {
            repaired = true;
            if self.any_instance_running(state).await? {
                bail!(ErrorKind::ContentStoreError(
                    "stop Minecraft instances before re-importing damaged shared content".into(),
                ));
            }
            filesystem::quarantine_file(&self.root, &stored_file.path, &hash)
                .await?;
        }
        let stored_file = self.save_staged_file(staged, &[]).await?;
        if repaired {
            self.restore_quarantined_hardlinks(&stored_file).await?;
        }
        Ok(stored_file)
    }

    pub(in crate::state::content_store) async fn restore_quarantined_hardlinks(
        &self,
        stored_file: &StoredFileHandle,
    ) -> crate::Result<()> {
        let quarantined = filesystem::quarantined_files(
            &self.root,
            &stored_file.metadata.sha512,
        )
        .await?;
        if quarantined.is_empty() {
            return Ok(());
        }
        let mut restoration_pending = false;
        for instance in instances::load_instance_rows(&self.pool).await? {
            for (mut file, binding) in
                self.instance_files_with_storage(&instance).await?
            {
                if binding.storage_kind != FileStorageKind::Hardlink
                    || binding.blob_sha512 != stored_file.metadata.sha512
                {
                    continue;
                }
                let status = self
                    .check_instance_file(&instance, &file, &binding)
                    .await?;
                if status == InstanceFileStatus::Healthy {
                    continue;
                }
                let path = self
                    .instance_path(&instance.path, &content_file_path(&file))
                    .await?;
                if status == InstanceFileStatus::Conflict {
                    let opposite = self
                        .instance_path(
                            &instance.path,
                            &crate::state::content_store::file_path_on_disk(
                                &file.relative_path,
                                !file.enabled,
                            ),
                        )
                        .await?;
                    if symlink_metadata_if_exists(&opposite).await?.is_some() {
                        restoration_pending = true;
                        continue;
                    }
                    if !filesystem::is_regular_file(&path, false).await? {
                        restoration_pending = true;
                        continue;
                    }
                    let is_quarantined =
                        filesystem::matches_any_file(&path, &quarantined)
                            .await?;
                    if !is_quarantined {
                        restoration_pending = true;
                        continue;
                    }
                    crate::state::content_store::adapters::filesystem::remove_instance_file(&path).await?;
                }
                let storage_kind =
                    self.create_instance_file(stored_file, &path).await?;
                self.save_repaired_instance_file(
                    &mut file,
                    &binding.blob_sha512,
                    storage_kind,
                )
                .await?;
            }
        }
        if !restoration_pending {
            for path in quarantined {
                let name = path.file_name().ok_or_else(|| {
                    ErrorKind::ContentStoreError(
                        "invalid quarantined content path".into(),
                    )
                })?;
                let backup = self.root.join("recovered-content").join(name);
                filesystem::move_instance_file(&path, &backup).await?;
                tracing::warn!(
                    sha512 = %stored_file.metadata.sha512,
                    path = %backup.display(),
                    "Preserved original content after repair; remove this recovery file manually when no longer needed",
                );
            }
        }
        Ok(())
    }

    pub(in crate::state::content_store) async fn repair_stored_file(
        &self,
        stored_file: &StoredFileMetadata,
        sources: &[String],
        state: &State,
    ) -> crate::Result<()> {
        let mirrors = sources.iter().map(String::as_str).collect::<Vec<_>>();
        let downloaded = downloads::download(
            &self.staging,
            &self.pool,
            &mirrors,
            None,
            &state.fetch_semaphore,
            None,
        )
        .await?;
        if downloaded.sha512 != stored_file.sha512
            || downloaded.size != stored_file.size as u64
        {
            bail!(ErrorKind::ContentStoreError(
                "repair download has an unexpected hash or size".into(),
            ));
        }
        let path = self.path(stored_file)?;
        filesystem::quarantine_file(&self.root, &path, &stored_file.sha512)
            .await?;
        let repaired = self
            .save_staged_file(downloaded.into_staged()?, sources)
            .await?;
        self.restore_quarantined_hardlinks(&repaired).await
    }
}

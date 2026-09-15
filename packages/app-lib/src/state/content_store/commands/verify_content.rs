use crate::State;
use crate::state::content_store::adapters::downloads;
use crate::state::content_store::adapters::filesystem;
use crate::state::content_store::adapters::filesystem::symlink_metadata_if_exists;
use crate::state::content_store::adapters::sqlite as catalog;
use crate::state::content_store::{
    ContentStore, FileContent, FileStorageKind, FileStoragePolicy,
    InstanceFileStatus, InstanceFileStorage, StoreIssue, StoreVerification,
    StoredFileHandle, StoredFileMetadata, content_file_path, input,
};
use crate::state::instances;
use crate::state::{Instance, InstanceFile};

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
                return Err(input(
                    "Stop Minecraft instances before re-importing damaged shared content",
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
                let storage_kind = self
                    .create_instance_file(
                        stored_file,
                        &path,
                        FileStoragePolicy::Shared,
                    )
                    .await?;
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
                filesystem::remove_unused_file(&path).await?;
            }
        }
        Ok(())
    }

    pub async fn verify(
        &self,
        state: &State,
        repair: bool,
    ) -> crate::Result<StoreVerification> {
        self.verify_with_progress(state, repair, &|_, _| {}).await
    }

    pub async fn verify_with_progress(
        &self,
        state: &State,
        repair: bool,
        on_progress: &(dyn Fn(u64, u64) + Send + Sync),
    ) -> crate::Result<StoreVerification> {
        let _files_lock = self.files_lock.lock().await;
        let _lease = self.lease().await;
        if repair && self.any_instance_running(state).await? {
            return Err(input(
                "Stop Minecraft instances before repairing shared content",
            ));
        }
        let stored_files = catalog::stored_files(&self.pool).await?;
        let total = stored_files
            .iter()
            .map(|stored_file| stored_file.size as u64)
            .sum::<u64>();
        let bytes_read = std::sync::atomic::AtomicU64::new(0);
        let on_read = |bytes| {
            let current = bytes_read
                .fetch_add(bytes, std::sync::atomic::Ordering::Relaxed)
                + bytes;
            on_progress(current, total);
        };
        on_progress(0, total);
        let mut report = StoreVerification {
            checked: 0,
            repaired: 0,
            issues: Vec::new(),
        };
        for stored_file in stored_files {
            report.checked += 1;
            if self
                .is_healthy_with_progress(&stored_file, true, &on_read)
                .await?
            {
                if repair
                    && let Some(healthy) =
                        self.lookup(Some(&stored_file.sha512), None).await?
                {
                    self.restore_quarantined_hardlinks(&healthy).await?;
                }
                continue;
            }
            let mut message = "Stored content is missing or damaged; re-import the original file".to_string();
            if repair {
                let mut sources: Vec<String> =
                    catalog::file_sources(&stored_file)?;
                if sources.is_empty() {
                    sources = downloads::repair_sources(
                        &self.pool,
                        &stored_file.sha512,
                        state,
                    )
                    .await;
                }
                if !sources.is_empty() {
                    let result = self
                        .repair_stored_file(&stored_file, &sources, state)
                        .await;
                    match result {
                        Ok(()) => {
                            report.repaired += 1;
                            continue;
                        }
                        Err(error) => {
                            message = format!("Repair failed: {error}")
                        }
                    }
                }
            }
            let instance_ids =
                catalog::instances_using_file(&self.pool, &stored_file.sha512)
                    .await?;
            report.issues.push(StoreIssue {
                sha512: stored_file.sha512,
                instance_ids,
                message,
            });
        }
        for instance in instances::load_instance_rows(&self.pool).await? {
            for (file, binding) in
                self.instance_files_with_storage(&instance).await?
            {
                self.verify_instance_file(
                    &instance,
                    file,
                    binding,
                    repair,
                    &mut report,
                )
                .await?;
            }
        }

        Ok(report)
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
            return Err(input(
                "Repair download has an unexpected hash or size",
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

    async fn verify_instance_file(
        &self,
        instance: &Instance,
        mut file: InstanceFile,
        binding: InstanceFileStorage,
        repair: bool,
        report: &mut StoreVerification,
    ) -> crate::Result<()> {
        let file_status =
            self.check_instance_file(instance, &file, &binding).await?;
        let content = self
            .file_content_with_binding(&file, binding.clone())
            .await?;
        if file_status == InstanceFileStatus::Healthy
            && matches!(&content, FileContent::Stored { .. })
            && !file.missing
        {
            return Ok(());
        }
        if repair && let FileContent::Stored { stored_file, .. } = &content {
            let storage_kind = match file_status {
                InstanceFileStatus::Missing => {
                    let path = self
                        .instance_path(
                            &instance.path,
                            &content_file_path(&file),
                        )
                        .await?;
                    Some(
                        self.create_instance_file(
                            stored_file,
                            &path,
                            binding.storage_kind.restore_policy(),
                        )
                        .await?,
                    )
                }
                InstanceFileStatus::Healthy => Some(binding.storage_kind),
                InstanceFileStatus::Conflict => None,
            };
            if let Some(storage_kind) = storage_kind {
                self.save_repaired_instance_file(
                    &mut file,
                    &binding.blob_sha512,
                    storage_kind,
                )
                .await?;
                report.repaired += 1;
                return Ok(());
            }
        }
        let reason = match (&content, file_status) {
            (FileContent::Damaged(_), _) => {
                "the stored content is missing or damaged"
            }
            (_, InstanceFileStatus::Conflict) => {
                "the instance path contains different content; preserve it and resolve the conflict"
            }
            (_, InstanceFileStatus::Missing) => "the instance file is missing",
            (_, InstanceFileStatus::Healthy) => {
                "the instance file metadata needs repair"
            }
        };
        report.issues.push(StoreIssue {
            sha512: binding.blob_sha512,
            instance_ids: vec![instance.id.clone()],
            message: format!("{}: {reason}", file.relative_path),
        });
        Ok(())
    }
}

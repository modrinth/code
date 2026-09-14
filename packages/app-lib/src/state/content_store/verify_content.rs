use super::{
    ContentStore, FileContent, FileStorageKind, InstanceFileStatus, StoreIssue,
    StoreVerification, StoredFileMetadata, catalog, content_file_path, input,
};
use crate::State;
use crate::state::instances::adapters::sqlite::{content_rows, instance_rows};
use crate::state::{Instance, InstanceFile};
use tokio::fs;

impl ContentStore {
    pub(crate) async fn import_file(
        &self,
        source: &std::path::Path,
        state: &State,
    ) -> crate::Result<super::StoredFileHandle> {
        let staged = self.stage_file(source).await?;
        let hash = staged.sha512.clone();
        let _files_lock = self.files_lock.lock().await;
        let mut repaired = false;
        if let Some(stored_file) = self.get_file_record(&hash).await?
            && !self.is_healthy(&stored_file.metadata, true).await?
        {
            repaired = true;
            for instance in instance_rows::list_instances(&self.pool).await? {
                if crate::state::instance_has_running_process(
                    &instance.id,
                    state,
                )
                .await?
                {
                    return Err(input(
                        "Stop Minecraft instances before re-importing damaged shared content",
                    ));
                }
            }
            if fs::symlink_metadata(&stored_file.path).await.is_ok() {
                let quarantine = self.root.join("quarantine");
                fs::create_dir_all(&quarantine).await?;
                fs::rename(
                    &stored_file.path,
                    quarantine.join(format!(
                        "{}-{}",
                        hash,
                        uuid::Uuid::new_v4()
                    )),
                )
                .await?;
            }
        }
        let stored_file = self.save_staged_file(staged, &[]).await?;
        if repaired {
            self.restore_quarantined_hardlinks(&stored_file).await?;
        }
        Ok(stored_file)
    }

    pub(super) async fn restore_quarantined_hardlinks(
        &self,
        stored_file: &super::StoredFileHandle,
    ) -> crate::Result<()> {
        let mut entries = match fs::read_dir(self.root.join("quarantine")).await
        {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        let prefix = format!("{}-", stored_file.metadata.sha512);
        let mut quarantined = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_name().to_string_lossy().starts_with(&prefix)
                && entry.file_type().await?.is_file()
            {
                quarantined.push(entry.path());
            }
        }
        if quarantined.is_empty() {
            return Ok(());
        }
        for instance in instance_rows::list_instances(&self.pool).await? {
            for mut file in
                content_rows::get_instance_files(&instance.id, &self.pool)
                    .await?
            {
                let Some(binding) =
                    catalog::file_storage(&self.pool, &file.id).await?
                else {
                    continue;
                };
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
                            &super::file_path_on_disk(
                                &file.relative_path,
                                !file.enabled,
                            ),
                        )
                        .await?;
                    match fs::symlink_metadata(opposite).await {
                        Ok(_) => continue,
                        Err(error)
                            if error.kind() == std::io::ErrorKind::NotFound => {
                        }
                        Err(error) => return Err(error.into()),
                    }
                    if !fs::symlink_metadata(&path).await?.is_file() {
                        continue;
                    }
                    let candidate = path.clone();
                    let originals = quarantined.clone();
                    let is_quarantined =
                        tokio::task::spawn_blocking(move || {
                            for original in originals {
                                if same_file::is_same_file(
                                    &candidate, original,
                                )? {
                                    return Ok::<_, std::io::Error>(true);
                                }
                            }
                            Ok(false)
                        })
                        .await??;
                    if !is_quarantined {
                        continue;
                    }
                    super::file_io::remove_instance_file(&path).await?;
                }
                let storage_kind = self
                    .create_instance_file(
                        stored_file,
                        &path,
                        super::FileStoragePolicy::Shared,
                    )
                    .await?;
                let mut tx = self.pool.begin().await?;
                file.missing = false;
                content_rows::upsert_instance_file(&file, &mut tx).await?;
                catalog::set_file_storage(
                    &mut tx,
                    &file.id,
                    &binding.blob_sha512,
                    storage_kind,
                )
                .await?;
                tx.commit().await?;
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
        if repair {
            for instance in instance_rows::list_instances(&self.pool).await? {
                if crate::state::instance_has_running_process(
                    &instance.id,
                    state,
                )
                .await?
                {
                    return Err(input(
                        "Stop Minecraft instances before repairing shared content",
                    ));
                }
            }
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
                    && let Some(healthy) = self
                        .lookup(Some(&stored_file.sha512), None, None)
                        .await?
                {
                    self.restore_quarantined_hardlinks(&healthy).await?;
                }
                continue;
            }
            let mut message = "Stored content is missing or damaged; re-import the original file".to_string();
            if repair {
                let mut sources: Vec<String> =
                    serde_json::from_str(&stored_file.sources)?;
                if sources.is_empty()
                    && let Ok(files) = crate::state::CachedEntry::get_file_many(
                        &[stored_file.sha1.as_str()],
                        None,
                        &self.pool,
                        &state.api_semaphore,
                    )
                    .await
                {
                    for file in files {
                        if let Ok(Some(version)) =
                            crate::state::CachedEntry::get_version(
                                &file.version_id,
                                None,
                                &self.pool,
                                &state.api_semaphore,
                            )
                            .await
                        {
                            for candidate in version.files {
                                if candidate.hashes.get("sha512")
                                    == Some(&stored_file.sha512)
                                {
                                    sources.push(candidate.url);
                                }
                            }
                        }
                    }
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
            let instance_ids = sqlx::query_scalar!("SELECT DISTINCT file.instance_id FROM instance_files file INNER JOIN store_instance_files binding ON binding.file_id = file.id WHERE binding.blob_sha512 = ?", stored_file.sha512).fetch_all(&self.pool).await?;
            report.issues.push(StoreIssue {
                sha512: stored_file.sha512,
                instance_ids,
                message,
            });
        }
        for instance in instance_rows::list_instances(&self.pool).await? {
            for file in
                content_rows::get_instance_files(&instance.id, &self.pool)
                    .await?
            {
                self.verify_instance_file(&instance, file, repair, &mut report)
                    .await?;
            }
        }
        Ok(report)
    }

    async fn repair_stored_file(
        &self,
        stored_file: &StoredFileMetadata,
        sources: &[String],
        state: &State,
    ) -> crate::Result<()> {
        let mirrors = sources.iter().map(String::as_str).collect::<Vec<_>>();
        let downloaded = crate::util::fetch::fetch_file_mirrors_in(
            &mirrors,
            Some(&stored_file.sha1),
            None,
            None,
            &state.fetch_semaphore,
            &self.pool,
            None,
            Some(&self.staging),
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
        if fs::symlink_metadata(&path).await.is_ok() {
            let quarantine = self.root.join("quarantine");
            fs::create_dir_all(&quarantine).await?;
            fs::rename(
                &path,
                quarantine.join(format!(
                    "{}-{}",
                    stored_file.sha512,
                    uuid::Uuid::new_v4()
                )),
            )
            .await?;
        }
        let repaired = self
            .save_staged_file(downloaded.into_staged()?, sources)
            .await?;
        self.restore_quarantined_hardlinks(&repaired).await
    }

    async fn verify_instance_file(
        &self,
        instance: &Instance,
        mut file: InstanceFile,
        repair: bool,
        report: &mut StoreVerification,
    ) -> crate::Result<()> {
        let Some(binding) = catalog::file_storage(&self.pool, &file.id).await?
        else {
            return Ok(());
        };
        let file_status =
            self.check_instance_file(instance, &file, &binding).await?;
        let content = self.file_content(&file).await?;
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
                let mut tx = self.pool.begin().await?;
                file.missing = false;
                content_rows::upsert_instance_file(&file, &mut tx).await?;
                catalog::set_file_storage(
                    &mut tx,
                    &file.id,
                    &binding.blob_sha512,
                    storage_kind,
                )
                .await?;
                tx.commit().await?;
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

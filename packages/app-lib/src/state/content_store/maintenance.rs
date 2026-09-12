use super::{
    ContentStore, FileContent, InstanceFileKind, InstanceFileStatus,
    StoredFileStatus, catalog, content_file_path, input, sync_directory,
};
use crate::State;
use crate::state::instances::adapters::sqlite::instance_rows;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use tokio::fs;

#[derive(Serialize)]
pub struct StoreUsage {
    pub unique_bytes: u64,
    pub shared_bytes: u64,
    pub unused_cache_bytes: u64,
    pub estimated_saved_bytes: u64,
    pub private_copy_bytes: u64,
    pub object_count: usize,
    pub damaged_objects: usize,
    pub cache_limit_bytes: u64,
}

#[derive(Serialize)]
pub struct StoreIssue {
    pub sha512: String,
    pub instance_ids: Vec<String>,
    pub message: String,
}

#[derive(Serialize)]
pub struct StoreVerification {
    pub checked: usize,
    pub repaired: usize,
    pub issues: Vec<StoreIssue>,
}

impl ContentStore {
    pub(crate) async fn import_file(
        &self,
        source: &std::path::Path,
        state: &State,
    ) -> crate::Result<super::StoredFileHandle> {
        let staged = self.stage_file(source).await?;
        let hash = staged.sha512.clone();
        let _files_lock = self.files_lock.lock().await;
        if let Some(stored_file) = self.get_file_record(&hash).await?
            && !self.is_healthy(&stored_file.metadata, true).await?
        {
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
        self.save_staged_file(staged, &[]).await
    }

    pub async fn usage(&self, state: &State) -> crate::Result<StoreUsage> {
        let _lease = self.lease().await;
        let _runtime_lease = self.runtime_gate.read().await;
        let runtime = super::runtime::RuntimeStorage::read(state).await?;
        let blobs = catalog::stored_files(&self.pool).await?;
        let roots = catalog::referenced_files(&self.pool)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();
        let placements = sqlx::query!("SELECT binding.blob_sha512, binding.materialization_kind, file.size FROM store_instance_files binding INNER JOIN instance_files file ON file.id = binding.file_id WHERE file.missing = 0").fetch_all(&self.pool).await?;
        let installed = placements
            .iter()
            .map(|placement| placement.blob_sha512.as_str())
            .collect::<HashSet<_>>();
        let logical_bytes = placements
            .iter()
            .map(|placement| placement.size.max(0) as u64)
            .sum::<u64>();
        let private_copy_bytes = placements
            .iter()
            .filter(|placement| {
                placement.materialization_kind
                    == InstanceFileKind::Copy.as_str()
            })
            .map(|placement| placement.size.max(0) as u64)
            .sum::<u64>();
        let referenced_unique = blobs
            .iter()
            .filter(|stored_file| {
                installed.contains(stored_file.sha512.as_str())
            })
            .map(|stored_file| stored_file.size as u64)
            .sum::<u64>();
        let mut shared_placements = HashMap::new();
        for placement in &placements {
            if placement.materialization_kind
                == InstanceFileKind::Symlink.as_str()
            {
                *shared_placements
                    .entry(placement.blob_sha512.as_str())
                    .or_insert(0usize) += 1;
            }
        }
        Ok(StoreUsage {
            unique_bytes: blobs
                .iter()
                .map(|stored_file| stored_file.size as u64)
                .sum::<u64>()
                + runtime.total_bytes(),
            shared_bytes: blobs
                .iter()
                .filter(|stored_file| {
                    shared_placements
                        .get(stored_file.sha512.as_str())
                        .copied()
                        .unwrap_or(0)
                        > 1
                })
                .map(|stored_file| stored_file.size as u64)
                .sum::<u64>()
                + runtime.shared_bytes(),
            unused_cache_bytes: blobs
                .iter()
                .filter(|stored_file| !roots.contains(&stored_file.sha512))
                .map(|stored_file| stored_file.size as u64)
                .sum::<u64>()
                + runtime.unused_bytes(),
            estimated_saved_bytes: logical_bytes.saturating_sub(
                referenced_unique.saturating_add(private_copy_bytes),
            ) + runtime.saved_bytes(),
            private_copy_bytes,
            object_count: blobs.len(),
            damaged_objects: blobs
                .iter()
                .filter(|stored_file| {
                    stored_file.status == StoredFileStatus::Quarantined
                })
                .count(),
            cache_limit_bytes: self.cache_limit().await?,
        })
    }

    async fn cache_limit(&self) -> crate::Result<u64> {
        catalog::setting(&self.pool, "store_cache_limit_bytes")
            .await?
            .unwrap_or_else(|| "5368709120".to_string())
            .parse()
            .map_err(|_| input("The shared content cache limit is invalid"))
    }

    pub async fn set_cache_limit(&self, bytes: u64) -> crate::Result<()> {
        if bytes > i64::MAX as u64 {
            return Err(input("The shared content cache limit is too large"));
        }
        catalog::set_setting(
            &self.pool,
            "store_cache_limit_bytes",
            &bytes.to_string(),
        )
        .await
    }

    /// Removes unreferenced content objects and managed game and Java cache files.
    pub async fn cleanup(
        &self,
        state: &State,
        purge_unused: bool,
    ) -> crate::Result<u64> {
        let _exclusive = self.gate.clone().try_write_owned().map_err(|_| input("The shared store is busy; try cleanup again after content operations finish"))?;
        if catalog::setting(&self.pool, "store_layout_version")
            .await?
            .as_deref()
            != Some("1")
        {
            return Err(input(
                "Finish shared-store migration before cleaning its cache",
            ));
        }
        let _runtime_exclusive = self.runtime_gate.try_write()
			.map_err(|_| input("Game or Java files are in use; try cleanup again after the operation finishes"))?;
        let runtime = super::runtime::RuntimeStorage::read(state).await?;
        let roots = catalog::referenced_files(&self.pool)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();
        let mut candidates = catalog::stored_files(&self.pool)
            .await?
            .into_iter()
            .filter(|stored_file| !roots.contains(&stored_file.sha512))
            .collect::<Vec<_>>();
        candidates.sort_by_key(|stored_file| stored_file.last_used_at);
        let mut unused = candidates
            .iter()
            .map(|stored_file| stored_file.size as u64)
            .sum::<u64>()
            + runtime.unused_bytes();
        let limit = if purge_unused {
            0
        } else {
            self.cache_limit().await?
        };
        let mut reclaimed = 0;
        for stored_file in candidates {
            if unused <= limit
                && stored_file.status != StoredFileStatus::Deleting
            {
                continue;
            }
            if !purge_unused
                && chrono::Utc::now()
                    .timestamp()
                    .saturating_sub(stored_file.last_used_at)
                    < 60
            {
                continue;
            }
            catalog::set_file_status(
                &self.pool,
                &stored_file.sha512,
                StoredFileStatus::Deleting,
            )
            .await?;
            let path = self.path(&stored_file)?;
            self.validate_object_parent(&path).await?;
            match fs::symlink_metadata(&path).await {
                Ok(metadata) => {
                    if !metadata.is_file() || metadata.file_type().is_symlink()
                    {
                        return Err(input(
                            "An unreferenced store object was replaced by an unexpected filesystem entry",
                        ));
                    }
                    #[cfg(windows)]
                    {
                        let mut permissions = metadata.permissions();
                        permissions.set_readonly(false);
                        fs::set_permissions(&path, permissions).await?;
                    }
                    fs::remove_file(&path).await?;
                    if let Some(parent) = path.parent() {
                        sync_directory(parent).await?;
                        let _ = fs::remove_dir(parent).await;
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            catalog::delete_file(&self.pool, &stored_file.sha512).await?;
            unused = unused.saturating_sub(stored_file.size as u64);
            reclaimed += stored_file.size as u64;
        }
        for instance in instance_rows::list_instances(&self.pool).await? {
            if crate::state::instance_has_running_process(&instance.id, state)
                .await?
            {
                return Ok(reclaimed);
            }
        }
        for job in crate::install::store::list(false, state).await? {
            if matches!(
                job.status,
                crate::install::InstallJobStatus::Queued
                    | crate::install::InstallJobStatus::Running
            ) {
                return Ok(reclaimed);
            }
            if let Some(backup) = job.state.paths.staging_dir
                && fs::try_exists(backup).await?
            {
                return Ok(reclaimed);
            }
        }
        let mut unused_runtime = runtime
            .files
            .into_iter()
            .filter(|file| file.references == 0)
            .collect::<Vec<_>>();
        unused_runtime.sort_by_key(|file| file.last_used_at);
        for file in unused_runtime {
            if !purge_unused
                && (unused <= limit
                    || chrono::Utc::now()
                        .timestamp()
                        .saturating_sub(file.last_used_at)
                        < 60)
            {
                continue;
            }
            let removed =
                super::runtime::remove_file(&file, &runtime.root).await?;
            unused = unused.saturating_sub(removed);
            reclaimed += removed;
        }
        Ok(reclaimed)
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
        let blobs = catalog::stored_files(&self.pool).await?;
        let total = blobs
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
        for stored_file in blobs {
            report.checked += 1;
            if self
                .is_healthy_with_progress(&stored_file, true, &on_read)
                .await?
            {
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
                    let result = async {
						let mirrors = sources.iter().map(String::as_str).collect::<Vec<_>>();
						let downloaded = crate::util::fetch::fetch_file_mirrors_in(&mirrors, Some(&stored_file.sha1), None, None, &state.fetch_semaphore, &self.pool, None, Some(&self.staging)).await?;
						if downloaded.sha512 != stored_file.sha512 || downloaded.size != stored_file.size as u64 { return Err(input("Repair download has an unexpected hash or size")); }
						let path = self.path(&stored_file)?;
						if fs::symlink_metadata(&path).await.is_ok() {
							let quarantine = self.root.join("quarantine");
							fs::create_dir_all(&quarantine).await?;
							fs::rename(&path, quarantine.join(format!("{}-{}", stored_file.sha512, uuid::Uuid::new_v4()))).await?;
						}
						self.save_staged_file(downloaded.into_staged()?, &sources).await?;
						Ok::<(), crate::Error>(())
					}.await;
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
            for mut file in crate::state::instances::adapters::sqlite::content_rows::get_instance_files(&instance.id, &self.pool).await? {
				let Some(binding) = catalog::file_storage(&self.pool, &file.id).await? else { continue; };
				let file_status = self.check_instance_file(&instance, &file, &binding).await?;
				let content = self.file_content(&file).await?;
				if file_status == InstanceFileStatus::Healthy
					&& matches!(&content, FileContent::Stored { .. })
					&& !file.missing
				{
					continue;
				}
				let mut fixed = false;
				if repair {
					if let FileContent::Stored { stored_file, .. } = &content
						&& file_status == InstanceFileStatus::Missing
					{
						let path = self.instance_path(&instance.path, &content_file_path(&file)).await?;
						let mode = self.create_instance_file(stored_file, &path, binding.materialization_kind == InstanceFileKind::Copy).await?;
						let mut tx = self.pool.begin().await?;
						file.missing = false;
						crate::state::instances::adapters::sqlite::content_rows::upsert_instance_file(&file, &mut tx).await?;
						catalog::set_file_storage(&mut tx, &file.id, &binding.blob_sha512, mode).await?;
						tx.commit().await?;
						fixed = true;
					} else if matches!(&content, FileContent::Stored { .. })
						&& file_status == InstanceFileStatus::Healthy
						&& file.missing
					{
						let mut tx = self.pool.begin().await?;
						file.missing = false;
						crate::state::instances::adapters::sqlite::content_rows::upsert_instance_file(&file, &mut tx).await?;
						tx.commit().await?;
						fixed = true;
					}
				}
				if fixed { report.repaired += 1; }
				else {
					let reason = match (&content, file_status) {
						(FileContent::Damaged(_), _) => "the stored content is missing or damaged",
						(_, InstanceFileStatus::Conflict) => "the instance path contains different content; preserve it and resolve the conflict",
						(_, InstanceFileStatus::Missing) => "the instance file is missing",
						(_, InstanceFileStatus::Healthy) => "the instance file metadata needs repair",
					};
					report.issues.push(StoreIssue {
						sha512: binding.blob_sha512, instance_ids: vec![instance.id.clone()],
						message: format!("{}: {reason}", file.relative_path),
					});
				}
			}
        }
        Ok(report)
    }

    pub(crate) async fn remove_abandoned_staging(&self) -> crate::Result<()> {
        let mut entries = fs::read_dir(&self.staging).await?;
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name();
            if name.to_string_lossy().starts_with(".tmp")
                && entry.file_type().await?.is_file()
            {
                fs::remove_file(entry.path()).await?;
            }
        }
        Ok(())
    }

    /// Registers payloads published before a crash interrupted their catalog write.
    pub(crate) async fn recover_unregistered_files(&self) -> crate::Result<()> {
        let known = catalog::stored_files(&self.pool)
            .await?
            .into_iter()
            .map(|stored_file| stored_file.sha512)
            .collect::<HashSet<_>>();
        let objects = self.root.join("objects/sha512");
        self.validate_object_parent(&objects.join("payload.jar"))
            .await?;
        let mut prefixes = match fs::read_dir(&objects).await {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        while let Some(prefix) = prefixes.next_entry().await? {
            let name = prefix.file_name().to_string_lossy().into_owned();
            if super::validate_digest(&name, 2).is_err()
                || !prefix.file_type().await?.is_dir()
            {
                continue;
            }
            let mut entries = fs::read_dir(prefix.path()).await?;
            while let Some(entry) = entries.next_entry().await? {
                let hash = entry.file_name().to_string_lossy().into_owned();
                if super::validate_digest(&hash, 128).is_err()
                    || !hash.starts_with(&name)
                    || known.contains(&hash)
                    || !entry.file_type().await?.is_dir()
                {
                    continue;
                }
                for filename in ["payload.jar", "payload.bin"] {
                    let path = entry.path().join(filename);
                    let metadata = match fs::symlink_metadata(&path).await {
                        Ok(metadata) if metadata.is_file() => metadata,
                        Ok(_) => continue,
                        Err(error)
                            if error.kind() == std::io::ErrorKind::NotFound =>
                        {
                            continue;
                        }
                        Err(error) => return Err(error.into()),
                    };
                    let (sha512, sha1, size) = super::hash_file(&path).await?;
                    if sha512 != hash {
                        tracing::warn!(path = %path.display(), "Preserving an unregistered store object with an unexpected hash");
                        continue;
                    }
                    let mut permissions = metadata.permissions();
                    permissions.set_readonly(true);
                    fs::set_permissions(&path, permissions).await?;
                    catalog::save_file(
                        &self.pool,
                        &super::StoredFile {
                            sha512,
                            sha1,
                            size: size.try_into().map_err(|_| {
                                input("Content file is too large")
                            })?,
                            relative_path: format!(
                                "objects/sha512/{name}/{hash}/{filename}"
                            ),
                            status: StoredFileStatus::Ready,
                            modified_at_ns: crate::state::file_modified_at_ns(
                                &metadata,
                            )?
                                as i64,
                            last_used_at: chrono::Utc::now().timestamp(),
                            sources: "[]".to_string(),
                        },
                    )
                    .await?;
                    break;
                }
            }
        }
        Ok(())
    }
}

use super::{
	BlobStatus, ContentProjectionStatus, ContentStore, FileContent,
	MaterializationKind, catalog, content_file_path, input, sync_directory,
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
    pub(crate) async fn ingest_local_file(
        &self,
        source: &std::path::Path,
        state: &State,
    ) -> crate::Result<super::BlobLease> {
		let staged = self.stage_file(source).await?;
		let hash = staged.sha512.clone();
        let _files_lock = self.files_lock.lock().await;
        if let Some(blob) = self.catalog_blob(&hash).await? {
            if !self.is_healthy(&blob.blob, true).await? {
                for instance in
                    instance_rows::list_instances(&self.pool).await?
                {
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
                if fs::symlink_metadata(&blob.path).await.is_ok() {
                    let quarantine = self.root.join("quarantine");
                    fs::create_dir_all(&quarantine).await?;
                    fs::rename(
                        &blob.path,
                        quarantine.join(format!(
                            "{}-{}",
                            hash,
                            uuid::Uuid::new_v4()
                        )),
                    )
                    .await?;
                }
            }
        }
		self.publish_staged(staged, &[]).await
    }

    pub async fn usage(&self) -> crate::Result<StoreUsage> {
        let _lease = self.lease().await;
        let blobs = catalog::blobs(&self.pool).await?;
        let roots = catalog::roots(&self.pool)
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
					== MaterializationKind::Copy.as_str()
			})
            .map(|placement| placement.size.max(0) as u64)
            .sum::<u64>();
        let referenced_unique = blobs
            .iter()
            .filter(|blob| installed.contains(blob.sha512.as_str()))
            .map(|blob| blob.size as u64)
            .sum::<u64>();
		let mut shared_placements = HashMap::new();
		for placement in &placements {
			if placement.materialization_kind
				== MaterializationKind::Symlink.as_str()
			{
				*shared_placements
					.entry(placement.blob_sha512.as_str())
					.or_insert(0usize) += 1;
			}
		}
        Ok(StoreUsage {
            unique_bytes: blobs.iter().map(|blob| blob.size as u64).sum(),
			shared_bytes: blobs
				.iter()
				.filter(|blob| {
					shared_placements.get(blob.sha512.as_str()).copied().unwrap_or(0) > 1
				})
				.map(|blob| blob.size as u64)
				.sum(),
			unused_cache_bytes: blobs
                .iter()
                .filter(|blob| !roots.contains(&blob.sha512))
                .map(|blob| blob.size as u64)
                .sum(),
            estimated_saved_bytes: logical_bytes.saturating_sub(
                referenced_unique.saturating_add(private_copy_bytes),
            ),
            private_copy_bytes,
            object_count: blobs.len(),
            damaged_objects: blobs
                .iter()
				.filter(|blob| blob.status == BlobStatus::Quarantined)
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

    /// Only unreferenced content objects are disposable; other store directories have separate owners.
    pub async fn cleanup(&self, purge_unused: bool) -> crate::Result<u64> {
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
        let roots = catalog::roots(&self.pool)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();
        let mut candidates = catalog::blobs(&self.pool)
            .await?
            .into_iter()
            .filter(|blob| !roots.contains(&blob.sha512))
            .collect::<Vec<_>>();
        candidates.sort_by_key(|blob| blob.last_used_at);
        let mut unused =
            candidates.iter().map(|blob| blob.size as u64).sum::<u64>();
        let limit = if purge_unused {
            0
        } else {
            self.cache_limit().await?
        };
        let mut reclaimed = 0;
        for blob in candidates {
			if unused <= limit && blob.status != BlobStatus::Deleting {
                continue;
            }
            if !purge_unused
                && chrono::Utc::now()
                    .timestamp()
                    .saturating_sub(blob.last_used_at)
                    < 60
            {
                continue;
            }
			catalog::set_status(
				&self.pool,
				&blob.sha512,
				BlobStatus::Deleting,
			)
			.await?;
            let path = self.path(&blob)?;
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
            catalog::delete(&self.pool, &blob.sha512).await?;
            unused = unused.saturating_sub(blob.size as u64);
            reclaimed += blob.size as u64;
        }
        Ok(reclaimed)
    }

    pub async fn verify(
        &self,
        state: &State,
        repair: bool,
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
        let blobs = catalog::blobs(&self.pool).await?;
        let mut report = StoreVerification {
            checked: 0,
            repaired: 0,
            issues: Vec::new(),
        };
        for blob in blobs {
            report.checked += 1;
            if self.is_healthy(&blob, true).await? {
                continue;
            }
            let mut message = "Stored content is missing or damaged; re-import the original file".to_string();
            if repair {
                let mut sources: Vec<String> =
                    serde_json::from_str(&blob.sources)?;
                if sources.is_empty() {
                    if let Ok(files) = crate::state::CachedEntry::get_file_many(
                        &[blob.sha1.as_str()],
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
                                        == Some(&blob.sha512)
                                    {
                                        sources.push(candidate.url);
                                    }
                                }
                            }
                        }
                    }
                }
                if !sources.is_empty() {
                    let result = async {
						let mirrors = sources.iter().map(String::as_str).collect::<Vec<_>>();
						let downloaded = crate::util::fetch::fetch_file_mirrors_in(&mirrors, Some(&blob.sha1), None, None, &state.fetch_semaphore, &self.pool, None, Some(&self.staging)).await?;
						if downloaded.sha512 != blob.sha512 || downloaded.size != blob.size as u64 { return Err(input("Repair download has an unexpected hash or size")); }
						let path = self.path(&blob)?;
						if fs::symlink_metadata(&path).await.is_ok() {
							let quarantine = self.root.join("quarantine");
							fs::create_dir_all(&quarantine).await?;
							fs::rename(&path, quarantine.join(format!("{}-{}", blob.sha512, uuid::Uuid::new_v4()))).await?;
						}
						self.publish_staged(downloaded.into_staged()?, &sources).await?;
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
            let instance_ids = sqlx::query_scalar!("SELECT DISTINCT file.instance_id FROM instance_files file INNER JOIN store_instance_files binding ON binding.file_id = file.id WHERE binding.blob_sha512 = ?", blob.sha512).fetch_all(&self.pool).await?;
            report.issues.push(StoreIssue {
                sha512: blob.sha512,
                instance_ids,
                message,
            });
        }
        for instance in instance_rows::list_instances(&self.pool).await? {
            for mut file in crate::state::instances::adapters::sqlite::content_rows::get_instance_files(&instance.id, &self.pool).await? {
				let Some(binding) = catalog::binding(&self.pool, &file.id).await? else { continue; };
				let projection = self.inspect_projection(&instance, &file, &binding).await?;
				let content = self.file_content(&file).await?;
				if projection == ContentProjectionStatus::Healthy
					&& matches!(&content, FileContent::Stored(_))
					&& !file.missing
				{
					continue;
				}
				let mut fixed = false;
				if repair {
					if let FileContent::Stored(blob) = &content
						&& projection == ContentProjectionStatus::Missing
					{
						let path = self.instance_path(&instance.path, &content_file_path(&file)).await?;
						let mode = self.materialize(blob, &path, binding.materialization_kind == MaterializationKind::Copy).await?;
						let mut tx = self.pool.begin().await?;
						file.missing = false;
						crate::state::instances::adapters::sqlite::content_rows::upsert_instance_file(&file, &mut tx).await?;
						catalog::bind(&mut tx, &file.id, &binding.blob_sha512, mode).await?;
						tx.commit().await?;
						fixed = true;
					} else if matches!(&content, FileContent::Stored(_))
						&& projection == ContentProjectionStatus::Healthy
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
					let reason = match (&content, projection) {
						(FileContent::Damaged(_), _) => "the stored content is missing or damaged",
						(_, ContentProjectionStatus::Conflict) => "the instance path contains different content; preserve it and resolve the conflict",
						(_, ContentProjectionStatus::Missing) => "the instance projection is missing",
						(_, ContentProjectionStatus::Healthy) => "the instance projection metadata needs repair",
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
}

pub(crate) mod catalog;
mod maintenance;
pub(crate) mod migration;
mod operations;
mod runtime;

pub use maintenance::{StoreUsage, StoreVerification};
pub(crate) use operations::{
    ContentProjectionStatus, PreparedProjection, content_file_path,
    materialized_content_path,
};

use crate::state::{DirectoryInfo, InstanceFile, file_modified_at_ns};
use crate::util::fetch::{self, DownloadMeta, FetchProgressFn, FetchSemaphore};
use fs4::tokio::AsyncFileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use sqlx::SqlitePool;
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, OwnedRwLockReadGuard, RwLock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Blob {
    pub sha512: String,
    pub sha1: String,
    pub size: i64,
    pub relative_path: String,
    pub status: BlobStatus,
    pub modified_at_ns: i64,
    pub last_used_at: i64,
    pub sources: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Binding {
    pub file_id: String,
    pub blob_sha512: String,
    pub materialization_kind: MaterializationKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MaterializationKind {
    Symlink,
    Copy,
}

impl MaterializationKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Symlink => "symlink",
            Self::Copy => "copy",
        }
    }

    fn from_db(value: &str) -> crate::Result<Self> {
        match value {
            "symlink" => Ok(Self::Symlink),
            "copy" => Ok(Self::Copy),
            _ => Err(input("Invalid content materialization kind")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BlobStatus {
    Ready,
    Quarantined,
    Deleting,
}

impl BlobStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Quarantined => "quarantined",
            Self::Deleting => "deleting",
        }
    }

    fn from_db(value: &str) -> crate::Result<Self> {
        match value {
            "ready" => Ok(Self::Ready),
            "quarantined" => Ok(Self::Quarantined),
            "deleting" => Ok(Self::Deleting),
            _ => Err(input("Invalid content object status")),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BlobLease {
    pub blob: Blob,
    pub path: PathBuf,
    _guard: Arc<OwnedRwLockReadGuard<()>>,
}

#[derive(Clone, Debug)]
pub(crate) struct CatalogBlob {
    pub blob: Blob,
    pub path: PathBuf,
    _guard: Arc<OwnedRwLockReadGuard<()>>,
}

#[derive(Clone, Debug)]
pub(crate) enum FileContent {
    Unmanaged,
    Stored(BlobLease),
    Damaged(Binding),
}

#[derive(Clone, Debug)]
pub(crate) enum ReadableContent {
    Local(PathBuf),
    Stored(BlobLease),
}

impl ReadableContent {
    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::Local(path) => path,
            Self::Stored(blob) => &blob.path,
        }
    }
}

pub(crate) struct AcquireResult {
    pub blob: BlobLease,
    pub reused: bool,
}

pub struct ContentStore {
    pub(crate) root: PathBuf,
    pub(crate) staging: PathBuf,
    pub(crate) profiles: PathBuf,
    pub(crate) pool: SqlitePool,
    gate: Arc<RwLock<()>>,
	pub(crate) runtime_gate: RwLock<()>,
    acquisitions: [Mutex<()>; 64],
    publications: [Mutex<()>; 64],
    pub(crate) files_lock: Mutex<()>,
    /// Background migration takes a reader per file; Play takes the writer to pause it.
    pub(crate) legacy_migration_priority: RwLock<()>,
    link_support: Mutex<HashMap<PathBuf, bool>>,
    _process_lock: File,
    _content_process_lock: Option<File>,
}

impl ContentStore {
    pub(crate) async fn lock_process(
        settings_dir: &Path,
    ) -> crate::Result<File> {
        fs::create_dir_all(settings_dir).await?;
        let file = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(settings_dir.join("store.lock"))
            .await?;
        if !file.try_lock_exclusive()? {
            return Err(input(
                "Another Modrinth process is using this application directory",
            ));
        }
        Ok(file)
    }

    pub(crate) async fn new(
        dirs: &DirectoryInfo,
        pool: SqlitePool,
        process_lock: File,
    ) -> crate::Result<Self> {
        fs::create_dir_all(dirs.content_store_dir()).await?;
        fs::create_dir_all(dirs.store_staging_dir()).await?;
        fs::create_dir_all(dirs.instances_dir()).await?;
        let settings_root = fs::canonicalize(&dirs.settings_dir).await?;
        let content_root = fs::canonicalize(&dirs.config_dir).await?;
        let content_process_lock = if settings_root != content_root {
            Some(Self::lock_process(&content_root).await?)
        } else {
            None
        };
        Ok(Self {
            root: fs::canonicalize(dirs.content_store_dir()).await?,
            staging: fs::canonicalize(dirs.store_staging_dir()).await?,
            profiles: fs::canonicalize(dirs.instances_dir()).await?,
            pool,
            gate: Arc::new(RwLock::new(())),
			runtime_gate: RwLock::new(()),
            acquisitions: std::array::from_fn(|_| Mutex::new(())),
            publications: std::array::from_fn(|_| Mutex::new(())),
            files_lock: Mutex::new(()),
            legacy_migration_priority: RwLock::new(()),
            link_support: Mutex::new(HashMap::new()),
            _process_lock: process_lock,
            _content_process_lock: content_process_lock,
        })
    }

    pub(crate) async fn lease(&self) -> Arc<OwnedRwLockReadGuard<()>> {
        Arc::new(self.gate.clone().read_owned().await)
    }

    fn acquisition_lock(&self, key: &str) -> &Mutex<()> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        &self.acquisitions[hasher.finish() as usize % self.acquisitions.len()]
    }

    fn publication_lock(&self, sha512: &str) -> &Mutex<()> {
        let mut hasher = DefaultHasher::new();
        sha512.hash(&mut hasher);
        &self.publications[hasher.finish() as usize % self.publications.len()]
    }

    fn path(&self, blob: &Blob) -> crate::Result<PathBuf> {
        validate_digest(&blob.sha512, 128)?;
        let prefix =
            format!("objects/sha512/{}/{}/", &blob.sha512[..2], blob.sha512);
        if !blob.relative_path.starts_with(&prefix)
            || !matches!(
                blob.relative_path.strip_prefix(&prefix),
                Some("payload.jar" | "payload.bin")
            )
        {
            return Err(input("Invalid content store object path"));
        }
        Ok(self.root.join(&blob.relative_path))
    }

    pub(crate) async fn lookup(
        &self,
        sha512: Option<&str>,
        sha1: Option<&str>,
        size: Option<u64>,
    ) -> crate::Result<Option<BlobLease>> {
        if let Some(hash) = sha512 {
            validate_digest(hash, 128)?;
        }
        if let Some(hash) = sha1 {
            validate_digest(hash, 40)?;
        }
        let guard = self.lease().await;
        self.lookup_with_guard(sha512, sha1, size, guard).await
    }

    async fn lookup_with_guard(
        &self,
        sha512: Option<&str>,
        sha1: Option<&str>,
        size: Option<u64>,
        guard: Arc<OwnedRwLockReadGuard<()>>,
    ) -> crate::Result<Option<BlobLease>> {
        let candidates = catalog::find(&self.pool, sha512, sha1)
            .await?
            .into_iter()
            .filter(|blob| sha1.is_none_or(|expected| blob.sha1 == expected))
            .filter(|blob| {
                sha512.is_some()
                    || size.is_none_or(|size| blob.size as u64 == size)
            })
            .collect::<Vec<_>>();
        if candidates.len() != 1 {
            return Ok(None);
        }
        let blob = candidates
            .into_iter()
            .next()
            .ok_or_else(|| input("Missing content object"))?;
        if !self.is_healthy(&blob, false).await? {
            return Ok(None);
        }
        if let Some(expected_size) = size
            && expected_size != blob.size as u64
        {
            tracing::warn!(
                expected_size,
                actual_size = blob.size,
                sha512 = %blob.sha512,
                "Cached content matches SHA-512 but differs from the declared size; using its actual size"
            );
        }
        catalog::touch(&self.pool, &blob.sha512).await?;
        Ok(Some(BlobLease {
            path: self.path(&blob)?,
            blob,
            _guard: guard,
        }))
    }

    pub(crate) async fn is_healthy(
        &self,
        blob: &Blob,
        verify: bool,
    ) -> crate::Result<bool> {
		self.is_healthy_with_progress(blob, verify, &|_| {}).await
	}

	pub(crate) async fn is_healthy_with_progress(
		&self,
		blob: &Blob,
		verify: bool,
		on_read: &(dyn Fn(u64) + Send + Sync),
	) -> crate::Result<bool> {
        if blob.status != BlobStatus::Ready && !verify {
            return Ok(false);
        }
        let path = self.path(blob)?;
        self.validate_object_parent(&path).await?;
        let metadata = match fs::symlink_metadata(&path).await {
            Ok(metadata) if metadata.is_file() => metadata,
            Ok(_) => {
                catalog::set_status(
                    &self.pool,
                    &blob.sha512,
                    BlobStatus::Quarantined,
                )
                .await?;
                return Ok(false);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                catalog::set_status(
                    &self.pool,
                    &blob.sha512,
                    BlobStatus::Quarantined,
                )
                .await?;
                return Ok(false);
            }
            Err(error) => return Err(error.into()),
        };
        let modified_at_ns = file_modified_at_ns(&metadata)? as i64;
        if !verify
            && metadata.len() == blob.size as u64
            && modified_at_ns == blob.modified_at_ns
        {
            return Ok(true);
        }
        let hashes = hash_file_with_progress(&path, on_read).await?;
        if hashes.0 != blob.sha512
            || hashes.1 != blob.sha1
            || hashes.2 != blob.size as u64
        {
            catalog::set_status(
                &self.pool,
                &blob.sha512,
                BlobStatus::Quarantined,
            )
            .await?;
            return Ok(false);
        }
        let mut verified = blob.clone();
        verified.modified_at_ns = modified_at_ns;
        catalog::put(&self.pool, &verified).await?;
        Ok(true)
    }

    pub(crate) async fn acquire(
        &self,
        mirrors: &[&str],
        sha512: Option<&str>,
        sha1: Option<&str>,
        size: Option<u64>,
        download_meta: Option<&DownloadMeta>,
        semaphore: &FetchSemaphore,
        progress: Option<&mut FetchProgressFn<'_>>,
    ) -> crate::Result<AcquireResult> {
        let key = sha512
            .map(|hash| format!("sha512:{hash}"))
            .or_else(|| sha1.map(|hash| format!("sha1:{hash}")));
        let _acquisition = if let Some(key) = &key {
            Some(self.acquisition_lock(key).lock().await)
        } else {
            None
        };
        if let Some(blob) = self.lookup(sha512, sha1, size).await? {
            return Ok(AcquireResult { blob, reused: true });
        }
        let download = fetch::fetch_file_mirrors_in(
            mirrors,
            sha1,
            download_meta,
            None,
            semaphore,
            &self.pool,
            progress,
            Some(&self.staging),
        )
        .await?;
        if let Some(expected_sha512) = sha512
            && expected_sha512 != download.sha512
        {
            return Err(input(format!(
                "Downloaded content SHA-512 mismatch: expected {expected_sha512}, got {}",
                download.sha512,
            )));
        }
        if let Some(expected_size) = size
            && expected_size != download.size
        {
            if sha512.is_none() {
                return Err(input(format!(
                    "Downloaded content size mismatch: expected {expected_size} bytes, got {} bytes",
                    download.size,
                )));
            }
            tracing::warn!(
                expected_size,
                actual_size = download.size,
                sha512 = %download.sha512,
                "Downloaded content matches SHA-512 but differs from the declared size; using its actual size"
            );
        }
        let sources = mirrors
            .iter()
            .filter_map(|source| {
                let url = url::Url::parse(source).ok()?;
                (url.scheme() == "https"
                    && url.username().is_empty()
                    && url.password().is_none()
                    && url.query().is_none())
                .then(|| source.to_string())
            })
            .collect::<Vec<_>>();
        let blob = self
            .publish_staged(download.into_staged()?, &sources)
            .await?;
        Ok(AcquireResult {
            blob,
            reused: false,
        })
    }

    pub(crate) async fn ingest_file(
        &self,
        source: &Path,
    ) -> crate::Result<BlobLease> {
        self.ingest_file_with_sources(source, &[]).await
    }

    async fn ingest_file_with_sources(
        &self,
        source: &Path,
        sources: &[String],
    ) -> crate::Result<BlobLease> {
        if let Some(blob) = self.owned_path(source).await? {
            return Ok(blob);
        }
        let (sha512, sha1, size) = hash_file(source).await?;
        if let Some(blob) =
            self.lookup(Some(&sha512), Some(&sha1), Some(size)).await?
        {
            return Ok(blob);
        }
        self.require_staging_space(size)?;
        let staged = self.stage_file(source).await?;
        self.publish_staged(staged, sources).await
    }

    pub(crate) fn require_staging_space(&self, size: u64) -> crate::Result<()> {
        if fs4::available_space(&self.staging)?
            < size.saturating_add(64 * 1024 * 1024)
        {
            return Err(std::io::Error::new(
				std::io::ErrorKind::StorageFull,
				"There is not enough space to import this file into the shared content store",
			).into());
        }
        Ok(())
    }

    pub(super) async fn stage_file(
        &self,
        source: &Path,
    ) -> crate::Result<fetch::StagedDownload> {
        let mut input_file = File::open(source).await?;
        let before = input_file.metadata().await?;
        if !before.is_file() {
            return Err(input("Only regular content files can be stored"));
        }
        let temporary = self.temporary().await?;
        let mut output = File::create(&temporary).await?;
        let mut sha512 = Sha512::new();
        let mut sha1 = sha1_smol::Sha1::new();
        let mut buffer = vec![0; 256 * 1024];
        let mut size = 0_u64;
        let mut prefix = [0_u8; 2];
        let mut prefix_len = 0;
        loop {
            let count = input_file.read(&mut buffer).await?;
            if count == 0 {
                break;
            }
            let prefix_count = (prefix.len() - prefix_len).min(count);
            prefix[prefix_len..prefix_len + prefix_count]
                .copy_from_slice(&buffer[..prefix_count]);
            prefix_len += prefix_count;
            output.write_all(&buffer[..count]).await?;
            sha512.update(&buffer[..count]);
            sha1.update(&buffer[..count]);
            size += count as u64;
        }
        output.sync_all().await?;
        drop(output);
        let after = input_file.metadata().await?;
        if before.len() != after.len()
            || before.modified()? != after.modified()?
            || size != after.len()
        {
            return Err(input(
                "Content changed while it was being imported; try again after closing the instance",
            ));
        }
        Ok(fetch::StagedDownload {
            path: temporary,
            size,
            sha1: sha1.hexdigest(),
            sha512: format!("{:x}", sha512.finalize()),
            archive: prefix_len == prefix.len() && prefix == *b"PK",
        })
    }

    pub(super) async fn publish_staged(
        &self,
        staged: fetch::StagedDownload,
        sources: &[String],
    ) -> crate::Result<BlobLease> {
        if staged.path.parent() != Some(self.staging.as_path()) {
            return Err(input("Content was not staged in the shared store"));
        }
        let fetch::StagedDownload {
            path: temporary,
            size,
            sha1,
            sha512: hash,
            archive,
        } = staged;
        let guard = self.lease().await;
        let _publication = self.publication_lock(&hash).lock().await;
        if let Some(existing) = self
            .lookup_with_guard(Some(&hash), None, Some(size), guard.clone())
            .await?
        {
            return Ok(existing);
        }
        let relative_path = format!(
            "objects/sha512/{}/{}/payload.{}",
            &hash[..2],
            hash,
            if archive { "jar" } else { "bin" }
        );
        let destination = self.root.join(&relative_path);
        let parent = destination
            .parent()
            .ok_or_else(|| input("Invalid store path"))?;
        self.validate_object_parent(&destination).await?;
        fs::create_dir_all(parent).await?;
        self.validate_object_parent(&destination).await?;
        if let Ok(metadata) = fs::symlink_metadata(&destination).await {
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || hash_file(&destination).await?.0 != hash
            {
                return Err(input(format!(
                    "Content object {hash} needs repair before it can be replaced"
                )));
            }
        } else {
            let destination_copy = destination.clone();
            tokio::task::spawn_blocking(move || {
                temporary
                    .persist_noclobber(destination_copy)
                    .map_err(|error| error.error)
            })
            .await??;
        }
        let mut permissions = fs::metadata(&destination).await?.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&destination, permissions).await?;
        sync_directory(parent).await?;
        let blob = Blob {
            sha512: hash,
            sha1,
            size: size
                .try_into()
                .map_err(|_| input("Content file is too large"))?,
            relative_path,
            status: BlobStatus::Ready,
            modified_at_ns: file_modified_at_ns(
                &fs::metadata(&destination).await?,
            )? as i64,
            last_used_at: chrono::Utc::now().timestamp(),
            sources: serde_json::to_string(sources)?,
        };
        catalog::put(&self.pool, &blob).await?;
        Ok(BlobLease {
            blob,
            path: destination,
            _guard: guard,
        })
    }

    pub(crate) async fn ingest_bytes(
        &self,
        bytes: &[u8],
        expected_sha1: Option<&str>,
    ) -> crate::Result<BlobLease> {
        let sha1 = sha1_smol::Sha1::from(bytes).hexdigest();
        if let Some(expected) = expected_sha1 {
            validate_digest(expected, 40)?;
            if sha1 != expected {
                return Err(input(
                    "Content bytes do not match the expected hash",
                ));
            }
        }
        let temporary = self.temporary().await?;
        let mut file = File::create(&temporary).await?;
        file.write_all(bytes).await?;
        file.sync_all().await?;
        drop(file);
        self.publish_staged(
            fetch::StagedDownload {
                path: temporary,
                size: bytes.len() as u64,
                sha1,
                sha512: format!("{:x}", Sha512::digest(bytes)),
                archive: bytes.starts_with(b"PK"),
            },
            &[],
        )
        .await
    }

    pub(crate) async fn temporary(&self) -> crate::Result<tempfile::TempPath> {
        let staging = self.staging.clone();
        Ok(tokio::task::spawn_blocking(move || {
            tempfile::NamedTempFile::new_in(staging)
                .map(|file| file.into_temp_path())
        })
        .await??)
    }

    pub(crate) async fn owned_path(
        &self,
        path: &Path,
    ) -> crate::Result<Option<BlobLease>> {
        let canonical = match fs::canonicalize(path).await {
            Ok(path) => path,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(error) => return Err(error.into()),
        };
        let Ok(relative) = canonical.strip_prefix(&self.root) else {
            return Ok(None);
        };
        let components = relative.components().collect::<Vec<_>>();
        if components.len() != 5 {
            return Ok(None);
        }
        let Some(hash) = components[3].as_os_str().to_str() else {
            return Ok(None);
        };
        if hash.len() != 128
            || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Ok(None);
        }
        let Some(blob) = self.lookup(Some(hash), None, None).await? else {
            return Ok(None);
        };
        Ok((blob.path == canonical).then_some(blob))
    }

    pub(crate) async fn file_content(
        &self,
        file: &InstanceFile,
    ) -> crate::Result<FileContent> {
        let Some(binding) = catalog::binding(&self.pool, &file.id).await?
        else {
            return Ok(FileContent::Unmanaged);
        };
        let blob = self
            .lookup(Some(&binding.blob_sha512), None, Some(file.size))
            .await?;
        Ok(match blob {
            Some(blob) => FileContent::Stored(blob),
            None => FileContent::Damaged(binding),
        })
    }

    pub(crate) async fn read_path(
        &self,
        file: &InstanceFile,
        instance_path: &str,
    ) -> crate::Result<ReadableContent> {
        match self.file_content(file).await? {
            FileContent::Stored(blob) => {
                return Ok(ReadableContent::Stored(blob));
            }
            FileContent::Damaged(_) => {
                return Err(input(format!(
                    "{} needs repair or re-import",
                    file.relative_path
                )));
            }
            FileContent::Unmanaged => {}
        }
        validate_relative(instance_path)?;
        let relative_path = content_file_path(file);
        validate_relative(&relative_path)?;
        let path = self.profiles.join(instance_path).join(relative_path);
        if !fs::metadata(&path).await?.is_file() {
            return Err(input("Only regular content files can be read"));
        }
        Ok(ReadableContent::Local(path))
    }

    pub(crate) async fn instance_path(
        &self,
        instance_path: &str,
        relative_path: &str,
    ) -> crate::Result<PathBuf> {
        validate_relative(instance_path)?;
        validate_relative(relative_path)?;
        let base = self.profiles.join(instance_path);
        let destination = base.join(relative_path);
        let parent = destination
            .parent()
            .ok_or_else(|| input("Invalid instance path"))?;
        let mut current = self.profiles.clone();
        for component in parent.strip_prefix(&self.profiles)?.components() {
            current.push(component);
            match fs::symlink_metadata(&current).await {
                Ok(metadata)
                    if !metadata.is_dir()
                        || metadata.file_type().is_symlink() =>
                {
                    return Err(input(
                        "Content directories must not be symbolic links",
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(destination)
    }

    pub(crate) async fn retain(
        &self,
        kind: &str,
        owner: &str,
        blobs: &[String],
    ) -> crate::Result<()> {
        let _lease = self.lease().await;
        let mut tx = self.pool.begin().await?;
        for blob in blobs {
            catalog::retain(&mut tx, kind, owner, blob).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub(crate) async fn replace_retained(
        &self,
        kind: &str,
        owner: &str,
        blobs: &[String],
    ) -> crate::Result<()> {
        let _lease = self.lease().await;
        let mut tx = self.pool.begin().await?;
        catalog::release(&mut tx, kind, owner).await?;
        for blob in blobs {
            catalog::retain(&mut tx, kind, owner, blob).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub(crate) async fn catalog_blob(
        &self,
        sha512: &str,
    ) -> crate::Result<Option<CatalogBlob>> {
        let guard = self.lease().await;
        let Some(blob) = catalog::find(&self.pool, Some(sha512), None)
            .await?
            .into_iter()
            .next()
        else {
            return Ok(None);
        };
        let path = self.path(&blob)?;
        self.validate_object_parent(&path).await?;
        Ok(Some(CatalogBlob {
            blob,
            path,
            _guard: guard,
        }))
    }

    async fn validate_object_parent(&self, path: &Path) -> crate::Result<()> {
        let parent = path
            .parent()
            .ok_or_else(|| input("Invalid store object path"))?;
        let mut current = self.root.clone();
        for component in parent.strip_prefix(&self.root)?.components() {
            current.push(component);
            match fs::symlink_metadata(&current).await {
                Ok(metadata)
                    if !metadata.is_dir()
                        || metadata.file_type().is_symlink() =>
                {
                    return Err(input(
                        "Store object directories must not be symbolic links",
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }

    pub(crate) async fn release(
        &self,
        kind: &str,
        owner: &str,
    ) -> crate::Result<()> {
        let mut tx = self.pool.begin().await?;
        catalog::release(&mut tx, kind, owner).await?;
        tx.commit().await?;
        Ok(())
    }
}

pub(crate) fn input(message: impl Into<String>) -> crate::Error {
    crate::ErrorKind::InputError(message.into()).into()
}

pub(crate) fn validate_digest(hash: &str, length: usize) -> crate::Result<()> {
    if hash.len() != length
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(input("Invalid content hash"));
    }
    Ok(())
}

pub(crate) fn validate_relative(path: &str) -> crate::Result<()> {
    if path.is_empty()
        || path.contains('\\')
        || path.contains(':')
        || path.split('/').any(|part| {
            part.is_empty()
                || part == "."
                || part == ".."
                || part.ends_with('.')
                || part.ends_with(' ')
        })
        || Path::new(path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(input("Invalid instance-relative path"));
    }
    Ok(())
}

pub(crate) fn eligible(path: &str) -> bool {
    if validate_relative(path).is_err() {
        return false;
    }
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() != 2 {
        return false;
    }
    let extension = Path::new(parts[1].trim_end_matches(".disabled"))
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    match parts[0] {
        "mods" => extension.eq_ignore_ascii_case("jar"),
        "resourcepacks" | "shaderpacks" | "datapacks" => {
            extension.eq_ignore_ascii_case("jar")
                || extension.eq_ignore_ascii_case("zip")
        }
        _ => false,
    }
}

pub(crate) async fn hash_file(
    path: &Path,
) -> crate::Result<(String, String, u64)> {
	hash_file_with_progress(path, &|_| {}).await
}

async fn hash_file_with_progress(
	path: &Path,
	on_read: &(dyn Fn(u64) + Send + Sync),
) -> crate::Result<(String, String, u64)> {
    let mut file = File::open(path).await?;
    let mut sha512 = Sha512::new();
    let mut sha1 = sha1_smol::Sha1::new();
    let mut size = 0;
    let mut buffer = vec![0; 256 * 1024];
    loop {
        let count = file.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        sha512.update(&buffer[..count]);
        sha1.update(&buffer[..count]);
        size += count as u64;
		on_read(count as u64);
    }
    Ok((format!("{:x}", sha512.finalize()), sha1.hexdigest(), size))
}

pub(crate) async fn sync_directory(path: &Path) -> crate::Result<()> {
    #[cfg(unix)]
    File::open(path).await?.sync_all().await?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

pub(crate) fn relative_link(source: &Path, parent: &Path) -> PathBuf {
    let source_parts = source.components().collect::<Vec<_>>();
    let parent_parts = parent.components().collect::<Vec<_>>();
    let common = source_parts
        .iter()
        .zip(&parent_parts)
        .take_while(|(a, b)| a == b)
        .count();
    if common == 0 {
        return source.to_path_buf();
    }
    let mut relative = PathBuf::new();
    for _ in common..parent_parts.len() {
        relative.push("..");
    }
    for part in &source_parts[common..] {
        relative.push(part.as_os_str());
    }
    relative
}

pub(crate) fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            part => normalized.push(part.as_os_str()),
        }
    }
    normalized
}

pub(crate) async fn writable_copy(
    source: &Path,
    destination: &Path,
) -> crate::Result<()> {
    fs::copy(source, destination).await?;
    let mut permissions = fs::metadata(destination).await?.permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(permissions.mode() | 0o200);
    }
    #[cfg(not(unix))]
    permissions.set_readonly(false);
    fs::set_permissions(destination, permissions).await?;
    Ok(())
}

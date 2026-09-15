use super::{
    InstanceFileStorage, StoredFileMetadata,
    adapters::verified_files::VerifiedFiles, input,
};
use crate::state::DirectoryInfo;
use fs4::tokio::AsyncFileExt;
use sqlx::SqlitePool;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::sync::{Mutex, OwnedRwLockReadGuard, RwLock};
pub struct ContentStore {
    pub(crate) root: PathBuf,
    pub(crate) staging: PathBuf,
    pub(crate) profiles: PathBuf,
    pub(crate) pool: SqlitePool,
    pub(super) cleanup_lock: Arc<RwLock<()>>,
    pub(crate) runtime_cache_lock: RwLock<()>,
    pub(super) download_locks: [Mutex<()>; 64],
    pub(super) publish_locks: [Mutex<()>; 64],
    pub(crate) files_lock: Mutex<()>,
    pub(super) verified_files: VerifiedFiles,
    pub(crate) legacy_migration_priority: RwLock<()>,
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
            cleanup_lock: Arc::new(RwLock::new(())),
            runtime_cache_lock: RwLock::new(()),
            download_locks: std::array::from_fn(|_| Mutex::new(())),
            publish_locks: std::array::from_fn(|_| Mutex::new(())),
            files_lock: Mutex::new(()),
            verified_files: VerifiedFiles::default(),
            legacy_migration_priority: RwLock::new(()),
            _process_lock: process_lock,
            _content_process_lock: content_process_lock,
        })
    }

    pub(crate) async fn lease(&self) -> Arc<OwnedRwLockReadGuard<()>> {
        Arc::new(self.cleanup_lock.clone().read_owned().await)
    }

    pub(super) fn download_lock(&self, key: &str) -> &Mutex<()> {
        Self::key_lock(&self.download_locks, key)
    }

    pub(super) fn publish_lock(&self, sha512: &str) -> &Mutex<()> {
        Self::key_lock(&self.publish_locks, sha512)
    }

    fn key_lock<'a>(locks: &'a [Mutex<()>], key: &str) -> &'a Mutex<()> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        &locks[hasher.finish() as usize % locks.len()]
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StoredFileHandle {
    pub metadata: StoredFileMetadata,
    pub path: PathBuf,
    pub(super) _guard: Arc<OwnedRwLockReadGuard<()>>,
}

#[derive(Clone, Debug)]
pub(crate) struct StoredFileRecord {
    pub metadata: StoredFileMetadata,
    pub path: PathBuf,
    pub(super) _guard: Arc<OwnedRwLockReadGuard<()>>,
}

#[derive(Clone, Debug)]
pub(crate) enum FileContent {
    Unmanaged,
    Stored {
        storage: InstanceFileStorage,
        stored_file: StoredFileHandle,
    },
    Damaged(InstanceFileStorage),
}

#[derive(Clone, Debug)]
pub(crate) enum ReadableContent {
    Local(PathBuf),
    Stored(StoredFileHandle),
}

impl ReadableContent {
    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::Local(path) => path,
            Self::Stored(stored_file) => &stored_file.path,
        }
    }
}

pub(crate) struct GetFileResult {
    pub stored_file: StoredFileHandle,
    pub reused: bool,
}

use parking_lot::Mutex;
use sha2::{Digest, Sha512};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Instant, SystemTime};

const MAX_VERIFIED_FILES: usize = 4096;

#[derive(Clone, Default)]
pub(in crate::state::content_store) struct VerifiedFiles {
    files: Arc<Mutex<HashMap<PathBuf, VerifiedFile>>>,
}

struct VerifiedFile {
    stamp: FileStamp,
    sha512: String,
    last_used: Instant,
}

#[derive(PartialEq, Eq)]
struct FileStamp {
    identity: (u64, u64),
    size: u64,
    modified: SystemTime,
    changed: (i64, i64),
}

impl FileStamp {
    fn read(file: &File) -> Option<Self> {
        let metadata = file.metadata().ok()?;
        #[cfg(unix)]
        let (identity, changed) = {
            use std::os::unix::fs::MetadataExt;
            (
                (metadata.dev(), metadata.ino()),
                (metadata.ctime(), metadata.ctime_nsec()),
            )
        };
        #[cfg(windows)]
        let (identity, changed) = {
            use std::os::windows::io::AsRawHandle;
            use windows::Win32::Foundation::HANDLE;
            use windows::Win32::Storage::FileSystem::{
                BY_HANDLE_FILE_INFORMATION, FILE_BASIC_INFO, FileBasicInfo,
                GetFileInformationByHandle, GetFileInformationByHandleEx,
            };
            let handle = HANDLE(file.as_raw_handle());
            let mut information = BY_HANDLE_FILE_INFORMATION::default();
            let mut basic = FILE_BASIC_INFO::default();
            unsafe {
                GetFileInformationByHandle(handle, &mut information).ok()?;
                GetFileInformationByHandleEx(
                    handle,
                    FileBasicInfo,
                    (&mut basic as *mut FILE_BASIC_INFO).cast(),
                    size_of::<FILE_BASIC_INFO>() as u32,
                )
                .ok()?;
            }
            if basic.ChangeTime == 0 {
                return None;
            }
            (
                (
                    u64::from(information.dwVolumeSerialNumber),
                    (u64::from(information.nFileIndexHigh) << 32)
                        | u64::from(information.nFileIndexLow),
                ),
                (basic.ChangeTime, 0),
            )
        };
        Some(Self {
            identity,
            size: metadata.len(),
            modified: metadata.modified().ok()?,
            changed,
        })
    }
}

impl VerifiedFiles {
    /// Reuses a hash only while identity, size, write time and change time match.
    /// Filesystems without the required metadata are always read in full.
    pub(in crate::state::content_store) async fn hash_file(
        &self,
        path: &Path,
    ) -> crate::Result<String> {
        let cache = self.clone();
        let path = path.to_path_buf();
        Ok(
            tokio::task::spawn_blocking(move || cache.hash(&path, &|_| {}))
                .await??,
        )
    }

    fn hash(&self, path: &Path, on_read: &dyn Fn(u64)) -> io::Result<String> {
        let mut file = File::open(path)?;
        let before = FileStamp::read(&file);
        if let Some(stamp) = &before {
            let mut files = self.files.lock();
            if let Some(verified) = files.get_mut(path)
                && &verified.stamp == stamp
            {
                verified.last_used = Instant::now();
                return Ok(verified.sha512.clone());
            }
        }
        let mut hasher = Sha512::new();
        let mut buffer = vec![0; 256 * 1024];
        let mut size = 0;
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
            size += read as u64;
            on_read(read as u64);
        }
        let sha512 = format!("{:x}", hasher.finalize());
        if let Some(stamp) = before
            && FileStamp::read(&file).as_ref() == Some(&stamp)
            && size == stamp.size
        {
            let mut files = self.files.lock();
            if files.len() >= MAX_VERIFIED_FILES
                && !files.contains_key(path)
                && let Some(oldest) = files
                    .iter()
                    .min_by_key(|(_, verified)| verified.last_used)
                    .map(|(path, _)| path.clone())
            {
                files.remove(&oldest);
            }
            files.insert(
                path.to_path_buf(),
                VerifiedFile {
                    stamp,
                    sha512: sha512.clone(),
                    last_used: Instant::now(),
                },
            );
        }
        Ok(sha512)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::fs;
    use std::io::Write;

    #[test]
    fn unchanged_files_are_not_read_again() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("mod.jar");
        fs::write(&path, b"cached content").unwrap();
        let cache = VerifiedFiles::default();
        let bytes = Cell::new(0);
        let on_read = |read| bytes.set(bytes.get() + read);
        let expected = cache.hash(&path, &on_read).unwrap();
        assert_eq!(bytes.get(), 14);
        assert_eq!(cache.hash(&path, &on_read).unwrap(), expected);
        assert_eq!(bytes.get(), 14);
    }

    #[test]
    fn edits_with_restored_size_and_write_time_are_rehashed() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("mod.jar");
        fs::write(&path, b"original").unwrap();
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        let cache = VerifiedFiles::default();
        let original = cache.hash(&path, &|_| {}).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut file = File::options().write(true).open(&path).unwrap();
        file.write_all(b"modified").unwrap();
        file.set_modified(modified).unwrap();
        drop(file);
        assert_ne!(cache.hash(&path, &|_| {}).unwrap(), original);
    }

    #[test]
    fn replacement_with_matching_size_and_write_time_is_rehashed() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("mod.jar");
        fs::write(&path, b"original").unwrap();
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        let cache = VerifiedFiles::default();
        let original = cache.hash(&path, &|_| {}).unwrap();
        fs::rename(&path, directory.path().join("old.jar")).unwrap();
        fs::write(&path, b"replaced").unwrap();
        File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(modified)
            .unwrap();
        assert_ne!(cache.hash(&path, &|_| {}).unwrap(), original);
        fs::remove_file(&path).unwrap();
        assert_eq!(
            cache.hash(&path, &|_| {}).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
    }

    #[test]
    fn files_changed_during_verification_are_not_cached() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("mod.jar");
        let original = vec![0; 512 * 1024];
        let changed = vec![1; original.len()];
        fs::write(&path, &original).unwrap();
        let cache = VerifiedFiles::default();
        let edited = Cell::new(false);
        cache
            .hash(&path, &|_| {
                if !edited.replace(true) {
                    fs::write(&path, &changed).unwrap();
                }
            })
            .unwrap();
        let bytes = Cell::new(0);
        let actual = cache
            .hash(&path, &|read| bytes.set(bytes.get() + read))
            .unwrap();
        assert_eq!(bytes.get(), changed.len() as u64);
        assert_eq!(actual, format!("{:x}", Sha512::digest(&changed)));
    }
}

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use sha2::{Digest, Sha256};

use crate::RENDERER_VERSION;
use crate::texture::Texture;

/// A hash of its fields and [`RENDERER_VERSION`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CacheKey([u8; 32]);

impl CacheKey {
    pub fn builder() -> CacheKeyBuilder {
        let mut builder = CacheKeyBuilder(Sha256::new());
        builder.update(&RENDERER_VERSION.to_le_bytes());
        builder
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for CacheKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Fields are length-prefixed: `("ab", "c")` and `("a", "bc")` give different keys.
pub struct CacheKeyBuilder(Sha256);

impl CacheKeyBuilder {
    pub fn field(mut self, bytes: impl AsRef<[u8]>) -> Self {
        self.update(bytes.as_ref());
        self
    }

    pub fn texture(self, texture: &Texture) -> Self {
        self.field(texture.width().to_le_bytes())
            .field(texture.as_rgba8())
    }

    pub fn finish(self) -> CacheKey {
        CacheKey(self.0.finalize().into())
    }

    fn update(&mut self, bytes: &[u8]) {
        self.0.update((bytes.len() as u64).to_le_bytes());
        self.0.update(bytes);
    }
}

pub trait RenderCache {
    type Error;

    fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>, Self::Error>;

    fn put(&self, key: &CacheKey, image: &[u8]) -> Result<(), Self::Error>;
}

/// Stores each image as `<key>.<extension>` in one directory.
#[derive(Clone, Debug)]
pub struct DirCache {
    directory: PathBuf,
    extension: String,
}

impl DirCache {
    pub fn new(
        directory: impl Into<PathBuf>,
        extension: impl Into<String>,
    ) -> Self {
        Self {
            directory: directory.into(),
            extension: extension.into(),
        }
    }

    pub fn path(&self, key: &CacheKey) -> PathBuf {
        self.directory.join(format!("{key}.{}", self.extension))
    }

    /// Updates the file's modification time, which [`DirCache::prune`] reads as last use.
    pub fn lookup(&self, key: &CacheKey) -> Option<PathBuf> {
        let path = self.path(key);
        if !path.is_file() {
            return None;
        }
        if let Ok(file) = std::fs::File::options().write(true).open(&path) {
            let _ = file.set_modified(SystemTime::now());
        }
        Some(path)
    }

    /// Writes to a temporary file, then renames it into place.
    pub fn store(&self, key: &CacheKey, image: &[u8]) -> io::Result<PathBuf> {
        std::fs::create_dir_all(&self.directory)?;
        let path = self.path(key);
        static WRITES: AtomicU64 = AtomicU64::new(0);
        let temporary = self.directory.join(format!(
            "{key}.{}.{}.tmp",
            std::process::id(),
            WRITES.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&temporary, image)?;
        if let Err(error) = std::fs::rename(&temporary, &path) {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }
        Ok(path)
    }

    /// Deletes least recently used files until the directory is at most `max_bytes`.
    pub fn prune(&self, max_bytes: u64) -> io::Result<()> {
        let mut files = Vec::new();
        let entries = match std::fs::read_dir(&self.directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(());
            }
            Err(error) => return Err(error),
        };
        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                files.push((
                    metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    metadata.len(),
                    entry.path(),
                ));
            }
        }

        let mut total: u64 = files.iter().map(|(_, size, _)| size).sum();
        files.sort();
        for (_, size, path) in files {
            if total <= max_bytes {
                break;
            }
            remove_cached_file(&path)?;
            total -= size;
        }
        Ok(())
    }
}

fn remove_cached_file(path: &Path) -> io::Result<()> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

impl RenderCache for DirCache {
    type Error = io::Error;

    fn get(&self, key: &CacheKey) -> io::Result<Option<Vec<u8>>> {
        match self.lookup(key) {
            Some(path) => std::fs::read(path).map(Some),
            None => Ok(None),
        }
    }

    fn put(&self, key: &CacheKey, image: &[u8]) -> io::Result<()> {
        self.store(key, image).map(drop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(fields: &[&str]) -> CacheKey {
        fields
            .iter()
            .fold(CacheKey::builder(), |builder, field| builder.field(field))
            .finish()
    }

    fn scratch_directory(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "modrinth-skin-rendering-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&directory);
        directory
    }

    #[test]
    fn keys_depend_on_field_boundaries_and_content() {
        assert_eq!(key(&["a", "b"]), key(&["a", "b"]));
        assert_ne!(key(&["ab", "c"]), key(&["a", "bc"]));
        assert_ne!(key(&["a"]), key(&["b"]));
        assert_eq!(key(&["a"]).to_string().len(), 64);
    }

    #[test]
    fn texture_keys_depend_on_pixels_and_shape() {
        let wide = Texture::new(4, 2);
        let tall = Texture::new(2, 4);
        let mut painted = Texture::new(4, 2);
        painted.set(0, 0, [1, 0, 0, 0]);

        let of =
            |texture: &Texture| CacheKey::builder().texture(texture).finish();
        assert_eq!(of(&wide), of(&Texture::new(4, 2)));
        assert_ne!(of(&wide), of(&tall));
        assert_ne!(of(&wide), of(&painted));
    }

    #[test]
    fn stored_images_can_be_read_back() {
        let directory = scratch_directory("round-trip");
        let cache = DirCache::new(&directory, "png");
        let key = key(&["skin"]);

        assert_eq!(cache.get(&key).unwrap(), None);
        assert!(cache.lookup(&key).is_none());
        cache.put(&key, b"image").unwrap();
        assert_eq!(cache.get(&key).unwrap().as_deref(), Some(&b"image"[..]));
        assert_eq!(cache.lookup(&key), Some(cache.path(&key)));
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn pruning_removes_the_least_recently_used_files_first() {
        let directory = scratch_directory("prune");
        let cache = DirCache::new(&directory, "png");
        let keys = [key(&["old"]), key(&["middle"]), key(&["new"])];
        for (age, key) in keys.iter().enumerate() {
            let path = cache.store(key, &[0; 10]).unwrap();
            let modified = SystemTime::UNIX_EPOCH
                + std::time::Duration::from_secs(1_000 + age as u64);
            std::fs::File::options()
                .write(true)
                .open(path)
                .unwrap()
                .set_modified(modified)
                .unwrap();
        }

        cache.prune(25).unwrap();
        assert!(!cache.path(&keys[0]).exists());
        assert!(cache.path(&keys[1]).exists());
        assert!(cache.path(&keys[2]).exists());

        cache.prune(0).unwrap();
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
        DirCache::new(directory.join("missing"), "png")
            .prune(0)
            .unwrap();

        std::fs::remove_dir_all(directory).unwrap();
    }
}

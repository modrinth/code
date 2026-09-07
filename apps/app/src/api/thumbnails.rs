use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::UNIX_EPOCH;

use sha2::{Digest, Sha256};
use tauri::{Manager, Runtime};
use tauri_plugin_fs::FsExt;
use tokio::sync::Semaphore;

use super::Result;

static THUMBNAIL_WORKER: Semaphore = Semaphore::const_new(1);
static GENERATED_THUMBNAILS: AtomicUsize = AtomicUsize::new(0);
const CACHE_LIMIT: u64 = 128 * 1024 * 1024;

#[tauri::command]
pub async fn get_image_thumbnail<R: Runtime>(
    app: tauri::AppHandle<R>,
    path: PathBuf,
    size: u32,
) -> Result<String> {
    let path = tokio::fs::canonicalize(path).await?;
    if !app.asset_protocol_scope().is_allowed(&path)
        && !app.fs_scope().is_allowed(&path)
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Image is outside the permitted scope",
        )
        .into());
    }
    let cache = theseus::State::get()
        .await?
        .directories
        .caches_dir()
        .join("thumbnails-v1");
    let permit = THUMBNAIL_WORKER
        .acquire()
        .await
        .map_err(std::io::Error::other)?;
    let thumbnail = tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        create_thumbnail(&path, &cache, size.clamp(32, 512))
    })
    .await
    .map_err(std::io::Error::other)??;
    Ok(super::utils::tauri_convert_file_src(&thumbnail)?.to_string())
}

fn create_thumbnail(
    path: &Path,
    cache: &Path,
    size: u32,
) -> std::io::Result<PathBuf> {
    let metadata = std::fs::metadata(path)?;
    let modified = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let mut hash = Sha256::new();
    hash.update(path.as_os_str().as_encoded_bytes());
    hash.update(metadata.len().to_le_bytes());
    hash.update(modified.as_nanos().to_le_bytes());
    hash.update(size.to_le_bytes());
    let target = cache.join(format!("{:x}.png", hash.finalize()));
    if target.is_file() {
        return Ok(target);
    }
    std::fs::create_dir_all(cache)?;
    let mut reader = image::ImageReader::open(path)?.with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(256 * 1024 * 1024);
    limits.max_image_width = Some(16384);
    limits.max_image_height = Some(16384);
    reader.limits(limits);
    let image = reader.decode().map_err(std::io::Error::other)?;
    let thumbnail = image.thumbnail(size, size);
    drop(image);
    let temporary = target.with_extension("tmp");
    thumbnail
        .save_with_format(&temporary, image::ImageFormat::Png)
        .map_err(std::io::Error::other)?;
    std::fs::rename(&temporary, &target)?;
    if GENERATED_THUMBNAILS
        .fetch_add(1, Ordering::Relaxed)
        .is_multiple_of(32)
        && let Err(error) = prune_cache(cache, &target)
    {
        tracing::warn!(%error, "Could not prune thumbnail cache");
    }
    Ok(target)
}

fn prune_cache(cache: &Path, current: &Path) -> std::io::Result<()> {
    let mut entries = Vec::new();
    let mut total = 0;
    for entry in std::fs::read_dir(cache)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if !metadata.is_file() {
            continue;
        }
        total += metadata.len();
        entries.push((
            metadata.modified().unwrap_or(UNIX_EPOCH),
            metadata.len(),
            entry.path(),
        ));
    }
    entries.sort_unstable_by_key(|(modified, _, _)| *modified);
    for (_, size, path) in entries {
        if total <= CACHE_LIMIT {
            break;
        }
        if path != current && std::fs::remove_file(path).is_ok() {
            total -= size;
        }
    }
    Ok(())
}

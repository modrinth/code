use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::{EditInstance, State};
use crate::util::fetch::{sha1_async, write};
use crate::util::io;
use bytes::Bytes;
use std::fs::File as StdFile;
use std::io::{BufRead, BufReader, Cursor, Seek};
use std::path::{Path, PathBuf};

const INSTANCE_ICON_MAX_BYTES: usize = 4 * 1024 * 1024;
const INSTANCE_ICON_MAX_DIMENSION: u32 = 512;
const INSTANCE_ICON_MAX_SOURCE_DIMENSION: u32 = 8_192;
const INSTANCE_ICON_MAX_DECODE_BYTES: u64 = 64 * 1024 * 1024;

enum LegacyIconAction {
    Keep,
    Normalize,
    Remove,
}

pub async fn edit_icon(
    instance_id: &str,
    icon_path: Option<&Path>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let icon_path = if let Some(icon_path) = icon_path {
        Some(
            cache_icon_from_path(icon_path, &state)
                .await?
                .to_string_lossy()
                .to_string(),
        )
    } else {
        None
    };

    apply_instance_icon(instance_id, icon_path, &state).await
}

pub(crate) async fn cache_icon(
    bytes: Bytes,
    state: &State,
) -> crate::Result<PathBuf> {
    let bytes = tokio::task::spawn_blocking(move || {
        if looks_like_svg(&bytes) {
            return Err(svg_not_supported_error());
        }

        normalize_raster(Cursor::new(bytes))
    })
    .await??;

    write_cached_icon(bytes, state).await
}

pub(crate) async fn cache_icon_from_path(
    icon_path: &Path,
    state: &State,
) -> crate::Result<PathBuf> {
    let icon_path = icon_path.to_path_buf();
    let bytes = tokio::task::spawn_blocking(move || {
        let file = StdFile::open(&icon_path).map_err(|error| {
            crate::ErrorKind::InputError(format!(
                "Could not open instance icon {}: {error}",
                icon_path.display()
            ))
        })?;
        let mut reader = BufReader::new(file);
        let looks_like_svg = {
            let bytes = reader.fill_buf().map_err(|error| {
                crate::ErrorKind::InputError(format!(
                    "Could not inspect instance icon {}: {error}",
                    icon_path.display()
                ))
            })?;
            looks_like_svg(bytes)
        };
        if has_svg_extension(&icon_path) || looks_like_svg {
            return Err(svg_not_supported_error());
        }

        normalize_raster(reader)
    })
    .await??;

    write_cached_icon(bytes, state).await
}

pub(crate) async fn migrate_legacy_icons() -> crate::Result<()> {
    let state = State::get().await?;
    let instances = instance_rows::list_instances(&state.pool).await?;

    for instance in instances {
        let Some(icon_path) = instance.icon_path.as_deref() else {
            continue;
        };
        let action = match inspect_legacy_icon(Path::new(icon_path)) {
            Ok(action) => action,
            Err(error) => {
                tracing::warn!(
                    instance_id = instance.id,
                    icon_path,
                    error = %error,
                    "Failed to inspect legacy instance icon"
                );
                continue;
            }
        };

        match action {
            LegacyIconAction::Keep => {}
            LegacyIconAction::Normalize => {
                if let Err(error) =
                    edit_icon(&instance.id, Some(Path::new(icon_path))).await
                {
                    tracing::warn!(
                        instance_id = instance.id,
                        icon_path,
                        error = %error,
                        "Failed to normalize legacy instance icon"
                    );
                }
            }
            LegacyIconAction::Remove => {
                if let Err(error) =
                    apply_instance_icon(&instance.id, None, &state).await
                {
                    tracing::warn!(
                        instance_id = instance.id,
                        icon_path,
                        error = %error,
                        "Failed to remove legacy SVG instance icon"
                    );
                }
            }
        }
    }

    Ok(())
}

async fn apply_instance_icon(
    instance_id: &str,
    icon_path: Option<String>,
    state: &State,
) -> crate::Result<()> {
    let instance =
        instance_rows::get_instance_display_info(instance_id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError("Unknown instance".to_string())
            })?;
    crate::state::edit_instance(
        instance_id,
        EditInstance {
            icon_path: Some(icon_path.clone()),
            ..EditInstance::default()
        },
        &state.pool,
    )
    .await?;

    if let Err(error) = super::shared::sync_shared_instance_icon(
        instance_id,
        icon_path.as_deref(),
        state,
    )
    .await
    {
        tracing::warn!(
            instance_id,
            error = %error,
            "Failed to sync shared instance icon"
        );
    }

    emit_instance(&instance.id, InstancePayloadType::Edited).await?;

    Ok(())
}

async fn write_cached_icon(
    bytes: Bytes,
    state: &State,
) -> crate::Result<PathBuf> {
    if bytes.len() >= INSTANCE_ICON_MAX_BYTES {
        return Err(icon_too_large_error());
    }

    let hash = sha1_async(bytes.clone()).await?;
    let path = state
        .directories
        .caches_dir()
        .join("icons")
        .join(format!("{hash}.png"));
    write(&path, &bytes, &state.io_semaphore).await?;

    Ok(io::canonicalize(path)?)
}

fn normalize_raster<R>(reader: R) -> crate::Result<Bytes>
where
    R: BufRead + Seek,
{
    let mut reader = image::ImageReader::new(reader)
        .with_guessed_format()
        .map_err(|error| {
            crate::ErrorKind::InputError(format!(
                "Could not identify instance icon format: {error}"
            ))
        })?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(INSTANCE_ICON_MAX_SOURCE_DIMENSION);
    limits.max_image_height = Some(INSTANCE_ICON_MAX_SOURCE_DIMENSION);
    limits.max_alloc = Some(INSTANCE_ICON_MAX_DECODE_BYTES);
    reader.limits(limits);

    let image = reader.decode().map_err(|error| {
        crate::ErrorKind::InputError(format!(
            "Could not decode instance icon: {error}"
        ))
    })?;
    let image = if image.width() > INSTANCE_ICON_MAX_DIMENSION
        || image.height() > INSTANCE_ICON_MAX_DIMENSION
    {
        image.resize(
            INSTANCE_ICON_MAX_DIMENSION,
            INSTANCE_ICON_MAX_DIMENSION,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        image
    };
    let mut normalized = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image.to_rgba8())
        .write_to(&mut normalized, image::ImageFormat::Png)
        .map_err(|error| {
            crate::ErrorKind::InputError(format!(
                "Could not encode instance icon as PNG: {error}"
            ))
        })?;

    validate_normalized_icon(normalized.into_inner())
}

fn inspect_legacy_icon(icon_path: &Path) -> crate::Result<LegacyIconAction> {
    let metadata = std::fs::metadata(icon_path).map_err(|error| {
        crate::ErrorKind::InputError(format!(
            "Could not inspect instance icon {}: {error}",
            icon_path.display()
        ))
    })?;
    let file = StdFile::open(icon_path).map_err(|error| {
        crate::ErrorKind::InputError(format!(
            "Could not open instance icon {}: {error}",
            icon_path.display()
        ))
    })?;
    let mut reader = BufReader::new(file);
    let bytes = reader.fill_buf().map_err(|error| {
        crate::ErrorKind::InputError(format!(
            "Could not inspect instance icon {}: {error}",
            icon_path.display()
        ))
    })?;

    if has_svg_extension(icon_path) || looks_like_svg(bytes) {
        return Ok(LegacyIconAction::Remove);
    }

    if metadata.len() < INSTANCE_ICON_MAX_BYTES as u64
        && image::guess_format(bytes).ok() == Some(image::ImageFormat::Png)
    {
        return Ok(LegacyIconAction::Keep);
    }

    Ok(LegacyIconAction::Normalize)
}

fn validate_normalized_icon(normalized: Vec<u8>) -> crate::Result<Bytes> {
    if normalized.len() >= INSTANCE_ICON_MAX_BYTES {
        return Err(icon_too_large_error());
    }

    Ok(Bytes::from(normalized))
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    if image::guess_format(bytes).is_ok() {
        return false;
    }

    bytes[..bytes.len().min(1_024)]
        .windows(4)
        .any(|window| window.eq_ignore_ascii_case(b"<svg"))
}

fn has_svg_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
}

fn icon_too_large_error() -> crate::Error {
    crate::ErrorKind::InputError(format!(
        "Instance icons must be smaller than {INSTANCE_ICON_MAX_BYTES} bytes"
    ))
    .into()
}

fn svg_not_supported_error() -> crate::Error {
    crate::ErrorKind::InputError(
        "SVG instance icons are not supported".to_string(),
    )
    .into()
}

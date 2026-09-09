use crate::state::{InstanceMetadata, State};
use serde_json::Value;
use std::path::Path;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct PackFormat {
    major: u64,
    minor: u64,
}

async fn read_zip_json(
    path: &Path,
    name: &str,
) -> crate::Result<Option<Value>> {
    let zip = async_zip::tokio::read::fs::ZipFileReader::new(path).await?;
    let Some(index) = zip.file().entries().iter().position(|entry| {
        entry.filename().as_str().is_ok_and(|value| value == name)
    }) else {
        return Ok(None);
    };
    if zip.file().entries()[index].uncompressed_size() > 1024 * 1024 {
        return Ok(None);
    }
    let mut bytes = Vec::new();
    zip.reader_with_entry(index)
        .await?
        .read_to_end_checked(&mut bytes)
        .await?;
    Ok(Some(serde_json::from_slice(&bytes)?))
}

pub(super) async fn game_format(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<Option<PackFormat>> {
    let content_set = &metadata.applied_content_set;
    let version = &content_set.game_version;
    if let Some(minor) = version
        .strip_prefix("1.")
        .and_then(|version| version.split('.').next()?.parse::<u32>().ok())
    {
        let major = match minor {
            6..=8 => Some(1),
            9..=10 => Some(2),
            11..=12 => Some(3),
            13 => Some(4),
            _ => None,
        };
        if let Some(major) = major {
            return Ok(Some(PackFormat { major, minor: 0 }));
        }
    }
    let version_jar = content_set.loader_version.as_ref().map_or_else(
        || version.clone(),
        |loader| format!("{version}-{loader}"),
    );
    let path = state
        .directories
        .version_dir(&version_jar)
        .join(format!("{version_jar}.jar"));
    if !path.exists() {
        return Ok(None);
    }
    let Some(data) = read_zip_json(&path, "version.json").await? else {
        return Ok(None);
    };
    let Some(pack) = data.get("pack_version") else {
        return Ok(None);
    };
    let major = pack.as_u64().or_else(|| {
        pack.get("resource")
            .or_else(|| pack.get("resource_major"))?
            .as_u64()
    });
    Ok(major.map(|major| PackFormat {
        major,
        minor: pack
            .get("resource_minor")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    }))
}

fn format_bound(value: &Value, default_minor: u64) -> Option<PackFormat> {
    if let Some(major) = value.as_u64() {
        return Some(PackFormat {
            major,
            minor: default_minor,
        });
    }
    let values = value.as_array()?;
    if !(1..=2).contains(&values.len()) {
        return None;
    }
    Some(PackFormat {
        major: values[0].as_u64()?,
        minor: match values.get(1) {
            Some(minor) => minor.as_u64()?,
            None => default_minor,
        },
    })
}

fn legacy_range(value: &Value) -> Option<(u64, u64)> {
    if let Some(format) = value.as_u64() {
        return Some((format, format));
    }
    if let Some(values) = value.as_array() {
        return (values.len() == 2)
            .then(|| Some((values[0].as_u64()?, values[1].as_u64()?)))?;
    }
    Some((
        value.get("min_inclusive")?.as_u64()?,
        value.get("max_inclusive")?.as_u64()?,
    ))
}

fn supports_format(pack: &Value, game: PackFormat) -> Option<bool> {
    if game.major >= 65 {
        let min = format_bound(pack.get("min_format")?, 0)?;
        let max = format_bound(pack.get("max_format")?, i32::MAX as u64)?;
        if min.major < 65 {
            let format = pack.get("pack_format")?.as_u64()?;
            let (legacy_min, legacy_max) =
                legacy_range(pack.get("supported_formats")?)?;
            if format < legacy_min || format > legacy_max {
                return Some(false);
            }
        } else if pack.get("supported_formats").is_some() {
            return Some(false);
        }
        return Some(min <= game && game <= max);
    }
    let format = pack.get("pack_format")?.as_u64()?;
    if game.major >= 16
        && let Some(range) = pack.get("supported_formats")
    {
        let (min, max) = legacy_range(range)?;
        return Some(
            min <= format
                && format <= max
                && min <= game.major
                && game.major <= max,
        );
    }
    Some(format == game.major)
}

pub(super) async fn compatible(path: &Path, game: Option<PackFormat>) -> bool {
    let Some(game) = game else {
        return false;
    };
    match read_zip_json(path, "pack.mcmeta").await {
        Ok(Some(data)) => data
            .get("pack")
            .and_then(|pack| supports_format(pack, game))
            .unwrap_or(false),
        Ok(None) => false,
        Err(error) => {
            tracing::warn!(
                "Could not check resource-pack compatibility for {}: {error}",
                path.display()
            );
            false
        }
    }
}

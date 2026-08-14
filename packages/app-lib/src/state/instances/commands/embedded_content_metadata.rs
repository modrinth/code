use crate::state::{
    CacheValue, CachedEmbeddedContentMetadata, CachedEntry, ContentFile,
    EmbeddedContentMetadata, Instance, ModLoader, ProjectType, State,
};
use bytes::Bytes;
use futures::stream::{self, StreamExt};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::Path;
use toml::Value as TomlValue;
use zip::ZipArchive;

const MAX_METADATA_BYTES: u64 = 1024 * 1024;
const MAX_ICON_BYTES: u64 = 8 * 1024 * 1024;
const PREFERRED_ICON_SIZE: u32 = 96;
const MAX_NAME_CHARS: usize = 256;
const MAX_VERSION_CHARS: usize = 128;

pub(crate) struct ArchiveInspection {
    pub metadata: Option<EmbeddedContentMetadata>,
    pub icon: Option<Bytes>,
}

#[derive(Clone, Copy)]
enum ModMetadataKind {
    Fabric,
    Quilt,
    NeoForge,
    Forge,
    LegacyForge,
}

pub(crate) fn infer_project_type_bytes(
    bytes: &Bytes,
) -> crate::Result<ProjectType> {
    let mut archive = ZipArchive::new(Cursor::new(&**bytes)).map_err(|_| {
        crate::ErrorKind::InputError(
            "Unable to infer project type for input file".to_string(),
        )
    })?;
    infer_project_type(&mut archive)
}

fn inspect_content_file(
    path: &Path,
    loader: ModLoader,
) -> crate::Result<ArchiveInspection> {
    let file = File::open(path).map_err(|error| {
        crate::ErrorKind::OtherError(format!(
            "Could not open content archive {}: {error}",
            path.display()
        ))
    })?;
    inspect_content_archive(file, Some(loader))
}

fn inspect_content_archive<R: Read + Seek>(
    reader: R,
    loader: Option<ModLoader>,
) -> crate::Result<ArchiveInspection> {
    let mut archive = ZipArchive::new(reader).map_err(|_| {
        crate::ErrorKind::InputError(
            "Unable to infer project type for input file".to_string(),
        )
    })?;
    let project_type = infer_project_type(&mut archive)?;
    let (metadata, icon) = match project_type {
        ProjectType::Mod => inspect_mod(&mut archive, loader),
        ProjectType::DataPack | ProjectType::ResourcePack => {
            inspect_pack(&mut archive)
        }
        ProjectType::ShaderPack => (None, None),
    };

    Ok(ArchiveInspection { metadata, icon })
}

fn infer_project_type<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> crate::Result<ProjectType> {
    if has_entry(archive, "fabric.mod.json")
        || has_entry(archive, "quilt.mod.json")
        || has_entry(archive, "META-INF/neoforge.mods.toml")
        || has_entry(archive, "META-INF/mods.toml")
        || has_entry(archive, "mcmod.info")
    {
        Ok(ProjectType::Mod)
    } else if has_entry(archive, "pack.mcmeta") {
        if archive.file_names().any(|name| name.starts_with("data/")) {
            Ok(ProjectType::DataPack)
        } else {
            Ok(ProjectType::ResourcePack)
        }
    } else if archive
        .file_names()
        .any(|name| name.starts_with("shaders/"))
    {
        Ok(ProjectType::ShaderPack)
    } else {
        Err(crate::ErrorKind::InputError(
            "Unable to infer project type for input file".to_string(),
        )
        .into())
    }
}

fn inspect_pack<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> (Option<EmbeddedContentMetadata>, Option<Bytes>) {
    let icon = read_icon_entry(archive, "pack.png");
    let metadata = EmbeddedContentMetadata::default();

    if metadata.is_empty() && icon.is_none() {
        (None, None)
    } else {
        (Some(metadata), icon)
    }
}

fn inspect_mod<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    loader: Option<ModLoader>,
) -> (Option<EmbeddedContentMetadata>, Option<Bytes>) {
    let manifest = read_manifest(archive);
    let order = metadata_order(loader);
    let parsed = order.iter().find_map(|kind| match kind {
        ModMetadataKind::Fabric => parse_fabric_metadata(archive),
        ModMetadataKind::Quilt => parse_quilt_metadata(archive),
        ModMetadataKind::NeoForge => parse_toml_metadata(
            archive,
            "META-INF/neoforge.mods.toml",
            &manifest,
        ),
        ModMetadataKind::Forge => {
            parse_toml_metadata(archive, "META-INF/mods.toml", &manifest)
        }
        ModMetadataKind::LegacyForge => parse_legacy_forge_metadata(archive),
    });
    let (mut metadata, icon_path) =
        parsed.unwrap_or_else(|| (EmbeddedContentMetadata::default(), None));

    if metadata.name.is_none() {
        metadata.name = manifest
            .get("implementation-title")
            .and_then(|value| clean_string(value, MAX_NAME_CHARS));
    }
    if metadata.version.is_none() {
        metadata.version = manifest
            .get("implementation-version")
            .and_then(|value| clean_string(value, MAX_VERSION_CHARS));
    }

    let icon = icon_path
        .as_deref()
        .and_then(|path| read_icon_entry(archive, path));
    if metadata.is_empty() && icon.is_none() {
        (None, None)
    } else {
        (Some(metadata), icon)
    }
}

fn metadata_order(loader: Option<ModLoader>) -> [ModMetadataKind; 5] {
    match loader {
        Some(ModLoader::Fabric) => [
            ModMetadataKind::Fabric,
            ModMetadataKind::Quilt,
            ModMetadataKind::NeoForge,
            ModMetadataKind::Forge,
            ModMetadataKind::LegacyForge,
        ],
        Some(ModLoader::Quilt) => [
            ModMetadataKind::Quilt,
            ModMetadataKind::Fabric,
            ModMetadataKind::NeoForge,
            ModMetadataKind::Forge,
            ModMetadataKind::LegacyForge,
        ],
        Some(ModLoader::Forge) => [
            ModMetadataKind::Forge,
            ModMetadataKind::LegacyForge,
            ModMetadataKind::NeoForge,
            ModMetadataKind::Fabric,
            ModMetadataKind::Quilt,
        ],
        Some(ModLoader::NeoForge) => [
            ModMetadataKind::NeoForge,
            ModMetadataKind::Forge,
            ModMetadataKind::LegacyForge,
            ModMetadataKind::Fabric,
            ModMetadataKind::Quilt,
        ],
        Some(ModLoader::Vanilla) | None => [
            ModMetadataKind::Fabric,
            ModMetadataKind::Quilt,
            ModMetadataKind::NeoForge,
            ModMetadataKind::Forge,
            ModMetadataKind::LegacyForge,
        ],
    }
}

fn parse_fabric_metadata<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Option<(EmbeddedContentMetadata, Option<String>)> {
    let root = read_json_entry(archive, "fabric.mod.json")?;
    let metadata = EmbeddedContentMetadata {
        name: root
            .get("name")
            .and_then(JsonValue::as_str)
            .or_else(|| root.get("id").and_then(JsonValue::as_str))
            .and_then(|value| clean_string(value, MAX_NAME_CHARS)),
        version: root
            .get("version")
            .and_then(json_scalar_string)
            .and_then(|value| clean_string(&value, MAX_VERSION_CHARS)),
        ..Default::default()
    };
    let icon = root.get("icon").and_then(json_icon_path);
    Some((metadata, icon))
}

fn parse_quilt_metadata<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Option<(EmbeddedContentMetadata, Option<String>)> {
    let text = read_text_entry(archive, "quilt.mod.json")?;
    let root = json5::from_str::<JsonValue>(&text).ok()?;
    let loader = root.get("quilt_loader").unwrap_or(&root);
    let display = loader.get("metadata").unwrap_or(loader);
    let metadata = EmbeddedContentMetadata {
        name: display
            .get("name")
            .and_then(JsonValue::as_str)
            .or_else(|| loader.get("id").and_then(JsonValue::as_str))
            .and_then(|value| clean_string(value, MAX_NAME_CHARS)),
        version: loader
            .get("version")
            .and_then(json_scalar_string)
            .and_then(|value| clean_string(&value, MAX_VERSION_CHARS)),
        ..Default::default()
    };
    let icon = display
        .get("icon")
        .or_else(|| loader.get("icon"))
        .and_then(json_icon_path);
    Some((metadata, icon))
}

fn parse_toml_metadata<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
    manifest: &HashMap<String, String>,
) -> Option<(EmbeddedContentMetadata, Option<String>)> {
    let text = read_text_entry(archive, path)?;
    let root = toml::from_str::<TomlValue>(&text).ok()?;
    let mod_table = root.get("mods")?.as_array()?.first()?.as_table()?;
    let name = mod_table
        .get("displayName")
        .or_else(|| mod_table.get("modId"))
        .and_then(toml_scalar_string)
        .and_then(|value| clean_string(&value, MAX_NAME_CHARS));
    let version = mod_table
        .get("version")
        .and_then(toml_scalar_string)
        .and_then(|value| resolve_toml_value(&value, &root, manifest))
        .and_then(|value| clean_string(&value, MAX_VERSION_CHARS));
    let icon = mod_table
        .get("logoFile")
        .and_then(TomlValue::as_str)
        .and_then(safe_archive_path);

    Some((
        EmbeddedContentMetadata {
            name,
            version,
            ..Default::default()
        },
        icon,
    ))
}

fn parse_legacy_forge_metadata<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Option<(EmbeddedContentMetadata, Option<String>)> {
    let root = read_json_entry(archive, "mcmod.info")?;
    let entry = root
        .as_array()
        .and_then(|entries| entries.first())
        .or_else(|| {
            root.get("modList")
                .and_then(JsonValue::as_array)
                .and_then(|entries| entries.first())
        })?;
    let metadata = EmbeddedContentMetadata {
        name: entry
            .get("name")
            .and_then(JsonValue::as_str)
            .or_else(|| entry.get("modid").and_then(JsonValue::as_str))
            .and_then(|value| clean_string(value, MAX_NAME_CHARS)),
        version: entry
            .get("version")
            .and_then(json_scalar_string)
            .and_then(|value| clean_string(&value, MAX_VERSION_CHARS)),
        ..Default::default()
    };
    let icon = entry
        .get("logoFile")
        .and_then(JsonValue::as_str)
        .and_then(safe_archive_path);
    Some((metadata, icon))
}

fn resolve_toml_value(
    value: &str,
    root: &TomlValue,
    manifest: &HashMap<String, String>,
) -> Option<String> {
    if value == "${file.jarVersion}" {
        return manifest.get("implementation-version").cloned();
    }
    if let Some(key) = value
        .strip_prefix("${file.")
        .and_then(|value| value.strip_suffix('}'))
    {
        return root
            .get("properties")
            .and_then(TomlValue::as_table)
            .and_then(|properties| properties.get(key))
            .and_then(toml_scalar_string);
    }
    Some(value.to_string())
}

fn json_icon_path(value: &JsonValue) -> Option<String> {
    if let Some(path) = value.as_str() {
        return safe_archive_path(path);
    }
    let icons = value.as_object()?;
    let mut candidates = icons
        .iter()
        .filter_map(|(size, path)| {
            Some((
                size.parse::<u32>().ok()?,
                safe_archive_path(path.as_str()?)?,
            ))
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(size, _)| *size);
    candidates
        .iter()
        .find(|(size, _)| *size >= PREFERRED_ICON_SIZE)
        .or_else(|| candidates.last())
        .map(|(_, path)| path.clone())
}

fn safe_archive_path(path: &str) -> Option<String> {
    let path = path.trim().trim_start_matches('/');
    if path.is_empty()
        || path.contains('\0')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    Some(path.to_string())
}

fn clean_string(value: &str, max_chars: usize) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.contains("${") {
        return None;
    }
    let value = value
        .chars()
        .filter(|character| !character.is_control())
        .take(max_chars)
        .collect::<String>();
    (!value.is_empty()).then_some(value)
}

fn json_scalar_string(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(value) => Some(value.clone()),
        JsonValue::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn toml_scalar_string(value: &TomlValue) -> Option<String> {
    match value {
        TomlValue::String(value) => Some(value.clone()),
        TomlValue::Integer(value) => Some(value.to_string()),
        TomlValue::Float(value) => Some(value.to_string()),
        _ => None,
    }
}

fn read_manifest<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> HashMap<String, String> {
    let Some(text) = read_text_entry(archive, "META-INF/MANIFEST.MF") else {
        return HashMap::new();
    };
    let mut attributes: HashMap<String, String> = HashMap::new();
    let mut current_key: Option<String> = None;
    for line in text.lines() {
        if let Some(continuation) = line.strip_prefix(' ') {
            if let Some(key) = current_key.as_ref()
                && let Some(value) = attributes.get_mut(key)
            {
                value.push_str(continuation);
            }
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            current_key = None;
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        attributes.insert(key.clone(), value.trim_start().to_string());
        current_key = Some(key);
    }
    attributes
}

fn has_entry<R: Read + Seek>(archive: &mut ZipArchive<R>, path: &str) -> bool {
    archive.by_name(path).is_ok()
}

fn read_json_entry<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Option<JsonValue> {
    let text = read_text_entry(archive, path)?;
    serde_json::from_str(&text).ok()
}

fn read_text_entry<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Option<String> {
    let bytes = read_entry(archive, path, MAX_METADATA_BYTES)?;
    String::from_utf8(bytes).ok()
}

fn read_icon_entry<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Option<Bytes> {
    let path = safe_archive_path(path)?;
    read_entry(archive, &path, MAX_ICON_BYTES).map(Bytes::from)
}

fn read_entry<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
    max_bytes: u64,
) -> Option<Vec<u8>> {
    let entry = archive.by_name(path).ok()?;
    if entry.size() > max_bytes {
        return None;
    }
    let mut bytes = Vec::with_capacity(entry.size() as usize);
    entry
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() as u64 <= max_bytes).then_some(bytes)
}

pub(crate) async fn resolve_embedded_content_metadata(
    instance: &Instance,
    loader: ModLoader,
    files: &[(String, ContentFile)],
    state: &State,
) -> crate::Result<HashMap<String, EmbeddedContentMetadata>> {
    let candidates = files
        .iter()
        .filter(|(_, file)| {
            file.metadata.is_none()
                && matches!(
                    file.project_type,
                    ProjectType::Mod
                        | ProjectType::DataPack
                        | ProjectType::ResourcePack
                )
        })
        .map(|(relative_path, file)| {
            (
                file.hash.clone(),
                state
                    .directories
                    .instances_dir()
                    .join(&instance.path)
                    .join(relative_path),
            )
        })
        .collect::<HashMap<_, _>>();
    if candidates.is_empty() {
        return Ok(HashMap::new());
    }

    let cache_keys = candidates
        .keys()
        .map(|hash| metadata_cache_key(hash, loader))
        .collect::<Vec<_>>();
    let cache_key_refs =
        cache_keys.iter().map(String::as_str).collect::<Vec<_>>();
    let cached = CachedEntry::get_embedded_content_metadata_many(
        &cache_key_refs,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let mut resolved = HashMap::new();
    let mut resolved_hashes = HashSet::new();
    for cached in cached {
        let icon_exists = cached
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.icon_path.as_deref())
            .is_none_or(|path| Path::new(path).is_file());
        if icon_exists {
            resolved_hashes.insert(cached.hash.clone());
            if let Some(metadata) = cached.metadata {
                resolved.insert(cached.hash, metadata);
            }
        }
    }

    let pending = candidates
        .into_iter()
        .filter(|(hash, _)| !resolved_hashes.contains(hash))
        .collect::<Vec<_>>();
    let inspected_metadata = stream::iter(pending)
        .map(|(hash, path)| async move {
            let inspection = tokio::task::spawn_blocking(move || {
                inspect_content_file(&path, loader)
            })
            .await;
            let (mut metadata, icon) = match inspection {
                Ok(Ok(inspection)) => {
                    (inspection.metadata.unwrap_or_default(), inspection.icon)
                }
                Ok(Err(error)) => {
                    tracing::debug!(
                        hash,
                        error = %error,
                        "Could not inspect content metadata"
                    );
                    return None;
                }
                Err(error) => {
                    tracing::debug!(
                        hash,
                        error = %error,
                        "Content metadata inspection task failed"
                    );
                    return None;
                }
            };
            if let Some(icon) = icon {
                match crate::api::instance::cache_icon(icon, state).await {
                    Ok(path) => {
                        metadata.icon_path =
                            Some(path.to_string_lossy().to_string());
                    }
                    Err(error) => {
                        tracing::debug!(
                            hash,
                            error = %error,
                            "Could not cache embedded content icon"
                        );
                    }
                }
            }
            let metadata = (!metadata.is_empty()).then_some(metadata);
            Some((hash, metadata))
        })
        .buffer_unordered(8)
        .collect::<Vec<_>>()
        .await;
    let mut entries = Vec::with_capacity(inspected_metadata.len());
    for (hash, metadata) in inspected_metadata.into_iter().flatten() {
        if let Some(metadata) = metadata.as_ref() {
            resolved.insert(hash.clone(), metadata.clone());
        }
        entries.push(
            CacheValue::EmbeddedContentMetadata(
                CachedEmbeddedContentMetadata {
                    cache_key: metadata_cache_key(&hash, loader),
                    hash,
                    metadata,
                },
            )
            .get_entry(),
        );
    }
    CachedEntry::upsert_many(&entries, &state.pool).await?;

    Ok(resolved)
}

fn metadata_cache_key(hash: &str, loader: ModLoader) -> String {
    format!("{hash}-{}", loader.as_str())
}

use indexmap::IndexMap;
use minecraft_meta_manifest::minecraft::VersionManifest;
use minecraft_meta_manifest::profile::{
    CURRENT_FABRIC_FORMAT_VERSION, CURRENT_FORGE_FORMAT_VERSION,
    CURRENT_NEOFORGE_FORMAT_VERSION, CURRENT_QUILT_FORMAT_VERSION,
    DUMMY_REPLACE_STRING, Processor, ProfileMetadata, SidedDataEntry,
};
use serde::{Deserialize, Serialize};

/// Resolved launch metadata, serialized in the installed version-cache format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedVersion {
    #[serde(flatten)]
    pub minecraft: VersionManifest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<IndexMap<String, SidedDataEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processors: Option<Vec<Processor>>,
}

impl From<VersionManifest> for ResolvedVersion {
    fn from(minecraft: VersionManifest) -> Self {
        Self {
            minecraft,
            data: None,
            processors: None,
        }
    }
}

impl std::ops::Deref for ResolvedVersion {
    type Target = VersionManifest;
    fn deref(&self) -> &Self::Target {
        &self.minecraft
    }
}

impl std::ops::DerefMut for ResolvedVersion {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.minecraft
    }
}

pub fn merge_partial_version(
    partial: ProfileMetadata,
    mut resolved: ResolvedVersion,
) -> ResolvedVersion {
    let version = &mut resolved.minecraft;
    let game_version = version.id.0.clone();
    let replacement = DUMMY_REPLACE_STRING;
    version.libraries.retain(|library| {
        let artifact = library.name.rsplit_once(':').map(|item| item.0);
        !partial.libraries.iter().any(|target| {
            target.include_in_classpath
                && artifact.is_some()
                && target.name.rsplit_once(':').map(|item| item.0) == artifact
        })
    });
    version.libraries.extend(partial.libraries);
    for library in &mut version.libraries {
        library.name = library.name.replace(replacement, &game_version);
    }
    if let Some(arguments) = partial.arguments {
        let merged = version.arguments.get_or_insert_default();
        for (kind, arguments) in arguments {
            merged.entry(kind).or_default().extend(arguments);
        }
    }
    version.id.0 = partial.id.replace(replacement, &game_version);
    if let Some(main_class) = partial.main_class {
        version.main_class = main_class;
    }
    version.minecraft_arguments = partial.minecraft_arguments;
    version.release_time = partial.release_time;
    version.time = partial.time;
    version.ty = partial.ty;
    resolved.data = partial.data;
    resolved.processors = partial.processors;
    resolved
}

/// Metadata for locating and caching a loader manifest.
#[derive(Debug, Clone)]
pub struct LoaderManifestMetadata {
    /// The canonical loader name used in launcher-meta paths.
    pub loader: String,
    /// The latest manifest format version for this loader.
    pub format_version: usize,
    /// The cache key that includes the loader format version.
    pub cache_key: String,
    /// The launcher-meta path to the manifest.
    pub path: String,
}

/// Returns metadata for the latest manifest format for the provided loader.
pub fn loader_manifest_metadata(loader: &str) -> LoaderManifestMetadata {
    let format_version = current_loader_manifest_format_version(loader);
    let cache_key = format!("{loader}-v{format_version}");
    let path = format!("{loader}/v{format_version}/manifest.json");

    LoaderManifestMetadata {
        loader: loader.to_string(),
        format_version,
        cache_key,
        path,
    }
}

/// Returns loader manifest metadata from a versioned cache key.
pub fn loader_manifest_metadata_from_cache_key(
    cache_key: &str,
) -> LoaderManifestMetadata {
    if let Some((loader, format_version)) =
        cache_key.rsplit_once("-v").and_then(|(loader, version)| {
            version
                .parse::<usize>()
                .ok()
                .map(|version| (loader, version))
        })
    {
        let cache_key = format!("{loader}-v{format_version}");
        let path = format!("{loader}/v{format_version}/manifest.json");

        LoaderManifestMetadata {
            loader: loader.to_string(),
            format_version,
            cache_key,
            path,
        }
    } else {
        loader_manifest_metadata(cache_key)
    }
}

fn current_loader_manifest_format_version(loader: &str) -> usize {
    match loader {
        "fabric" => CURRENT_FABRIC_FORMAT_VERSION,
        "forge" => CURRENT_FORGE_FORMAT_VERSION,
        "quilt" => CURRENT_QUILT_FORMAT_VERSION,
        "neo" => CURRENT_NEOFORGE_FORMAT_VERSION,
        _ => 0,
    }
}

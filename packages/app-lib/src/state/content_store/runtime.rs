use super::{input, normalize};
use crate::State;
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::{
    DirectoryInfo, InstanceLaunchContext, JavaVersion, ModLoader,
};
use daedalus::minecraft::{AssetsIndex, LoggingConfiguration, VersionInfo};
use serde::de::DeserializeOwned;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub(super) struct RuntimeFile {
    pub path: PathBuf,
    pub size: u64,
    pub last_used_at: i64,
    pub references: usize,
    pub directory: bool,
}

pub(super) struct RuntimeStorage {
    pub root: PathBuf,
    pub files: Vec<RuntimeFile>,
}

impl RuntimeStorage {
    pub async fn read(state: &State) -> crate::Result<Self> {
        let mut contexts = Vec::new();
        let mut incomplete = false;
        for instance in instance_rows::list_instances(&state.pool).await? {
            match instance_rows::get_instance_launch_context(
                &instance.id,
                &state.pool,
            )
            .await
            {
                Ok(Some(context)) => contexts.push(context),
                Ok(None) => incomplete = true,
                Err(error) => {
                    tracing::debug!(instance_id = %instance.id, "Retaining runtime files without an instance launch context: {error}");
                    incomplete = true;
                }
            }
        }
        let java = JavaVersion::get_all(&state.pool)
            .await?
            .into_iter()
            .collect::<HashMap<_, _>>();
        let mut dirs = state.directories.clone();
        dirs.config_dir = tokio::fs::canonicalize(&dirs.config_dir).await?;
        tokio::task::spawn_blocking(move || {
            Self::scan(dirs, contexts, java, incomplete)
        })
        .await?
    }

    fn scan(
        dirs: DirectoryInfo,
        contexts: Vec<InstanceLaunchContext>,
        java: HashMap<u32, JavaVersion>,
        incomplete: bool,
    ) -> crate::Result<Self> {
        let root = dirs.metadata_dir();
        match fs::symlink_metadata(&root) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => {
                return Err(input(
                    "Runtime storage must be a directory, not a symlink",
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self {
                    root,
                    files: Vec::new(),
                });
            }
            Err(error) => return Err(error.into()),
        }

        let mut references = References::default();
        if incomplete {
            references.tree(&root, "unresolved-instance");
        }
        for context in contexts {
            if let Err(error) = references.instance(&dirs, &context, &java) {
                tracing::debug!(instance_id = %context.instance.id, "Retaining runtime files with unresolved dependencies: {error}");
                references.tree(&root, &context.instance.id);
            }
        }
        let mut files = Vec::new();
        let mut pending = vec![
            dirs.versions_dir(),
            dirs.assets_dir(),
            dirs.libraries_dir(),
            dirs.java_versions_dir(),
            dirs.natives_dir(),
            dirs.log_configs_dir(),
            dirs.legacy_assets_dir(),
        ];
        while let Some(path) = pending.pop() {
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    continue;
                }
                Err(error) => return Err(error.into()),
            };
            if metadata.is_symlink() && path.parent() == Some(root.as_path()) {
                continue;
            }
            if metadata.is_dir() {
                for entry in fs::read_dir(&path)? {
                    pending.push(entry?.path());
                }
            } else if metadata.is_file() || metadata.is_symlink() {
                let last_used_at = metadata
                    .accessed()
                    .ok()
                    .into_iter()
                    .chain(metadata.modified().ok())
                    .filter_map(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map(|time| time.as_secs() as i64)
                    .max()
                    .unwrap_or(i64::MAX);
                files.push(RuntimeFile {
                    references: references.count(&path),
                    path,
                    size: if metadata.is_file() {
                        metadata.len()
                    } else {
                        0
                    },
                    last_used_at,
                    directory: false,
                });
            }
        }
        let mut distributions: HashMap<PathBuf, RuntimeFile> = HashMap::new();
        let mut individual_files = Vec::new();
        for file in files {
            let distribution = file
                .path
                .strip_prefix(dirs.java_versions_dir())
                .ok()
                .filter(|relative| relative.components().count() > 1)
                .and_then(|relative| relative.components().next())
                .map(|name| dirs.java_versions_dir().join(name));
            if let Some(path) = distribution {
                let entry =
                    distributions.entry(path.clone()).or_insert(RuntimeFile {
                        path,
                        size: 0,
                        last_used_at: 0,
                        references: 0,
                        directory: true,
                    });
                entry.size += file.size;
                entry.last_used_at = entry.last_used_at.max(file.last_used_at);
                entry.references = entry.references.max(file.references);
            } else {
                individual_files.push(file);
            }
        }
        individual_files.extend(distributions.into_values());
        Ok(Self {
            root,
            files: individual_files,
        })
    }

    pub fn total_bytes(&self) -> u64 {
        self.files.iter().map(|file| file.size).sum()
    }

    pub fn shared_bytes(&self) -> u64 {
        self.files
            .iter()
            .filter(|file| file.references > 1)
            .map(|file| file.size)
            .sum()
    }

    pub fn unused_bytes(&self) -> u64 {
        self.files
            .iter()
            .filter(|file| file.references == 0)
            .map(|file| file.size)
            .sum()
    }

    pub fn saved_bytes(&self) -> u64 {
        self.files
            .iter()
            .map(|file| {
                file.size
                    .saturating_mul(file.references.saturating_sub(1) as u64)
            })
            .sum()
    }
}

#[derive(Default)]
struct References {
    files: HashMap<PathBuf, HashSet<String>>,
    trees: HashMap<PathBuf, HashSet<String>>,
}

impl References {
    fn file(&mut self, path: PathBuf, owner: &str) {
        self.files
            .entry(normalize(&path))
            .or_default()
            .insert(owner.to_owned());
    }

    fn tree(&mut self, path: &Path, owner: &str) {
        self.trees
            .entry(normalize(path))
            .or_default()
            .insert(owner.to_owned());
    }

    fn count(&self, path: &Path) -> usize {
        let mut owners = self.files.get(path).cloned().unwrap_or_default();
        for (tree, references) in &self.trees {
            if path.starts_with(tree) {
                owners.extend(references.iter().cloned());
            }
        }
        owners.len()
    }

    fn instance(
        &mut self,
        dirs: &DirectoryInfo,
        context: &InstanceLaunchContext,
        java: &HashMap<u32, JavaVersion>,
    ) -> crate::Result<()> {
        let owner = &context.instance.id;
        let content = &context.applied_content_set;
        let version_id = if content.loader == ModLoader::Vanilla {
            content.game_version.clone()
        } else {
            let loader = content
                .loader_version
                .as_deref()
                .filter(|version| !matches!(*version, "latest" | "stable"))
                .ok_or_else(|| {
                    input("The installed loader version is unresolved")
                })?;
            format!("{}-{loader}", content.game_version)
        };
        let version_dir = dirs.version_dir(&version_id);
        let version: VersionInfo =
            read_json(&version_dir.join(format!("{version_id}.json")))?;
        self.tree(&version_dir, owner);
        self.tree(&dirs.version_natives_dir(&version_id), owner);
        for library in &version.libraries {
            self.file(
                dirs.libraries_dir()
                    .join(daedalus::get_path_from_artifact(&library.name)?),
                owner,
            );
        }
        if version
            .processors
            .as_ref()
            .is_some_and(|processors| !processors.is_empty())
        {
            // Processor outputs can use arbitrary paths in the shared library directory.
            self.tree(&dirs.libraries_dir(), owner);
        }
        if let Some(logging) = &version.logging {
            for LoggingConfiguration::Log4j2Xml { file, .. } in logging.values()
            {
                self.file(dirs.log_configs_dir().join(&file.id), owner);
            }
        }
        let index_path = dirs
            .assets_index_dir()
            .join(format!("{}.json", version.asset_index.id));
        let index: AssetsIndex = read_json(&index_path)?;
        self.file(index_path, owner);
        for (name, asset) in index.objects {
            if asset.hash.len() != 40
                || !asset.hash.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(input("Invalid Minecraft asset hash"));
            }
            self.file(dirs.object_dir(&asset.hash), owner);
            self.file(dirs.legacy_assets_dir().join(name), owner);
        }
        let major = version
            .java_version
            .as_ref()
            .map_or(8, |java| java.major_version);
        let selected = context
            .launch_overrides
            .java_path
            .as_deref()
            .filter(|path| Path::new(path).is_file())
            .into_iter()
            .chain(java.get(&major).map(|java| java.path.as_str()));
        for selected in selected {
            let selected = fs::canonicalize(selected)?;
            if let Ok(relative) =
                selected.strip_prefix(dirs.java_versions_dir())
                && let Some(distribution) = relative.components().next()
            {
                self.tree(&dirs.java_versions_dir().join(distribution), owner);
            }
        }
        Ok(())
    }
}

fn read_json<T: DeserializeOwned>(path: &Path) -> crate::Result<T> {
    Ok(serde_json::from_reader(std::io::BufReader::new(
        fs::File::open(path)?,
    ))?)
}

pub(super) async fn remove_file(
    file: &RuntimeFile,
    root: &Path,
) -> crate::Result<u64> {
    let path = &file.path;
    let mut parent = path.parent();
    while let Some(directory) = parent {
        if !directory.starts_with(root)
            || tokio::fs::symlink_metadata(directory).await?.is_symlink()
        {
            return Err(input(
                "Runtime cache path is outside the managed storage directory",
            ));
        }
        if directory == root {
            break;
        }
        parent = directory.parent();
    }
    let metadata = match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(0);
        }
        Err(error) => return Err(error.into()),
    };
    if metadata.is_dir() != file.directory {
        return Err(input("Runtime cache entry changed during cleanup"));
    }
    if file.directory {
        tokio::fs::remove_dir_all(path).await?;
        return Ok(file.size);
    }
    #[cfg(windows)]
    if metadata.is_file() && metadata.permissions().readonly() {
        let mut permissions = metadata.permissions();
        permissions.set_readonly(false);
        tokio::fs::set_permissions(path, permissions).await?;
    }
    tokio::fs::remove_file(path).await?;
    let mut parent = path.parent();
    while let Some(directory) = parent {
        if directory == root || tokio::fs::remove_dir(directory).await.is_err()
        {
            break;
        }
        parent = directory.parent();
    }
    Ok(if metadata.is_file() {
        metadata.len()
    } else {
        0
    })
}

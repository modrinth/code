use super::content::get_projects;
use super::get::get;
use super::paths::get_full_path;
use crate::InvocationContext;
use crate::event::LoadingBarType;
use crate::event::emit::{emit_loading, init_loading};
use crate::pack::install_from::{
    EnvType, PackDependency, PackFile, PackFileHash, PackFormat,
};
use crate::state::{
    CacheBehaviour, CachedEntry, InstanceMetadata, ModLoader, SideType, State,
};
use crate::util::io::{self, IOError};
use async_zip::tokio::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use path_util::SafeRelativeUtf8UnixPathBuf;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[tracing::instrument(skip_all)]
pub async fn export_mrpack(
    context: &InvocationContext,
    instance_id: &str,
    export_path: PathBuf,
    included_export_candidates: Vec<String>,
    version_id: Option<String>,
    description: Option<String>,
    _name: Option<String>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _permit: tokio::sync::SemaphorePermit =
        state.io_semaphore.0.acquire().await?;
    let metadata = get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to export a nonexistent instance {instance_id}!"
        ))
    })?;
    let included_export_candidates = included_export_candidates
        .into_iter()
        .filter(|x| {
            if let Some(f) = PathBuf::from(x).file_name()
                && f.to_string_lossy().starts_with(".DS_Store")
            {
                return false;
            }
            true
        })
        .collect::<Vec<_>>();

    let instance_base_path = get_full_path(instance_id).await?;
    let mut file = File::create(&export_path)
        .await
        .map_err(|e| IOError::with_path(e, &export_path))?;
    let mut writer = ZipFileWriter::with_tokio(&mut file);
    let version_id = version_id.unwrap_or("1.0.0".to_string());
    let mut packfile =
        create_mrpack_json(context, &metadata, version_id, description).await?;
    packfile.files.retain(|f| {
        is_export_candidate_included(
            f.path.as_str(),
            &included_export_candidates,
        )
    });

    let mut path_list = Vec::new();
    add_all_recursive_folder_paths(&instance_base_path, &mut path_list).await?;
    let loading_bar = init_loading(
        LoadingBarType::ZipExtract {
            instance_id: metadata.instance.id.clone(),
            instance_name: metadata.instance.name.clone(),
        },
        path_list.len() as f64,
        "Exporting instance to .mrpack",
    )
    .await?;

    for path in path_list {
        emit_loading(&loading_bar, 1.0, None)?;
        let relative_path = pack_get_relative_path(&instance_base_path, &path)?;

        if packfile.files.iter().any(|f| f.path == relative_path)
            || !is_export_candidate_included(
                relative_path.as_str(),
                &included_export_candidates,
            )
        {
            continue;
        }

        if path.is_file() {
            let mut file = File::open(&path)
                .await
                .map_err(|e| IOError::with_path(e, &path))?;
            let mut data = Vec::new();
            file.read_to_end(&mut data).await.map_err(IOError::from)?;
            let builder = ZipEntryBuilder::new(
                format!("overrides/{relative_path}").into(),
                Compression::Deflate,
            );
            writer.write_entry_whole(builder, &data).await?;
        }
    }

    let data = serde_json::to_vec_pretty(&packfile)?;
    let builder = ZipEntryBuilder::new(
        "modrinth.index.json".to_string().into(),
        Compression::Deflate,
    );
    writer.write_entry_whole(builder, &data).await?;
    writer.close().await?;

    Ok(())
}

fn is_export_candidate_included(
    path: &str,
    included_export_candidates: &[String],
) -> bool {
    included_export_candidates.iter().any(|candidate| {
        path == candidate
            || path
                .strip_prefix(candidate)
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
}

#[tracing::instrument]
pub async fn get_pack_export_candidates(
    instance_id: &str,
) -> crate::Result<Vec<SafeRelativeUtf8UnixPathBuf>> {
    let mut path_list = Vec::new();
    let instance_base_dir = get_full_path(instance_id).await?;
    let mut read_dir = io::read_dir(&instance_base_dir).await?;
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, &instance_base_dir))?
    {
        let path = entry.path();
        if path.is_dir() {
            let mut read_dir = io::read_dir(&path).await?;
            while let Some(entry) = read_dir
                .next_entry()
                .await
                .map_err(|e| IOError::with_path(e, &instance_base_dir))?
            {
                path_list.push(pack_get_relative_path(
                    &instance_base_dir,
                    &entry.path(),
                )?);
            }
        } else {
            path_list.push(pack_get_relative_path(&instance_base_dir, &path)?);
        }
    }
    Ok(path_list)
}

fn pack_get_relative_path(
    instance_path: &PathBuf,
    path: &PathBuf,
) -> crate::Result<SafeRelativeUtf8UnixPathBuf> {
    Ok(SafeRelativeUtf8UnixPathBuf::try_from(
        path.strip_prefix(instance_path)
            .map_err(|_| {
                crate::ErrorKind::FSError(format!(
                    "Path {path:?} does not correspond to an instance"
                ))
            })?
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/"),
    )?)
}

#[tracing::instrument(skip_all)]
pub async fn create_mrpack_json(
    context: &InvocationContext,
    metadata: &InstanceMetadata,
    version_id: String,
    description: Option<String>,
) -> crate::Result<PackFormat> {
    let mut dependencies = HashMap::new();
    match (
        metadata.applied_content_set.loader,
        metadata.applied_content_set.loader_version.clone(),
    ) {
        (ModLoader::Forge, Some(v)) => {
            dependencies.insert(PackDependency::Forge, v)
        }
        (ModLoader::NeoForge, Some(v)) => {
            dependencies.insert(PackDependency::NeoForge, v)
        }
        (ModLoader::Fabric, Some(v)) => {
            dependencies.insert(PackDependency::FabricLoader, v)
        }
        (ModLoader::Quilt, Some(v)) => {
            dependencies.insert(PackDependency::QuiltLoader, v)
        }
        (ModLoader::Vanilla, _) => None,
        _ => {
            return Err(crate::ErrorKind::OtherError(
                "Loader version mismatch".to_string(),
            )
            .into());
        }
    };
    dependencies.insert(
        PackDependency::Minecraft,
        metadata.applied_content_set.game_version.clone(),
    );

    let state = State::get().await?;
    let projects = get_projects(
        context,
        &metadata.instance.id,
        Some(CacheBehaviour::MustRevalidate),
    )
    .await?
    .into_iter()
    .filter_map(|(path, file)| match file.metadata {
        Some(metadata) => Some((path, metadata.version_id)),
        _ => None,
    })
    .collect::<Vec<_>>();
    let versions = CachedEntry::get_version_many(
        context,
        &projects.iter().map(|x| &*x.1).collect::<Vec<_>>(),
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let files = projects
        .into_iter()
        .filter_map(|(path, version_id)| {
            if let Some(version) = versions.iter().find(|x| x.id == version_id)
            {
                let mut env = HashMap::new();
                env.insert(EnvType::Client, SideType::Required);
                env.insert(EnvType::Server, SideType::Required);
                let Some(primary_file) = version.files.first() else {
                    return Some(Err(crate::ErrorKind::OtherError(format!(
                        "No primary file found for mod at: {path}"
                    ))
                    .as_error()));
                };
                let file_size = primary_file.size;
                let downloads = vec![primary_file.url.clone()];
                let hashes = primary_file
                    .hashes
                    .clone()
                    .into_iter()
                    .map(|(h1, h2)| (PackFileHash::from(h1), h2))
                    .collect();

                Some(Ok(PackFile {
                    path: match path.try_into() {
                        Ok(path) => path,
                        Err(_) => {
                            return Some(Err(crate::ErrorKind::OtherError(
                                "Invalid file path in project".into(),
                            )
                            .as_error()));
                        }
                    },
                    hashes,
                    env: Some(env),
                    downloads,
                    file_size,
                }))
            } else {
                None
            }
        })
        .collect::<crate::Result<Vec<PackFile>>>()?;

    Ok(PackFormat {
        game: "minecraft".to_string(),
        format_version: 1,
        version_id,
        name: metadata.instance.name.clone(),
        summary: description,
        files,
        dependencies,
    })
}

#[async_recursion::async_recursion]
async fn add_all_recursive_folder_paths(
    folder: &PathBuf,
    output: &mut Vec<PathBuf>,
) -> crate::Result<()> {
    let mut read_dir = io::read_dir(folder).await?;
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, folder))?
    {
        let path = entry.path();
        if path.is_dir() {
            add_all_recursive_folder_paths(&path, output).await?;
        } else {
            output.push(path);
        }
    }

    Ok(())
}

//! Theseus instance management interface

use crate::event::emit::{emit_instance, emit_loading, init_loading};
use crate::event::{InstancePayloadType, LoadingBarType};
use crate::pack::install_from::{
    EnvType, PackDependency, PackFile, PackFileHash, PackFormat,
};
use crate::server_address::ServerAddress;
use crate::state::{
    CacheBehaviour, CachedEntry, ContentFile, ContentItem, ContentSet,
    CreateInstance, Credentials, Dependency, EditInstance,
    InstanceInstallStage, InstanceLink, InstanceMetadata, JavaVersion,
    LinkedModpackInfo, ModLoader, ProcessMetadata, ProjectType, Settings,
    SideType, State,
};
use crate::util::fetch;
use crate::util::io::{self, IOError};
use async_zip::tokio::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use dashmap::DashMap;
use path_util::SafeRelativeUtf8UnixPathBuf;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub enum QuickPlayType {
    None,
    Singleplayer(String),
    Server(ServerAddress),
}

#[tracing::instrument]
pub async fn remove(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = get(instance_id).await?;
    crate::state::remove_instance(instance_id, &state).await?;

    if let Some(metadata) = metadata {
        emit_instance(&metadata.instance.id, InstancePayloadType::Removed)
            .await?;
    }

    Ok(())
}

#[tracing::instrument]
pub async fn get(instance_id: &str) -> crate::Result<Option<InstanceMetadata>> {
    let state = State::get().await?;
    crate::state::get_instance(instance_id, &state.pool).await
}

#[tracing::instrument]
pub async fn get_many(
    instance_ids: &[&str],
) -> crate::Result<Vec<InstanceMetadata>> {
    let state = State::get().await?;
    let mut instances = Vec::with_capacity(instance_ids.len());

    for instance_id in instance_ids {
        if let Some(instance) =
            crate::state::get_instance(instance_id, &state.pool).await?
        {
            instances.push(instance);
        }
    }

    Ok(instances)
}

#[tracing::instrument]
pub async fn list() -> crate::Result<Vec<InstanceMetadata>> {
    let state = State::get().await?;
    crate::state::list_instances(&state.pool).await
}

#[tracing::instrument]
#[allow(clippy::too_many_arguments)]
pub async fn create(
    name: String,
    game_version: String,
    modloader: ModLoader,
    loader_version: Option<String>,
    icon_path: Option<String>,
    link: InstanceLink,
    skip_install: Option<bool>,
) -> crate::Result<InstanceMetadata> {
    let state = State::get().await?;
    let instance = crate::state::create_instance(
        CreateInstance {
            name,
            path: None,
            game_version,
            loader: modloader,
            loader_version,
            icon_path,
            link,
        },
        &state,
    )
    .await?;

    let result = async {
        emit_instance(&instance.id, InstancePayloadType::Created).await?;

        if !skip_install.unwrap_or(false) {
            let context =
                crate::state::instances::commands::get_instance_launch_context(
                    &instance.id,
                    &state.pool,
                )
                .await?
                .ok_or_else(|| {
                    crate::ErrorKind::OtherError(format!(
                        "Missing launch context for instance {}",
                        instance.id
                    ))
                })?;
            crate::launcher::install_minecraft(&context, None, false).await?;
        }

        crate::state::get_instance(&instance.id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(
                    "Created instance could not be loaded".to_string(),
                )
                .into()
            })
    }
    .await;

    if result.is_err() {
        let _ = crate::state::remove_instance(&instance.id, &state).await;
    }

    result
}

pub async fn duplicate(copy_from: &str) -> crate::Result<String> {
    let metadata = get(copy_from).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let created = create(
        metadata.instance.name.clone(),
        metadata.applied_content_set.game_version.clone(),
        metadata.applied_content_set.loader,
        metadata.applied_content_set.loader_version.clone(),
        metadata.instance.icon_path.clone(),
        metadata.link.clone(),
        Some(true),
    )
    .await?;

    let state = State::get().await?;
    let bar = crate::pack::import::copy_dotminecraft(
        &created.instance.id,
        get_full_path(copy_from).await?,
        &state.io_semaphore,
        None,
    )
    .await?;

    let context =
        crate::state::instances::commands::get_instance_launch_context(
            &created.instance.id,
            &state.pool,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    crate::launcher::install_minecraft(&context, Some(bar), false).await?;
    emit_instance(&created.instance.id, InstancePayloadType::Edited).await?;

    Ok(created.instance.id)
}

#[tracing::instrument]
pub async fn sync_content_files(
    instance_id: &str,
) -> crate::Result<Vec<crate::state::instances::InstanceFile>> {
    let state = State::get().await?;
    crate::state::sync_content_files(instance_id, &state).await
}

#[tracing::instrument]
pub async fn list_content_sets(
    instance_id: &str,
) -> crate::Result<Vec<ContentSet>> {
    let state = State::get().await?;
    crate::state::list_content_sets(instance_id, &state.pool).await
}

#[tracing::instrument]
pub async fn get_projects(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<DashMap<String, ContentFile>> {
    let state = State::get().await?;
    crate::state::get_content_projects(
        instance_id,
        None,
        cache_behaviour,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn get_installed_project_ids(
    instance_id: &str,
) -> crate::Result<Vec<String>> {
    let state = State::get().await?;
    crate::state::get_installed_project_ids_for_instance(
        instance_id,
        None,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn get_content_items(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<Vec<ContentItem>> {
    let state = State::get().await?;
    crate::state::list_content(instance_id, None, cache_behaviour, &state).await
}

#[tracing::instrument]
pub async fn get_linked_modpack_content(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<Vec<ContentItem>> {
    let state = State::get().await?;
    crate::state::list_linked_modpack_content(
        instance_id,
        None,
        cache_behaviour,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn get_dependencies_as_content_items(
    dependencies: Vec<Dependency>,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<Vec<ContentItem>> {
    let state = State::get().await?;
    crate::state::dependencies_to_content_items(
        &dependencies,
        cache_behaviour,
        &state.pool,
        &state.api_semaphore,
    )
    .await
}

#[tracing::instrument]
pub async fn get_linked_modpack_info(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<Option<LinkedModpackInfo>> {
    let state = State::get().await?;
    crate::state::get_linked_modpack_info(
        instance_id,
        None,
        cache_behaviour,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn get_full_path(instance_id: &str) -> crate::Result<PathBuf> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;

    Ok(io::canonicalize(
        state
            .directories
            .instances_dir()
            .join(metadata.instance.path),
    )?)
}

#[tracing::instrument]
pub async fn get_mod_full_path(
    instance_id: &str,
    project_path: &str,
) -> crate::Result<PathBuf> {
    Ok(get_full_path(instance_id).await?.join(project_path))
}

pub async fn edit(
    instance_id: &str,
    patch: EditInstance,
) -> crate::Result<InstanceMetadata> {
    let state = State::get().await?;
    crate::state::edit_instance(instance_id, patch, &state.pool).await?;

    crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string()).into()
        })
}

pub async fn edit_icon(
    instance_id: &str,
    icon_path: Option<&Path>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let icon_path = if let Some(icon) = icon_path {
        let bytes = io::read(icon).await?;
        let file = crate::util::fetch::write_cached_icon(
            &icon.to_string_lossy(),
            &state.directories.caches_dir(),
            bytes::Bytes::from(bytes),
            &state.io_semaphore,
        )
        .await?;
        Some(file.to_string_lossy().to_string())
    } else {
        None
    };

    crate::state::edit_instance(
        instance_id,
        EditInstance {
            icon_path: Some(icon_path),
            ..EditInstance::default()
        },
        &state.pool,
    )
    .await?;
    emit_instance(&metadata.instance.id, InstancePayloadType::Edited).await?;

    Ok(())
}

pub async fn get_optimal_jre_key(
    instance_id: &str,
) -> crate::Result<Option<JavaVersion>> {
    let state = State::get().await?;
    let context =
        crate::state::instances::commands::get_instance_launch_context(
            instance_id,
            &state.pool,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "Tried to resolve a nonexistent instance {instance_id}!"
            ))
        })?;
    let (minecraft, version_index) =
        crate::launcher::resolve_minecraft_manifest(
            &context.applied_content_set.game_version,
            &state,
        )
        .await?;
    let version = &minecraft.versions[version_index];
    let loader_version = crate::launcher::get_loader_version_from_profile(
        &context.applied_content_set.game_version,
        context.applied_content_set.loader,
        context.applied_content_set.loader_version.as_deref(),
    )
    .await?;
    let version_info = crate::launcher::download::download_version_info(
        &state,
        version,
        loader_version.as_ref(),
        None,
        None,
    )
    .await?;

    crate::launcher::get_java_version_from_launch_context(
        &context,
        &version_info,
    )
    .await
}

#[tracing::instrument]
pub async fn install(instance_id: &str, force: bool) -> crate::Result<()> {
    let state = State::get().await?;
    let context =
        crate::state::instances::commands::get_instance_launch_context(
            instance_id,
            &state.pool,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "Tried to install a nonexistent instance {instance_id}!"
            ))
        })?;
    let result =
        crate::launcher::install_minecraft(&context, None, force).await;
    if result.is_err() {
        let current_stage =
            crate::state::instances::commands::get_instance_launch_context(
                instance_id,
                &state.pool,
            )
            .await
            .ok()
            .flatten()
            .map(|context| context.instance.install_stage)
            .unwrap_or(InstanceInstallStage::NotInstalled);
        if current_stage != InstanceInstallStage::Installed {
            crate::state::instances::commands::set_instance_install_stage(
                &context.instance.id,
                InstanceInstallStage::NotInstalled,
                &state.pool,
            )
            .await?;
        }
    }

    result
}

#[tracing::instrument]
pub async fn update_all_projects(
    instance_id: &str,
) -> crate::Result<HashMap<String, String>> {
    let state = State::get().await?;
    let metadata = get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let loading_bar = init_loading(
        LoadingBarType::InstanceUpdate {
            instance_id: metadata.instance.id.clone(),
            instance_name: metadata.instance.name.clone(),
        },
        100.0,
        "Updating instance",
    )
    .await?;
    let map = crate::state::instances::commands::update_all_projects(
        instance_id,
        &state,
    )
    .await?;
    emit_loading(&loading_bar, 100.0, Some("Updated instance"))?;
    emit_instance(&metadata.instance.id, InstancePayloadType::Edited).await?;

    Ok(map)
}

#[tracing::instrument]
pub async fn update_project(
    instance_id: &str,
    project_path: &str,
    skip_send_event: Option<bool>,
) -> crate::Result<String> {
    let state = State::get().await?;
    let metadata = get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let path = crate::state::instances::commands::update_project(
        instance_id,
        project_path,
        &state,
    )
    .await?;

    if !skip_send_event.unwrap_or(false) {
        emit_instance(&metadata.instance.id, InstancePayloadType::Edited)
            .await?;
    }

    Ok(path)
}

#[tracing::instrument]
pub async fn add_project_from_version(
    instance_id: &str,
    version_id: &str,
    reason: fetch::DownloadReason,
    dependent_on_version_id: Option<String>,
) -> crate::Result<String> {
    let state = State::get().await?;
    let metadata = get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let project_path =
        crate::state::instances::commands::add_project_from_version(
            instance_id,
            version_id,
            reason,
            dependent_on_version_id,
            crate::state::ContentSourceKind::Local,
            &state,
        )
        .await?;
    emit_instance(&metadata.instance.id, InstancePayloadType::Edited).await?;

    Ok(project_path)
}

#[tracing::instrument]
pub async fn add_project_from_path(
    instance_id: &str,
    path: &Path,
    project_type: Option<ProjectType>,
) -> crate::Result<String> {
    let state = State::get().await?;
    crate::state::instances::commands::add_project_from_path(
        instance_id,
        path,
        project_type,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn toggle_disable_project(
    instance_id: &str,
    project: &str,
) -> crate::Result<String> {
    let state = State::get().await?;
    let metadata = get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let res = crate::state::instances::commands::toggle_disable_project(
        instance_id,
        project,
        &state,
    )
    .await?;
    emit_instance(&metadata.instance.id, InstancePayloadType::Edited).await?;

    Ok(res)
}

#[tracing::instrument]
pub async fn remove_project(
    instance_id: &str,
    project: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    crate::state::instances::commands::remove_project(
        instance_id,
        project,
        &state,
    )
    .await?;
    emit_instance(&metadata.instance.id, InstancePayloadType::Edited).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn update_managed_modrinth_version(
    instance_id: &str,
    version_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    crate::state::instances::commands::update_managed_modrinth_version(
        instance_id,
        version_id,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn repair_managed_modrinth(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    crate::state::instances::commands::repair_managed_modrinth(
        instance_id,
        &state,
    )
    .await
}

#[tracing::instrument(skip_all)]
pub async fn export_mrpack(
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
        create_mrpack_json(&metadata, version_id, description).await?;
    let included_candidates_set = HashSet::<_>::from_iter(
        included_export_candidates.iter().map(|x| x.as_str()),
    );
    packfile
        .files
        .retain(|f| included_candidates_set.contains(f.path.as_str()));

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
            || !included_candidates_set
                .iter()
                .any(|x| relative_path.starts_with(&**x))
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

#[tracing::instrument]
pub async fn run(
    instance_id: &str,
    quick_play_type: QuickPlayType,
) -> crate::Result<ProcessMetadata> {
    let state = State::get().await?;
    let default_account = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or_else(|| crate::ErrorKind::NoCredentialsError.as_error())?;

    run_credentials(instance_id, &default_account, quick_play_type).await
}

#[tracing::instrument(skip(credentials))]
async fn run_credentials(
    instance_id: &str,
    credentials: &Credentials,
    quick_play_type: QuickPlayType,
) -> crate::Result<ProcessMetadata> {
    let state = State::get().await?;
    let settings = Settings::get(&state.pool).await?;
    let context =
        crate::state::instances::commands::get_instance_launch_context(
            instance_id,
            &state.pool,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "Tried to run a nonexistent instance {instance_id}!"
            ))
        })?;

    let pre_launch_hooks = context
        .launch_overrides
        .hooks
        .pre_launch
        .as_ref()
        .or(settings.hooks.pre_launch.as_ref())
        .filter(|hook_command| !hook_command.is_empty());
    if let Some(hook) = pre_launch_hooks {
        let mut cmd = shlex::split(hook)
            .ok_or_else(|| {
                crate::ErrorKind::LauncherError(format!(
                    "Invalid pre-launch command: {hook}",
                ))
            })?
            .into_iter();

        if let Some(command) = cmd.next() {
            let full_path = get_full_path(&context.instance.id).await?;
            let result = Command::new(command)
                .args(cmd)
                .current_dir(&full_path)
                .spawn()
                .map_err(|e| IOError::with_path(e, &full_path))?
                .wait()
                .await
                .map_err(IOError::from)?;

            if !result.success() {
                return Err(crate::ErrorKind::LauncherError(format!(
                    "Non-zero exit code for pre-launch hook: {}",
                    result.code().unwrap_or(-1)
                ))
                .as_error());
            }
        }
    }

    let java_args = context
        .launch_overrides
        .extra_launch_args
        .clone()
        .unwrap_or(settings.extra_launch_args);
    let wrapper = context
        .launch_overrides
        .hooks
        .wrapper
        .clone()
        .or(settings.hooks.wrapper)
        .filter(|hook_command| !hook_command.is_empty());
    let memory = context.launch_overrides.memory.unwrap_or(settings.memory);
    let resolution = context
        .launch_overrides
        .game_resolution
        .unwrap_or(settings.game_resolution);
    let env_args = context
        .launch_overrides
        .custom_env_vars
        .clone()
        .unwrap_or(settings.custom_env_vars);
    let post_exit_hook = context
        .launch_overrides
        .hooks
        .post_exit
        .clone()
        .or(settings.hooks.post_exit)
        .filter(|hook_command| !hook_command.is_empty());

    let mut mc_set_options: Vec<(String, String)> = vec![];
    if let Some(fullscreen) = context.launch_overrides.force_fullscreen {
        mc_set_options.push(("fullscreen".to_string(), fullscreen.to_string()));
    } else if settings.force_fullscreen {
        mc_set_options.push(("fullscreen".to_string(), "true".to_string()));
    }

    if let Some(project_id) = server_play_project_id(&context.link)
        && !project_id.trim().is_empty()
    {
        let server_id = uuid::Uuid::new_v4().to_string();
        let join_result = fetch::INSECURE_REQWEST_CLIENT
			.post("https://sessionserver.mojang.com/session/minecraft/join")
			.json(&json!({
				"accessToken": &credentials.access_token,
				"selectedProfile": credentials.offline_profile.id.simple().to_string(),
				"serverId": &server_id,
			}))
			.timeout(Duration::from_secs(5))
			.send()
			.await;

        match join_result {
            Ok(resp) if resp.status().is_success() => {
                let result = fetch::post_json(
                    concat!(
                        env!("MODRINTH_API_BASE_URL"),
                        "analytics/minecraft-server-play"
                    ),
                    json!({
                        "project_id": project_id,
                        "username": &credentials.offline_profile.name,
                        "server_id": &server_id,
                    }),
                    &state.api_semaphore,
                    &state.pool,
                )
                .await;

                match result {
                    Ok(()) => {
                        info!(
                            "Tracked server play for '{project_id}' in analytics"
                        )
                    }
                    Err(err) => warn!("Failed to report server play: {err:?}"),
                }
            }
            Ok(resp) => warn!(
                "Failed to join Mojang session server: HTTP {}",
                resp.status()
            ),
            Err(err) => warn!("Failed to join Mojang session server: {err:?}"),
        }
    }

    crate::minecraft_skins::flush_pending_skin_change().await?;
    crate::launcher::launch_minecraft(
        &java_args,
        &env_args,
        &mc_set_options,
        &wrapper,
        &memory,
        &resolution,
        credentials,
        post_exit_hook,
        &context,
        quick_play_type,
    )
    .await
}

fn server_play_project_id(link: &InstanceLink) -> Option<&String> {
    match link {
        InstanceLink::ServerProject { project_id }
        | InstanceLink::ServerProjectModpack {
            server_project_id: project_id,
            ..
        } => Some(project_id),
        InstanceLink::Unmanaged
        | InstanceLink::ModrinthModpack { .. }
        | InstanceLink::ModrinthHosting { .. }
        | InstanceLink::ImportedModpack { .. }
        | InstanceLink::SharedInstance { .. } => None,
    }
}

pub async fn kill(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let processes =
        crate::api::process::get_by_instance_id(instance_id).await?;

    for process in processes {
        state.process_manager.kill(process.uuid).await?;
    }

    Ok(())
}

#[tracing::instrument]
pub async fn try_update_playtime_by_instance_id(
    instance_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let context =
		crate::state::instances::commands::get_instance_launch_context(
			instance_id,
			&state.pool,
		)
		.await?
		.ok_or_else(|| {
			crate::ErrorKind::OtherError(format!(
				"Tried to update playtime for nonexistent instance {instance_id}!"
			))
		})?;
    let updated_recent_playtime = context.instance.recent_time_played;
    let res = if updated_recent_playtime > 0 {
        let modrinth_pack_version_id = match &context.link {
            InstanceLink::ModrinthModpack { version_id, .. }
            | InstanceLink::ServerProjectModpack {
                content_version_id: version_id,
                ..
            }
            | InstanceLink::ImportedModpack {
                version_id: Some(version_id),
                ..
            } => Some(version_id.clone()),
            InstanceLink::Unmanaged
            | InstanceLink::ServerProject { .. }
            | InstanceLink::ModrinthHosting { .. }
            | InstanceLink::ImportedModpack { .. }
            | InstanceLink::SharedInstance { .. } => None,
        };
        let playtime_update_json = json!({
            "seconds": updated_recent_playtime,
            "loader": context.applied_content_set.loader.as_str(),
            "game_version": &context.applied_content_set.game_version,
            "parent": modrinth_pack_version_id,
        });
        let mut hashmap: HashMap<String, serde_json::Value> = HashMap::new();

        for (_, project) in get_projects(instance_id, None).await? {
            if let Some(metadata) = project.metadata {
                hashmap
                    .insert(metadata.version_id, playtime_update_json.clone());
            }
        }

        fetch::post_json(
            concat!(env!("MODRINTH_API_BASE_URL"), "analytics/playtime"),
            serde_json::to_value(hashmap)?,
            &state.api_semaphore,
            &state.pool,
        )
        .await
    } else {
        Ok(())
    };

    if res.is_ok() {
        crate::state::instances::commands::mark_instance_playtime_submitted(
            &context.instance.id,
            updated_recent_playtime,
            &state.pool,
        )
        .await?;
    }

    res
}

#[tracing::instrument(skip_all)]
pub async fn create_mrpack_json(
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

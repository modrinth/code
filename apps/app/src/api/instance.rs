use crate::api::Result;
use dashmap::DashMap;
use path_util::SafeRelativeUtf8UnixPathBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use theseus::DownloadReason;
use theseus::data::{
    AppliedContentSetPatch, ContentItem, Dependency,
    EditInstance as CoreEditInstance, InstanceInstallCandidate,
    InstanceInstallTarget, InstanceLaunchOverridesPatch,
    InstanceLink as CoreInstanceLink, InstanceMetadata, LinkedModpackInfo,
};
use theseus::instance::InstallProjectWithDependenciesRequest;
use theseus::instance::QuickPlayType;
use theseus::prelude::*;
use theseus::server_address::ServerAddress;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("instance")
        .invoke_handler(tauri::generate_handler![
            instance_remove,
            instance_get,
            instance_get_many,
            instance_list,
            instance_get_projects,
            instance_get_installed_project_ids,
            instance_get_install_candidates,
            instance_content,
            instance_get_content_items,
            instance_get_dependencies_as_content_items,
            instance_get_linked_modpack_info,
            instance_get_linked_modpack_content,
            instance_get_optimal_jre_key,
            instance_get_full_path,
            instance_get_mod_full_path,
            instance_check_installed,
            instance_update_all,
            instance_update_project,
            instance_add_project_from_version,
            instance_install_project_with_dependencies,
            instance_switch_project_version_with_dependencies,
            instance_add_project_from_path,
            instance_is_file_on_modrinth,
            instance_toggle_disable_project,
            instance_remove_project,
            instance_update_managed_modrinth_version,
            instance_repair_managed_modrinth,
            instance_run,
            instance_kill,
            instance_edit,
            instance_edit_icon,
            instance_export_mrpack,
            instance_get_pack_export_candidates,
        ])
        .build()
}

#[derive(Serialize, Debug, Clone)]
pub struct Instance {
    pub id: String,
    pub path: String,
    pub install_stage: String,
    pub launcher_feature_version: String,
    pub name: String,
    pub icon_path: Option<String>,
    pub game_version: String,
    pub protocol_version: Option<u32>,
    pub loader: ModLoader,
    pub loader_version: Option<String>,
    pub groups: Vec<String>,
    pub link: Option<InstanceLink>,
    pub update_channel: ReleaseChannel,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub last_played: Option<chrono::DateTime<chrono::Utc>>,
    pub submitted_time_played: u64,
    pub recent_time_played: u64,
    pub java_path: Option<String>,
    pub extra_launch_args: Option<Vec<String>>,
    pub custom_env_vars: Option<Vec<(String, String)>>,
    pub memory: Option<MemorySettings>,
    pub force_fullscreen: Option<bool>,
    pub game_resolution: Option<WindowSize>,
    pub hooks: Hooks,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstanceLink {
    ModrinthModpack {
        project_id: String,
        version_id: String,
    },
    ServerProject {
        project_id: String,
    },
    ServerProjectModpack {
        server_project_id: String,
        content_project_id: Option<String>,
        content_version_id: String,
        project_id: Option<String>,
        version_id: Option<String>,
    },
    ImportedModpack {
        project_id: Option<String>,
        version_id: Option<String>,
        name: Option<String>,
        version_number: Option<String>,
        filename: Option<String>,
    },
    ModrinthHosting {
        server_id: String,
        instance_ids: Vec<String>,
        active_instance_id: Option<String>,
    },
    SharedInstance {
        shared_instance_id: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditInstance {
    pub name: Option<String>,

    pub game_version: Option<String>,
    pub loader: Option<ModLoader>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub loader_version: Option<Option<String>>,

    pub groups: Option<Vec<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub link: Option<Option<InstanceLink>>,
    pub update_channel: Option<ReleaseChannel>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub java_path: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub extra_launch_args: Option<Option<Vec<String>>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub custom_env_vars: Option<Option<Vec<(String, String)>>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub memory: Option<Option<MemorySettings>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub force_fullscreen: Option<Option<bool>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub game_resolution: Option<Option<WindowSize>>,
    pub hooks: Option<Hooks>,
}

impl From<InstanceMetadata> for Instance {
    fn from(metadata: InstanceMetadata) -> Self {
        Self {
            id: metadata.instance.id,
            path: metadata.instance.path,
            install_stage: metadata.instance.install_stage.as_str().to_string(),
            launcher_feature_version: metadata
                .instance
                .launcher_feature_version
                .as_str()
                .to_string(),
            name: metadata.instance.name,
            icon_path: metadata.instance.icon_path,
            game_version: metadata.applied_content_set.game_version,
            protocol_version: metadata.applied_content_set.protocol_version,
            loader: metadata.applied_content_set.loader,
            loader_version: metadata.applied_content_set.loader_version,
            groups: metadata.groups,
            link: InstanceLink::from_core(metadata.link),
            update_channel: metadata.instance.update_channel,
            created: metadata.instance.created,
            modified: metadata.instance.modified,
            last_played: metadata.instance.last_played,
            submitted_time_played: metadata.instance.submitted_time_played,
            recent_time_played: metadata.instance.recent_time_played,
            java_path: metadata.launch_overrides.java_path,
            extra_launch_args: metadata.launch_overrides.extra_launch_args,
            custom_env_vars: metadata.launch_overrides.custom_env_vars,
            memory: metadata.launch_overrides.memory,
            force_fullscreen: metadata.launch_overrides.force_fullscreen,
            game_resolution: metadata.launch_overrides.game_resolution,
            hooks: metadata.launch_overrides.hooks,
        }
    }
}

impl InstanceLink {
    fn from_core(link: CoreInstanceLink) -> Option<Self> {
        match link {
            CoreInstanceLink::Unmanaged => None,
            CoreInstanceLink::ModrinthModpack {
                project_id,
                version_id,
            } => Some(Self::ModrinthModpack {
                project_id,
                version_id,
            }),
            CoreInstanceLink::ServerProject { project_id } => {
                Some(Self::ServerProject { project_id })
            }
            CoreInstanceLink::ServerProjectModpack {
                server_project_id,
                content_project_id,
                content_version_id,
            } => Some(Self::ServerProjectModpack {
                project_id: Some(server_project_id.clone()),
                version_id: Some(content_version_id.clone()),
                server_project_id,
                content_project_id: Some(content_project_id),
                content_version_id,
            }),
            CoreInstanceLink::ImportedModpack {
                project_id,
                version_id,
                name,
                version_number,
                filename,
            } => Some(Self::ImportedModpack {
                project_id,
                version_id,
                name,
                version_number,
                filename,
            }),
            CoreInstanceLink::ModrinthHosting {
                server_id,
                instance_ids,
                active_instance_id,
            } => Some(Self::ModrinthHosting {
                server_id: server_id.to_string(),
                instance_ids: instance_ids
                    .into_iter()
                    .map(|id| id.to_string())
                    .collect(),
                active_instance_id: active_instance_id.map(|id| id.to_string()),
            }),
            CoreInstanceLink::SharedInstance { shared_instance_id } => {
                Some(Self::SharedInstance {
                    shared_instance_id: shared_instance_id.to_string(),
                })
            }
        }
    }

    pub(crate) fn into_core(self) -> Result<CoreInstanceLink> {
        match self {
            Self::ModrinthModpack {
                project_id,
                version_id,
            } => Ok(CoreInstanceLink::ModrinthModpack {
                project_id,
                version_id,
            }),
            Self::ServerProject { project_id } => {
                Ok(CoreInstanceLink::ServerProject { project_id })
            }
            Self::ServerProjectModpack {
                server_project_id,
                content_project_id,
                content_version_id,
                ..
            } => Ok(CoreInstanceLink::ServerProjectModpack {
                server_project_id,
                content_project_id: content_project_id.unwrap_or_default(),
                content_version_id,
            }),
            Self::ImportedModpack {
                project_id,
                version_id,
                name,
                version_number,
                filename,
            } => Ok(CoreInstanceLink::ImportedModpack {
                project_id,
                version_id,
                name,
                version_number,
                filename,
            }),
            Self::ModrinthHosting {
                server_id,
                instance_ids,
                active_instance_id,
            } => Ok(CoreInstanceLink::ModrinthHosting {
                server_id: server_id.parse().map_err(|err| {
                    theseus::Error::from(theseus::ErrorKind::InputError(
                        format!("Invalid server id: {err}"),
                    ))
                })?,
                instance_ids: instance_ids
                    .into_iter()
                    .map(|id| {
                        id.parse().map_err(|err| {
                            theseus::Error::from(
                                theseus::ErrorKind::InputError(format!(
                                    "Invalid hosted instance id: {err}"
                                )),
                            )
                        })
                    })
                    .collect::<std::result::Result<Vec<_>, _>>()?,
                active_instance_id: active_instance_id
                    .map(|id| {
                        id.parse().map_err(|err| {
                            theseus::Error::from(
                                theseus::ErrorKind::InputError(format!(
                                    "Invalid active instance id: {err}"
                                )),
                            )
                        })
                    })
                    .transpose()?,
            }),
            Self::SharedInstance { shared_instance_id } => {
                Ok(CoreInstanceLink::SharedInstance {
                    shared_instance_id: shared_instance_id.parse().map_err(
                        |err| {
                            theseus::Error::from(
                                theseus::ErrorKind::InputError(format!(
                                    "Invalid shared instance id: {err}"
                                )),
                            )
                        },
                    )?,
                })
            }
        }
    }
}

fn edit_to_core(edit_instance: EditInstance) -> Result<CoreEditInstance> {
    Ok(CoreEditInstance {
        install_stage: None,
        launcher_feature_version: None,
        name: edit_instance.name,
        icon_path: None,
        update_channel: edit_instance.update_channel,
        groups: edit_instance.groups,
        link: edit_instance
            .link
            .map(|link| match link {
                Some(link) => link.into_core(),
                None => Ok(CoreInstanceLink::Unmanaged),
            })
            .transpose()?,
        launch_overrides: Some(InstanceLaunchOverridesPatch {
            java_path: edit_instance.java_path,
            extra_launch_args: edit_instance.extra_launch_args,
            custom_env_vars: edit_instance.custom_env_vars,
            memory: edit_instance.memory,
            force_fullscreen: edit_instance.force_fullscreen,
            game_resolution: edit_instance.game_resolution,
            hooks: edit_instance.hooks,
        }),
        content_set_patch: Some(AppliedContentSetPatch {
            source_kind: None,
            game_version: edit_instance.game_version,
            protocol_version: Some(None),
            loader: edit_instance.loader,
            loader_version: edit_instance.loader_version,
        }),
        last_played: None,
        submitted_time_played: None,
        recent_time_played: None,
    })
}

#[tauri::command]
pub async fn instance_remove(instance_id: &str) -> Result<()> {
    theseus::instance::remove(instance_id).await?;
    Ok(())
}

#[tauri::command]
pub async fn instance_get(instance_id: &str) -> Result<Option<Instance>> {
    Ok(theseus::instance::get(instance_id)
        .await?
        .map(Instance::from))
}

#[tauri::command]
pub async fn instance_get_many(
    instance_ids: Vec<String>,
) -> Result<Vec<Instance>> {
    let ids = instance_ids.iter().map(|x| &**x).collect::<Vec<&str>>();
    Ok(theseus::instance::get_many(&ids)
        .await?
        .into_iter()
        .map(Instance::from)
        .collect())
}

#[tauri::command]
pub async fn instance_list() -> Result<Vec<Instance>> {
    Ok(theseus::instance::list()
        .await?
        .into_iter()
        .map(Instance::from)
        .collect())
}

#[tauri::command]
pub async fn instance_get_projects(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> Result<DashMap<String, ContentFile>> {
    Ok(theseus::instance::get_projects(instance_id, cache_behaviour).await?)
}

#[tauri::command]
pub async fn instance_get_installed_project_ids(
    instance_id: &str,
) -> Result<Vec<String>> {
    Ok(theseus::instance::get_installed_project_ids(instance_id).await?)
}

#[tauri::command]
pub async fn instance_get_install_candidates(
    project_id: &str,
    project_type: ProjectType,
    targets: Vec<InstanceInstallTarget>,
) -> Result<Vec<InstanceInstallCandidate>> {
    Ok(theseus::instance::get_install_candidates(
        project_id,
        project_type,
        targets,
    )
    .await?)
}

#[tauri::command]
pub async fn instance_content(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> Result<Vec<ContentItem>> {
    instance_get_content_items(instance_id, cache_behaviour).await
}

#[tauri::command]
pub async fn instance_get_content_items(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> Result<Vec<ContentItem>> {
    Ok(
        theseus::instance::get_content_items(instance_id, cache_behaviour)
            .await?,
    )
}

#[tauri::command]
pub async fn instance_get_dependencies_as_content_items(
    dependencies: Vec<Dependency>,
    cache_behaviour: Option<CacheBehaviour>,
) -> Result<Vec<ContentItem>> {
    Ok(theseus::instance::get_dependencies_as_content_items(
        dependencies,
        cache_behaviour,
    )
    .await?)
}

#[tauri::command]
pub async fn instance_get_linked_modpack_info(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> Result<Option<LinkedModpackInfo>> {
    Ok(
        theseus::instance::get_linked_modpack_info(
            instance_id,
            cache_behaviour,
        )
        .await?,
    )
}

#[tauri::command]
pub async fn instance_get_linked_modpack_content(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> Result<Vec<ContentItem>> {
    Ok(theseus::instance::get_linked_modpack_content(
        instance_id,
        cache_behaviour,
    )
    .await?)
}

#[tauri::command]
pub async fn instance_get_full_path(instance_id: &str) -> Result<PathBuf> {
    Ok(theseus::instance::get_full_path(instance_id).await?)
}

#[tauri::command]
pub async fn instance_get_mod_full_path(
    instance_id: &str,
    project_path: &str,
) -> Result<PathBuf> {
    Ok(theseus::instance::get_mod_full_path(instance_id, project_path).await?)
}

#[tauri::command]
pub async fn instance_get_optimal_jre_key(
    instance_id: &str,
) -> Result<Option<JavaVersion>> {
    Ok(theseus::instance::get_optimal_jre_key(instance_id).await?)
}

#[tauri::command]
pub async fn instance_check_installed(
    instance_id: &str,
    project_id: &str,
) -> Result<bool> {
    let check_project_id = project_id;

    if let Ok(projects) =
        theseus::instance::get_projects(instance_id, None).await
    {
        Ok(projects.into_iter().any(|(_, project)| {
            project
                .metadata
                .as_ref()
                .is_some_and(|metadata| check_project_id == metadata.project_id)
        }))
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn instance_update_all(
    instance_id: &str,
) -> Result<HashMap<String, String>> {
    Ok(theseus::instance::update_all_projects(instance_id).await?)
}

#[tauri::command]
pub async fn instance_update_project(
    instance_id: &str,
    project_path: &str,
) -> Result<String> {
    Ok(
        theseus::instance::update_project(instance_id, project_path, None)
            .await?,
    )
}

#[tauri::command]
pub async fn instance_add_project_from_version(
    instance_id: &str,
    version_id: &str,
    reason: DownloadReason,
    dependent_on_version_id: Option<String>,
) -> Result<String> {
    Ok(theseus::instance::add_project_from_version(
        instance_id,
        version_id,
        reason,
        dependent_on_version_id,
    )
    .await?)
}

#[tauri::command]
pub async fn instance_install_project_with_dependencies(
    instance_id: &str,
    request: InstallProjectWithDependenciesRequest,
) -> Result<ResolveContentPlan> {
    Ok(theseus::instance::install_project_with_dependencies(
        instance_id,
        request,
    )
    .await?)
}

#[tauri::command]
pub async fn instance_switch_project_version_with_dependencies(
    instance_id: &str,
    project_path: &str,
    version_id: &str,
) -> Result<String> {
    Ok(theseus::instance::switch_project_version_with_dependencies(
        instance_id,
        project_path,
        version_id,
    )
    .await?)
}

#[tauri::command]
pub async fn instance_add_project_from_path(
    instance_id: &str,
    project_path: &Path,
    project_type: Option<ProjectType>,
) -> Result<String> {
    Ok(theseus::instance::add_project_from_path(
        instance_id,
        project_path,
        project_type,
    )
    .await?)
}

#[tauri::command]
pub async fn instance_is_file_on_modrinth(project_path: &Path) -> Result<bool> {
    Ok(theseus::instance::is_file_on_modrinth(project_path).await?)
}

#[tauri::command]
pub async fn instance_toggle_disable_project(
    instance_id: &str,
    project_path: &str,
    desired_enabled: Option<bool>,
) -> Result<String> {
    Ok(theseus::instance::toggle_disable_project(
        instance_id,
        project_path,
        desired_enabled,
    )
    .await?)
}

#[tauri::command]
pub async fn instance_remove_project(
    instance_id: &str,
    project_path: &str,
) -> Result<()> {
    theseus::instance::remove_project(instance_id, project_path).await?;
    Ok(())
}

#[tauri::command]
pub async fn instance_update_managed_modrinth_version(
    instance_id: String,
    version_id: String,
) -> Result<theseus::install::InstallJobSnapshot> {
    Ok(theseus::instance::update_managed_modrinth_version(
        &instance_id,
        &version_id,
    )
    .await?)
}

#[tauri::command]
pub async fn instance_repair_managed_modrinth(
    instance_id: &str,
) -> Result<theseus::install::InstallJobSnapshot> {
    Ok(theseus::instance::repair_managed_modrinth(instance_id).await?)
}

#[tauri::command]
pub async fn instance_export_mrpack(
    instance_id: &str,
    export_location: PathBuf,
    included_overrides: Vec<String>,
    version_id: Option<String>,
    description: Option<String>,
    name: Option<String>,
) -> Result<()> {
    theseus::instance::export_mrpack(
        instance_id,
        export_location,
        included_overrides,
        version_id,
        description,
        name,
    )
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn instance_get_pack_export_candidates(
    instance_id: &str,
) -> Result<Vec<SafeRelativeUtf8UnixPathBuf>> {
    Ok(theseus::instance::get_pack_export_candidates(instance_id).await?)
}

#[tauri::command]
pub async fn instance_run(
    instance_id: &str,
    server_address: Option<String>,
) -> Result<ProcessMetadata> {
    let quick_play = match server_address {
        Some(addr) => QuickPlayType::Server(ServerAddress::Unresolved(addr)),
        None => QuickPlayType::None,
    };
    Ok(theseus::instance::run(instance_id, quick_play).await?)
}

#[tauri::command]
pub async fn instance_kill(instance_id: &str) -> Result<()> {
    theseus::instance::kill(instance_id).await?;
    Ok(())
}

#[tauri::command]
pub async fn instance_edit(
    instance_id: &str,
    edit_instance: EditInstance,
) -> Result<()> {
    theseus::instance::edit(instance_id, edit_to_core(edit_instance)?).await?;
    Ok(())
}

#[tauri::command]
pub async fn instance_edit_icon(
    instance_id: &str,
    icon_path: Option<&Path>,
) -> Result<()> {
    theseus::instance::edit_icon(instance_id, icon_path).await?;
    Ok(())
}

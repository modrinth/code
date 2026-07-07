use crate::api::Result;
use crate::api::instance::InstanceLink;
use serde::Deserialize;
use std::path::PathBuf;
use theseus::data::ModLoader;
use theseus::install::{
    InstallJobSnapshot, InstallModpackPreview, InstallPostInstallEdit,
};
use theseus::instance::{
    SharedInstanceInstallPreview, SharedInstanceUpdatePreview,
};
use theseus::pack::import::ImportLauncherType;
use theseus::pack::install_from::CreatePackLocation;
use uuid::Uuid;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("install")
        .invoke_handler(tauri::generate_handler![
            install_get_modpack_preview,
            install_create_instance,
            install_create_modpack_instance,
            install_get_shared_instance_preview,
            install_get_shared_instance_update_preview,
            install_shared_instance,
            install_update_shared_instance,
            install_import_instance,
            install_duplicate_instance,
            install_existing_instance,
            install_pack_to_existing_instance,
            install_job_list,
            install_job_get,
            install_job_retry,
            install_job_cancel,
            install_job_dismiss,
        ])
        .build()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallCreateInstanceRequest {
    pub name: String,
    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<String>,
    pub icon_path: Option<String>,
    pub link: Option<InstanceLink>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPostInstallEditRequest {
    pub name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub icon_path: Option<Option<String>>,
    pub link: Option<InstanceLink>,
}

impl InstallPostInstallEditRequest {
    fn into_core(self) -> Result<InstallPostInstallEdit> {
        Ok(InstallPostInstallEdit {
            name: self.name,
            icon_path: self.icon_path,
            link: self.link.map(|link| link.into_core()).transpose()?,
        })
    }
}

#[tauri::command]
pub async fn install_get_modpack_preview(
    location: CreatePackLocation,
) -> Result<InstallModpackPreview> {
    Ok(theseus::pack::install_from::get_instance_from_pack(location).await?)
}

#[tauri::command]
pub async fn install_create_instance(
    request: InstallCreateInstanceRequest,
) -> Result<InstallJobSnapshot> {
    Ok(theseus::install::create_instance(
        request.name.trim().to_string(),
        request.game_version,
        request.loader,
        request.loader_version,
        request.icon_path,
        match request.link {
            Some(link) => link.into_core()?,
            None => theseus::data::InstanceLink::Unmanaged,
        },
    )
    .await?)
}

#[tauri::command]
pub async fn install_create_modpack_instance(
    location: CreatePackLocation,
    post_install_edit: Option<InstallPostInstallEditRequest>,
) -> Result<InstallJobSnapshot> {
    Ok(theseus::install::create_modpack_instance(
        location,
        post_install_edit.map(|edit| edit.into_core()).transpose()?,
    )
    .await?)
}

#[tauri::command]
pub async fn install_get_shared_instance_preview(
    shared_instance_id: String,
    name: String,
) -> Result<SharedInstanceInstallPreview> {
    Ok(theseus::instance::get_shared_instance_install_preview(
        &shared_instance_id,
        name,
    )
    .await?)
}

#[tauri::command]
pub async fn install_get_shared_instance_update_preview(
    instance_id: String,
) -> Result<Option<SharedInstanceUpdatePreview>> {
    Ok(
        theseus::instance::get_shared_instance_update_preview(&instance_id)
            .await?,
    )
}

#[tauri::command]
pub async fn install_shared_instance(
    shared_instance_id: String,
    name: String,
    manager_id: Option<String>,
) -> Result<InstallJobSnapshot> {
    Ok(theseus::instance::install_shared_instance(
        &shared_instance_id,
        name,
        manager_id,
    )
    .await?)
}

#[tauri::command]
pub async fn install_update_shared_instance(
    instance_id: String,
) -> Result<InstallJobSnapshot> {
    Ok(theseus::instance::update_shared_instance(&instance_id).await?)
}

#[tauri::command]
pub async fn install_import_instance(
    launcher_type: ImportLauncherType,
    base_path: PathBuf,
    instance_folder: String,
) -> Result<InstallJobSnapshot> {
    Ok(theseus::install::import_instance(
        launcher_type,
        base_path,
        instance_folder,
    )
    .await?)
}

#[tauri::command]
pub async fn install_duplicate_instance(
    source_instance_id: String,
) -> Result<InstallJobSnapshot> {
    Ok(theseus::install::duplicate_instance(source_instance_id).await?)
}

#[tauri::command]
pub async fn install_existing_instance(
    instance_id: String,
    force: bool,
) -> Result<InstallJobSnapshot> {
    Ok(theseus::install::install_existing_instance(instance_id, force).await?)
}

#[tauri::command]
pub async fn install_pack_to_existing_instance(
    instance_id: String,
    location: CreatePackLocation,
    post_install_edit: Option<InstallPostInstallEditRequest>,
) -> Result<InstallJobSnapshot> {
    Ok(theseus::install::install_pack_to_existing_instance(
        instance_id,
        location,
        post_install_edit.map(|edit| edit.into_core()).transpose()?,
    )
    .await?)
}

#[tauri::command]
pub async fn install_job_list(
    include_finished: bool,
) -> Result<Vec<InstallJobSnapshot>> {
    Ok(theseus::install::list_jobs(include_finished).await?)
}

#[tauri::command]
pub async fn install_job_get(job_id: Uuid) -> Result<InstallJobSnapshot> {
    Ok(theseus::install::get_job(job_id).await?)
}

#[tauri::command]
pub async fn install_job_retry(job_id: Uuid) -> Result<InstallJobSnapshot> {
    Ok(theseus::install::retry_job(job_id).await?)
}

#[tauri::command]
pub async fn install_job_cancel(job_id: Uuid) -> Result<InstallJobSnapshot> {
    Ok(theseus::install::cancel_job(job_id).await?)
}

#[tauri::command]
pub async fn install_job_dismiss(job_id: Uuid) -> Result<()> {
    Ok(theseus::install::dismiss_job(job_id).await?)
}

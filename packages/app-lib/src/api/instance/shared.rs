use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::instances::{InstanceLink, SharedInstanceAttachment};
use crate::state::{
    ContentSetSyncStatus, ModrinthCredentials, ProjectType,
    SharedInstanceRole, State,
};
use crate::util::fetch::{INSECURE_REQWEST_CLIENT, REQWEST_CLIENT};
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedInstanceUsers {
    pub user_ids: Vec<String>,
}

#[derive(Clone, Debug)]
struct ExternalFileCandidate {
    file_name: String,
    file_type: String,
    file_path: String,
}

#[derive(Clone, Debug, Serialize)]
struct ExternalFileData {
    file_name: String,
    file_type: String,
}

#[derive(Clone, Debug, Deserialize)]
struct CreateInstanceResponse {
    #[serde(alias = "instance_id")]
    id: String,
}

#[derive(Clone, Debug, Deserialize)]
struct InstanceVersionResponse {
    version: i32,
    ready: bool,
    external_files: Vec<ExternalFileResponse>,
}

#[derive(Clone, Debug, Deserialize)]
struct ExternalFileResponse {
    file_name: String,
    file_type: String,
    url: String,
}

#[tracing::instrument]
pub async fn get_shared_instance_users(
    instance_id: &str,
) -> crate::Result<SharedInstanceUsers> {
    let state = State::get().await?;
    let Some(attachment) = shared_attachment(instance_id, &state).await?
    else {
        return Ok(SharedInstanceUsers { user_ids: Vec::new() });
    };

    get_remote_users(&attachment.id, &state).await
}

#[tracing::instrument]
pub async fn invite_shared_instance_users(
    instance_id: &str,
    user_ids: Vec<String>,
) -> crate::Result<SharedInstanceUsers> {
    let state = State::get().await?;
    let attachment = match shared_attachment(instance_id, &state).await? {
        Some(attachment) => {
            tracing::debug!(
                instance_id,
                shared_instance_id = %attachment.id,
                role = attachment.role.as_str(),
                user_count = user_ids.len(),
                "Using existing shared instance attachment for invite"
            );
            attachment
        }
        None => {
            ensure_shareable_instance(instance_id, &state).await?;
            tracing::info!(
                instance_id,
                user_count = user_ids.len(),
                "Creating shared instance before first invite"
            );
            let remote = create_remote_instance(&state).await?;
            tracing::info!(
                instance_id,
                shared_instance_id = %remote.id,
                "Created remote shared instance"
            );
            crate::state::attach_shared_instance(
                instance_id,
                &remote.id,
                SharedInstanceRole::Owner,
                ContentSetSyncStatus::Unknown,
                None,
                None,
                &state.pool,
            )
            .await?;
            tracing::debug!(
                instance_id,
                shared_instance_id = %remote.id,
                "Attached local instance as shared instance owner"
            );
            publish_shared_instance_inner(instance_id, &state).await?;
            shared_attachment(instance_id, &state)
                .await?
                .ok_or_else(|| {
                    crate::ErrorKind::InputError(
                        "Shared instance attachment was not persisted"
                            .to_string(),
                    )
                })?
        }
    };

    ensure_owner(&attachment)?;
    if !user_ids.is_empty() {
        tracing::info!(
            instance_id,
            shared_instance_id = %attachment.id,
            user_count = user_ids.len(),
            "Adding users to shared instance"
        );
        add_remote_users(&attachment.id, user_ids.clone(), &state).await?;
    }
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(SharedInstanceUsers { user_ids })
}

#[tracing::instrument]
pub async fn remove_shared_instance_users(
    instance_id: &str,
    user_ids: Vec<String>,
) -> crate::Result<SharedInstanceUsers> {
    let state = State::get().await?;
    let Some(attachment) = shared_attachment(instance_id, &state).await?
    else {
        return Ok(SharedInstanceUsers { user_ids: Vec::new() });
    };
    ensure_owner(&attachment)?;

    if !user_ids.is_empty() {
        remove_remote_users(&attachment.id, user_ids, &state).await?;
    }

    let remaining_users = get_remote_users(&attachment.id, &state).await?;
    if remaining_users.user_ids.is_empty() {
        delete_remote_instance(&attachment.id, &state).await?;
        crate::state::clear_shared_instance(instance_id, &state.pool).await?;
    }

    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(remaining_users)
}

#[tracing::instrument]
pub async fn publish_shared_instance(
    instance_id: &str,
) -> crate::Result<SharedInstanceAttachment> {
    let state = State::get().await?;
    publish_shared_instance_inner(instance_id, &state).await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    shared_attachment(instance_id, &state)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "Shared instance attachment was not persisted".to_string(),
            )
            .into()
        })
}

pub(crate) async fn mark_shared_instance_stale(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    crate::state::mark_shared_instance_stale(instance_id, &state.pool).await
}

async fn publish_shared_instance_inner(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    let attachment = shared_attachment(instance_id, state)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "Instance is not attached to a shared instance".to_string(),
            )
        })?;
    ensure_owner(&attachment)?;
    tracing::info!(
        instance_id,
        shared_instance_id = %attachment.id,
        applied_version = attachment.applied_version,
        latest_version = attachment.latest_version,
        "Publishing shared instance content"
    );

    crate::state::set_shared_instance_sync_status(
        instance_id,
        ContentSetSyncStatus::Applying,
        attachment.applied_version,
        attachment.latest_version,
        &state.pool,
    )
    .await?;

    let result = publish_current_content(instance_id, &attachment.id, state).await;

    match result {
        Ok(version) => {
            tracing::info!(
                instance_id,
                shared_instance_id = %attachment.id,
                version,
                "Published shared instance content"
            );
            crate::state::set_shared_instance_sync_status(
                instance_id,
                ContentSetSyncStatus::UpToDate,
                Some(version),
                Some(version),
                &state.pool,
            )
            .await?;
            Ok(())
        }
        Err(error) => {
            tracing::warn!(
                instance_id,
                shared_instance_id = %attachment.id,
                error = %error,
                "Failed to publish shared instance content"
            );
            crate::state::set_shared_instance_sync_status(
                instance_id,
                ContentSetSyncStatus::Error,
                attachment.applied_version,
                attachment.latest_version,
                &state.pool,
            )
            .await?;
            Err(error)
        }
    }
}

async fn publish_current_content(
    instance_id: &str,
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<i32> {
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    ensure_shareable_link(&metadata.link)?;
    let modpack_id = shared_modpack_id(&metadata.link);
    let (modrinth_ids, external_files) =
        collect_publish_content(instance_id, state).await?;
    tracing::debug!(
        instance_id,
        shared_instance_id,
        modpack_id = modpack_id.as_deref().unwrap_or("none"),
        modrinth_id_count = modrinth_ids.len(),
        external_file_count = external_files.len(),
        "Creating shared instance version"
    );
    let external_file_data = external_files
        .iter()
        .map(|file| ExternalFileData {
            file_name: file.file_name.clone(),
            file_type: file.file_type.clone(),
        })
        .collect::<Vec<_>>();
    let response = request_json::<InstanceVersionResponse>(
        "create_instance_version",
        Method::POST,
        &format!("/instances/{shared_instance_id}/versions"),
        Some(json!({
            "modrinth_ids": modrinth_ids,
            "external_files": external_file_data,
            "modpack_id": modpack_id,
            "game_version": metadata.applied_content_set.game_version.clone(),
            "loader": metadata.applied_content_set.loader.as_str(),
            "loader_version": metadata
                .applied_content_set
                .loader_version
                .clone()
                .unwrap_or_default(),
        })),
        state,
    )
    .await?;

    if !response.external_files.is_empty() {
        upload_external_files(
            &metadata.instance.path,
            &external_files,
            &response.external_files,
            state,
        )
        .await?;
        request_empty(
            "mark_version_files_ready",
            Method::POST,
            &format!(
                "/instances/{shared_instance_id}/versions/{}/files",
                response.version
            ),
            None,
            state,
        )
        .await?;
    } else if !response.ready {
        tracing::debug!(
            "Shared instance version {} was not ready but had no external files",
            response.version
        );
    }

    Ok(response.version)
}

async fn collect_publish_content(
    instance_id: &str,
    state: &State,
) -> crate::Result<(Vec<String>, Vec<ExternalFileCandidate>)> {
    let items = crate::state::list_content(instance_id, None, None, state)
        .await?;

    let mut modrinth_ids = Vec::new();
    let mut seen_modrinth_ids = HashSet::new();
    let mut external_files = Vec::new();
    let mut seen_external_files = HashSet::new();

    for item in items {
        if !item.enabled {
            continue;
        }

        if let Some(version) = item.version {
            if seen_modrinth_ids.insert(version.id.clone()) {
                modrinth_ids.push(version.id);
            }
            continue;
        }

        if item.file_path.is_empty() {
            continue;
        }

        let file_type = file_type(item.project_type);
        let external_key = format!("{}:{file_type}", item.file_path);
        if seen_external_files.insert(external_key) {
            external_files.push(ExternalFileCandidate {
                file_name: item.file_name,
                file_type,
                file_path: item.file_path,
            });
        }
    }

    Ok((modrinth_ids, external_files))
}

fn shared_modpack_id(link: &InstanceLink) -> Option<String> {
    match link {
        InstanceLink::ModrinthModpack { version_id, .. } => {
            Some(version_id.clone())
        }
        InstanceLink::ServerProjectModpack {
            content_version_id,
            ..
        } => Some(content_version_id.clone()),
        _ => None,
    }
}

async fn ensure_shareable_instance(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    ensure_shareable_link(&metadata.link)
}

fn ensure_shareable_link(link: &InstanceLink) -> crate::Result<()> {
    if matches!(link, InstanceLink::ImportedModpack { .. }) {
        return Err(crate::ErrorKind::InputError(
            "You must unlink this modpack to share your instance".to_string(),
        )
        .into());
    }

    Ok(())
}

async fn upload_external_files(
    instance_path: &str,
    candidates: &[ExternalFileCandidate],
    uploads: &[ExternalFileResponse],
    state: &State,
) -> crate::Result<()> {
    for upload in uploads {
        let candidate = candidates
            .iter()
            .find(|candidate| {
                candidate.file_name == upload.file_name
                    && candidate.file_type == upload.file_type
            })
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Shared instance service requested unknown external file {}",
                    upload.file_name
                ))
            })?;
        let path = state
            .directories
            .instances_dir()
            .join(instance_path)
            .join(&candidate.file_path);
        let bytes = crate::util::io::read(path).await?;
        let response = REQWEST_CLIENT.put(&upload.url).body(bytes).send().await?;

        if !response.status().is_success() {
            return Err(crate::ErrorKind::OtherError(format!(
                "External file upload failed with status {}",
                response.status()
            ))
            .into());
        }
    }

    Ok(())
}

async fn shared_attachment(
    instance_id: &str,
    state: &State,
) -> crate::Result<Option<SharedInstanceAttachment>> {
    Ok(crate::state::get_instance(instance_id, &state.pool)
        .await?
        .and_then(|metadata| metadata.shared_instance))
}

fn ensure_owner(attachment: &SharedInstanceAttachment) -> crate::Result<()> {
    if attachment.role == SharedInstanceRole::Owner {
        return Ok(());
    }

    Err(crate::ErrorKind::InputError(
        "Only the owner instance can manage shared instance users".to_string(),
    )
    .into())
}

fn file_type(project_type: ProjectType) -> String {
    project_type.get_name().to_string()
}

async fn create_remote_instance(
    state: &State,
) -> crate::Result<CreateInstanceResponse> {
    request_json("create_instance", Method::POST, "/instances", None, state)
        .await
}

async fn delete_remote_instance(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    request_empty(
        "delete_instance",
        Method::DELETE,
        &format!("/instances/{shared_instance_id}"),
        None,
        state,
    )
    .await
}

async fn get_remote_users(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<SharedInstanceUsers> {
    request_json(
        "get_instance_users",
        Method::GET,
        &format!("/instances/{shared_instance_id}/users"),
        None,
        state,
    )
    .await
}

async fn add_remote_users(
    shared_instance_id: &str,
    user_ids: Vec<String>,
    state: &State,
) -> crate::Result<()> {
    request_empty(
        "add_instance_users",
        Method::POST,
        &format!("/instances/{shared_instance_id}/users"),
        Some(json!({ "user_ids": user_ids })),
        state,
    )
    .await
}

async fn remove_remote_users(
    shared_instance_id: &str,
    user_ids: Vec<String>,
    state: &State,
) -> crate::Result<()> {
    request_empty(
        "remove_instance_users",
        Method::DELETE,
        &format!("/instances/{shared_instance_id}/users"),
        Some(json!({ "user_ids": user_ids })),
        state,
    )
    .await
}

async fn request_json<T>(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let response = request(operation, method, path, body, state).await?;
    let status = response.status();
    let body = response.text().await?;
    tracing::debug!(
        operation,
        path,
        status = status.as_u16(),
        response_body = %body,
        "Decoding shared instances API response"
    );

    serde_json::from_str::<T>(&body).map_err(|error| {
        crate::ErrorKind::OtherError(format!(
            "Shared instances API request {operation} {path} failed to decode JSON response with status {status}: {error}; body: {body}"
        ))
        .into()
    })
}

async fn request_empty(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
) -> crate::Result<()> {
    request(operation, method, path, body, state).await?;
    Ok(())
}

async fn request(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
) -> crate::Result<reqwest::Response> {
    let credentials =
        ModrinthCredentials::get_and_refresh(&state.pool, &state.api_semaphore)
            .await?
            .ok_or(crate::ErrorKind::NoCredentialsError)?;
    let _permit = state.api_semaphore.0.acquire().await?;
    let base_url = service_base_url();
    let url = service_url(base_url, path);
    tracing::debug!(
        operation,
        method = method.as_str(),
        path,
        url = %url,
        user_id = %credentials.user_id,
        has_body = body.is_some(),
        "Sending shared instances API request"
    );
    let mut request = shared_instances_client(base_url)
        .request(method.clone(), &url)
        .bearer_auth(credentials.session);

    if let Some(body) = body {
        request = request.json(&body);
    }

    let response = request.send().await?;
    if response.status().is_success() {
        tracing::debug!(
            operation,
            method = method.as_str(),
            path,
            url = %url,
            status = response.status().as_u16(),
            "Shared instances API request succeeded"
        );
        return Ok(response);
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    tracing::warn!(
        operation,
        method = method.as_str(),
        path,
        url = %url,
        user_id = %credentials.user_id,
        status = status.as_u16(),
        response_body = %body,
        "Shared instances API request failed"
    );
    Err(crate::ErrorKind::OtherError(format!(
        "Shared instances API request {operation} {method} {path} failed with status {status}: {body}"
    ))
    .into())
}

fn service_url(base_url: &str, path: &str) -> String {
    format!("{base_url}/v1{path}")
}

fn service_base_url() -> &'static str {
    env!("SHARED_INSTANCES_API_BASE_URL").trim_end_matches('/')
}

fn shared_instances_client(base_url: &str) -> &'static reqwest::Client {
    if base_url.starts_with("https://") {
        &REQWEST_CLIENT
    } else {
        &INSECURE_REQWEST_CLIENT
    }
}

use super::install::*;
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum SharedInstancesRequestAuth {
    ModrinthSession,
    None,
}

impl SharedInstancesRequestAuth {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::ModrinthSession => "modrinth_session",
            Self::None => "none",
        }
    }
}

impl SharedInstanceUnavailableReason {
    pub(super) fn from_status(status: StatusCode) -> Option<Self> {
        match status {
            StatusCode::NOT_FOUND => Some(Self::Deleted),
            StatusCode::UNAUTHORIZED => Some(Self::AccessRevoked),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(super) enum SharedInstanceRemoteResponse<T> {
    Available(T),
    Unavailable(SharedInstanceUnavailableReason),
}

#[derive(Clone, Debug)]
pub(super) struct ExternalFileCandidate {
    pub(super) file_name: String,
    pub(super) file_type: String,
    pub(super) source: ExternalFileSource,
}

#[derive(Clone, Debug)]
pub(super) enum ExternalFileSource {
    InstanceFile(String),
    ConfigBundle(Vec<u8>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct ConfigFile {
    pub(super) path: String,
    pub(super) hash: String,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ExternalFileData {
    pub(super) file_name: String,
    pub(super) file_type: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct CreateInstanceResponse {
    #[serde(alias = "instance_id")]
    pub(super) id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct CreateInstanceInviteResponse {
    pub(super) id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct InstanceInviteInfoResponse {
    pub(super) instance_id: String,
    pub(super) instance_name: String,
    #[serde(default)]
    pub(super) instance_icon: Option<String>,
    #[serde(default)]
    pub(super) managers: Vec<InstanceInviteManagerResponse>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum InstanceInviteManagerResponse {
    User { id: String },
    Server { name: String, icon: Option<String> },
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct InstanceVersionResponse {
    pub(super) version: i32,
    #[serde(default)]
    pub(super) modrinth_ids: Vec<String>,
    pub(super) ready: bool,
    #[serde(default)]
    pub(super) external_files: Vec<ExternalFileResponse>,
    #[serde(default)]
    pub(super) modpack_id: Option<String>,
    pub(super) game_version: String,
    pub(super) loader: ModLoader,
    pub(super) loader_version: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct ExternalFileResponse {
    pub(super) file_name: String,
    pub(super) file_type: String,
    pub(super) url: String,
    #[serde(default)]
    pub(super) file_size: Option<i64>,
    #[serde(default)]
    pub(super) metadata: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct RemoteSharedInstanceUsers {
    #[serde(default)]
    pub(super) users: Vec<SharedInstanceUser>,
    #[serde(default)]
    pub(super) tokens: i32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum RemoteSharedInstanceUsersResponse {
    Current(RemoteSharedInstanceUsers),
    Legacy(Vec<String>),
}

impl RemoteSharedInstanceUsersResponse {
    pub(super) fn into_shared_users(self) -> SharedInstanceUsers {
        match self {
            Self::Current(response) => {
                SharedInstanceUsers::from_users(response.users, response.tokens)
            }
            Self::Legacy(user_ids) => {
                SharedInstanceUsers::from_user_ids(user_ids)
            }
        }
    }
}

pub(super) async fn create_remote_instance(
    name: String,
    state: &State,
) -> crate::Result<CreateInstanceResponse> {
    request_json(
        "create_instance",
        Method::POST,
        "/instances",
        Some(json!({ "name": name })),
        state,
    )
    .await
}

pub(super) async fn delete_remote_instance(
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

pub(super) async fn update_remote_instance(
    shared_instance_id: &str,
    name: String,
    state: &State,
) -> crate::Result<()> {
    let operation = "update_instance";
    let method = Method::PATCH;
    let path = format!("/instances/{shared_instance_id}");
    let response = send_request(
        operation,
        method.clone(),
        &path,
        Some(json!({ "name": name })),
        state,
    )
    .await?;

    if response.status().is_success() {
        return Ok(());
    }

    if response.status() == StatusCode::METHOD_NOT_ALLOWED {
        let request_id = response_request_id(&response);
        tracing::warn!(
            operation,
            method = method.as_str(),
            path,
            status = StatusCode::METHOD_NOT_ALLOWED.as_u16(),
            request_id = request_id.as_deref().unwrap_or("none"),
            "Shared instances API does not support remote instance updates; skipping name sync"
        );
        return Ok(());
    }

    shared_instances_request_error(operation, method, &path, response).await
}

pub(super) async fn get_remote_instance_access(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<SharedInstanceRemoteResponse<()>> {
    let operation = "get_instance";
    let method = Method::GET;
    let path = format!("/instances/{shared_instance_id}");
    let response =
        send_request(operation, method.clone(), &path, None, state).await?;

    if let Some(reason) =
        SharedInstanceUnavailableReason::from_status(response.status())
    {
        if reason == SharedInstanceUnavailableReason::AccessRevoked
            && !active_modrinth_session_is_valid(state).await?
        {
            return Err(crate::ErrorKind::NoCredentialsError.into());
        }

        return Ok(SharedInstanceRemoteResponse::Unavailable(reason));
    }

    if !response.status().is_success() {
        return shared_instances_request_error(
            operation, method, &path, response,
        )
        .await;
    }

    Ok(SharedInstanceRemoteResponse::Available(()))
}

pub(super) async fn update_remote_instance_icon(
    shared_instance_id: &str,
    icon_path: Option<&str>,
    state: &State,
) -> crate::Result<()> {
    let path = format!("/instances/{shared_instance_id}/icon");
    let Some(icon_path) = icon_path else {
        return request_empty(
            "delete_instance_icon",
            Method::DELETE,
            &path,
            None,
            state,
        )
        .await;
    };

    let bytes = crate::util::io::read(icon_path).await?;
    let operation = "upload_instance_icon";
    let method = Method::PUT;
    let response =
        send_bytes_request(operation, method.clone(), &path, bytes, state)
            .await?;

    if response.status().is_success() {
        return Ok(());
    }

    shared_instances_request_error(operation, method, &path, response).await
}

pub(super) async fn get_remote_users(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<SharedInstanceUsers> {
    let users = request_json::<RemoteSharedInstanceUsersResponse>(
        "get_instance_users",
        Method::GET,
        &format!("/instances/{shared_instance_id}/users"),
        None,
        state,
    )
    .await?;

    Ok(users.into_shared_users())
}

pub(super) async fn get_latest_remote_version(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<InstanceVersionResponse> {
    match get_latest_remote_version_optional_unavailable(
        shared_instance_id,
        state,
    )
    .await?
    {
        SharedInstanceRemoteResponse::Available(version) => Ok(version),
        SharedInstanceRemoteResponse::Unavailable(
            reason @ SharedInstanceUnavailableReason::AccessRevoked,
        ) => {
            if !accept_pending_remote_invite(shared_instance_id, state).await? {
                return Err(shared_instance_unavailable_error(reason));
            }

            match get_latest_remote_version_optional_unavailable(
                shared_instance_id,
                state,
            )
            .await?
            {
                SharedInstanceRemoteResponse::Available(version) => Ok(version),
                SharedInstanceRemoteResponse::Unavailable(reason) => {
                    Err(shared_instance_unavailable_error(reason))
                }
            }
        }
        SharedInstanceRemoteResponse::Unavailable(reason) => {
            Err(shared_instance_unavailable_error(reason))
        }
    }
}

pub(super) async fn get_latest_remote_version_optional_unavailable(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<SharedInstanceRemoteResponse<InstanceVersionResponse>> {
    get_latest_remote_version_optional_unavailable_with_auth(
        shared_instance_id,
        state,
        SharedInstancesRequestAuth::ModrinthSession,
    )
    .await
}

pub(super) async fn get_latest_remote_version_optional_unavailable_with_auth(
    shared_instance_id: &str,
    state: &State,
    auth: SharedInstancesRequestAuth,
) -> crate::Result<SharedInstanceRemoteResponse<InstanceVersionResponse>> {
    request_json_optional_unavailable(
        "get_latest_instance_version",
        Method::GET,
        &format!("/instances/{shared_instance_id}/versions"),
        None,
        state,
        auth,
    )
    .await
}

pub(super) async fn add_remote_users(
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

pub(super) async fn remove_remote_users(
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

pub(super) async fn accept_pending_remote_invite(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<bool> {
    let operation = "accept_pending_instance_invite";
    let method = Method::POST;
    let path = format!("/instances/{shared_instance_id}/invites/pending");
    let response =
        send_request(operation, method.clone(), &path, None, state).await?;

    match response.status() {
        StatusCode::OK | StatusCode::NO_CONTENT => Ok(true),
        StatusCode::NOT_FOUND => Ok(false),
        status if status.is_success() => Ok(true),
        _ => {
            shared_instances_request_error(operation, method, &path, response)
                .await
        }
    }
}

pub(super) async fn accept_shared_instance_invite(
    shared_instance_id: &str,
    invite_id: &str,
    state: &State,
) -> crate::Result<()> {
    let path = format!("/instances/{shared_instance_id}/invites/{invite_id}");
    let operation = "accept_instance_invite";
    let method = Method::POST;

    let response = send_request_with_auth(
        operation,
        method.clone(),
        &path,
        None,
        state,
        SharedInstancesRequestAuth::ModrinthSession,
    )
    .await?;
    if response.status().is_success() {
        return Ok(());
    }

    let status = response.status();
    let request_id = response_request_id(&response);
    let body = response.text().await.unwrap_or_default();
    if status == StatusCode::BAD_REQUEST && body.contains("already has access")
    {
        return Ok(());
    }

    tracing::warn!(
        operation,
        method = method.as_str(),
        path,
        status = status.as_u16(),
        request_id = request_id.as_deref().unwrap_or("none"),
        "Shared instances API request failed"
    );
    Err(crate::ErrorKind::OtherError(format!(
        "Shared instances API request {operation} {method} {path} failed with status {status}"
    ))
    .into())
}

pub(super) async fn get_shared_instance_invite_info(
    invite_id: &str,
    state: &State,
) -> crate::Result<InstanceInviteInfoResponse> {
    let operation = "get_instance_invite";
    let method = Method::GET;
    let path = format!("/invites/{invite_id}");
    let response = send_request_with_auth(
        operation,
        method.clone(),
        &path,
        None,
        state,
        SharedInstancesRequestAuth::None,
    )
    .await?;
    if !response.status().is_success() {
        return shared_instances_request_error(
            operation, method, &path, response,
        )
        .await;
    }

    decode_json_response(operation, method, &path, response).await
}

pub(super) async fn decline_pending_remote_invite(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    request_empty(
        "decline_pending_instance_invite",
        Method::DELETE,
        &format!("/instances/{shared_instance_id}/invites/pending"),
        None,
        state,
    )
    .await
}

pub(super) async fn request_json<T>(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let response =
        request(operation, method.clone(), path, body, state).await?;
    decode_json_response(operation, method, path, response).await
}

pub(super) async fn request_json_optional_unavailable<T>(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
    auth: SharedInstancesRequestAuth,
) -> crate::Result<SharedInstanceRemoteResponse<T>>
where
    T: DeserializeOwned,
{
    let response = send_request_with_auth(
        operation,
        method.clone(),
        path,
        body,
        state,
        auth,
    )
    .await?;
    if let Some(reason) =
        SharedInstanceUnavailableReason::from_status(response.status())
    {
        if reason == SharedInstanceUnavailableReason::AccessRevoked
            && matches!(auth, SharedInstancesRequestAuth::ModrinthSession)
            && !active_modrinth_session_is_valid(state).await?
        {
            tracing::warn!(
                operation,
                method = method.as_str(),
                path,
                status = response.status().as_u16(),
                "Shared instances API returned unauthorized while Modrinth auth is unavailable"
            );
            return Err(crate::ErrorKind::NoCredentialsError.into());
        }

        tracing::warn!(
            operation,
            method = method.as_str(),
            path,
            status = response.status().as_u16(),
            "Shared instances API resource is unavailable"
        );
        return Ok(SharedInstanceRemoteResponse::Unavailable(reason));
    }

    if !response.status().is_success() {
        return shared_instances_request_error(
            operation, method, path, response,
        )
        .await;
    }

    decode_json_response(operation, method, path, response)
        .await
        .map(SharedInstanceRemoteResponse::Available)
}

pub(super) async fn active_modrinth_session_is_valid(
    state: &State,
) -> crate::Result<bool> {
    let Some(credentials) =
        ModrinthCredentials::get_and_refresh(&state.pool, &state.api_semaphore)
            .await?
    else {
        return Ok(false);
    };

    let _permit = state.api_semaphore.0.acquire().await?;
    let response = INSECURE_REQWEST_CLIENT
        .get(concat!(env!("MODRINTH_API_URL"), "user"))
        .header("Authorization", &credentials.session)
        .send()
        .await?;

    if response.status() == StatusCode::UNAUTHORIZED {
        ModrinthCredentials::remove(&credentials.user_id, &state.pool).await?;
        return Ok(false);
    }

    if response.status().is_success() {
        return Ok(true);
    }

    let status = response.status();
    let request_id = response_request_id(&response);
    tracing::warn!(
        operation = "validate_modrinth_session",
        method = Method::GET.as_str(),
        path = "/user",
        status = status.as_u16(),
        request_id = request_id.as_deref().unwrap_or("none"),
        "Modrinth auth validation request failed"
    );
    Err(crate::ErrorKind::OtherError(format!(
        "Modrinth auth validation failed with status {status}"
    ))
    .into())
}

pub(super) async fn decode_json_response<T>(
    operation: &'static str,
    method: Method,
    path: &str,
    response: reqwest::Response,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let status = response.status();
    let request_id = response_request_id(&response);
    let body = response.text().await?;
    serde_json::from_str::<T>(&body).map_err(|error| {
        tracing::warn!(
            operation,
            method = method.as_str(),
            path,
            status = status.as_u16(),
            request_id = request_id.as_deref().unwrap_or("none"),
            error_category = ?error.classify(),
            error_line = error.line(),
            error_column = error.column(),
            "Shared instances API returned an invalid JSON response"
        );
        crate::ErrorKind::OtherError(format!(
            "Shared instances API request {operation} {method} {path} returned invalid JSON with status {status}"
        ))
        .into()
    })
}

pub(super) async fn request_empty(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
) -> crate::Result<()> {
    request(operation, method, path, body, state).await?;
    Ok(())
}

pub(super) async fn request(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
) -> crate::Result<reqwest::Response> {
    let response =
        send_request(operation, method.clone(), path, body, state).await?;
    if response.status().is_success() {
        return Ok(response);
    }

    shared_instances_request_error(operation, method, path, response).await
}

pub(super) async fn send_request(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
) -> crate::Result<reqwest::Response> {
    send_request_with_auth(
        operation,
        method,
        path,
        body,
        state,
        SharedInstancesRequestAuth::ModrinthSession,
    )
    .await
}

pub(super) async fn send_bytes_request(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Vec<u8>,
    state: &State,
) -> crate::Result<reqwest::Response> {
    let base_url = service_base_url();
    let url = service_url(base_url, path);
    send_bytes_request_to_url(operation, method, path, &url, body, state).await
}

pub(super) async fn send_bytes_request_to_url(
    operation: &'static str,
    method: Method,
    path: &str,
    url: &str,
    body: Vec<u8>,
    state: &State,
) -> crate::Result<reqwest::Response> {
    let service_origin = url::Url::parse(service_base_url())
        .map_err(|error| {
            crate::ErrorKind::OtherError(format!(
                "Invalid shared instances API base URL: {error}"
            ))
        })?
        .origin();
    let upload_origin = url::Url::parse(url)
        .map_err(|error| {
            crate::ErrorKind::OtherError(format!(
                "Invalid shared instances upload URL: {error}"
            ))
        })?
        .origin();
    if service_origin != upload_origin {
        return Err(crate::ErrorKind::OtherError(
            "Shared instances upload URL has an unexpected origin".to_string(),
        )
        .into());
    }

    let credentials =
        ModrinthCredentials::get_and_refresh(&state.pool, &state.api_semaphore)
            .await?
            .ok_or(crate::ErrorKind::NoCredentialsError)?;
    let _permit = state.api_semaphore.0.acquire().await?;

    tracing::debug!(
        operation,
        method = method.as_str(),
        path,
        url = %url,
        user_id = credentials.user_id.as_str(),
        auth = SharedInstancesRequestAuth::ModrinthSession.label(),
        has_body = true,
        "Sending shared instances API request"
    );

    let response = shared_instances_client(url)
        .request(method.clone(), url)
        .bearer_auth(credentials.session)
        .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
        .body(body)
        .send()
        .await?;

    if response.status().is_success() {
        let request_id = response_request_id(&response);
        tracing::debug!(
            operation,
            method = method.as_str(),
            path,
            url = %url,
            status = response.status().as_u16(),
            request_id = request_id.as_deref().unwrap_or("none"),
            "Shared instances API request succeeded"
        );
    }

    Ok(response)
}

pub(super) async fn send_request_with_auth(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
    auth: SharedInstancesRequestAuth,
) -> crate::Result<reqwest::Response> {
    let modrinth_credentials =
        if matches!(auth, SharedInstancesRequestAuth::ModrinthSession) {
            Some(
                ModrinthCredentials::get_and_refresh(
                    &state.pool,
                    &state.api_semaphore,
                )
                .await?
                .ok_or(crate::ErrorKind::NoCredentialsError)?,
            )
        } else {
            None
        };
    let _permit = state.api_semaphore.0.acquire().await?;
    let base_url = service_base_url();
    let url = service_url(base_url, path);
    let mut request =
        shared_instances_client(base_url).request(method.clone(), &url);
    let mut user_id = None;

    match auth {
        SharedInstancesRequestAuth::ModrinthSession => {
            let credentials = modrinth_credentials
                .expect("Modrinth session credentials were loaded");
            user_id = Some(credentials.user_id);
            request = request.bearer_auth(credentials.session);
        }
        SharedInstancesRequestAuth::None => {}
    }

    tracing::debug!(
        operation,
        method = method.as_str(),
        path,
        url = %url,
        user_id = user_id.as_deref(),
        auth = auth.label(),
        has_body = body.is_some(),
        "Sending shared instances API request"
    );

    if let Some(body) = body {
        request = request.json(&body);
    }

    let response = request.send().await?;
    if response.status().is_success() {
        let request_id = response_request_id(&response);
        tracing::debug!(
            operation,
            method = method.as_str(),
            path,
            url = %url,
            status = response.status().as_u16(),
            request_id = request_id.as_deref().unwrap_or("none"),
            "Shared instances API request succeeded"
        );
    }
    Ok(response)
}

pub(super) async fn shared_instances_request_error<T>(
    operation: &'static str,
    method: Method,
    path: &str,
    response: reqwest::Response,
) -> crate::Result<T> {
    let status = response.status();
    let request_id = response_request_id(&response);
    tracing::warn!(
        operation,
        method = method.as_str(),
        path,
        status = status.as_u16(),
        request_id = request_id.as_deref().unwrap_or("none"),
        "Shared instances API request failed"
    );
    Err(crate::ErrorKind::OtherError(format!(
        "Shared instances API request {operation} {method} {path} failed with status {status}"
    ))
    .into())
}

pub(super) fn response_request_id(
    response: &reqwest::Response,
) -> Option<String> {
    let request_id = response.headers().get("x-request-id")?.to_str().ok()?;
    if request_id.is_empty()
        || request_id.len() > 128
        || !request_id.chars().all(|character| {
            character.is_ascii_alphanumeric() || "-_.:".contains(character)
        })
    {
        return None;
    }

    Some(request_id.to_string())
}

pub(super) fn service_url(base_url: &str, path: &str) -> String {
    format!("{base_url}/v1{path}")
}

pub(super) fn service_base_url() -> &'static str {
    env!("SHARED_INSTANCES_API_BASE_URL").trim_end_matches('/')
}

pub(super) fn shared_instances_client(
    base_url: &str,
) -> &'static reqwest::Client {
    if base_url.starts_with("https://") {
        &REQWEST_CLIENT
    } else {
        &INSECURE_REQWEST_CLIENT
    }
}

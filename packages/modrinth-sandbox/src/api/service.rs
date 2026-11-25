use std::{
    collections::HashMap,
    ffi::OsStr,
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    time::Duration,
};

use parking_lot::Mutex;
use rand::{RngExt, distr::Alphanumeric};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Semaphore,
    time::Instant,
};
use tracing::{error, info, warn};
use url::Url;
use uuid::Uuid;

use crate::{
    ExposedPath, PathPermission, SandboxHandle,
    api::desktop::{OpenFileDialogArgs, OpenFileDialogFilter},
};

const DEFAULT_SESSION_JOIN: &str =
    "https://sessionserver.mojang.com/session/minecraft/join";

const MAX_CONNECTIONS: usize = 64;

const UPSTREAM_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const UPSTREAM_READ_TIMEOUT: Duration = Duration::from_secs(30);
const WRITE_TIMEOUT: Duration = Duration::from_secs(30);

/// macOS's sandbox handles file dialogs without any intervention.
const ALLOW_OPEN_FILE_DIALOG: bool = cfg!(not(target_os = "macos"));

/// This is only necessary on Linux X11.
/// On Windows, the drag_intercept.rs code intercepts the drop, grants access, and forwards it.
/// On macOS, the OS grants access before sending the drop
/// On Wayland, the game receives a FD which it can use with the xdg document portal to
/// receive access to the file through FUSE.
const ALLOW_REQUEST_ACCESS: bool = cfg!(not(any(windows, target_os = "macos")));

#[derive(Default)]
struct DiscoveredEndpoints {
    custom_session_join: Option<Url>,
    replace_authorization_header: HashMap<Arc<str>, Url>,
}

// todo(boris): idk how this will look in ffi, probably going to be a bit of a pain so feel free
// to adjust as much as you want
pub trait SandboxApiLauncher: Send + Sync {
    /// Retrieve the session token for a given account
    fn login(&self, uuid: Uuid, token: tokio::sync::oneshot::Sender<Arc<str>>);
}

/// Contains info required for the game to communicate with the sandbox api
/// Dropping this handle will result in the child being unregistered
/// Dropping all handles will result in the localhost server being stopped until the next call to start
#[derive(Clone)]
pub struct SandboxApiChild(Arc<SandboxApiChildInner>);

struct SandboxApiChildInner {
    dummy_access_token: Arc<str>,
    secret: Arc<str>,
    port: u16,
    notify: Arc<tokio::sync::Notify>,
    child_count: Arc<AtomicU32>,
    inner: Arc<Mutex<SandboxApiServiceInner>>,
}

impl std::fmt::Debug for SandboxApiChild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SandboxApiChild").finish()
    }
}

impl SandboxApiChild {
    pub fn get_dummy_access_token(&self) -> Arc<str> {
        self.0.dummy_access_token.clone()
    }

    pub fn get_secret(&self) -> Arc<str> {
        self.0.secret.clone()
    }

    pub fn get_port(&self) -> u16 {
        self.0.port
    }

    /// Arguments which make the game use the sandbox api, when running it with the
    /// sandbox agent.
    pub fn get_jvm_arguments(&self) -> Vec<String> {
        let port = self.0.port;
        let mut args = vec![
            format!("-Dmodrinth.sandboxagent.secret={}", self.0.secret),
            format!("-Dmodrinth.sandboxagent.port={port}"),
            "-Dmodrinth.sandboxagent.transform.auth=true".to_owned(),
            "-Dmodrinth.sandboxagent.transform.openuri=true".to_owned(),
        ];
        for (transform, allowed) in [
            ("dragdrop", ALLOW_REQUEST_ACCESS),
            ("filedialog", ALLOW_OPEN_FILE_DIALOG),
        ] {
            if allowed {
                args.push(format!(
                    "-Dmodrinth.sandboxagent.transform.{transform}=true"
                ));
            }
        }

        args.extend([
            "-Drealms.environment=local".to_owned(),
            "-Dminecraft.api.env=sandboxauth".to_owned(),
            // used in 26.3 (authlib 10)+
            format!("-Dminecraft.api.discovery.host=http://127.0.0.1:{port}/sandboxauth/discovery"),
            // used in 1.16 (authlib 2) - 26.2 (authlib 9)
            format!("-Dminecraft.api.session.host=http://127.0.0.1:{port}/sandboxauth/session"),
            // used in 1.16 (authlib 2) - 26.2 (authlib 9)
            format!("-Dminecraft.api.services.host=http://127.0.0.1:{port}/sandboxauth/services"),
            // used in 1.21.9 (authlib 7) - 26.2 (authlib 9)
            "-Dminecraft.api.profiles.host=https://api.mojang.com".to_owned(),
            // used in 1.16 (authlib 2) - 1.20.3 (authlib 5)
            "-Dminecraft.api.account.host=https://api.mojang.com".to_owned(),
            // used in 1.16 (authlib 2) - 1.20.2 (authlib 4)
            "-Dminecraft.api.auth.host=https://authserver.mojang.com".to_owned(),
        ]);
        args
    }

    pub fn set_sandbox_handle(&self, sandbox_handle: crate::SandboxHandle) {
        if let Some(child) = self
            .0
            .inner
            .lock()
            .children_by_secret
            .get_mut(&self.0.secret)
        {
            child.handle = Some(sandbox_handle);
        }
    }
}

impl Drop for SandboxApiChildInner {
    fn drop(&mut self) {
        {
            let mut inner = self.inner.lock();
            inner.children_by_secret.remove(&self.secret);
            inner.dummy_access_tokens.remove(&self.dummy_access_token);
        }

        self.child_count.fetch_sub(1, Ordering::SeqCst);
        self.notify.notify_one();
    }
}

#[derive(Clone)]
pub struct SandboxApiService {
    port: u16,
    notify: Arc<tokio::sync::Notify>,
    child_count: Arc<AtomicU32>,
    http_client: reqwest::Client,
    launcher_api: Arc<dyn SandboxApiLauncher>,
    inner: Arc<Mutex<SandboxApiServiceInner>>,
    connections: Arc<Semaphore>,
}

#[derive(Default, Clone, Debug)]
struct ServiceChildState {
    handle: Option<SandboxHandle>,
    disallow_request_access: Arc<AtomicBool>,
}

#[derive(Default)]
struct SandboxApiServiceInner {
    reuse_buffer: Option<Vec<u8>>,
    discovered_endpoints: DiscoveredEndpoints,
    children_by_secret: HashMap<Arc<str>, ServiceChildState>,
    dummy_access_tokens: HashMap<Arc<str>, Uuid>,
    running_server: bool,
}

impl SandboxApiService {
    pub fn new(port: u16, launcher_api: Arc<dyn SandboxApiLauncher>) -> Self {
        let http_client = reqwest::Client::builder()
            .connect_timeout(UPSTREAM_CONNECT_TIMEOUT)
            .read_timeout(UPSTREAM_READ_TIMEOUT)
            .build()
            .expect("client configuration should be valid");
        Self::new_with_client(port, launcher_api, http_client)
    }

    pub fn new_with_client(
        port: u16,
        launcher_api: Arc<dyn SandboxApiLauncher>,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            port,
            notify: Arc::default(),
            child_count: Arc::default(),
            http_client,
            launcher_api,
            inner: Arc::default(),
            connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        }
    }

    pub fn start(self, account_uuid: Uuid) -> SandboxApiChild {
        // Create dummy access token
        let mut dummy_access_token = String::from("SandboxAccessToken_");
        for c in rand::rng()
            .sample_iter(Alphanumeric)
            .take(32)
            .map(char::from)
        {
            dummy_access_token.push(c);
        }
        let dummy_access_token: Arc<str> = dummy_access_token.into();

        // Create secret
        let secret = rand::rng()
            .sample_iter(Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>();
        let secret: Arc<str> = secret.into();

        // Run localhost server if it isn't already running
        self.child_count.fetch_add(1, Ordering::SeqCst);
        tokio::task::spawn(self.clone().run_localhost_server());

        {
            let mut inner = self.inner.lock();
            inner
                .children_by_secret
                .insert(secret.clone(), ServiceChildState::default());
            inner
                .dummy_access_tokens
                .insert(dummy_access_token.clone(), account_uuid);
        }

        SandboxApiChild(Arc::new(SandboxApiChildInner {
            dummy_access_token,
            secret,
            port: self.port,
            notify: self.notify.clone(),
            child_count: self.child_count.clone(),
            inner: self.inner,
        }))
    }

    async fn run_localhost_server(self) {
        let inner = self.inner.clone();
        {
            let mut inner = inner.lock();
            if inner.running_server {
                return;
            }
            inner.running_server = true;
        }

        if let Err(err) = self.run_localhost_server_inner().await {
            error!("error running sandboxapi service: {err}");

            let mut inner = inner.lock();
            inner.running_server = false;
        }
    }

    async fn run_localhost_server_inner(self) -> std::io::Result<()> {
        let socket_addr =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), self.port);
        info!("starting sandboxapi service on {}", socket_addr);
        let listener = tokio::net::TcpListener::bind(socket_addr).await?;

        loop {
            let notify = self.notify.clone().notified_owned();
            let accept = async {
                let permit = self
                    .connections
                    .clone()
                    .acquire_owned()
                    .await
                    .expect("the semaphore is never closed");
                (listener.accept().await, permit)
            };
            let notified = tokio::select! {
                (res, permit) = accept => {
                    match res {
                        Ok((stream, _)) => {
                            let service = self.clone();
                            tokio::task::spawn(async move {
                                _ = service.handle_incoming(stream).await;
                                drop(permit);
                            });
                        }
                        Err(err) => {
                            // Errors are usually temporary, so keep the server running
                            warn!("sandboxapi service failed to accept connection: {err}");
                            tokio::time::sleep(Duration::from_millis(50)).await;
                        }
                    }
                    false
                },
                _ = notify => true,
            };

            if notified {
                let mut inner = self.inner.lock();
                if self.child_count.load(Ordering::SeqCst) == 0 {
                    // Close the listener before clearing the flag so a server started
                    // after this point doesn't fail to bind because the port is still in use
                    drop(listener);
                    inner.running_server = false;
                    return Ok(());
                }
            }
        }
    }

    async fn handle_incoming(self, stream: TcpStream) -> std::io::Result<()> {
        let mut buf = self
            .inner
            .lock()
            .reuse_buffer
            .take()
            .unwrap_or_else(|| vec![0_u8; 1024]);
        let res = self
            .clone()
            .handle_incoming_with_buffer(stream, &mut buf)
            .await;
        match &mut self.inner.lock().reuse_buffer {
            Some(old) => {
                if old.len() < buf.len() {
                    *old = buf;
                }
            }
            v => *v = Some(buf),
        }
        res
    }

    async fn handle_incoming_with_buffer(
        self,
        mut stream: TcpStream,
        buf: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        let deadline = Instant::now() + Duration::from_secs(5);

        let mut read = 0;
        'read_loop: loop {
            if read == buf.len() {
                if buf.len() >= 1048576 {
                    return Err(ErrorKind::OutOfMemory.into());
                }
                buf.resize(buf.len() * 2, 0);
            }

            let n = tokio::select! {
                n = stream.read(&mut buf[read..]) => n?,
                _ = tokio::time::sleep_until(deadline) => {
                    return Err(ErrorKind::TimedOut.into());
                }
            };
            read += n;

            if read == 0 {
                return Ok(());
            }

            let mut headers = [httparse::EMPTY_HEADER; 32];
            let mut request = httparse::Request::new(&mut headers);
            let Ok(parsed) = request.parse(&buf[..read]) else {
                return Err(ErrorKind::InvalidData.into());
            };

            let httparse::Status::Complete(body_offset) = parsed else {
                if n == 0 {
                    return Err(ErrorKind::InvalidData.into());
                } else {
                    continue;
                }
            };

            // Make sure we're read enough content according to Content-Length
            if n > 0 {
                for header in request.headers.iter() {
                    if !header.name.eq_ignore_ascii_case("Content-Length") {
                        continue;
                    }
                    let Ok(content_length_str) = str::from_utf8(header.value)
                    else {
                        break;
                    };
                    let Ok(content_length) =
                        content_length_str.parse::<usize>()
                    else {
                        break;
                    };
                    let received_length = read - body_offset;
                    if received_length < content_length {
                        continue 'read_loop;
                    }
                    break;
                }
            }

            self.handle_request(request, &buf[body_offset..read], stream)
                .await;
            return Ok(());
        }
    }

    async fn handle_request(
        self,
        request: httparse::Request<'_, '_>,
        body: &[u8],
        mut stream: TcpStream,
    ) {
        let Some(path) = request.path else {
            RequestStatus::NotFound.write(stream).await;
            return;
        };
        let (path, params) = path.split_once('?').unwrap_or((path, ""));
        let params_or_none = if params.is_empty() {
            None
        } else {
            Some(params)
        };

        // We check for this authorization header in order to prevent other applications
        // on the system from interacting with our API. This isn't strictly necessary, but
        // is good practice
        let child_state = if path.starts_with("/sandboxapi/") {
            match self.clone().check_authorization_secret(&request) {
                Ok(child_state) => Some(child_state),
                Err(error_status) => {
                    error_status.write(stream).await;
                    return;
                }
            }
        } else {
            None
        };

        if let Some(realms_path) = path.strip_prefix("/sandboxrealms/") {
            if let Ok(mut url) = Url::parse(&format!(
                "https://pc.realms.minecraft.net/{realms_path}"
            )) {
                url.set_query(params_or_none);
                self.proxy_with_replacement(
                    &request,
                    body.to_vec(),
                    url,
                    stream,
                )
                .await;
            }
            return;
        }

        match path {
            "/sandboxapi/open-uri" => {
                if request.method != Some("POST") {
                    RequestStatus::MethodNotAllowed.write(stream).await;
                    return;
                }
                let status =
                    self.handle_request_open_uri(params, child_state).await;
                status.write(stream).await;
            }
            "/sandboxapi/request-access" => {
                if !ALLOW_REQUEST_ACCESS {
                    RequestStatus::Forbidden.write(stream).await;
                    return;
                }
                if request.method != Some("GET") {
                    RequestStatus::MethodNotAllowed.write(stream).await;
                    return;
                }
                match self
                    .handle_request_request_access(params, child_state)
                    .await
                {
                    Ok(path) => {
                        write_text_response(stream, &path.to_string_lossy())
                            .await
                    }
                    Err(error_status) => error_status.write(stream).await,
                };
            }
            "/sandboxapi/open-file-dialog" => {
                if !ALLOW_OPEN_FILE_DIALOG {
                    RequestStatus::Forbidden.write(stream).await;
                    return;
                }
                if request.method != Some("GET") {
                    RequestStatus::MethodNotAllowed.write(stream).await;
                    return;
                }
                match self
                    .handle_request_open_file_dialog(params, child_state)
                    .await
                {
                    Ok(paths) => {
                        let mut response = String::new();
                        for path in paths {
                            if !response.is_empty() {
                                response.push('\n');
                            }
                            response.push_str(&path.to_string_lossy());
                        }
                        write_text_response(stream, &response).await
                    }
                    Err(error_status) => error_status.write(stream).await,
                };
            }
            "/sandboxauth/session/session/minecraft/join" => {
                // Need special handling for the join request since it puts the
                // access token in the body instead of in the Authorization header
                // like all other requests
                let Ok(mut join_request) =
                    serde_json::from_slice::<MinecraftJoinRequest>(body)
                else {
                    RequestStatus::BadRequest.write(stream).await;
                    return;
                };

                let real_uuid = self
                    .inner
                    .lock()
                    .dummy_access_tokens
                    .get(&join_request.access_token)
                    .copied();
                if let Some(real_uuid) = real_uuid {
                    if real_uuid != join_request.selected_profile {
                        RequestStatus::Unauthorized.write(stream).await;
                        return;
                    }

                    let (sender, receiver) = tokio::sync::oneshot::channel();
                    self.launcher_api.login(real_uuid, sender);
                    let Ok(real_token) = receiver.await else {
                        RequestStatus::Unauthorized.write(stream).await;
                        return;
                    };
                    join_request.access_token = real_token;
                }

                let Ok(new_body) = serde_json::to_vec(&join_request) else {
                    RequestStatus::InternalServerError.write(stream).await;
                    return;
                };

                let to = self
                    .inner
                    .lock()
                    .discovered_endpoints
                    .custom_session_join
                    .clone()
                    .or_else(|| Url::parse(DEFAULT_SESSION_JOIN).ok());

                let Some(mut to) = to else {
                    RequestStatus::InternalServerError.write(stream).await;
                    return;
                };

                to.set_query(params_or_none);
                self.proxy_with_replacement(&request, new_body, to, stream)
                    .await;
            }
            "/sandboxauth/discovery" => {
                let Ok(discovery) = self.http_client.get("https://discovery.minecraftservices.com/minecraft/client").send().await else {
                    RequestStatus::InternalServerError.write(stream).await;
                    return;
                };

                let mut discovery_result = match discovery
                    .json::<MinecraftDiscoveryResult>()
                    .await
                {
                    Ok(discovery_result) => discovery_result,
                    Err(err) => {
                        error!("failed to get discovery: {:?}", err);
                        RequestStatus::InternalServerError.write(stream).await;
                        return;
                    }
                };

                discovery_result.environment = Some("sandboxauth".into());
                for (service_name, endpoints) in
                    &mut discovery_result.discovery.services
                {
                    for (endpoint_name, endpoint) in &mut endpoints.endpoints {
                        let Some(old_uri) = &endpoint.uri else {
                            continue;
                        };
                        if old_uri.contains('{') && old_uri.contains('}') {
                            // Skip URIs with path replacements for now, currently these are only
                            // used for profile/skin lookups which are unauthenticated anyways
                            continue;
                        }
                        let Ok(old_url) = Url::parse(old_uri) else {
                            continue;
                        };
                        if !old_url.has_host() {
                            continue;
                        }

                        // Custom behaviour for session join since it puts the access token in the
                        // body instead of the Authorization header
                        if &**service_name == "session"
                            && &**endpoint_name == "join"
                        {
                            // Update custom_session_join
                            self.inner
                                .lock()
                                .discovered_endpoints
                                .custom_session_join =
                                if &**old_uri == DEFAULT_SESSION_JOIN {
                                    None
                                } else {
                                    Some(old_url)
                                };

                            // Override join with our custom endpoint
                            *endpoint = MinecraftEndpoint {
                                uri: Some(format!("http://127.0.0.1:{}/sandboxauth/session/session/minecraft/join", self.port).into()),
                                valid_uris: None,
                                other_values: HashMap::default()
                            };
                        } else {
                            // Create new endpoint string
                            let new_endpoint_path: Arc<str> = format!("/sandboxauth/discovered-service/{service_name}/{endpoint_name}").into();
                            let new_endpoint: Arc<str> = format!(
                                "http://127.0.0.1:{}{}",
                                self.port, new_endpoint_path
                            )
                            .into();

                            self.inner
                                .lock()
                                .discovered_endpoints
                                .replace_authorization_header
                                .insert(new_endpoint_path.clone(), old_url);

                            *endpoint = MinecraftEndpoint {
                                uri: Some(new_endpoint),
                                valid_uris: None,
                                other_values: HashMap::default(),
                            };
                        }
                    }
                }

                if write_json_response(&mut stream, &discovery_result)
                    .await
                    .is_err()
                {
                    RequestStatus::InternalServerError.write(stream).await;
                }
            }
            _ => {
                if let Some(session_path) =
                    path.strip_prefix("/sandboxauth/session/")
                {
                    // Proxy minecraft.api.session.host requests
                    let Ok(mut to) = Url::parse(&format!(
                        "https://sessionserver.mojang.com/{session_path}"
                    )) else {
                        RequestStatus::BadRequest.write(stream).await;
                        return;
                    };
                    to.set_query(params_or_none);
                    self.proxy_with_replacement(
                        &request,
                        body.to_vec(),
                        to,
                        stream,
                    )
                    .await;
                    return;
                } else if let Some(service_path) =
                    path.strip_prefix("/sandboxauth/services/")
                {
                    // Proxy minecraft.api.services.host requests
                    let Ok(mut to) = Url::parse(&format!(
                        "https://api.minecraftservices.com/{service_path}"
                    )) else {
                        RequestStatus::BadRequest.write(stream).await;
                        return;
                    };
                    to.set_query(params_or_none);
                    self.proxy_with_replacement(
                        &request,
                        body.to_vec(),
                        to,
                        stream,
                    )
                    .await;
                    return;
                } else if path.starts_with("/sandboxauth/discovered-service") {
                    // Arbitrary service discovered through discovery.minecraftservices.com (26.3+)
                    let to = self
                        .inner
                        .lock()
                        .discovered_endpoints
                        .replace_authorization_header
                        .get(path)
                        .cloned();
                    if let Some(mut to) = to {
                        to.set_query(params_or_none);
                        self.proxy_with_replacement(
                            &request,
                            body.to_vec(),
                            to,
                            stream,
                        )
                        .await;
                        return;
                    }
                }
                RequestStatus::NotFound.write(stream).await;
            }
        }
    }

    async fn proxy_with_replacement(
        self,
        request: &httparse::Request<'_, '_>,
        body: Vec<u8>,
        to: Url,
        mut stream: TcpStream,
    ) {
        let headers = match self.create_headers_with_replacement(request).await
        {
            Ok(headers) => headers,
            Err(status) => {
                status.write(stream).await;
                return;
            }
        };

        let method = request
            .method
            .and_then(|method| {
                reqwest::Method::from_bytes(method.as_bytes()).ok()
            })
            .unwrap_or(reqwest::Method::GET);

        let response = self
            .http_client
            .request(method.clone(), to.clone())
            .headers(headers)
            .body(body)
            .send()
            .await;

        let response = match response {
            Ok(response) => response,
            Err(err) => {
                tracing::error!(
                    "Sandbox service was unable to send request to upstream {to}: {err}"
                );
                RequestStatus::InternalServerError.write(stream).await;
                return;
            }
        };

        let status = response.status();

        let mut builder = Vec::from("HTTP/1.1 ");
        builder.extend_from_slice(response.status().as_str().as_bytes());
        if let Some(canonical_reason) = response.status().canonical_reason() {
            builder.push(b' ');
            builder.extend_from_slice(canonical_reason.as_bytes());
        }
        builder.extend_from_slice(b"\r\n");

        let no_body = status.is_informational()
            || status == reqwest::StatusCode::NO_CONTENT
            || status == reqwest::StatusCode::NOT_MODIFIED;
        let override_content_length =
            !no_body && method != reqwest::Method::HEAD;

        let connection_listed_headers: Vec<String> = response
            .headers()
            .get_all(reqwest::header::CONNECTION)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(','))
            .map(|name| name.trim().to_ascii_lowercase())
            .filter(|name| !name.is_empty())
            .collect();

        for (name, value) in response.headers() {
            let lowercase_name = name.as_str().trim().to_ascii_lowercase();
            if is_hop_by_hop_header(lowercase_name.as_str())
                || connection_listed_headers
                    .iter()
                    .any(|listed| listed == lowercase_name.as_str())
            {
                continue;
            }
            if override_content_length
                && name.as_str().eq_ignore_ascii_case("content-length")
            {
                continue;
            }
            builder.extend_from_slice(name.as_str().as_bytes());
            builder.extend_from_slice(b": ");
            builder.extend_from_slice(value.as_bytes());
            builder.extend_from_slice(b"\r\n");
        }

        let content = match response.bytes().await {
            Ok(content) => content,
            Err(err) => {
                tracing::error!(
                    "Sandbox service was unable to read upstream response from {to}: {err}"
                );
                RequestStatus::InternalServerError.write(stream).await;
                return;
            }
        };

        if override_content_length {
            builder.extend_from_slice(
                format!("Content-Length: {}\r\n", content.len()).as_bytes(),
            );
        }
        builder.extend_from_slice(b"Connection: close\r\n\r\n");
        builder.extend_from_slice(&content);

        write_response(&mut stream, &builder).await;
    }

    async fn create_headers_with_replacement(
        &self,
        request: &httparse::Request<'_, '_>,
    ) -> Result<HeaderMap, RequestStatus> {
        let mut headers = HeaderMap::new();

        for old_header in request.headers.iter() {
            if is_hop_by_hop_header(&old_header.name.to_ascii_lowercase()) {
                continue;
            }
            if old_header.name.eq_ignore_ascii_case("Host")
                || old_header.name.eq_ignore_ascii_case("Content-Length")
                || old_header.name.eq_ignore_ascii_case("Connection")
            {
                // Reqwest will automatically set these headers
                continue;
            }
            // Access token authorization header
            if old_header.name.eq_ignore_ascii_case("Authorization")
                && let Some(bearer) = old_header.value.strip_prefix(b"Bearer")
                && let Ok(bearer) = str::from_utf8(bearer)
            {
                let access_token = bearer.trim_ascii_start();
                let real_uuid = self
                    .inner
                    .lock()
                    .dummy_access_tokens
                    .get(access_token)
                    .copied();

                if let Some(real_uuid) = real_uuid {
                    let (sender, receiver) = tokio::sync::oneshot::channel();
                    self.launcher_api.login(real_uuid, sender);
                    let Ok(real_token) = receiver.await else {
                        return Err(RequestStatus::Unauthorized);
                    };

                    if let Ok(header_value) =
                        HeaderValue::from_str(&format!("Bearer {real_token}"))
                    {
                        headers.insert("Authorization", header_value);
                    }
                    continue;
                }
            }
            // Realms cookie header
            if old_header.name.eq_ignore_ascii_case("Cookie")
                && let Ok(cookie) = str::from_utf8(old_header.value)
            {
                let mut new_cookie = String::new();
                let mut first = true;
                for element in cookie.split(';') {
                    if first {
                        first = false;
                    } else {
                        new_cookie.push(';');
                    }

                    // Auth cookie is `sid=token:access:uuid`
                    if let Some((key, value)) = element.split_once('=')
                        && key == "sid"
                        && let Some(right) = value.strip_prefix("token:")
                        && let Some((token, uuid_str)) = right.split_once(':')
                        && let Ok(uuid) = Uuid::from_str(uuid_str)
                    {
                        let real_uuid = self
                            .inner
                            .lock()
                            .dummy_access_tokens
                            .get(token)
                            .copied();
                        if let Some(real_uuid) = real_uuid
                            && real_uuid == uuid
                        {
                            let (sender, receiver) =
                                tokio::sync::oneshot::channel();
                            self.launcher_api.login(real_uuid, sender);
                            let Ok(real_token) = receiver.await else {
                                return Err(RequestStatus::Unauthorized);
                            };

                            new_cookie.push_str("sid=token:");
                            new_cookie.push_str(&real_token);
                            new_cookie.push(':');
                            new_cookie.push_str(uuid_str);
                            continue;
                        }
                    }
                    new_cookie.push_str(element);
                }

                if let Ok(header_value) = HeaderValue::from_str(&new_cookie) {
                    headers.insert("Cookie", header_value);
                }
                continue;
            }
            if let Ok(header_name) = HeaderName::from_str(old_header.name)
                && let Ok(header_value) =
                    HeaderValue::from_bytes(old_header.value)
            {
                headers.append(header_name, header_value);
            }
        }

        Ok(headers)
    }

    fn check_authorization_secret(
        self,
        request: &httparse::Request<'_, '_>,
    ) -> Result<ServiceChildState, RequestStatus> {
        for header in request.headers.iter() {
            if header.name == "Authorization" {
                let Some(token) =
                    header.value.trim_ascii_start().strip_prefix(b"Bearer")
                else {
                    return Err(RequestStatus::Forbidden);
                };
                let Ok(token) = str::from_utf8(token) else {
                    return Err(RequestStatus::BadRequest);
                };
                if let Some(child) = self
                    .inner
                    .lock()
                    .children_by_secret
                    .get(token.trim_ascii_start())
                {
                    return Ok(child.clone());
                } else {
                    return Err(RequestStatus::Forbidden);
                }
            }
        }
        Err(RequestStatus::Unauthorized)
    }

    async fn handle_request_open_uri(
        self,
        params: &str,
        child_state: Option<ServiceChildState>,
    ) -> RequestStatus {
        let mut uri = None;
        for (key, value) in
            params.split("&").filter_map(|arg| arg.split_once('='))
        {
            if key == "uri" {
                uri = Some(value);
            } else {
                warn!("unrecognized option for open-uri: {key}");
            }
        }

        let Some(encoded) = uri else {
            return RequestStatus::BadRequest;
        };

        let encoded = encoded.replace("+", "%20");
        let Ok(decoded) = urlencoding::decode(&encoded) else {
            return RequestStatus::BadRequest;
        };

        let Ok(url) = url::Url::parse(&decoded) else {
            return RequestStatus::BadRequest;
        };

        if matches!(url.scheme(), "http" | "https" | "steam" | "signalrgb") {
            if url.scheme() == "steam" {
                let allowed = match url.host_str() {
                    Some("forceinputappid") => true,
                    Some("open") | Some("close") => url.path() == "/keyboard",
                    _ => false,
                };
                if !allowed {
                    return RequestStatus::Forbidden;
                }
            }
            if url.scheme() == "signalrgb"
                && !(url.host_str() == Some("effect")
                    && url.path().starts_with("/apply/"))
            {
                return RequestStatus::Forbidden;
            }

            let sandbox_handle =
                child_state.and_then(|child_state| child_state.handle);
            tokio::task::spawn(super::desktop::open_url(url, sandbox_handle));
            RequestStatus::OK
        } else if url.scheme() == "file" && url.host_str().is_none() {
            let Ok(path) = url.to_file_path() else {
                return RequestStatus::BadRequest;
            };
            let Some(sandbox_handle) =
                child_state.and_then(|child_state| child_state.handle)
            else {
                return RequestStatus::InternalServerError;
            };
            if sandbox_handle.is_path_risky_or_blocked(&path) {
                warn!(
                    path = %path.display(),
                    "refusing to open risky or blocked path"
                );
                return RequestStatus::OK;
            }
            if let Ok(path) = path.canonicalize() {
                if path.is_dir() {
                    tokio::task::spawn(super::desktop::open_dir(path));
                } else {
                    tokio::task::spawn(super::desktop::launch_file_safe(
                        path,
                        sandbox_handle,
                    ));
                }
            };
            // If the path doesn't exist, do nothing. This prevents enumeration of existing files
            RequestStatus::OK
        } else {
            RequestStatus::Forbidden
        }
    }

    /// Exposes a file which was drag-dropped on the game.
    async fn handle_request_request_access(
        self,
        params: &str,
        child_state: Option<ServiceChildState>,
    ) -> Result<PathBuf, RequestStatus> {
        let Some(child_state) = child_state else {
            return Err(RequestStatus::InternalServerError);
        };

        if child_state.disallow_request_access.load(Ordering::SeqCst) {
            return Err(RequestStatus::Unauthorized);
        }

        let Some(sandbox_handle) = child_state.handle else {
            return Err(RequestStatus::InternalServerError);
        };

        let mut path = None;
        for (key, value) in
            params.split("&").filter_map(|arg| arg.split_once('='))
        {
            if key == "path" {
                path = Some(value);
            } else {
                warn!("unrecognized option for request-access: {key}");
            }
        }

        let Some(encoded) = path else {
            return Err(RequestStatus::BadRequest);
        };

        let encoded = encoded.replace("+", "%20");
        let Ok(decoded) = urlencoding::decode(&encoded) else {
            return Err(RequestStatus::BadRequest);
        };

        let path = Path::new(&*decoded);
        info!(
            path = %path.display(),
            "got request to access dropped file"
        );
        if sandbox_handle.is_path_risky_or_blocked(path) {
            warn!(
                path = %path.display(),
                "rejecting dropped-file access because the path is risky or blocked"
            );
            return Err(RequestStatus::Unauthorized);
        }
        if !path.exists() {
            // This API should only be used for DnD file paths
            // Therefore, it should only ever be called for files that actually exist
            // If we're receiving a request for a file that doesn't exist, it's possible
            // that a malicious program is attempting to enumerate files on the computer
            // To prevent this, we return Unauthorized and block the process from further requests
            warn!(
                path = %path.display(),
                "rejecting dropped-file access because the path doesn't exist"
            );
            child_state
                .disallow_request_access
                .store(true, Ordering::SeqCst);
            return Err(RequestStatus::Unauthorized);
        }

        // We only allow certain zip file formats (resourcepacks, mods, shaders, etc.)
        let Ok(archive) =
            async_zip::tokio::read::fs::ZipFileReader::new(path).await
        else {
            warn!(
                path = %path.display(),
                "rejecting dropped-file access because the file is not a readable ZIP archive"
            );
            return Err(RequestStatus::InternalServerError);
        };

        let mut allow = false;
        let is_zip = path.extension() == Some(OsStr::new("zip"));
        for entry in archive.file().entries() {
            let Ok(filename) = entry.filename().as_str() else {
                continue;
            };
            let filename = filename.strip_prefix('/').unwrap_or(filename);
            let is_content_file = match filename {
                "pack.mcmeta" => true,        // Resourcepack/datapack
                "mcmod.info" => true,         // Old forge mod
                "fabric.mod.json" => true,    // Fabric mod
                "META-INF/mods.toml" => true, // New forge mod
                "META-INF/neoforge.mods.toml" => true, // Neoforge mod
                "META-INF/jarjar/metadata.json" => true, // Forge jarjar
                "modrinth.index.json" => true, // Mrpack
                "metaData.json" => path.extension() == Some(OsStr::new("mcpr")), // Replaymod replay
                _ => {
                    let is_shaderpack =
                        filename.starts_with("shaders/") && is_zip;
                    let is_flashback_replay =
                        filename.ends_with(".flashback") && is_zip;
                    is_shaderpack || is_flashback_replay
                }
            };
            if is_content_file {
                allow = true;
                break;
            }
        }

        if !allow {
            warn!(
                path = %path.display(),
                "rejecting dropped-file access because this content type is not allowed"
            );
            return Err(RequestStatus::Unauthorized);
        }

        info!(
            path = %path.display(),
            "dropped file passed content validation"
        );
        match sandbox_handle.expose_path(path, PathPermission::ReadOnly) {
            Ok(ExposedPath::Direct) => {
                info!(
                    path = %path.display(),
                    "granted sandbox direct access to dropped file"
                );
                Ok(path.to_path_buf())
            }
            Ok(ExposedPath::Proxied(proxied)) => {
                info!(
                    path = %path.display(),
                    exposed_path = %proxied.display(),
                    "granted sandbox proxied access to dropped file"
                );
                Ok(proxied)
            }
            Err(err) => {
                error!("unable to expose path to sandbox: {err:#}");
                Err(RequestStatus::InternalServerError)
            }
        }
    }

    async fn handle_request_open_file_dialog(
        self,
        params: &str,
        child_state: Option<ServiceChildState>,
    ) -> Result<Vec<PathBuf>, RequestStatus> {
        let Some(child_state) = child_state else {
            return Err(RequestStatus::InternalServerError);
        };

        let Some(sandbox_handle) = child_state.handle else {
            return Err(RequestStatus::InternalServerError);
        };

        let mut open_args = OpenFileDialogArgs::default();

        for (key, value) in
            params.split("&").filter_map(|arg| arg.split_once('='))
        {
            let value = value.replace("+", "%20");
            let Ok(value) = urlencoding::decode(&value) else {
                return Err(RequestStatus::BadRequest);
            };

            match key {
                "title" => {
                    open_args.title = Some(value.to_string());
                }
                "accept_label" => {
                    open_args.accept_label = Some(value.to_string());
                }
                "save" => {
                    open_args.save = value
                        .parse::<bool>()
                        .map_err(|_| RequestStatus::BadRequest)?;
                }
                "multiple" => {
                    open_args.multiple = value
                        .parse::<bool>()
                        .map_err(|_| RequestStatus::BadRequest)?;
                }
                "directory" => {
                    open_args.directory = value
                        .parse::<bool>()
                        .map_err(|_| RequestStatus::BadRequest)?;
                }
                "filter" => {
                    let (extension, friendly) =
                        value.split_once(',').unwrap_or((&*value, &*value));
                    open_args.filters.push(OpenFileDialogFilter {
                        user_friendly_name: friendly.to_string(),
                        extension: extension.to_string(),
                    });
                }
                "default_location" => {
                    let path = Path::new(&*value);
                    if sandbox_handle.is_path_risky_or_blocked(path) {
                        warn!(
                            path = %path.display(),
                            "ignoring default location because the path is risky or blocked"
                        );
                    } else {
                        open_args.default_location = Some(path.to_path_buf());
                    }
                }
                _ => {
                    warn!("unrecognized option for open-file-dialog: {key}");
                }
            }
        }

        let mut paths = match super::desktop::open_file_dialog(open_args).await
        {
            Ok(paths) => paths,
            Err(err) => {
                error!("failed to open file dialog: {err:#}");
                return Err(RequestStatus::InternalServerError);
            }
        };

        for path in &mut paths {
            if !sandbox_handle.is_path_writeable(path) {
                match sandbox_handle
                    .expose_path(path, PathPermission::ReadWrite)
                {
                    Ok(ExposedPath::Direct) => {}
                    Ok(ExposedPath::Proxied(proxied)) => {
                        *path = proxied;
                    }
                    Err(err) => {
                        error!("unable to expose path to sandbox: {err:#}");
                        return Err(RequestStatus::InternalServerError);
                    }
                }
            }
        }

        Ok(paths)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MinecraftDiscoveryResult {
    #[serde(skip_serializing_if = "skip_if_none")]
    environment: Option<Arc<str>>,
    discovery: MinecraftDiscovery,

    #[serde(flatten)]
    other_values: HashMap<Arc<str>, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MinecraftDiscovery {
    #[serde(skip_serializing_if = "skip_if_none")]
    product: Option<Arc<str>>,
    #[serde(flatten)]
    services: HashMap<Arc<str>, MinecraftEndpoints>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MinecraftEndpoints {
    endpoints: HashMap<Arc<str>, MinecraftEndpoint>,

    #[serde(flatten)]
    other_values: HashMap<Arc<str>, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MinecraftEndpoint {
    #[serde(skip_serializing_if = "skip_if_none")]
    uri: Option<Arc<str>>,
    #[serde(skip_serializing_if = "skip_if_none")]
    valid_uris: Option<Arc<[Arc<str>]>>,

    #[serde(flatten)]
    other_values: HashMap<Arc<str>, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MinecraftJoinRequest {
    access_token: Arc<str>,
    selected_profile: Uuid,
    server_id: Arc<str>,
}

fn skip_if_none<T>(value: &Option<T>) -> bool {
    value.is_none()
}

#[derive(Clone, Copy)]
enum RequestStatus {
    OK,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    InternalServerError,
}

async fn write_json_response<T: Serialize>(
    stream: &mut TcpStream,
    t: &T,
) -> serde_json::Result<()> {
    let raw = serde_json::to_string(t)?;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        raw.len(),
        raw
    );
    write_response(stream, response.as_bytes()).await;
    Ok(())
}

fn is_hop_by_hop_header(lowercase_name: &str) -> bool {
    const HOP_BY_HOP_HEADERS: &[&str] = &[
        "connection",
        "keep-alive",
        "proxy-connection",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
    ];
    HOP_BY_HOP_HEADERS.contains(&lowercase_name)
}

async fn write_text_response(mut stream: TcpStream, t: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        t.len(),
        t
    );
    write_response(&mut stream, response.as_bytes()).await;
}

/// Gives up if the client isn't reading the response.
async fn write_response(stream: &mut TcpStream, response: &[u8]) {
    _ = tokio::time::timeout(WRITE_TIMEOUT, async {
        stream.write_all(response).await?;
        stream.shutdown().await
    })
    .await;
}

impl RequestStatus {
    pub async fn write(self, mut stream: TcpStream) {
        let response = match self {
            RequestStatus::OK => {
                "HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            }
            RequestStatus::BadRequest => {
                "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            }
            RequestStatus::Unauthorized => {
                "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            }
            RequestStatus::Forbidden => {
                "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            }
            RequestStatus::NotFound => {
                "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            }
            RequestStatus::MethodNotAllowed => {
                "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            }
            RequestStatus::InternalServerError => {
                "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            }
        };
        write_response(&mut stream, response.as_bytes()).await;
    }
}

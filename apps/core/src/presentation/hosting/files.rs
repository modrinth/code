//! Kyros filesystem contracts, backed by the instance filesystem.

use std::{
    collections::{BTreeMap, HashMap},
    path::{Component, Path as FilePath, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    application::{fs_service, state::AppState},
    domain::{
        event::{Event, FsOperationKind},
        instance::{InstanceRecord, ModLoader},
    },
    presentation::{
        error::ApiError, extractors::AuthUser, handlers::properties,
        instance_path::resolve_instance_path,
    },
};

type Sessions = Arc<DashMap<String, Arc<tokio::sync::Mutex<UploadSession>>>>;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/servers/:id/modrinth/v0/fs/list", get(list))
        .route("/servers/:id/modrinth/v0/fs/download", get(download))
        .route("/servers/:id/modrinth/v0/fs/create", post(create))
        .route("/servers/:id/modrinth/v0/fs/update", put(write))
        .route("/servers/:id/modrinth/v0/fs/move", post(move_file))
        .route("/servers/:id/modrinth/v0/fs/delete", delete(remove))
        .route("/servers/:id/modrinth/v0/fs/copy", post(copy))
        .route("/servers/:id/modrinth/v0/fs/zip", post(zip))
        .route("/servers/:id/v1/worlds/:world/files/stat", post(stat))
        .route("/servers/:id/v1/worlds/:world/files/zip", post(zip_world_paths))
        .route("/servers/:id/v1/fs/unarchive", post(unarchive))
        .route("/servers/:id/v1/fs/ops/:action", post(modify_operation))
        .route("/servers/:id/v1/worlds/:world/:scope/upload-session", post(create_session).get(get_session))
        .route("/servers/:id/v1/worlds/:world/:scope/upload-session/:upload/files", post(upload_session_files))
        .route("/servers/:id/v1/worlds/:world/:scope/upload-session/:upload/finalize", post(finalize_session))
        .route("/servers/:id/v1/worlds/:world/:scope/upload-session/:upload", delete(cancel_session))
        .route("/servers/:id/v1/worlds/:world/content/upload-addon-file", post(upload_addons))
        .route("/servers/:id/v1/worlds/:world/content/upload-modpack-file", post(upload_modpack))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(Extension(Sessions::default()))
}

async fn instance(
    state: &Arc<AppState>,
    id: &str,
) -> Result<InstanceRecord, ApiError> {
    resolve_instance_path(state, id).await
}

/// Check existing ancestors before a service creates directories. Refuse symlinks,
/// Windows alternate streams, and root mutations at this adapter boundary.
fn checked_path(
    record: &InstanceRecord,
    path: &str,
    allow_root: bool,
) -> Result<PathBuf, ApiError> {
    if path.contains('\\') || path.contains(':') || path.contains('\0') {
        return Err(ApiError::BadRequest("invalid file path".into()));
    }
    let relative = FilePath::new(path.trim_start_matches('/'));
    let base = std::fs::canonicalize(&record.data_dir).map_err(io_error)?;
    let mut result = base.clone();
    for component in relative.components() {
        match component {
            Component::Normal(name) => {
                let name_text = name.to_string_lossy();
                if name_text.ends_with('.') || name_text.ends_with(' ') {
                    return Err(ApiError::BadRequest(
                        "invalid file path".into(),
                    ));
                }
                result.push(name);
            }
            Component::CurDir => continue,
            _ => {
                return Err(ApiError::BadRequest(
                    "path traversal rejected".into(),
                ))
            }
        }
        match std::fs::symlink_metadata(&result) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink()
                    || !std::fs::canonicalize(&result)
                        .map_err(io_error)?
                        .starts_with(&base)
                {
                    return Err(ApiError::BadRequest(
                        "symlink paths are not supported".into(),
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io_error(error)),
        }
    }
    if !allow_root && result == base {
        return Err(ApiError::BadRequest(
            "cannot modify the server root".into(),
        ));
    }
    Ok(result)
}

fn io_error(error: std::io::Error) -> ApiError {
    match error.kind() {
        std::io::ErrorKind::NotFound => {
            ApiError::NotFound("file not found".into())
        }
        std::io::ErrorKind::AlreadyExists => {
            ApiError::Conflict("file already exists".into())
        }
        std::io::ErrorKind::PermissionDenied => {
            ApiError::Forbidden("file access denied".into())
        }
        _ => ApiError::Internal(error.to_string()),
    }
}

fn fs_error(error: fs_service::FsError) -> ApiError {
    match error {
        fs_service::FsError::Io(error) => io_error(error),
        other => other.into(),
    }
}

#[derive(Deserialize)]
struct FileQuery {
    #[serde(default)]
    path: String,
}
#[derive(Deserialize)]
struct ListQuery {
    #[serde(default)]
    path: String,
    page: Option<usize>,
    page_size: Option<usize>,
}
#[derive(Deserialize)]
struct CreateQuery {
    path: String,
    r#type: String,
}
#[derive(Deserialize)]
struct DeleteQuery {
    path: String,
    #[serde(default)]
    recursive: bool,
}
#[derive(Deserialize)]
struct MoveBody {
    source: String,
    destination: String,
}
#[derive(Deserialize)]
struct CopyBody {
    sources: Vec<String>,
    #[serde(alias = "dest")]
    destination: String,
}

fn timestamp(value: std::io::Result<SystemTime>) -> u64 {
    value
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |value| value.as_secs())
}

async fn list(
    _user: AuthUser,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Value>, ApiError> {
    let record = instance(&state, &id).await?;
    let directory = checked_path(&record, &query.path, true)?;
    let root = std::fs::canonicalize(&record.data_dir).map_err(io_error)?;
    let page = query.page.unwrap_or(1);
    let size = query.page_size.unwrap_or(100);
    if page == 0
        || size == 0
        || size > 10_000
        || page.checked_mul(size).is_none()
    {
        return Err(ApiError::BadRequest("invalid pagination".into()));
    }
    let listing = fs_service::list_directory(
        &state,
        &record.id.to_string(),
        &query.path,
        page - 1,
        size,
    )
    .await
    .map_err(fs_error)?;
    let mut items = Vec::new();
    for entry in listing.items {
        // The service may return an absolute Windows path when its canonical
        // directory has a verbatim prefix. Derive the client path from the
        // checked directory and filename instead of exposing that path.
        let entry_path = directory.join(&entry.name);
        let relative = entry_path
            .strip_prefix(&root)
            .map_err(|_| {
                ApiError::BadRequest("path traversal rejected".into())
            })?
            .to_string_lossy()
            .replace('\\', "/");
        let path = checked_path(&record, &relative, true)?;
        let metadata = tokio::fs::metadata(&path).await.map_err(io_error)?;
        let mut item = json!({ "name": entry.name, "path": format!("/{relative}"), "type": entry.r#type, "modified": timestamp(metadata.modified()), "created": timestamp(metadata.created()) });
        if metadata.is_dir() {
            let mut children =
                tokio::fs::read_dir(path).await.map_err(io_error)?;
            let mut count = 0;
            while children.next_entry().await.map_err(io_error)?.is_some() {
                count += 1;
            }
            item["count"] = json!(count);
        } else {
            item["size"] = json!(metadata.len());
        }
        items.push(item);
    }
    Ok(Json(
        json!({ "items": items, "total": listing.total, "current": page }),
    ))
}

async fn download(
    _user: AuthUser,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<FileQuery>,
) -> Result<Response, ApiError> {
    let record = instance(&state, &id).await?;
    checked_path(&record, &query.path, false)?;
    let (file, _) =
        fs_service::download_file(&state, &record.id.to_string(), &query.path)
            .await
            .map_err(fs_error)?;
    let length = file.metadata().await.map_err(io_error)?.len();
    Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, length)
        .body(Body::from_stream(tokio_util::io::ReaderStream::new(file)))
        .map_err(|error| ApiError::Internal(error.to_string()))
}

async fn create(
    _user: AuthUser,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<CreateQuery>,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let record = instance(&state, &id).await?;
    checked_path(&record, &query.path, false)?;
    match query.r#type.as_str() {
        "file" => fs_service::write_file(
            &state,
            &record.id.to_string(),
            &query.path,
            body,
        )
        .await
        .map_err(fs_error)?,
        "directory" => {
            fs_service::create_dir(&state, &record.id.to_string(), &query.path)
                .await
                .map_err(fs_error)?
        }
        _ => {
            return Err(ApiError::BadRequest(
                "type must be file or directory".into(),
            ))
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn write(
    _user: AuthUser,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<FileQuery>,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let record = instance(&state, &id).await?;
    checked_path(&record, &query.path, false)?;
    fs_service::write_file(&state, &record.id.to_string(), &query.path, body)
        .await
        .map_err(fs_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn move_file(
    _user: AuthUser,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<MoveBody>,
) -> Result<StatusCode, ApiError> {
    let record = instance(&state, &id).await?;
    checked_path(&record, &body.source, false)?;
    checked_path(&record, &body.destination, false)?;
    fs_service::move_entry(
        &state,
        &record.id.to_string(),
        &body.source,
        &body.destination,
    )
    .await
    .map_err(fs_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove(
    _user: AuthUser,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<DeleteQuery>,
) -> Result<StatusCode, ApiError> {
    let record = instance(&state, &id).await?;
    checked_path(&record, &query.path, false)?;
    fs_service::delete_entry(
        &state,
        &record.id.to_string(),
        &query.path,
        query.recursive,
    )
    .await
    .map_err(fs_error)?;
    Ok(StatusCode::NO_CONTENT)
}

fn check_tree(record: &InstanceRecord, path: &str) -> Result<(), ApiError> {
    let path = checked_path(record, path, false)?;
    for entry in walkdir::WalkDir::new(&path) {
        let entry =
            entry.map_err(|error| ApiError::BadRequest(error.to_string()))?;
        if entry.file_type().is_symlink() {
            return Err(ApiError::BadRequest(
                "symlink paths are not supported".into(),
            ));
        }
    }
    Ok(())
}

async fn copy(
    _user: AuthUser,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CopyBody>,
) -> Result<StatusCode, ApiError> {
    let record = instance(&state, &id).await?;
    let destination = checked_path(&record, &body.destination, true)?;
    for source in &body.sources {
        check_tree(&record, source)?;
        let source = checked_path(&record, source, false)?;
        if destination.starts_with(&source) {
            return Err(ApiError::BadRequest(
                "cannot copy a directory into itself".into(),
            ));
        }
        if let Some(name) = source.file_name() {
            let target = destination.join(name);
            if target.exists() {
                // fs_extra follows existing destination paths; inspect their descendants too.
                for entry in walkdir::WalkDir::new(target) {
                    if entry
                        .map_err(|error| {
                            ApiError::BadRequest(error.to_string())
                        })?
                        .file_type()
                        .is_symlink()
                    {
                        return Err(ApiError::BadRequest(
                            "symlink paths are not supported".into(),
                        ));
                    }
                }
            }
        }
    }
    fs_service::copy_files(
        &state,
        &record.id.to_string(),
        body.sources,
        &body.destination,
    )
    .await
    .map_err(fs_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn zip(
    _user: AuthUser,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CopyBody>,
) -> Result<StatusCode, ApiError> {
    let record = instance(&state, &id).await?;
    let destination = checked_path(&record, &body.destination, false)?;
    for source in &body.sources {
        check_tree(&record, source)?;
        if destination.starts_with(checked_path(&record, source, false)?) {
            return Err(ApiError::BadRequest(
                "archive cannot contain itself".into(),
            ));
        }
    }
    fs_service::zip_files(
        &state,
        &record.id.to_string(),
        body.sources,
        &body.destination,
    )
    .await
    .map_err(fs_error)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(tag = "target_type")]
enum ZipRequest {
    Directory {
        path: String,
    },
    ManyPaths {
        parent: String,
        include: Vec<String>,
        target: String,
    },
}

async fn stat(
    _user: AuthUser,
    Path((id, world)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<FileQuery>,
) -> Result<Json<Value>, ApiError> {
    let record = instance(&state, &id).await?;
    if world != record.id.to_string() {
        return Err(ApiError::NotFound("world not found".into()));
    }
    let path = checked_path(&record, &body.path, true)?;
    let metadata = tokio::fs::metadata(&path).await.map_err(io_error)?;
    let root = std::fs::canonicalize(&record.data_dir).map_err(io_error)?;
    let relative = path
        .strip_prefix(root)
        .map_err(|_| ApiError::BadRequest("path traversal rejected".into()))?
        .to_string_lossy()
        .replace('\\', "/");
    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
    let created = metadata.created().unwrap_or(modified);
    Ok(Json(json!({
        "name": path.file_name().unwrap_or_default().to_string_lossy(),
        "full_path": format!("/{relative}"),
        "size_bytes": metadata.len(),
        "type": if metadata.is_dir() { "directory" } else if metadata.is_file() { "regular" } else { "other" },
        "mtime": chrono::DateTime::<chrono::Utc>::from(modified).to_rfc3339(),
        "ctime": chrono::DateTime::<chrono::Utc>::from(created).to_rfc3339(),
    })))
}

async fn zip_world_paths(
    user: AuthUser,
    Path((id, world)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<ZipRequest>,
) -> Result<Response, ApiError> {
    let record = instance(&state, &id).await?;
    if world != record.id.to_string() {
        return Err(ApiError::NotFound("world not found".into()));
    }
    let (sources, destination) = match body {
        ZipRequest::Directory { path } => {
            let source = checked_path(&record, &path, false)?;
            if !tokio::fs::metadata(source)
                .await
                .map_err(io_error)?
                .is_dir()
            {
                return Err(ApiError::BadRequest(
                    "ZIP source must be a directory".into(),
                ));
            }
            let destination = format!("{}.zip", path.trim_end_matches('/'));
            (vec![path], destination)
        }
        ZipRequest::ManyPaths {
            parent,
            include,
            target,
        } => {
            // These are child names, not paths relative to some other directory.
            if include.is_empty()
                || include.iter().chain(std::iter::once(&target)).any(|name| {
                    name.is_empty()
                        || name == "."
                        || name == ".."
                        || name.contains('/')
                        || name.contains('\\')
                })
            {
                return Err(ApiError::BadRequest(
                    "ZIP entries must be child filenames".into(),
                ));
            }
            let parent = parent.trim_end_matches('/');
            let sources = include
                .into_iter()
                .map(|name| format!("{parent}/{name}"))
                .collect();
            (sources, format!("{parent}/{target}"))
        }
    };
    if checked_path(&record, &destination, false)?.exists() {
        return Err(ApiError::Conflict("file already exists".into()));
    }
    zip(
        user,
        Path(id),
        State(state),
        Json(CopyBody {
            sources,
            destination,
        }),
    )
    .await?;
    // The existing ZIP service reports completion, not byte-level progress.
    Response::builder()
        .header(header::CONTENT_TYPE, "application/json-seq")
        .body(Body::from("\u{001e}{\"progress\":100,\"done\":true}\n"))
        .map_err(|error| ApiError::Internal(error.to_string()))
}

#[derive(Deserialize)]
struct ArchiveQuery {
    src: String,
    #[serde(default)]
    trg: String,
    #[serde(default, rename = "override")]
    overwrite: bool,
    #[serde(default)]
    dry: bool,
}

async fn unarchive(
    _user: AuthUser,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ArchiveQuery>,
) -> Result<Json<Value>, ApiError> {
    let record = instance(&state, &id).await?;
    let source = checked_path(&record, &query.src, false)?;
    checked_path(&record, &query.trg, true)?;
    let event_id = record.id.clone();
    let event_path = query.trg.clone();
    let dry = query.dry;
    let conflicts = tokio::task::spawn_blocking(
        move || -> Result<Vec<String>, ApiError> {
            let file = std::fs::File::open(source).map_err(io_error)?;
            let mut archive = zip::ZipArchive::new(file).map_err(|error| {
                ApiError::BadRequest(format!(
                    "cannot open ZIP archive: {error}"
                ))
            })?;
            let mut conflicts = Vec::new();
            let mut paths = Vec::new();
            for index in 0..archive.len() {
                let entry = archive
                    .by_index(index)
                    .map_err(|error| ApiError::BadRequest(error.to_string()))?;
                if entry
                    .unix_mode()
                    .is_some_and(|mode| mode & 0o170000 == 0o120000)
                    || entry.enclosed_name().is_none()
                {
                    return Err(ApiError::BadRequest(
                        "unsafe archive entry".into(),
                    ));
                }
                let target = format!(
                    "{}/{}",
                    query.trg.trim_end_matches('/'),
                    entry.name()
                );
                let path = checked_path(&record, &target, false)?;
                if path.exists() && !entry.is_dir() {
                    conflicts.push(target);
                }
                paths.push(path);
            }
            if !query.dry {
                for (index, path) in paths.iter().enumerate() {
                    let mut entry =
                        archive.by_index(index).map_err(|error| {
                            ApiError::BadRequest(error.to_string())
                        })?;
                    if entry.is_dir() {
                        std::fs::create_dir_all(path).map_err(io_error)?;
                    } else if query.overwrite || !path.exists() {
                        if let Some(parent) = path.parent() {
                            std::fs::create_dir_all(parent)
                                .map_err(io_error)?;
                        }
                        let mut file =
                            std::fs::File::create(path).map_err(io_error)?;
                        std::io::copy(&mut entry, &mut file)
                            .map_err(io_error)?;
                    }
                }
            }
            Ok(conflicts)
        },
    )
    .await
    .map_err(|error| ApiError::Internal(error.to_string()))??;
    if !dry {
        state.broadcaster.send(Event::FsChanged {
            instance_id: event_id,
            operation: FsOperationKind::Unzip,
            path: event_path,
        });
    }
    Ok(Json(
        json!({ "modpack_name": null, "conflicting_files": conflicts }),
    ))
}

async fn modify_operation(
    _user: AuthUser,
    Path((id, _action)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {
    instance(&state, &id).await?;
    Err(ApiError::NotFound("filesystem operation not found; local operations finish in their request".into()))
}

#[derive(Clone, Serialize)]
struct SessionResponse {
    upload_id: String,
    status: &'static str,
    created_at: i64,
    updated_at: i64,
    last_upload_at: Option<i64>,
    expires_at: i64,
    entry_count: usize,
    uploaded_byte_count: u64,
}

struct UploadSession {
    instance_id: String,
    user_id: String,
    scope: String,
    response: SessionResponse,
    staging: Option<tempfile::TempDir>,
    files: BTreeMap<String, (PathBuf, u64)>,
}

#[derive(Deserialize)]
struct SessionPath {
    id: String,
    world: String,
    scope: String,
    upload: Option<String>,
}

async fn session_instance(
    state: &Arc<AppState>,
    path: &SessionPath,
) -> Result<InstanceRecord, ApiError> {
    let record = instance(state, &path.id).await?;
    if path.world != record.id.to_string() {
        return Err(ApiError::NotFound("world not found".into()));
    }
    if path.scope != "files" && path.scope != "content" {
        return Err(ApiError::NotFound("upload scope not found".into()));
    }
    Ok(record)
}

fn find_session(
    sessions: &Sessions,
    path: &SessionPath,
) -> Result<Arc<tokio::sync::Mutex<UploadSession>>, ApiError> {
    let id = path
        .upload
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("missing upload ID".into()))?;
    sessions
        .get(id)
        .map(|entry| entry.value().clone())
        .ok_or_else(|| ApiError::NotFound("upload session not found".into()))
}

fn check_session(
    session: &mut UploadSession,
    record: &InstanceRecord,
    user: &str,
    scope: &str,
    active: bool,
) -> Result<(), ApiError> {
    if session.instance_id != record.id.to_string()
        || session.user_id != user
        || session.scope != scope
    {
        return Err(ApiError::NotFound("upload session not found".into()));
    }
    if session.response.expires_at <= chrono::Utc::now().timestamp() {
        session.response.status = "expired";
        session.staging.take();
        session.files.clear();
    }
    if active && session.response.status != "active" {
        return Err(ApiError::Conflict(format!(
            "upload session is {}",
            session.response.status
        )));
    }
    Ok(())
}

async fn create_session(
    AuthUser(account): AuthUser,
    Path(path): Path<SessionPath>,
    State(state): State<Arc<AppState>>,
    Extension(sessions): Extension<Sessions>,
) -> Result<Json<SessionResponse>, ApiError> {
    let record = session_instance(&state, &path).await?;
    let now = chrono::Utc::now().timestamp();
    // Dropping expired sessions also removes their temporary directories.
    sessions.retain(|_, value| {
        value
            .try_lock()
            .map_or(true, |session| session.response.expires_at > now)
    });
    let response = SessionResponse {
        upload_id: Uuid::new_v4().to_string(),
        status: "active",
        created_at: now,
        updated_at: now,
        last_upload_at: None,
        expires_at: now + 3600,
        entry_count: 0,
        uploaded_byte_count: 0,
    };
    let session = UploadSession {
        instance_id: record.id.to_string(),
        user_id: account.id,
        scope: path.scope,
        response: response.clone(),
        staging: Some(tempfile::tempdir().map_err(io_error)?),
        files: BTreeMap::new(),
    };
    sessions.insert(
        response.upload_id.clone(),
        Arc::new(tokio::sync::Mutex::new(session)),
    );
    Ok(Json(response))
}

async fn get_session(
    AuthUser(account): AuthUser,
    Path(path): Path<SessionPath>,
    State(state): State<Arc<AppState>>,
    Extension(sessions): Extension<Sessions>,
) -> Result<Json<Value>, ApiError> {
    let record = session_instance(&state, &path).await?;
    let candidates: Vec<_> =
        sessions.iter().map(|entry| entry.value().clone()).collect();
    let mut latest: Option<SessionResponse> = None;
    for candidate in candidates {
        let mut session = candidate.lock().await;
        if session.instance_id != record.id.to_string()
            || session.user_id != account.id
            || session.scope != path.scope
        {
            continue;
        }
        check_session(&mut session, &record, &account.id, &path.scope, false)?;
        if session.response.status == "active"
            && latest.as_ref().is_none_or(|last| {
                last.created_at < session.response.created_at
            })
        {
            latest = Some(session.response.clone());
        }
    }
    Ok(Json(json!({ "session": latest })))
}

fn content_path(
    record: &InstanceRecord,
    filename: &str,
) -> Result<String, ApiError> {
    if filename.contains('/') || filename.contains('\\') {
        return Err(ApiError::BadRequest(
            "addon filename must not contain a directory".into(),
        ));
    }
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".jar") {
        let folder = if matches!(record.loader, ModLoader::Paper) {
            "plugins"
        } else {
            "mods"
        };
        Ok(format!("{folder}/{filename}"))
    } else if lower.ends_with(".zip") {
        let properties = std::fs::read_to_string(
            FilePath::new(&record.data_dir).join("server.properties"),
        )
        .unwrap_or_default();
        let world = properties
            .lines()
            .find_map(|line| line.trim().strip_prefix("level-name="))
            .unwrap_or("world");
        Ok(format!("{world}/datapacks/{filename}"))
    } else {
        Err(ApiError::BadRequest(
            "addon must be a JAR or ZIP file".into(),
        ))
    }
}

async fn stage_files(
    session: &mut UploadSession,
    record: &InstanceRecord,
    multipart: &mut Multipart,
) -> Result<(), ApiError> {
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|error| ApiError::BadRequest(error.to_string()))?
    {
        let filename = field
            .file_name()
            .ok_or_else(|| {
                ApiError::BadRequest("missing upload filename".into())
            })?
            .to_string();
        let destination = if session.scope == "content" {
            content_path(record, &filename)?
        } else {
            filename
        };
        checked_path(record, &destination, false)?;
        let staging = session.staging.as_ref().ok_or_else(|| {
            ApiError::Conflict("upload session is closed".into())
        })?;
        let staged_path = staging.path().join(Uuid::new_v4().to_string());
        let mut file = tokio::fs::File::create(&staged_path)
            .await
            .map_err(io_error)?;
        let mut length = 0;
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|error| ApiError::BadRequest(error.to_string()))?
        {
            length += chunk.len() as u64;
            file.write_all(&chunk).await.map_err(io_error)?;
        }
        file.flush().await.map_err(io_error)?;
        if let Some((old_path, _)) =
            session.files.insert(destination, (staged_path, length))
        {
            tokio::fs::remove_file(old_path).await.map_err(io_error)?;
        }
        let now = chrono::Utc::now().timestamp();
        session.response.last_upload_at = Some(now);
        session.response.updated_at = now;
        session.response.entry_count = session.files.len();
        session.response.uploaded_byte_count =
            session.files.values().map(|(_, size)| size).sum();
    }
    Ok(())
}

async fn upload_session_files(
    AuthUser(account): AuthUser,
    Path(path): Path<SessionPath>,
    State(state): State<Arc<AppState>>,
    Extension(sessions): Extension<Sessions>,
    mut multipart: Multipart,
) -> Result<Json<SessionResponse>, ApiError> {
    let record = session_instance(&state, &path).await?;
    let session = find_session(&sessions, &path)?;
    let mut session = session.lock().await;
    check_session(&mut session, &record, &account.id, &path.scope, true)?;
    stage_files(&mut session, &record, &mut multipart).await?;
    Ok(Json(session.response.clone()))
}

async fn commit_files(
    state: &Arc<AppState>,
    record: &InstanceRecord,
    session: &mut UploadSession,
) -> Result<(), ApiError> {
    for destination in session.files.keys() {
        checked_path(record, destination, false)?;
    }
    for (destination, (source, _)) in &session.files {
        let target = checked_path(record, destination, false)?;
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(io_error)?;
        }
        let parent = target.parent().ok_or_else(|| {
            ApiError::BadRequest("invalid destination".into())
        })?;
        let staged =
            tempfile::NamedTempFile::new_in(parent).map_err(io_error)?;
        tokio::fs::copy(source, staged.path())
            .await
            .map_err(io_error)?;
        staged
            .persist(&target)
            .map_err(|error| io_error(error.error))?;
        state.broadcaster.send(Event::FsChanged {
            instance_id: record.id.clone(),
            operation: FsOperationKind::Upload,
            path: destination.clone(),
        });
    }
    session.response.status = "finalized";
    session.response.updated_at = chrono::Utc::now().timestamp();
    session.staging.take();
    session.files.clear();
    Ok(())
}

async fn finalize_session(
    AuthUser(account): AuthUser,
    Path(path): Path<SessionPath>,
    State(state): State<Arc<AppState>>,
    Extension(sessions): Extension<Sessions>,
) -> Result<Json<SessionResponse>, ApiError> {
    let record = session_instance(&state, &path).await?;
    let session = find_session(&sessions, &path)?;
    let mut session = session.lock().await;
    check_session(&mut session, &record, &account.id, &path.scope, false)?;
    if session.response.status == "finalized" {
        return Ok(Json(session.response.clone()));
    }
    check_session(&mut session, &record, &account.id, &path.scope, true)?;
    commit_files(&state, &record, &mut session).await?;
    Ok(Json(session.response.clone()))
}

async fn cancel_session(
    AuthUser(account): AuthUser,
    Path(path): Path<SessionPath>,
    State(state): State<Arc<AppState>>,
    Extension(sessions): Extension<Sessions>,
) -> Result<Json<SessionResponse>, ApiError> {
    let record = session_instance(&state, &path).await?;
    let session = find_session(&sessions, &path)?;
    let mut session = session.lock().await;
    check_session(&mut session, &record, &account.id, &path.scope, false)?;
    if session.response.status == "finalized" {
        return Err(ApiError::Conflict(
            "upload session is already finalized".into(),
        ));
    }
    session.response.status = "cancelled";
    session.response.updated_at = chrono::Utc::now().timestamp();
    session.staging.take();
    session.files.clear();
    Ok(Json(session.response.clone()))
}

async fn upload_addons(
    AuthUser(account): AuthUser,
    Path((id, world)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<StatusCode, ApiError> {
    let path = SessionPath {
        id,
        world,
        scope: "content".into(),
        upload: None,
    };
    let record = session_instance(&state, &path).await?;
    let now = chrono::Utc::now().timestamp();
    let mut session = UploadSession {
        instance_id: record.id.to_string(),
        user_id: account.id,
        scope: path.scope,
        response: SessionResponse {
            upload_id: Uuid::new_v4().to_string(),
            status: "active",
            created_at: now,
            updated_at: now,
            last_upload_at: None,
            expires_at: now + 3600,
            entry_count: 0,
            uploaded_byte_count: 0,
        },
        staging: Some(tempfile::tempdir().map_err(io_error)?),
        files: BTreeMap::new(),
    };
    stage_files(&mut session, &record, &mut multipart).await?;
    commit_files(&state, &record, &mut session).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct InstanceField {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct ModpackUploadQuery {
    #[serde(default)]
    soft_override: bool,
}

async fn upload_modpack(
    _user: AuthUser,
    Path((id, world)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ModpackUploadQuery>,
    mut multipart: Multipart,
) -> Result<StatusCode, ApiError> {
    let record = resolve_instance_path(&state, &id).await?;
    if world != record.id.to_string() {
        return Err(ApiError::NotFound("world not found".into()));
    }
    if !query.soft_override && !record.game_version.is_empty() {
        return Err(ApiError::UnprocessableEntity(
            "Hard content reset is not implemented by Core".into(),
        ));
    }
    let operation_lock = state
        .instance_operation_locks
        .entry(record.id.clone())
        .or_default()
        .clone();
    let _operation_guard = operation_lock.lock().await;
    // A linked server takes an update while running; it applies when the server stops.
    let linked = crate::application::server_source_service::get_status(
        &state,
        &record.id.to_string(),
    )
    .await?
    .is_some();
    if !linked && state.instances.contains_key(&record.id) {
        return Err(ApiError::Conflict(
            "Stop the server before installing a modpack".into(),
        ));
    }
    let mut pack = None;
    let mut updates = None;
    let mut instance = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|error| ApiError::BadRequest(error.to_string()))?
    {
        match field.name() {
            Some("file") => {
                if pack.is_some() {
                    return Err(ApiError::BadRequest(
                        "send exactly one modpack file".into(),
                    ));
                }
                let temporary =
                    tempfile::NamedTempFile::new().map_err(io_error)?;
                let mut output = tokio::fs::File::create(temporary.path())
                    .await
                    .map_err(io_error)?;
                while let Some(bytes) = field
                    .chunk()
                    .await
                    .map_err(|error| ApiError::BadRequest(error.to_string()))?
                {
                    output.write_all(&bytes).await.map_err(io_error)?;
                }
                output.flush().await.map_err(io_error)?;
                pack = Some(temporary);
            }
            Some("properties") => {
                let text = field
                    .text()
                    .await
                    .map_err(|error| ApiError::BadRequest(error.to_string()))?;
                let fields: super::models::PropertiesPatch =
                    serde_json::from_str(&text).map_err(|error| {
                        ApiError::BadRequest(format!(
                            "invalid properties: {error}"
                        ))
                    })?;
                let mut values: HashMap<String, String> = fields
                    .custom
                    .into_iter()
                    .filter_map(|(key, value)| value.map(|value| (key, value)))
                    .collect();
                for (key, value) in fields.known {
                    if !super::models::KNOWN_PROPERTIES.contains(&key.as_str())
                    {
                        return Err(ApiError::BadRequest(format!(
                            "unknown property {key}"
                        )));
                    }
                    // Null in creation properties selects the Minecraft default.
                    if let Some(value) = value {
                        values.insert(key.replace('_', "-"), value);
                    }
                }
                for (key, value) in &values {
                    if key.is_empty()
                        || key.chars().any(|c| {
                            !(c.is_ascii_alphanumeric()
                                || matches!(c, '-' | '_' | '.'))
                        })
                        || value.chars().any(char::is_control)
                    {
                        return Err(ApiError::BadRequest(format!(
                            "invalid property {key}"
                        )));
                    }
                }
                updates = Some(values);
            }
            // Set by the App when the pack is an export of one of its instances.
            Some("instance") => {
                let text = field
                    .text()
                    .await
                    .map_err(|error| ApiError::BadRequest(error.to_string()))?;
                let from: InstanceField =
                    serde_json::from_str(&text).map_err(|error| {
                        ApiError::BadRequest(format!(
                            "invalid instance: {error}"
                        ))
                    })?;
                instance = Some((from.id, from.name));
            }
            _ => {
                return Err(ApiError::BadRequest(
                    "unexpected multipart field".into(),
                ))
            }
        }
    }
    let pack =
        pack.ok_or_else(|| ApiError::BadRequest("missing file field".into()))?;
    let updates = updates.ok_or_else(|| {
        ApiError::BadRequest("missing properties field".into())
    })?;
    if !updates.is_empty() {
        resolve_instance_path(&state, &id).await?;
        checked_path(&record, "server.properties", false)?;
    }
    let metadata =
        crate::infrastructure::minecraft::mrpack::extract_metadata(pack.path())
            .await
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    if metadata.game != "minecraft"
        || metadata
            .dependencies
            .get("minecraft")
            .is_none_or(|version| version.trim().is_empty())
    {
        return Err(ApiError::BadRequest(
            "Modpack must specify a Minecraft version".into(),
        ));
    }
    checked_path(&record, "mods", false)?;
    for file in &metadata.files {
        checked_path(&record, &file.path, false)?;
    }
    // The existing installer guards lexical paths and final symlinks. Check
    // override ancestors as well before it creates any directories or files.
    let archive = std::fs::File::open(pack.path()).map_err(io_error)?;
    let mut archive = zip::ZipArchive::new(archive)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;
        if let Some(path) = entry
            .name()
            .strip_prefix("overrides/")
            .or_else(|| entry.name().strip_prefix("server-overrides/"))
        {
            if !path.is_empty() {
                checked_path(&record, path, false)?;
                if entry
                    .unix_mode()
                    .is_some_and(|mode| mode & 0o170000 == 0o120000)
                {
                    return Err(ApiError::BadRequest(
                        "symlink archive entries are not supported".into(),
                    ));
                }
            }
        }
    }
    drop(archive);
    drop(_operation_guard);
    crate::application::server_source_service::link_mrpack(
        &state,
        &record.id.to_string(),
        pack.path(),
        instance,
    )
    .await?;
    if !updates.is_empty() {
        let _ = properties::patch_properties_handler(
            Path(id),
            State(state),
            Json(updates),
        )
        .await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

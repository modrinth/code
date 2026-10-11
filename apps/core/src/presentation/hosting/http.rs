//! Archon HTTP contracts backed by Core's existing management services.

#[path = "backup_queue.rs"]
mod backup_queue;

use super::models::*;
use crate::{
    application::{
        backup_service, instance_status_service, mod_service,
        server_source_service,
        state::{AppState, WsTicket},
    },
    domain::{
        event::{Event, FsOperationKind},
        instance::{
            InstanceId, InstanceInstallStatus, InstanceRecord, InstanceStatus,
            MemorySettings, ModLoader,
        },
        server_source::{
            ContentKind, ContentOrigin, ServerSource, SourceApplyState,
        },
    },
    infrastructure::minecraft::server_properties,
    presentation::{
        error::ApiError,
        extractors::{bearer_token, AuthUser},
        handlers::{backups, instance_control, instances, mods, properties},
        instance_path::resolve_instance_path,
    },
};
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    routing::{delete, get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

type ApiResult = Result<Json<Value>, ApiError>;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .merge(backup_queue::router())
        .route(
            "/servers/:id/modrinth/v0/backups/:bid/download",
            get(download_backup),
        )
        .route("/local/servers", post(create_server))
        .route("/local/servers/:id", delete(delete_server))
        .route("/modrinth/v0/servers", get(list_v0))
        .route("/modrinth/v0/servers/:id", get(get_v0))
        .route("/modrinth/v0/servers/:id/power", post(power))
        .route("/modrinth/v0/servers/:id/name", post(rename))
        .route("/modrinth/v0/servers/:id/fs", get(filesystem_auth))
        .route("/modrinth/v0/servers/:id/ws", get(websocket_auth))
        .route("/modrinth/v0/servers/:id/reinstall", post(reinstall))
        .route(
            "/modrinth/v0/servers/:id/allocations",
            get(allocations).post(unsupported),
        )
        .route(
            "/modrinth/v0/servers/:id/allocations/:port",
            axum::routing::put(unsupported).delete(unsupported),
        )
        .route("/modrinth/v0/servers/:id/subdomain", post(unsupported))
        .route(
            "/modrinth/v0/subdomains/:subdomain/isavailable",
            get(unsupported),
        )
        .route(
            "/modrinth/v0/servers/:id/startup",
            get(startup_v0).post(patch_startup_v0),
        )
        .route(
            "/modrinth/v0/servers/:id/notices/:notice/dismiss",
            post(unsupported),
        )
        .route("/v1/servers", get(list_v1))
        .route("/v1/servers/:id", get(get_v1))
        .route(
            "/v1/servers/:id/worlds/:wid/select-download",
            get(select_download),
        )
        .route("/v1/servers/:id/flows/intro", delete(end_intro))
        .route("/v1/servers/:id/worlds/:wid/onboard", post(unsupported))
        .route(
            "/v1/servers/:id/worlds/:wid/options/startup",
            get(startup_v1).patch(patch_startup_v1),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/properties",
            get(get_properties).patch(patch_properties),
        )
        .route("/v1/servers/:id/worlds/:wid/content", post(install_content))
        .route("/v1/servers/:id/worlds/:wid/content/repair", post(repair))
        .route(
            "/v1/servers/:id/worlds/:wid/content/unlink-modpack",
            post(unlink_modpack),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/addons",
            get(get_addons).post(add_addon),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/addons/install-many",
            post(add_addons),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/addons/delete",
            post(delete_addon),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/addons/delete-many",
            post(delete_addons),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/addons/disable",
            post(disable_addon),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/addons/enable",
            post(enable_addon),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/addons/disable-many",
            post(disable_addons),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/addons/enable-many",
            post(enable_addons),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/addons/update",
            get(unsupported).post(update_addon),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/addons/update-many",
            post(update_addons),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/content/modpack/update",
            get(get_modpack_update).post(update_modpack),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/content/update-game-version",
            get(unsupported).post(unsupported),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/backups",
            get(list_backups).post(create_backup),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/backups/:bid",
            get(get_backup).patch(rename_backup).delete(delete_backup),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/backups/:bid/restore",
            post(restore_backup),
        )
        .route(
            "/v1/servers/:id/worlds/:wid/backups/:bid/retry",
            post(unsupported),
        )
}

async fn unsupported(_user: AuthUser) -> ApiResult {
    Err(ApiError::UnprocessableEntity(
        "This operation is not implemented by the local Core".into(),
    ))
}

fn host(state: &AppState) -> String {
    reqwest::Url::parse(&state.config.public_url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .unwrap_or_else(|| "127.0.0.1".into())
}

fn world(id: String, wid: String) -> Result<String, ApiError> {
    if id != wid {
        return Err(ApiError::NotFound(
            "world not found on this server".into(),
        ));
    }
    Ok(id)
}

async fn record(
    state: &Arc<AppState>,
    id: &str,
) -> Result<InstanceRecord, ApiError> {
    resolve_instance_path(state, id).await
}

async fn v0(
    state: &Arc<AppState>,
    user: &AuthUser,
    r: &InstanceRecord,
) -> Result<Value, ApiError> {
    let source =
        server_source_service::get_status(state, &r.id.to_string()).await?;
    let upstream = match source.as_ref().map(|status| &status.source) {
        Some(ServerSource::Modrinth {
            project_id,
            version_id,
        }) => {
            json!({"kind": "modpack", "project_id": project_id, "version_id": version_id})
        }
        _ => Value::Null,
    };
    let count = backup_service::list_backups(state, &r.id.to_string())
        .await?
        .len();
    // Only the Core's owner reaches the hosting API, so the caller is the owner.
    let node_url = format!(
        "{}/hosting/servers/{}",
        state.config.public_url.trim_end_matches('/'),
        r.id
    );
    let mut server = server_v0(
        r,
        &user.0.id,
        -32768,
        count,
        &host(state),
        upstream,
        &node_url,
    );
    // The App lists an instance's servers from these two fields, without opening a socket.
    server["linked_instance"] = match &source {
        Some(status) => match &status.source {
            ServerSource::Instance { instance_id, name } => {
                json!({"id": instance_id, "name": name, "state": status.state, "error": status.error})
            }
            _ => Value::Null,
        },
        None => Value::Null,
    };
    server["power_state"] = json!(r.status);
    // The first install of a linked pack sets the game version when it applies. Until then the
    // server is installing, not waiting for setup.
    if let (Some(status), true) = (&source, r.game_version.is_empty()) {
        server["flows"]["intro"] = json!(false);
        if status.state != SourceApplyState::Failed {
            server["status"] = json!("installing");
        }
    }
    Ok(server)
}

async fn v1(
    state: &Arc<AppState>,
    r: &InstanceRecord,
) -> Result<Value, ApiError> {
    let rows = backup_service::list_backups(state, &r.id.to_string()).await?;
    Ok(server_v1(
        r,
        &host(state),
        &format!(
            "{}/hosting/servers/{}",
            state.config.public_url.trim_end_matches('/'),
            r.id
        ),
        rows.iter().map(backup).collect(),
    ))
}

async fn select_download(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
) -> ApiResult {
    let id = world(id, wid)?;
    record(&state, &id).await?;
    Ok(Json(json!({"method_type": "unavailable"})))
}

async fn list_v0(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<Pagination>,
) -> ApiResult {
    let records = state.instance_store.list().await?;
    let count = records.len();
    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    let offset = q.offset.unwrap_or(0);
    let mut servers = Vec::new();
    for r in records.into_iter().skip(offset).take(limit) {
        servers.push(v0(&state, &user, &r).await?);
    }
    Ok(Json(
        json!({"servers": servers, "users": {&user.0.id: {"id": user.0.id, "username": user.0.username, "avatar_url": null}},
        "pagination": {"current_page": offset / limit + 1, "page_size": limit, "total_pages": count.div_ceil(limit), "total_items": count}}),
    ))
}

async fn get_v0(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult {
    let r = record(&state, &id).await?;
    Ok(Json(v0(&state, &user, &r).await?))
}

async fn list_v1(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> ApiResult {
    let mut servers = Vec::new();
    for r in state.instance_store.list().await? {
        servers.push(v1(&state, &r).await?);
    }
    Ok(Json(json!(servers)))
}

async fn get_v1(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult {
    let r = record(&state, &id).await?;
    Ok(Json(v1(&state, &r).await?))
}

async fn create_server(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateServer>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let name = body.name.trim();
    if name.is_empty() || name.len() > 100 {
        return Err(ApiError::BadRequest(
            "name must contain 1-100 characters".into(),
        ));
    }
    let memory = body.memory_mb.unwrap_or(4096);
    if memory < 512 {
        return Err(ApiError::BadRequest(
            "memory_mb must be at least 512".into(),
        ));
    }
    let existing = state.instance_store.list().await?;
    let port = match body.port {
        Some(0) => {
            return Err(ApiError::BadRequest("port cannot be zero".into()))
        }
        Some(port) if existing.iter().any(|r| r.port == port) => {
            return Err(ApiError::Conflict("port is already assigned".into()))
        }
        Some(port) => port,
        None => (25565..=65535)
            .find(|port| !existing.iter().any(|r| r.port == *port))
            .ok_or_else(|| ApiError::Conflict("no unassigned ports".into()))?,
    };
    let id = InstanceId::new();
    let directory =
        state.config.data_dir.join("instances").join(id.to_string());
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    server_properties::write_initial_properties(&directory, port)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let now = chrono::Utc::now();
    // An empty game version means onboarding has not selected server content yet.
    let r = InstanceRecord {
        id: id.clone(),
        path: id.to_string(),
        name: name.into(),
        game_version: String::new(),
        loader: ModLoader::Vanilla,
        loader_version: None,
        port,
        memory: MemorySettings {
            min_mb: 512,
            max_mb: memory,
        },
        java_version: None,
        jvm_args: None,
        server_args: None,
        install_status: InstanceInstallStatus::Installing,
        status: InstanceStatus::Offline,
        data_dir: directory.display().to_string(),
        installation_id: None,
        total_uptime_seconds: 0,
        created_at: now,
        updated_at: now,
    };
    state.instance_store.create(&r).await?;
    if let Some(instance) = body.instance {
        server_source_service::link_awaiting_upload(
            &state,
            &id.to_string(),
            ServerSource::Instance {
                instance_id: instance.id,
                name: instance.name,
            },
        )
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    }
    state
        .broadcaster
        .send(Event::InstanceCreated { instance: r });
    Ok((StatusCode::CREATED, Json(json!({"id": id}))))
}

/// Deletes a server with its world, files, and backups. A running server is killed first.
async fn delete_server(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let r = record(&state, &id).await?;
    let lock = state
        .instance_operation_locks
        .entry(r.id.clone())
        .or_default()
        .clone();
    let _guard = lock.lock().await;
    if state.instances.contains_key(&r.id) {
        instance_status_service::kill_instance(&state, &r.id).await?;
        let deadline = Instant::now() + Duration::from_secs(30);
        while state.instances.contains_key(&r.id) {
            if Instant::now() >= deadline {
                return Err(ApiError::Conflict(
                    "The server did not stop; try again".into(),
                ));
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }
    let key = r.id.to_string();
    // Child rows (mods, backups, the linked source) go with the instance row.
    sqlx::query("DELETE FROM instances WHERE id = ?")
        .bind(&key)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    for dir in [
        std::path::PathBuf::from(&r.data_dir),
        state.config.data_dir.join("backups").join(&key),
        state.config.data_dir.join("source_staging").join(&key),
    ] {
        match tokio::fs::remove_dir_all(&dir).await {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                tracing::warn!("Could not remove {}: {e}", dir.display());
            }
            _ => {}
        }
    }
    drop(_guard);
    state.instance_operation_locks.remove(&r.id);
    state
        .broadcaster
        .send(Event::InstanceDeleted { instance_id: r.id });
    Ok(StatusCode::NO_CONTENT)
}

async fn power(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Power>,
) -> ApiResult {
    match body.action {
        PowerAction::Start => {
            instance_control::start(Path(id), State(state)).await
        }
        PowerAction::Stop => {
            instance_control::stop(Path(id), State(state)).await
        }
        PowerAction::Restart => {
            instance_control::restart(Path(id), State(state)).await
        }
        PowerAction::Kill => {
            instance_control::kill(Path(id), State(state)).await
        }
    }
}

async fn rename(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Name>,
) -> ApiResult {
    instances::patch_instance(
        Path(id),
        State(state),
        Json(instances::PatchBody {
            name: Some(body.name),
            java_version: None,
            memory: None,
            jvm_args: None,
            server_args: None,
        }),
    )
    .await
}

async fn end_intro(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let r = record(&state, &id).await?;
    if r.game_version.is_empty() {
        return Err(ApiError::Conflict(
            "Choose server content before finishing setup".into(),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn allocations(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult {
    let r = record(&state, &id).await?;
    Ok(Json(json!([{"port": r.port, "name": "Minecraft"}])))
}

/// File routes take the same account token as the rest of the hosting API, so
/// the caller's own token is handed back as the node token.
async fn filesystem_auth(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult {
    let r = record(&state, &id).await?;
    Ok(Json(
        json!({"url": format!("{}/hosting/servers/{}/modrinth/v0/fs", state.config.public_url.trim_end_matches('/'), r.id), "token": bearer_token(&headers)}),
    ))
}

async fn websocket_auth(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult {
    let r = record(&state, &id).await?;
    let token = uuid::Uuid::new_v4().to_string();
    state
        .ws_tickets
        .retain(|_, ticket| ticket.expires_at > Instant::now());
    state.ws_tickets.insert(
        token.clone(),
        WsTicket {
            instance_id: r.id.clone(),
            expires_at: Instant::now() + Duration::from_secs(60),
        },
    );
    let base = state
        .config
        .public_url
        .trim_end_matches('/')
        .replacen("http", "ws", 1);
    Ok(Json(
        json!({"url": format!("{base}/hosting/servers/{}/ws", r.id), "token": token}),
    ))
}

async fn reinstall(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(q): Query<ReinstallQuery>,
    Json(body): Json<Reinstall>,
) -> ApiResult {
    if q.hard {
        return Err(ApiError::UnprocessableEntity(
            "Hard reset is not implemented by the local Core".into(),
        ));
    }
    match body {
        Reinstall::Loader {
            loader: value,
            loader_version,
            game_version,
        } => {
            instance_control::change_version_handler(
                Path(id),
                State(state),
                Json(instance_control::ChangeVersionBody {
                    game_version,
                    loader: Some(loader(&value)?),
                    loader_version: Some(loader_version),
                }),
            )
            .await
        }
        Reinstall::Modpack {
            project_id,
            version_id,
        } => {
            let version_id = version_id.ok_or_else(|| {
                ApiError::BadRequest(
                    "version_id is required for a modpack installation".into(),
                )
            })?;
            install_modrinth_pack(&state, &id, &project_id, &version_id)
                .await?;
            Ok(Json(json!({})))
        }
    }
}

async fn startup_v0(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult {
    let Json(startup) = instances::get_startup(Path(id), State(state)).await?;
    Ok(Json(
        json!({"invocation": startup["effective_command"], "original_invocation": startup["default_command"],
        "jdk_version": startup["java_version"].as_i64().map(|v| format!("lts{v}")), "jdk_build": null}),
    ))
}

async fn startup_v1(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
) -> ApiResult {
    let Json(startup) =
        instances::get_startup(Path(world(id, wid)?), State(state)).await?;
    Ok(Json(
        json!({"java_version": startup["java_version"], "jre_vendor": null,
        "original_invocation": startup["default_command"], "startup_command": startup["effective_command"]}),
    ))
}

async fn patch_startup_v0(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> ApiResult {
    if body.get("invocation").is_some_and(|v| !v.is_null())
        || body.get("jdk_build").is_some_and(|v| !v.is_null())
    {
        return Err(ApiError::UnprocessableEntity(
            "Custom startup commands and JDK vendors are not supported by Core"
                .into(),
        ));
    }
    let version = body
        .get("jdk_version")
        .map(|v| {
            if v.is_null() {
                Ok(None)
            } else {
                v.as_str()
                    .and_then(|s| s.strip_prefix("lts"))
                    .and_then(|s| s.parse::<i64>().ok())
                    .map(Some)
                    .ok_or_else(|| {
                        ApiError::BadRequest("invalid JDK version".into())
                    })
            }
        })
        .transpose()?;
    patch_java(user, state, id, version).await
}

async fn patch_startup_v1(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> ApiResult {
    let id = world(id, wid)?;
    if body.get("startup_command").is_some_and(|v| !v.is_null())
        || body.get("jre_vendor").is_some_and(|v| !v.is_null())
    {
        return Err(ApiError::UnprocessableEntity(
            "Custom startup commands and JDK vendors are not supported by Core"
                .into(),
        ));
    }
    let version = body
        .get("java_version")
        .map(|v| {
            if v.is_null() {
                Ok(None)
            } else {
                v.as_i64().filter(|v| *v > 0).map(Some).ok_or_else(|| {
                    ApiError::BadRequest("invalid Java version".into())
                })
            }
        })
        .transpose()?;
    patch_java(user, state, id, version).await
}

async fn patch_java(
    _user: AuthUser,
    state: Arc<AppState>,
    id: String,
    version: Option<Option<i64>>,
) -> ApiResult {
    instances::patch_instance(
        Path(id),
        State(state),
        Json(instances::PatchBody {
            name: None,
            java_version: version,
            memory: None,
            jvm_args: None,
            server_args: None,
        }),
    )
    .await
}

async fn get_properties(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
) -> ApiResult {
    let Json(value) =
        properties::get_properties_handler(Path(world(id, wid)?), State(state))
            .await?;
    let mut known = serde_json::Map::new();
    let mut custom = serde_json::Map::new();
    if let Some(props) = value["properties"].as_object() {
        for (key, value) in props {
            let normalized = key.replace('-', "_");
            if KNOWN_PROPERTIES.contains(&normalized.as_str()) {
                known.insert(normalized, value.clone());
            } else {
                custom.insert(key.clone(), value.clone());
            }
        }
    }
    Ok(Json(json!({"known": known, "custom": custom})))
}

async fn patch_properties(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
    Json(body): Json<PropertiesPatch>,
) -> ApiResult {
    let id = world(id, wid)?;
    apply_properties(&state, &id, body).await?;
    get_properties(user, State(state), Path((id.clone(), id))).await
}

async fn apply_properties(
    state: &Arc<AppState>,
    id: &str,
    body: PropertiesPatch,
) -> Result<(), ApiError> {
    // Null means remove a key in Archon. Core's patch handler cannot remove keys,
    // so reject the operation instead of silently changing it to an empty value.
    if body
        .known
        .values()
        .chain(body.custom.values())
        .any(Option::is_none)
    {
        return Err(ApiError::UnprocessableEntity("Removing properties is not supported through this endpoint; edit server.properties in Files".into()));
    }
    let mut updates: HashMap<String, String> = body
        .custom
        .into_iter()
        .map(|(k, v)| (k, v.unwrap()))
        .collect();
    for (key, value) in body.known {
        if !KNOWN_PROPERTIES.contains(&key.as_str()) {
            return Err(ApiError::BadRequest(format!(
                "unknown property {key}"
            )));
        }
        updates.insert(key.replace('_', "-"), value.unwrap());
    }
    let _ = properties::patch_properties_handler(
        Path(id.into()),
        State(state.clone()),
        Json(updates),
    )
    .await?;
    Ok(())
}

async fn install_content(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
    Json(body): Json<InstallContent>,
) -> ApiResult {
    let id = world(id, wid)?;
    let r = record(&state, &id).await?;
    let props = match body {
        InstallContent::Bare {
            loader: value,
            version,
            game_version,
            soft_override,
            properties,
        } => {
            if !soft_override && !r.game_version.is_empty() {
                return Err(ApiError::UnprocessableEntity(
                    "Hard content reset is not implemented by Core".into(),
                ));
            }
            let target = loader(&value)?;
            let loader_version =
                if target == ModLoader::Vanilla || version.is_empty() {
                    None
                } else {
                    Some(version.clone())
                };
            let game_version = game_version
                .or_else(|| {
                    if target == ModLoader::Vanilla {
                        Some(version.clone())
                    } else {
                        None
                    }
                })
                .filter(|v| !v.trim().is_empty())
                .ok_or_else(|| {
                    ApiError::BadRequest("game_version is required".into())
                })?;
            let _ = instance_control::change_version_handler(
                Path(id.clone()),
                State(state.clone()),
                Json(instance_control::ChangeVersionBody {
                    game_version: Some(game_version),
                    loader: Some(target),
                    loader_version: Some(loader_version),
                }),
            )
            .await?;
            properties
        }
        InstallContent::Modpack {
            spec,
            soft_override,
            properties,
        } => {
            if !soft_override && !r.game_version.is_empty() {
                return Err(ApiError::UnprocessableEntity(
                    "Hard content reset is not implemented by Core".into(),
                ));
            }
            match spec {
                ModpackSpec::Modrinth {
                    project_id,
                    version_id,
                } => {
                    install_modrinth_pack(
                        &state,
                        &id,
                        &project_id,
                        &version_id,
                    )
                    .await?;
                }
                ModpackSpec::LocalFile { filename } => {
                    return Err(ApiError::UnprocessableEntity(format!(
                        "Install {filename} using the mrpack upload endpoint"
                    )))
                }
            }
            properties
        }
    };
    if let Some(mut props) = props {
        // Setup sends null for unset defaults such as a random world seed.
        // This is an initial install, not an instruction to remove properties.
        props.known.retain(|_, value| value.is_some());
        props.custom.retain(|_, value| value.is_some());
        apply_properties(&state, &id, props).await?;
    }
    Ok(Json(json!({})))
}

async fn repair(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
) -> ApiResult {
    instance_control::repair(Path(world(id, wid)?), State(state)).await
}

async fn install_modrinth_pack(
    state: &Arc<AppState>,
    id: &str,
    project: &str,
    version: &str,
) -> Result<(), ApiError> {
    let r = record(&state, id).await?;
    if state.instances.contains_key(&r.id) {
        return Err(ApiError::Conflict(
            "Stop the server before installing a modpack".into(),
        ));
    }
    // Installs through the linked-source engine: the first pack links the server, later packs
    // update it, removing files the previous pack provided.
    let id = r.id.to_string();
    let linked = server_source_service::get_status(state, &id).await?;
    if linked.is_some() {
        server_source_service::request_source_update(state, &id, version)
            .await?;
    } else {
        server_source_service::link(
            state,
            &id,
            ServerSource::Modrinth {
                project_id: project.to_string(),
                version_id: version.to_string(),
            },
            version,
        )
        .await?;
    }
    Ok(())
}

/// The linked pack's files stay on the server and become its own content.
async fn unlink_modpack(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
) -> ApiResult {
    let r = record(&state, &world(id, wid)?).await?;
    server_source_service::unlink(&state, &r.id.to_string()).await?;
    Ok(Json(json!({})))
}

async fn get_modpack_update(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
) -> ApiResult {
    let r = record(&state, &world(id, wid)?).await?;
    let status = server_source_service::get_status(&state, &r.id.to_string())
        .await?
        .ok_or_else(|| ApiError::NotFound("no modpack is linked".into()))?;
    Ok(Json(modpack(&status)))
}

/// Applies the newest pushed version now. A running server restarts; a stopped one stays
/// stopped. This is also the retry for a failed update.
async fn update_modpack(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
) -> ApiResult {
    let r = record(&state, &world(id, wid)?).await?;
    let status =
        server_source_service::apply_now(&state, &r.id.to_string()).await?;
    Ok(Json(modpack(&status)))
}

#[derive(serde::Deserialize, Default)]
struct AddonsQuery {
    from_modpack: Option<bool>,
}

/// Mods and datapacks on the server. `from_modpack` selects the linked pack's content or the
/// server's own additions. Client-only pack content is listed with `installed: false`.
async fn get_addons(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
    Query(query): Query<AddonsQuery>,
) -> ApiResult {
    let id = world(id, wid)?;
    let r = record(&state, &id).await?;
    let id = r.id.to_string();
    let content = server_source_service::content(&state, &id).await?;
    let mods = mod_service::list_mods(&state, &id).await?;
    let data_dir = std::path::Path::new(&r.data_dir);
    let level = server_properties::read_properties(data_dir)
        .await
        .ok()
        .and_then(|props| props.get("level-name").cloned())
        .unwrap_or_else(|| "world".into());
    let mut addons = Vec::new();
    for item in &content.items {
        let from_pack = item.origin == ContentOrigin::Source;
        if query.from_modpack.is_some_and(|wanted| wanted != from_pack) {
            continue;
        }
        // Source items are listed by their enabled name; a disabled one has the suffix on disk.
        let on_disk = if item.disabled && !item.filename.ends_with(".disabled")
        {
            format!("{}.disabled", item.filename)
        } else {
            item.filename.clone()
        };
        let path = match item.kind {
            ContentKind::Datapack => {
                data_dir.join(&level).join("datapacks").join(&on_disk)
            }
            _ => data_dir.join("mods").join(&on_disk),
        };
        let size = tokio::fs::metadata(path)
            .await
            .map(|m| m.len())
            .unwrap_or(0);
        let info = mods
            .iter()
            .find(|m| m.filename == item.filename || m.filename == on_disk);
        addons.push(addon(item, info, size));
    }
    let mut result = json!({"modloader": r.loader.to_string(), "modloader_version": r.loader_version,
        "game_version": r.game_version, "modpack": content.source.as_ref().map(modpack), "addons": addons});
    match r.install_status {
        InstanceInstallStatus::Installing if !r.game_version.is_empty() => {
            result["installing"] = json!("loader")
        }
        InstanceInstallStatus::Failed => {
            result["error"] =
                json!({"message": "Server content installation failed"})
        }
        _ => {}
    }
    match content.source.as_ref().map(|status| status.state) {
        Some(SourceApplyState::Downloading | SourceApplyState::Applying) => {
            result["installing"] = json!("modpack")
        }
        Some(SourceApplyState::Failed) => {
            result["error"] = json!({"message": content.source.as_ref().and_then(|status| status.error.clone())
                .unwrap_or_else(|| "The update failed".into())})
        }
        _ => {}
    }
    Ok(Json(result))
}

/// Content the linked pack provides is read-only on the server: the next update would put it
/// back, so it is changed on the instance instead.
async fn require_own(
    state: &Arc<AppState>,
    id: &str,
    filename: &str,
) -> Result<(), ApiError> {
    let r = record(state, id).await?;
    if server_source_service::provides(state, &r.id.to_string(), filename)
        .await?
    {
        return Err(ApiError::UnprocessableEntity(
            "This comes from the linked pack. Change it there, or unlink the server.".into(),
        ));
    }
    Ok(())
}

fn require_mod(kind: Option<&str>) -> Result<(), ApiError> {
    if kind.is_some_and(|kind| kind != "mod") {
        return Err(ApiError::UnprocessableEntity("Only mod addons are currently managed by Core; use Files for other content".into()));
    }
    Ok(())
}

async fn add_addon(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
    Json(body): Json<AddAddon>,
) -> ApiResult {
    require_mod(body.kind.as_deref())?;
    mods::add_mod_handler(
        Path(world(id, wid)?),
        State(state),
        Json(mods::AddModBody {
            project_id: Some(body.project_id),
            version_id: body.version_id,
        }),
    )
    .await
}

async fn add_addons(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(pair): Path<(String, String)>,
    Json(items): Json<Vec<AddAddon>>,
) -> ApiResult {
    for item in &items {
        require_mod(item.kind.as_deref())?;
    }
    for item in items {
        let _ = add_addon(
            AuthUser(user.0.clone()),
            State(state.clone()),
            Path(pair.clone()),
            Json(item),
        )
        .await?;
    }
    Ok(Json(json!({})))
}

async fn delete_addon(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
    Json(body): Json<AddonFile>,
) -> ApiResult {
    require_mod(Some(&body.kind))?;
    let id = world(id, wid)?;
    require_own(&state, &id, &body.filename).await?;
    mods::delete_mod_handler(Path((id, body.filename)), State(state)).await
}

async fn delete_addons(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(pair): Path<(String, String)>,
    Json(body): Json<AddonFiles>,
) -> ApiResult {
    for item in &body.items {
        require_mod(Some(&item.kind))?;
    }
    for item in body.items {
        let _ = delete_addon(
            AuthUser(user.0.clone()),
            State(state.clone()),
            Path(pair.clone()),
            Json(item),
        )
        .await?;
    }
    Ok(Json(json!({})))
}

async fn toggle(
    _user: AuthUser,
    state: Arc<AppState>,
    pair: (String, String),
    body: AddonFile,
    enabled: bool,
) -> ApiResult {
    require_mod(Some(&body.kind))?;
    let id = world(pair.0, pair.1)?;
    require_own(&state, &id, &body.filename).await?;
    mods::toggle_mod_handler(
        Path((id, body.filename)),
        State(state),
        Json(mods::ToggleBody { enabled }),
    )
    .await
}

async fn disable_addon(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(pair): Path<(String, String)>,
    Json(body): Json<AddonFile>,
) -> ApiResult {
    toggle(user, state, pair, body, false).await
}
async fn enable_addon(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(pair): Path<(String, String)>,
    Json(body): Json<AddonFile>,
) -> ApiResult {
    toggle(user, state, pair, body, true).await
}

async fn toggle_many(
    user: AuthUser,
    state: Arc<AppState>,
    pair: (String, String),
    body: AddonFiles,
    enabled: bool,
) -> ApiResult {
    for item in &body.items {
        require_mod(Some(&item.kind))?;
    }
    for item in body.items {
        let _ = toggle(
            AuthUser(user.0.clone()),
            state.clone(),
            pair.clone(),
            item,
            enabled,
        )
        .await?;
    }
    Ok(Json(json!({})))
}
async fn disable_addons(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(pair): Path<(String, String)>,
    Json(body): Json<AddonFiles>,
) -> ApiResult {
    toggle_many(user, state, pair, body, false).await
}
async fn enable_addons(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(pair): Path<(String, String)>,
    Json(body): Json<AddonFiles>,
) -> ApiResult {
    toggle_many(user, state, pair, body, true).await
}

async fn update_addon(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
    Json(body): Json<UpdateAddon>,
) -> ApiResult {
    let id = world(id, wid)?;
    let r = record(&state, &id).await?;
    let updated = mod_service::update_mod_version(
        &state,
        &r.id.to_string(),
        &body.filename,
        body.version_id.as_deref(),
    )
    .await?;
    Ok(Json(json!({"updated": updated})))
}

async fn update_addons(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(pair): Path<(String, String)>,
    Json(body): Json<UpdateAddons>,
) -> ApiResult {
    for item in body.addons {
        let _ = update_addon(
            AuthUser(user.0.clone()),
            State(state.clone()),
            Path(pair.clone()),
            Json(item),
        )
        .await?;
    }
    Ok(Json(json!({})))
}

async fn backup_rows(
    state: &Arc<AppState>,
    id: &str,
) -> Result<Vec<Value>, ApiError> {
    let r = record(&state, id).await?;
    Ok(backup_service::list_backups(state, &r.id.to_string())
        .await?
        .iter()
        .map(backup)
        .collect())
}

async fn list_backups(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
) -> ApiResult {
    Ok(Json(json!(backup_rows(&state, &world(id, wid)?).await?)))
}

async fn get_backup(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid, bid)): Path<(String, String, String)>,
) -> ApiResult {
    let rows = backup_rows(&state, &world(id, wid)?).await?;
    rows.into_iter()
        .find(|b| b["id"].as_str() == Some(&bid))
        .map(Json)
        .ok_or_else(|| ApiError::NotFound("backup not found".into()))
}

async fn create_backup(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid)): Path<(String, String)>,
    body: Json<backups::CreateBody>,
) -> ApiResult {
    let id = world(id, wid)?;
    let Json(created) =
        backups::create_handler(Path(id.clone()), State(state.clone()), body)
            .await?;
    notify_changed(&state, &id, false).await?;
    Ok(Json(json!({"id": created["id"]})))
}

async fn delete_backup(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid, bid)): Path<(String, String, String)>,
) -> ApiResult {
    let id = world(id, wid)?;
    let result =
        backups::delete_handler(Path((id.clone(), bid)), State(state.clone()))
            .await?;
    notify_changed(&state, &id, false).await?;
    Ok(result)
}

async fn rename_backup(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid, bid)): Path<(String, String, String)>,
    body: Json<backups::RenameBody>,
) -> ApiResult {
    let id = world(id, wid)?;
    let result = backups::rename_handler(
        Path((id.clone(), bid)),
        State(state.clone()),
        body,
    )
    .await?;
    notify_changed(&state, &id, false).await?;
    Ok(result)
}

async fn restore_backup(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, wid, bid)): Path<(String, String, String)>,
) -> ApiResult {
    let id = world(id, wid)?;
    let result =
        backups::restore_handler(Path((id.clone(), bid)), State(state.clone()))
            .await?;
    notify_changed(&state, &id, true).await?;
    Ok(result)
}

async fn notify_changed(
    state: &Arc<AppState>,
    id: &str,
    files: bool,
) -> Result<(), ApiError> {
    let r = resolve_instance_path(state, id).await?;
    if files {
        state.broadcaster.send(Event::FsChanged {
            instance_id: r.id.clone(),
            operation: FsOperationKind::Unzip,
            path: "/".into(),
        });
    }
    state
        .broadcaster
        .send(Event::InstanceUpdated { instance: r });
    Ok(())
}

async fn download_backup(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, bid)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let r = record(&state, &id).await?;
    // Resolve against this server's database rows before deriving any disk path.
    let rows = backup_service::list_backups(&state, &r.id.to_string()).await?;
    let row = rows
        .into_iter()
        .find(|b| b.id == bid)
        .ok_or_else(|| ApiError::NotFound("backup not found".into()))?;
    let backup_id = uuid::Uuid::parse_str(&row.id)
        .map_err(|_| ApiError::BadRequest("invalid backup ID".into()))?;
    let path = state
        .config
        .data_dir
        .join("backups")
        .join(r.id.to_string())
        .join(format!("{backup_id}.zip"));
    let file = tokio::fs::File::open(path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ApiError::NotFound("backup archive not found".into())
        } else {
            ApiError::Internal(e.to_string())
        }
    })?;
    let size = file
        .metadata()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .len();
    Response::builder()
        .header(header::CONTENT_TYPE, "application/zip")
        .header(header::CONTENT_LENGTH, size)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{backup_id}.zip\""),
        )
        .body(Body::from_stream(tokio_util::io::ReaderStream::new(file)))
        .map_err(|e| ApiError::Internal(e.to_string()))
}

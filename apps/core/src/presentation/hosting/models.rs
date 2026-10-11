//! Translations at the Hosting boundary. Core instances remain the source of truth.

use crate::{
    application::{backup_service::BackupRecord, mod_service::ModInfo},
    domain::{
        instance::{InstanceInstallStatus, InstanceRecord, ModLoader},
        server_source::{
            ContentKind, ContentOrigin, ServerContentItem, ServerSource,
            ServerSourceStatus,
        },
    },
    presentation::error::ApiError,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize, Default)]
pub struct Pagination {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Deserialize)]
pub struct CreateServer {
    pub name: String,
    pub port: Option<u16>,
    pub memory_mb: Option<u32>,
    /// The App instance this server is created from. Its pack is uploaded afterwards.
    pub instance: Option<LinkedInstanceRef>,
}

#[derive(Deserialize)]
pub struct LinkedInstanceRef {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct Power {
    pub action: PowerAction,
}

#[derive(Deserialize)]
pub enum PowerAction {
    Start,
    Stop,
    Restart,
    Kill,
}

#[derive(Deserialize)]
pub struct Name {
    pub name: String,
}

#[derive(Deserialize, Default)]
pub struct ReinstallQuery {
    #[serde(default)]
    pub hard: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Reinstall {
    Modpack {
        project_id: String,
        version_id: Option<String>,
    },
    Loader {
        loader: String,
        loader_version: Option<String>,
        game_version: Option<String>,
    },
}

#[derive(Deserialize)]
#[serde(tag = "content_variant", rename_all = "snake_case")]
pub enum InstallContent {
    Bare {
        loader: String,
        version: String,
        game_version: Option<String>,
        soft_override: bool,
        properties: Option<PropertiesPatch>,
    },
    Modpack {
        spec: ModpackSpec,
        soft_override: bool,
        properties: Option<PropertiesPatch>,
    },
}

#[derive(Deserialize)]
#[serde(tag = "platform", rename_all = "snake_case")]
pub enum ModpackSpec {
    Modrinth {
        project_id: String,
        version_id: String,
    },
    LocalFile {
        filename: String,
    },
}

#[derive(Deserialize, Default)]
pub struct PropertiesPatch {
    #[serde(default)]
    pub known: std::collections::HashMap<String, Option<String>>,
    #[serde(default)]
    pub custom: std::collections::HashMap<String, Option<String>>,
}

#[derive(Deserialize)]
pub struct AddAddon {
    pub project_id: String,
    pub version_id: Option<String>,
    pub kind: Option<String>,
}

#[derive(Deserialize)]
pub struct AddonFile {
    pub kind: String,
    pub filename: String,
}

#[derive(Deserialize)]
pub struct AddonFiles {
    pub items: Vec<AddonFile>,
}

#[derive(Deserialize)]
pub struct UpdateAddon {
    pub filename: String,
    pub version_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateAddons {
    pub addons: Vec<UpdateAddon>,
}

#[derive(Deserialize)]
pub struct DeleteBackups {
    pub backup_ids: Vec<String>,
}

pub fn loader(value: &str) -> Result<ModLoader, ApiError> {
    value
        .to_lowercase()
        .replace("neo_forge", "neoforge")
        .parse()
        .map_err(ApiError::BadRequest)
}

pub fn server_v0(
    r: &InstanceRecord,
    owner: &str,
    permissions: i64,
    backup_count: usize,
    host: &str,
    upstream: Value,
    node_url: &str,
) -> Value {
    let configured = !r.game_version.is_empty();
    let status = if !configured {
        "available"
    } else {
        match r.install_status {
            InstanceInstallStatus::Installing => "installing",
            InstanceInstallStatus::Ready => "available",
            InstanceInstallStatus::Failed => "broken",
        }
    };
    json!({
        "server_id": r.id, "name": r.name, "owner_id": owner,
        "net": {"ip": host, "port": r.port, "domain": null},
        "game": "Minecraft", "backup_quota": u32::MAX, "used_backup_quota": backup_count,
        "status": status, "suspension_reason": null,
        "loader": if configured { Some(format!("{:?}", r.loader)) } else { None },
        "loader_version": r.loader_version, "mc_version": if configured { Some(&r.game_version) } else { None },
        "upstream": upstream, "sftp_username": "", "sftp_password": "", "sftp_host": "",
        "datacenter": "local", "notices": [], "node": {"instance": node_url, "token": "local-dev"},
        "flows": {"intro": !configured}, "is_medal": false, "locked_since": null,
        "current_user_permissions": permissions,
    })
}

pub fn server_v1(
    r: &InstanceRecord,
    host: &str,
    node_url: &str,
    backups: Vec<Value>,
) -> Value {
    let content = if r.game_version.is_empty() {
        Value::Null
    } else {
        json!({
            "modloader": r.loader.to_string(), "modloader_version": r.loader_version.as_deref().unwrap_or(""),
            "game_version": r.game_version, "java_version": r.java_version,
            "invocation": null, "original_invocation": null,
        })
    };
    json!({
        "id": r.id, "name": r.name, "subdomain": host,
        "specs": {"cpu": std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
            "memory_mb": r.memory.max_mb, "storage_mb": 0, "swap_mb": 0},
        "sftp_username": "", "sftp_password": "", "tags": ["self-hosted"],
        "location": {"status": "assigned", "location_metadata": {"region": "local",
            "region_should_be_user_displayed": false, "hostname": host, "url_host": node_url, "is_decommissioned_node": false}},
        "worlds": [{"id": r.id, "name": r.name, "created_at": r.created_at,
            "is_active": true, "download_method": {"method_type": "unavailable"}, "backups": backups, "content": content,
            "readiness": {"data_synchronized_fetched": true}}],
    })
}

pub fn backup(b: &BackupRecord) -> Value {
    json!({"id": b.id, "physical_id": b.id, "name": b.name, "created_at": b.created_at,
        "automated": b.trigger == "scheduled", "status": "done", "interrupted": false,
        "ongoing": false, "locked": b.locked, "history": []})
}

/// One row of the content list. `info` is Core's record of a mod it installed from Modrinth.
/// `installed` is false for client-only pack content, which is listed but never written.
pub fn addon(
    item: &ServerContentItem,
    info: Option<&ModInfo>,
    filesize: u64,
) -> Value {
    let project_id = item
        .project_id
        .as_ref()
        .or(info.and_then(|m| m.modrinth_project_id.as_ref()));
    let version_id = item
        .version_id
        .as_ref()
        .or(info.and_then(|m| m.modrinth_version_id.as_ref()));
    let kind = match item.kind {
        ContentKind::Datapack => "datapack",
        _ => "mod",
    };
    json!({"id": info.and_then(|m| m.id.as_deref()).unwrap_or(&item.filename), "filename": item.filename,
        "filesize": filesize, "disabled": item.disabled, "kind": kind,
        "from_modpack": item.origin == ContentOrigin::Source, "installed": !item.client_only,
        "status": "installed", "pack_client_retained": false, "pack_client_depends": false,
        "has_update": null, "name": info.and_then(|m| m.display_name.as_ref()), "project_id": project_id,
        "version": version_id.map(|id| json!({"id": id, "name": info.and_then(|m| m.version_number.as_ref()),
            "environment": if item.client_only { Some("client_only") } else { None }})),
        "owner": null, "icon_url": null})
}

/// The linked pack, in the shape of Modrinth's modpack card. An instance is a local pack named
/// after the instance, with `instance_id` set; `state` and `error` report its last update.
pub fn modpack(status: &ServerSourceStatus) -> Value {
    let (spec, title, instance_id) = match &status.source {
        ServerSource::Modrinth {
            project_id,
            version_id,
        } => (
            json!({"platform": "modrinth", "project_id": project_id, "version_id": version_id}),
            None,
            None,
        ),
        ServerSource::Mrpack { filename, name } => (
            json!({"platform": "local_file", "filename": filename, "name": name, "description": null}),
            Some(name),
            None,
        ),
        ServerSource::Instance { instance_id, name } => (
            json!({"platform": "local_file", "filename": "instance.mrpack", "name": name, "description": null}),
            Some(name),
            Some(instance_id),
        ),
    };
    json!({"spec": spec, "has_update": null, "title": title, "description": null,
        "icon_url": null, "owner": null, "version_number": null,
        "date_published": null, "downloads": null, "followers": null,
        "instance_id": instance_id, "state": status.state, "error": status.error})
}

pub const KNOWN_PROPERTIES: &[&str] = &[
    "allow_cheats",
    "allow_flight",
    "difficulty",
    "enforce_whitelist",
    "force_gamemode",
    "gamemode",
    "generate_structures",
    "generator_settings",
    "hardcore",
    "level_seed",
    "level_type",
    "max_players",
    "max_tick_time",
    "motd",
    "pause_when_empty_seconds",
    "player_idle_timeout",
    "require_resource_pack",
    "resource_pack",
    "resource_pack_id",
    "resource_pack_sha1",
    "simulation_distance",
    "spawn_protection",
    "sync_chunk_writes",
    "view_distance",
    "white_list",
];

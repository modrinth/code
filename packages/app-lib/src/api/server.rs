//! Public interface for managing locally-hosted dedicated Minecraft servers.
//!
//! This downloads and runs server software (Vanilla, Paper, Purpur, Fabric) on
//! the user's own machine, alongside the existing client launcher. See
//! [`crate::state::server_instance`] for the storage model and process
//! tracking.

use crate::State;
use crate::util::fetch::{fetch, fetch_json};
use crate::util::io;
use bytes::Bytes;
use reqwest::Method;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::process::Command;

pub use crate::state::{
    RunningServerInfo, ServerInstallStage, ServerInstance, ServerSoftware,
};

const MOJANG_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// Config files the UI is allowed to read/write within a server directory.
pub const EDITABLE_CONFIG_FILES: &[&str] = &[
    "server.properties",
    "bukkit.yml",
    "spigot.yml",
    "paper-global.yml",
    "paper-world-defaults.yml",
    "purpur.yml",
    "fabric-server-launcher.properties",
];

pub async fn list() -> crate::Result<Vec<ServerInstance>> {
    ServerInstance::list().await
}

pub async fn get(id: &str) -> crate::Result<Option<ServerInstance>> {
    ServerInstance::get(id).await
}

/// Create a new (not-yet-installed) server instance and persist its metadata.
pub async fn create(
    name: &str,
    software: ServerSoftware,
    minecraft_version: &str,
) -> crate::Result<ServerInstance> {
    let name = name.trim();
    if name.is_empty() {
        return Err(crate::ErrorKind::InputError(
            "Server name cannot be empty".to_string(),
        )
        .into());
    }
    if minecraft_version.trim().is_empty() {
        return Err(crate::ErrorKind::InputError(
            "A Minecraft version must be selected".to_string(),
        )
        .into());
    }

    let state = State::get().await?;
    let id = generate_unique_id(name, &state).await?;
    let memory_mb = crate::api::jre::default_memory_max_mb();

    let mut server = ServerInstance::new(
        id,
        name.to_string(),
        software,
        minecraft_version.to_string(),
        memory_mb,
    );
    server.save().await?;

    Ok(server)
}

pub async fn remove(id: &str) -> crate::Result<()> {
    ServerInstance::remove(id).await
}

/// Download the server software and write the EULA + default properties.
pub async fn install(id: &str) -> crate::Result<ServerInstance> {
    let state = State::get().await?;
    let mut server = ServerInstance::get_or_err(id).await?;

    server.install_stage = ServerInstallStage::Installing;
    server.save().await?;

    match install_inner(&state, &mut server).await {
        Ok(()) => {
            server.install_stage = ServerInstallStage::Installed;
            server.save().await?;
            Ok(server)
        }
        Err(e) => {
            server.install_stage = ServerInstallStage::Failed;
            let _ = server.save().await;
            Err(e)
        }
    }
}

async fn install_inner(
    state: &State,
    server: &mut ServerInstance,
) -> crate::Result<()> {
    let dir = state.directories.server_dir(&server.id);
    io::create_dir_all(&dir).await?;

    let (bytes, software_version) = match server.software {
        ServerSoftware::Vanilla => {
            download_vanilla(state, &server.minecraft_version).await?
        }
        ServerSoftware::Paper => {
            download_paper(state, &server.minecraft_version).await?
        }
        ServerSoftware::Purpur => {
            download_purpur(state, &server.minecraft_version).await?
        }
        ServerSoftware::Fabric => {
            download_fabric(state, &server.minecraft_version).await?
        }
    };

    let jar_name = "server.jar";
    crate::util::fetch::write(
        &dir.join(jar_name),
        &bytes,
        &state.io_semaphore,
    )
    .await?;

    server.jar_file = Some(jar_name.to_string());
    server.software_version = software_version;

    write_eula(&dir).await?;
    write_default_properties_if_absent(&dir, &server.name).await?;

    Ok(())
}

/// Start a previously-installed server, returning its running process info.
pub async fn start(id: &str) -> crate::Result<RunningServerInfo> {
    let state = State::get().await?;
    let mut server = ServerInstance::get_or_err(id).await?;

    if server.install_stage != ServerInstallStage::Installed {
        return Err(crate::ErrorKind::InputError(
            "Server must be installed before it can be started".to_string(),
        )
        .into());
    }

    let jar = server.jar_file.clone().ok_or_else(|| {
        crate::ErrorKind::InputError(
            "Server has no installed jar".to_string(),
        )
    })?;

    let dir = state.directories.server_dir(id);
    let jar_path = dir.join(&jar);
    if !jar_path.exists() {
        return Err(crate::ErrorKind::InputError(
            "Server jar is missing; reinstall the server".to_string(),
        )
        .into());
    }

    let java_path = resolve_java(&server).await?;

    let mut command = Command::new(&java_path);
    command.current_dir(&dir);
    command.arg(format!("-Xmx{}M", server.memory_mb));
    command.arg(format!("-Xms{}M", (server.memory_mb / 2).max(512)));
    for arg in &server.extra_java_args {
        command.arg(arg);
    }
    command.arg("-jar").arg(&jar).arg("nogui");

    let info = crate::state::spawn_server_process(id, command).await?;

    server.last_started = Some(chrono::Utc::now());
    server.save().await?;

    Ok(info)
}

/// Gracefully stop a running server by issuing the `stop` console command,
/// falling back to a force-kill if the process can't be reached.
pub async fn stop(id: &str) -> crate::Result<()> {
    if !crate::state::is_server_running(id) {
        return Ok(());
    }
    if !crate::state::send_server_command(id, "stop") {
        crate::state::force_kill_server(id);
    }
    Ok(())
}

/// Forcibly terminate a running server process.
pub async fn kill(id: &str) -> crate::Result<()> {
    crate::state::force_kill_server(id);
    Ok(())
}

/// Send an arbitrary console command to a running server.
pub async fn send_command(id: &str, command: &str) -> crate::Result<()> {
    if !crate::state::send_server_command(id, command) {
        return Err(crate::ErrorKind::InputError(
            "Server is not running".to_string(),
        )
        .into());
    }
    Ok(())
}

/// Get the buffered console output for a server.
pub async fn get_log(id: &str) -> crate::Result<Vec<String>> {
    Ok(crate::state::get_server_log(id))
}

/// List all servers that currently have a running process.
pub async fn get_running() -> crate::Result<Vec<RunningServerInfo>> {
    Ok(crate::state::get_running_servers())
}

/// Read a config file (e.g. `server.properties`) from a server directory.
pub async fn get_config(id: &str, file: &str) -> crate::Result<String> {
    validate_config_name(file)?;
    let state = State::get().await?;
    let path = state.directories.server_dir(id).join(file);
    if !path.exists() {
        return Ok(String::new());
    }
    let bytes = io::read(&path).await?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

/// Write a config file (e.g. `server.properties`) into a server directory.
pub async fn set_config(
    id: &str,
    file: &str,
    contents: &str,
) -> crate::Result<()> {
    validate_config_name(file)?;
    let state = State::get().await?;
    let dir = state.directories.server_dir(id);
    if !dir.exists() {
        return Err(crate::ErrorKind::InputError(format!(
            "No local server with id {id}"
        ))
        .into());
    }
    io::write(dir.join(file), contents).await?;
    Ok(())
}

/// List the Minecraft versions available for the given server software,
/// newest first.
pub async fn get_versions(
    software: ServerSoftware,
) -> crate::Result<Vec<String>> {
    let state = State::get().await?;
    match software {
        ServerSoftware::Vanilla => {
            let manifest: MojangManifest = fetch_json(
                Method::GET,
                MOJANG_MANIFEST_URL,
                None,
                None,
                None,
                &state.api_semaphore,
                &state.pool,
            )
            .await?;
            Ok(manifest
                .versions
                .into_iter()
                .filter(|v| v.version_type == "release")
                .map(|v| v.id)
                .collect())
        }
        ServerSoftware::Paper => {
            let project: PaperProject = fetch_json(
                Method::GET,
                "https://api.papermc.io/v2/projects/paper",
                None,
                None,
                None,
                &state.api_semaphore,
                &state.pool,
            )
            .await?;
            let mut versions = project.versions;
            versions.reverse();
            Ok(versions)
        }
        ServerSoftware::Purpur => {
            let project: PurpurProject = fetch_json(
                Method::GET,
                "https://api.purpurmc.org/v2/purpur",
                None,
                None,
                None,
                &state.api_semaphore,
                &state.pool,
            )
            .await?;
            let mut versions = project.versions;
            versions.reverse();
            Ok(versions)
        }
        ServerSoftware::Fabric => {
            let games: Vec<FabricGameVersion> = fetch_json(
                Method::GET,
                "https://meta.fabricmc.net/v2/versions/game",
                None,
                None,
                None,
                &state.api_semaphore,
                &state.pool,
            )
            .await?;
            Ok(games
                .into_iter()
                .filter(|g| g.stable)
                .map(|g| g.version)
                .collect())
        }
    }
}

async fn resolve_java(server: &ServerInstance) -> crate::Result<PathBuf> {
    if let Some(path) = &server.java_path {
        let path = PathBuf::from(path);
        if crate::api::jre::check_jre(path.clone()).await.is_ok() {
            return Ok(path);
        }
        tracing::warn!(
            "Configured Java for server {} is invalid; auto-resolving",
            server.id
        );
    }

    let required = required_java_major(&server.minecraft_version);
    if let Some(jre) =
        crate::api::jre::find_filtered_jres(Some(required)).await?.into_iter().next()
    {
        return Ok(PathBuf::from(jre.path));
    }

    crate::api::jre::auto_install_java(required).await
}

/// Map a Minecraft version to the major Java version its server requires.
fn required_java_major(minecraft_version: &str) -> u32 {
    let mut parts = minecraft_version.split(['.', '-', ' ']);
    let _major = parts.next();
    let minor = parts.next().and_then(|p| p.parse::<u32>().ok());
    let patch = parts.next().and_then(|p| p.parse::<u32>().ok()).unwrap_or(0);

    match minor {
        Some(m) if m >= 21 => 21,
        Some(20) if patch >= 5 => 21,
        Some(m) if m >= 17 => 17,
        Some(_) => 8,
        // Snapshots and unrecognized versions: assume the newest LTS.
        None => 21,
    }
}

fn validate_config_name(file: &str) -> crate::Result<()> {
    if file.is_empty()
        || file.contains('/')
        || file.contains('\\')
        || file.contains("..")
    {
        return Err(crate::ErrorKind::InputError(
            "Invalid config file name".to_string(),
        )
        .into());
    }
    Ok(())
}

fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !slug.is_empty() && !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "server".to_string()
    } else {
        slug
    }
}

async fn generate_unique_id(
    name: &str,
    state: &State,
) -> crate::Result<String> {
    let base = slugify(name);
    let servers_dir = state.directories.servers_dir();

    let mut candidate = base.clone();
    let mut counter = 1;
    while servers_dir.join(&candidate).exists() {
        counter += 1;
        candidate = format!("{base}-{counter}");
    }
    Ok(candidate)
}

async fn write_eula(dir: &std::path::Path) -> crate::Result<()> {
    io::write(dir.join("eula.txt"), "eula=true\n").await?;
    Ok(())
}

async fn write_default_properties_if_absent(
    dir: &std::path::Path,
    name: &str,
) -> crate::Result<()> {
    let path = dir.join("server.properties");
    if path.exists() {
        return Ok(());
    }
    let motd = name.replace(['\n', '\r'], " ");
    let contents = format!(
        "# Minecraft server properties\n# Created by the Modrinth App\nmotd={motd}\nserver-port=25565\nmax-players=20\nonline-mode=true\nview-distance=10\nspawn-protection=16\n"
    );
    io::write(path, contents).await?;
    Ok(())
}

async fn download_vanilla(
    state: &State,
    minecraft_version: &str,
) -> crate::Result<(Bytes, Option<String>)> {
    let manifest: MojangManifest = fetch_json(
        Method::GET,
        MOJANG_MANIFEST_URL,
        None,
        None,
        None,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    let entry = manifest
        .versions
        .into_iter()
        .find(|v| v.id == minecraft_version)
        .ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "Minecraft version {minecraft_version} not found"
            ))
        })?;

    let detail: MojangVersionDetail = fetch_json(
        Method::GET,
        &entry.url,
        None,
        None,
        None,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    let server = detail.downloads.server.ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Minecraft {minecraft_version} does not provide a dedicated server download"
        ))
    })?;

    let bytes = fetch(
        &server.url,
        Some(&server.sha1),
        None,
        None,
        &state.fetch_semaphore,
        &state.pool,
    )
    .await?;

    Ok((bytes, Some(minecraft_version.to_string())))
}

async fn download_paper(
    state: &State,
    minecraft_version: &str,
) -> crate::Result<(Bytes, Option<String>)> {
    let version: PaperVersion = fetch_json(
        Method::GET,
        &format!(
            "https://api.papermc.io/v2/projects/paper/versions/{minecraft_version}"
        ),
        None,
        None,
        None,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    let build = version.builds.into_iter().max().ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "No Paper builds found for Minecraft {minecraft_version}"
        ))
    })?;

    let detail: PaperBuild = fetch_json(
        Method::GET,
        &format!(
            "https://api.papermc.io/v2/projects/paper/versions/{minecraft_version}/builds/{build}"
        ),
        None,
        None,
        None,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    let jar_name = detail.downloads.application.name;
    let url = format!(
        "https://api.papermc.io/v2/projects/paper/versions/{minecraft_version}/builds/{build}/downloads/{jar_name}"
    );

    let bytes = fetch(
        &url,
        None,
        None,
        None,
        &state.fetch_semaphore,
        &state.pool,
    )
    .await?;

    Ok((bytes, Some(build.to_string())))
}

async fn download_purpur(
    state: &State,
    minecraft_version: &str,
) -> crate::Result<(Bytes, Option<String>)> {
    let version: PurpurVersion = fetch_json(
        Method::GET,
        &format!("https://api.purpurmc.org/v2/purpur/{minecraft_version}"),
        None,
        None,
        None,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    let build = version.builds.latest;
    let url = format!(
        "https://api.purpurmc.org/v2/purpur/{minecraft_version}/{build}/download"
    );

    let bytes = fetch(
        &url,
        None,
        None,
        None,
        &state.fetch_semaphore,
        &state.pool,
    )
    .await?;

    Ok((bytes, Some(build)))
}

async fn download_fabric(
    state: &State,
    minecraft_version: &str,
) -> crate::Result<(Bytes, Option<String>)> {
    let loaders: Vec<FabricLoaderEntry> = fetch_json(
        Method::GET,
        &format!(
            "https://meta.fabricmc.net/v2/versions/loader/{minecraft_version}"
        ),
        None,
        None,
        None,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    let loader = loaders
        .iter()
        .find(|l| l.loader.stable)
        .or_else(|| loaders.first())
        .map(|l| l.loader.version.clone())
        .ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "No Fabric loader available for Minecraft {minecraft_version}"
            ))
        })?;

    let installers: Vec<FabricInstallerEntry> = fetch_json(
        Method::GET,
        "https://meta.fabricmc.net/v2/versions/installer",
        None,
        None,
        None,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    let installer = installers
        .iter()
        .find(|i| i.stable)
        .or_else(|| installers.first())
        .map(|i| i.version.clone())
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "No Fabric installer available".to_string(),
            )
        })?;

    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{minecraft_version}/{loader}/{installer}/server/jar"
    );

    let bytes = fetch(
        &url,
        None,
        None,
        None,
        &state.fetch_semaphore,
        &state.pool,
    )
    .await?;

    Ok((bytes, Some(loader)))
}

#[derive(Deserialize)]
struct MojangManifest {
    versions: Vec<MojangVersionEntry>,
}

#[derive(Deserialize)]
struct MojangVersionEntry {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    url: String,
}

#[derive(Deserialize)]
struct MojangVersionDetail {
    downloads: MojangDownloads,
}

#[derive(Deserialize)]
struct MojangDownloads {
    server: Option<MojangArtifact>,
}

#[derive(Deserialize)]
struct MojangArtifact {
    url: String,
    sha1: String,
}

#[derive(Deserialize)]
struct PaperProject {
    versions: Vec<String>,
}

#[derive(Deserialize)]
struct PaperVersion {
    builds: Vec<u32>,
}

#[derive(Deserialize)]
struct PaperBuild {
    downloads: PaperDownloads,
}

#[derive(Deserialize)]
struct PaperDownloads {
    application: PaperApplication,
}

#[derive(Deserialize)]
struct PaperApplication {
    name: String,
}

#[derive(Deserialize)]
struct PurpurProject {
    versions: Vec<String>,
}

#[derive(Deserialize)]
struct PurpurVersion {
    builds: PurpurBuilds,
}

#[derive(Deserialize)]
struct PurpurBuilds {
    latest: String,
}

#[derive(Deserialize)]
struct FabricGameVersion {
    version: String,
    stable: bool,
}

#[derive(Deserialize)]
struct FabricLoaderEntry {
    loader: FabricLoaderInfo,
}

#[derive(Deserialize)]
struct FabricLoaderInfo {
    version: String,
    stable: bool,
}

#[derive(Deserialize)]
struct FabricInstallerEntry {
    version: String,
    stable: bool,
}

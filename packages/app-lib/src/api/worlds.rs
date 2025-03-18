use crate::data::ModLoader;
use crate::launcher::get_loader_version_from_profile;
use crate::state::ProfileInstallStage;
pub use crate::util::server_ping::{
    ServerGameProfile, ServerPlayers, ServerStatus, ServerVersion,
};
use crate::util::{io, server_ping};
use crate::{launcher, Error, ErrorKind, Result, State};
use chrono::{DateTime, TimeZone, Utc};
use either::Either;
use flate2::read::GzDecoder;
use hickory_resolver::error::ResolveErrorKind;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use tokio::io::AsyncReadExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct World {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_played: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "either::serde_untagged_optional"
    )]
    pub icon: Option<Either<PathBuf, Url>>,
    pub pinned: bool,
    #[serde(flatten)]
    pub details: WorldDetails,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorldDetails {
    Singleplayer {
        path: PathBuf,
        game_mode: SingleplayerGameMode,
        hardcore: bool,
    },
    Server {
        address: String,
    },
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SingleplayerGameMode {
    Creative,
    Survival,
    Adventure,
    Spectator,
    Unknown,
}

pub async fn get_profile_worlds(profile_path: &Path) -> Result<Vec<World>> {
    let mut result = vec![];
    get_singleplayer_worlds(profile_path, &mut result).await?;
    get_server_worlds(profile_path, &mut result).await?;
    Ok(result)
}

async fn get_singleplayer_worlds(
    instance_dir: &Path,
    worlds: &mut Vec<World>,
) -> Result<()> {
    let saves_dir = instance_dir.join("saves");
    if !saves_dir.exists() {
        return Ok(());
    }
    let mut saves_dir = io::read_dir(saves_dir).await?;
    while let Some(world_dir) = saves_dir.next_entry().await? {
        let world_path = world_dir.path();
        let level_dat_path = world_path.join("level.dat");
        if !level_dat_path.exists() {
            continue;
        }
        if let Ok(world) = read_singleplayer_world(world_path).await {
            worlds.push(world);
        }
    }

    Ok(())
}

async fn read_singleplayer_world(world_path: PathBuf) -> Result<World> {
    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    struct LevelDataRoot {
        data: LevelData,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    struct LevelData {
        level_name: String,
        last_played: i64,
        game_type: i32,
        #[serde(rename = "hardcore")]
        hardcore: bool,
    }

    let level_data = io::read(world_path.join("level.dat")).await?;
    let level_data = GzDecoder::new(&level_data[..]);
    let level_data: LevelDataRoot = fastnbt::from_reader(level_data)?;
    let level_data = level_data.data;

    let icon = Some(world_path.join("icon.png")).filter(|i| i.exists());

    let game_mode = match level_data.game_type {
        0 => SingleplayerGameMode::Survival,
        1 => SingleplayerGameMode::Creative,
        2 => SingleplayerGameMode::Adventure,
        3 => SingleplayerGameMode::Spectator,
        _ => SingleplayerGameMode::Unknown,
    };

    Ok(World {
        name: level_data.level_name,
        last_played: Utc.timestamp_millis_opt(level_data.last_played).single(),
        icon: icon.map(Either::Left),
        pinned: false, // TODO
        details: WorldDetails::Singleplayer {
            path: world_path,
            game_mode,
            hardcore: level_data.hardcore,
        },
    })
}

async fn get_server_worlds(
    instance_dir: &Path,
    worlds: &mut Vec<World>,
) -> Result<()> {
    #[derive(Deserialize, Debug)]
    struct ServersData {
        servers: Vec<ServerData>,
    }

    #[derive(Deserialize, Debug)]
    struct ServerData {
        #[serde(default)]
        hidden: bool,
        icon: Option<String>,
        ip: String,
        name: String,
    }

    let servers_dat_path = instance_dir.join("servers.dat");
    if !servers_dat_path.exists() {
        return Ok(());
    }
    let servers_data = io::read(servers_dat_path).await?;
    let servers_data: ServersData = fastnbt::from_bytes(&servers_data)?;

    let join_log = parse_join_log(instance_dir).await.ok();

    for server in servers_data.servers {
        if server.hidden {
            // TODO: Figure out whether we want to hide or show direct connect servers
            continue;
        }
        let icon = server.icon.and_then(|icon| {
            Url::parse(&format!("data:image/png;base64,{}", icon)).ok()
        });
        let last_played = join_log
            .as_ref()
            .and_then(|log| {
                let address = parse_server_address(&server.ip).ok()?;
                log.get(&(address.0.to_owned(), address.1))
            })
            .and_then(|time| Utc.timestamp_millis_opt(*time).single());
        let world = World {
            name: server.name,
            last_played,
            icon: icon.map(Either::Right),
            pinned: false, // TODO
            details: WorldDetails::Server { address: server.ip },
        };
        worlds.push(world);
    }

    Ok(())
}

async fn parse_join_log(
    instance_dir: &Path,
) -> Result<HashMap<(String, u16), i64>> {
    let mut result = HashMap::new();
    let join_log_path = instance_dir.join("logs/server_join_log.txt");
    if !join_log_path.exists() {
        return Ok(result);
    }

    let reader = io::open_file(&join_log_path).await?;
    let reader = BufReader::new(reader);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        let mut parts = line.split_whitespace();
        let Some(time) = parts.next().and_then(|s| s.parse::<i64>().ok())
        else {
            continue;
        };
        let Some(host) = parts.nth(2).filter(|s| s.ends_with(',')) else {
            continue;
        };
        let Some(port) = parts.next().and_then(|s| s.parse::<u16>().ok())
        else {
            continue;
        };
        result
            .entry((host[..host.len() - 1].to_owned(), port))
            .and_modify(|old| *old = max(*old, time))
            .or_insert(time);
    }

    Ok(result)
}

pub async fn get_profile_protocol_version(
    profile: &str,
) -> Result<Option<i32>> {
    let mut profile = super::profile::get(profile).await?.ok_or_else(|| {
        ErrorKind::UnmanagedProfileError(format!(
            "Could not find profile {}",
            profile
        ))
    })?;
    if profile.install_stage != ProfileInstallStage::Installed {
        return Ok(None);
    }

    if let Some(protocol_version) = profile.protocol_version {
        return Ok(Some(protocol_version));
    }

    let minecraft = crate::api::metadata::get_minecraft_versions().await?;
    let version_index = minecraft
        .versions
        .iter()
        .position(|it| it.id == profile.game_version)
        .ok_or(crate::ErrorKind::LauncherError(format!(
            "Invalid game version: {}",
            profile.game_version
        )))?;
    let version = &minecraft.versions[version_index];

    let loader_version = get_loader_version_from_profile(
        &profile.game_version,
        profile.loader,
        profile.loader_version.as_deref(),
    )
    .await?;
    if profile.loader != ModLoader::Vanilla && loader_version.is_none() {
        return Ok(None);
    }

    let version_jar =
        loader_version.as_ref().map_or(version.id.clone(), |it| {
            format!("{}-{}", version.id.clone(), it.id.clone())
        });

    let state = State::get().await?;
    let client_path = state
        .directories
        .version_dir(&version_jar)
        .join(format!("{version_jar}.jar"));

    if !client_path.exists() {
        return Ok(None);
    }

    let version = launcher::read_protocol_version_from_jar(client_path).await?;
    if version.is_some() {
        profile.protocol_version = version;
        profile.upsert(&state.pool).await?;
    }
    Ok(version)
}

pub async fn get_server_status(
    address: &str,
    protocol_version: Option<i32>,
) -> Result<ServerStatus> {
    let (original_host, original_port) = parse_server_address(address)?;
    let (host, port) =
        resolve_server_address(original_host, original_port).await?;
    Ok(server_ping::get_server_status(
        &(&host as &str, port),
        (original_host, original_port),
        protocol_version,
    )
    .await?)
}

pub fn parse_server_address(address: &str) -> Result<(&str, u16)> {
    parse_server_address_inner(address)
        .map_err(|e| Error::from(ErrorKind::InputError(e)))
}

// Reimplementation of Guava's HostAndPort#fromString with a default port of 25565
fn parse_server_address_inner(
    address: &str,
) -> std::result::Result<(&str, u16), String> {
    let (host, port_str) = if address.starts_with("[") {
        let colon_index = address.find(':');
        let close_bracket_index = address.rfind(']');
        if colon_index.is_none() || close_bracket_index.is_none() {
            return Err(format!("Invalid bracketed host/port: {address}"));
        }
        let close_bracket_index = close_bracket_index.unwrap();

        let host = &address[1..close_bracket_index];
        if close_bracket_index + 1 == address.len() {
            (host, "")
        } else {
            if address.as_bytes().get(close_bracket_index).copied()
                != Some(b':')
            {
                return Err(format!(
                    "Only a colon may follow a close bracket: {address}"
                ));
            }
            let port_str = &address[close_bracket_index + 2..];
            for c in port_str.chars() {
                if !c.is_ascii_digit() {
                    return Err(format!("Port must be numeric: {address}"));
                }
            }
            (host, port_str)
        }
    } else {
        let colon_pos = address.find(':');
        if let Some(colon_pos) = colon_pos {
            (&address[..colon_pos], &address[colon_pos + 1..])
        } else {
            (address, "")
        }
    };

    let mut port = None;
    if !port_str.is_empty() {
        if port_str.starts_with('+') {
            return Err(format!("Unparseable port number: {port_str}"));
        }
        port = port_str.parse::<u16>().ok();
        if port.is_none() {
            return Err(format!("Unparseable port number: {port_str}"));
        }
    }

    Ok((host, port.unwrap_or(25565)))
}

async fn resolve_server_address(
    host: &str,
    port: u16,
) -> Result<(String, u16)> {
    if host.parse::<Ipv4Addr>().is_ok() || host.parse::<Ipv6Addr>().is_ok() {
        return Ok((host.to_owned(), port));
    }
    let resolver =
        hickory_resolver::TokioAsyncResolver::tokio_from_system_conf()?;
    Ok(match resolver
        .srv_lookup(format!("_minecraft._tcp.{}", host))
        .await
    {
        Err(e)
            if matches!(e.kind(), ResolveErrorKind::NoRecordsFound { .. }) =>
        {
            None
        }
        Err(e) => return Err(e.into()),
        Ok(lookup) => lookup
            .into_iter()
            .next()
            .map(|r| (r.target().to_string(), port)),
    }
    .unwrap_or_else(|| (host.to_owned(), port)))
}

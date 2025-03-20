use crate::data::ModLoader;
use crate::launcher::get_loader_version_from_profile;
use crate::state::ProfileInstallStage;
pub use crate::util::server_ping::{
    ServerGameProfile, ServerPlayers, ServerStatus, ServerVersion,
};
use crate::util::{io, server_ping};
use crate::{launcher, Error, ErrorKind, Result, State};
use async_walkdir::WalkDir;
use async_zip::{Compression, ZipEntryBuilder};
use chrono::{DateTime, Local, TimeZone, Utc};
use either::Either;
use fs4::tokio::AsyncFileExt;
use futures::StreamExt;
use hickory_resolver::error::ResolveErrorKind;
use lazy_static::lazy_static;
use quartz_nbt::{NbtCompound, NbtTag};
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use std::io::Cursor;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::compat::FuturesAsyncWriteCompatExt;
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
    #[serde(flatten)]
    pub details: WorldDetails,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorldDetails {
    Singleplayer {
        path: String,
        game_mode: SingleplayerGameMode,
        hardcore: bool,
    },
    Server {
        index: usize,
        address: String,
        pack_status: ServerPackStatus,
    },
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum SingleplayerGameMode {
    #[default]
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum ServerPackStatus {
    Enabled,
    Disabled,
    #[default]
    Prompt,
}

impl From<Option<bool>> for ServerPackStatus {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(true) => ServerPackStatus::Enabled,
            Some(false) => ServerPackStatus::Disabled,
            None => ServerPackStatus::Prompt,
        }
    }
}

impl From<ServerPackStatus> for Option<bool> {
    fn from(val: ServerPackStatus) -> Self {
        match val {
            ServerPackStatus::Enabled => Some(true),
            ServerPackStatus::Disabled => Some(false),
            ServerPackStatus::Prompt => None,
        }
    }
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
        if let Some(_lock) = try_get_world_session_lock(&world_path).await? {
            if let Ok(world) = read_singleplayer_world(world_path).await {
                worlds.push(world);
            }
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
        #[serde(default)]
        level_name: String,
        #[serde(default)]
        last_played: i64,
        #[serde(default)]
        game_type: i32,
        #[serde(default, rename = "hardcore")]
        hardcore: bool,
    }

    let level_data = io::read(world_path.join("level.dat")).await?;
    let level_data: LevelDataRoot = quartz_nbt::serde::deserialize(
        &level_data,
        quartz_nbt::io::Flavor::GzCompressed,
    )?
    .0;
    let level_data = level_data.data;

    let icon = Some(world_path.join("icon.png")).filter(|i| i.exists());

    let game_mode = match level_data.game_type {
        0 => SingleplayerGameMode::Survival,
        1 => SingleplayerGameMode::Creative,
        2 => SingleplayerGameMode::Adventure,
        3 => SingleplayerGameMode::Spectator,
        _ => SingleplayerGameMode::Survival,
    };

    Ok(World {
        name: level_data.level_name,
        last_played: Utc.timestamp_millis_opt(level_data.last_played).single(),
        icon: icon.map(Either::Left),
        details: WorldDetails::Singleplayer {
            path: world_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            game_mode,
            hardcore: level_data.hardcore,
        },
    })
}

async fn get_server_worlds(
    instance_dir: &Path,
    worlds: &mut Vec<World>,
) -> Result<()> {
    let servers = servers_data::read(instance_dir).await?;
    if servers.is_empty() {
        return Ok(());
    }

    let join_log = parse_join_log(instance_dir).await.ok();

    for (index, server) in servers.into_iter().enumerate() {
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
            details: WorldDetails::Server {
                index,
                address: server.ip,
                pack_status: server.accept_textures.into(),
            },
        };
        worlds.push(world);
    }

    Ok(())
}

pub async fn rename_world(
    instance: &Path,
    world: &str,
    new_name: &str,
) -> Result<()> {
    let world = get_world_dir(instance, world);
    let level_dat_path = world.join("level.dat");
    if !level_dat_path.exists() {
        return Ok(());
    }
    let _lock = get_world_session_lock(&world).await?;

    let level_data = io::read(&level_dat_path).await?;
    let (mut root_data, _) = quartz_nbt::io::read_nbt(
        &mut Cursor::new(level_data),
        quartz_nbt::io::Flavor::GzCompressed,
    )?;
    let data = root_data.get_mut::<_, &mut NbtCompound>("Data")?;

    data.insert(
        "LevelName",
        NbtTag::String(new_name.trim_ascii().to_string()),
    );

    let mut level_data = vec![];
    quartz_nbt::io::write_nbt(
        &mut level_data,
        None,
        &root_data,
        quartz_nbt::io::Flavor::GzCompressed,
    )?;
    io::write(level_dat_path, level_data).await?;
    Ok(())
}

pub async fn reset_world_icon(instance: &Path, world: &str) -> Result<()> {
    let world = get_world_dir(instance, world);
    let icon = world.join("icon.png");
    if let Some(_lock) = try_get_world_session_lock(&world).await? {
        let _ = io::remove_file(icon).await;
    }
    Ok(())
}

pub async fn backup_world(instance: &Path, world: &str) -> Result<u64> {
    let world_dir = get_world_dir(instance, world);
    let _lock = get_world_session_lock(&world_dir).await?;
    let backups_dir = instance.join("backups");

    io::create_dir_all(&backups_dir).await?;

    let name_base = {
        let now = Local::now();
        let formatted_time = now.format("%Y-%m-%d_%H-%M-%S");
        format!("{}_{}", formatted_time, world)
    };
    let output_path =
        backups_dir.join(find_available_name(&backups_dir, &name_base, ".zip"));

    let writer = tokio::fs::File::create(&output_path).await?;
    let mut writer = async_zip::tokio::write::ZipFileWriter::with_tokio(writer);

    let mut walker = WalkDir::new(&world_dir);
    while let Some(entry) = walker.next().await {
        let entry = entry.map_err(|e| io::IOError::IOPathError {
            path: e.path().unwrap().to_string_lossy().to_string(),
            source: e.into_io().unwrap(),
        })?;
        if !entry.file_type().await?.is_file() {
            continue;
        }
        if entry.file_name() == "session.lock" {
            continue;
        }
        let zip_filename = format!(
            "{world}/{}",
            entry
                .path()
                .strip_prefix(&world_dir)?
                .display()
                .to_string()
                .replace('\\', "/")
        );
        let mut stream = writer
            .write_entry_stream(
                ZipEntryBuilder::new(zip_filename.into(), Compression::Deflate)
                    .build(),
            )
            .await?
            .compat_write();
        let mut source = tokio::fs::File::open(entry.path()).await?;
        tokio::io::copy(&mut source, &mut stream).await?;
        stream.into_inner().close().await?;
    }

    writer.close().await?;
    Ok(io::metadata(output_path).await?.len())
}

fn find_available_name(dir: &Path, file_name: &str, extension: &str) -> String {
    lazy_static! {
        static ref RESERVED_WINDOWS_FILENAMES: Regex = RegexBuilder::new(r#"^.*\.|(?:COM|CLOCK\$|CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\..*)?$"#)
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref COPY_COUNTER_PATTERN: Regex = RegexBuilder::new(r#"^(?<name>.*) \((?<count>\d*)\)$"#)
            .case_insensitive(true)
            .unicode(true)
            .build()
            .unwrap();
    }

    let mut file_name = file_name.replace(
        [
            '/', '\n', '\r', '\t', '\0', '\x0c', '`', '?', '*', '\\', '<', '>',
            '|', '"', ':', '.', '/', '"',
        ],
        "_",
    );
    if RESERVED_WINDOWS_FILENAMES.is_match(&file_name) {
        file_name.insert(0, '_');
        file_name.push('_');
    }

    let mut count = 0;
    if let Some(find) = COPY_COUNTER_PATTERN.captures(&file_name) {
        count = find
            .name("count")
            .unwrap()
            .as_str()
            .parse::<i32>()
            .unwrap_or(0);
        let end = find.name("name").unwrap().end();
        drop(find);
        file_name.truncate(end);
    }

    if file_name.len() > 255 - extension.len() {
        file_name.truncate(255 - extension.len());
    }

    let mut current_attempt = file_name.clone();
    loop {
        if count != 0 {
            let with_count = format!(" ({count})");
            if file_name.len() > 255 - with_count.len() {
                current_attempt.truncate(255 - with_count.len());
            }
            current_attempt.push_str(&with_count);
        }

        current_attempt.push_str(extension);

        let result = dir.join(&current_attempt);
        if !result.exists() {
            return current_attempt;
        }

        count += 1;
        current_attempt.replace_range(..current_attempt.len(), &file_name);
    }
}

pub async fn delete_world(instance: &Path, world: &str) -> Result<()> {
    let world = get_world_dir(instance, world);
    let lock = get_world_session_lock(&world).await?;
    let lock_path = world.join("session.lock");

    let mut dir = io::read_dir(&world).await?;
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        if entry.file_type().await?.is_dir() {
            io::remove_dir_all(path).await?;
            continue;
        }
        if path != lock_path {
            io::remove_file(path).await?;
        }
    }

    drop(lock);
    io::remove_file(lock_path).await?;
    io::remove_dir(world).await?;

    Ok(())
}

fn get_world_dir(instance: &Path, world: &str) -> PathBuf {
    instance.join("saves").join(world)
}

async fn get_world_session_lock(world: &Path) -> Result<tokio::fs::File> {
    let lock_path = world.join("session.lock");
    let mut file = tokio::fs::File::options()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .await?;
    file.write_all("☃".as_bytes()).await?;
    file.sync_all().await?;
    let locked = file.try_lock_exclusive()?;
    locked.then_some(file).ok_or_else(|| {
        io::IOError::IOPathError {
            source: std::io::Error::new(
                std::io::ErrorKind::ResourceBusy,
                "already locked by Minecraft",
            ),
            path: lock_path.to_string_lossy().into_owned(),
        }
        .into()
    })
}

async fn try_get_world_session_lock(
    world: &Path,
) -> Result<Option<tokio::fs::File>> {
    let file = tokio::fs::File::options()
        .create(true)
        .write(true)
        .truncate(false)
        .open(world.join("session.lock"))
        .await?;
    file.sync_all().await?;
    let locked = file.try_lock_exclusive()?;
    Ok(locked.then_some(file))
}

pub async fn add_server_to_profile(
    profile_path: &Path,
    name: String,
    address: String,
    pack_status: ServerPackStatus,
) -> Result<usize> {
    let mut servers = servers_data::read(profile_path).await?;
    let insert_index = servers
        .iter()
        .position(|x| x.hidden)
        .unwrap_or(servers.len());
    servers.insert(
        insert_index,
        servers_data::ServerData {
            name,
            ip: address,
            accept_textures: pack_status.into(),
            hidden: false,
            icon: None,
        },
    );
    servers_data::write(profile_path, &servers).await?;
    Ok(insert_index)
}

pub async fn edit_server_in_profile(
    profile_path: &Path,
    index: usize,
    name: String,
    address: String,
    pack_status: ServerPackStatus,
) -> Result<()> {
    let mut servers = servers_data::read(profile_path).await?;
    let server =
        servers
            .get_mut(index)
            .filter(|x| !x.hidden)
            .ok_or_else(|| {
                ErrorKind::InputError(format!(
                    "No editable server at index {index}"
                ))
                .as_error()
            })?;
    server.name = name;
    server.ip = address;
    server.accept_textures = pack_status.into();
    servers_data::write(profile_path, &servers).await?;
    Ok(())
}

pub async fn remove_server_from_profile(
    profile_path: &Path,
    index: usize,
) -> Result<()> {
    let mut servers = servers_data::read(profile_path).await?;
    if servers.get(index).filter(|x| !x.hidden).is_none() {
        return Err(ErrorKind::InputError(format!(
            "No removable server at index {index}"
        ))
        .into());
    }
    servers.remove(index);
    servers_data::write(profile_path, &servers).await?;
    Ok(())
}

mod servers_data {
    use crate::util::io;
    use crate::Result;
    use serde::{Deserialize, Serialize};
    use std::path::Path;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct ServerData {
        #[serde(default)]
        pub hidden: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub icon: Option<String>,
        #[serde(default)]
        pub ip: String,
        #[serde(default)]
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub accept_textures: Option<bool>,
    }

    pub async fn read(instance_dir: &Path) -> Result<Vec<ServerData>> {
        #[derive(Deserialize, Debug)]
        struct ServersData {
            #[serde(default)]
            servers: Vec<ServerData>,
        }

        let servers_dat_path = instance_dir.join("servers.dat");
        if !servers_dat_path.exists() {
            return Ok(vec![]);
        }
        let servers_data = io::read(servers_dat_path).await?;
        let servers_data: ServersData = quartz_nbt::serde::deserialize(
            &servers_data,
            quartz_nbt::io::Flavor::Uncompressed,
        )?
        .0;
        Ok(servers_data.servers)
    }

    pub async fn write(
        instance_dir: &Path,
        servers: &[ServerData],
    ) -> Result<()> {
        #[derive(Serialize, Debug)]
        struct ServersData<'a> {
            servers: &'a [ServerData],
        }

        let servers_dat_path = instance_dir.join("servers.dat");
        let data = quartz_nbt::serde::serialize(
            &ServersData { servers },
            None,
            quartz_nbt::io::Flavor::Uncompressed,
        )?;
        io::write(servers_dat_path, data).await?;
        Ok(())
    }
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
    server_ping::get_server_status(
        &(&host as &str, port),
        (original_host, original_port),
        protocol_version,
    )
    .await
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

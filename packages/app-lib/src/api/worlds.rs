use crate::data::ModLoader;
use crate::launcher::get_loader_version_from_profile;
use crate::profile::get_full_path;
use crate::state::attached_world_data::AttachedWorldData;
use crate::state::{
    Profile, ProfileInstallStage, attached_world_data, server_join_log,
};
pub use crate::util::server_ping::{
    ServerGameProfile, ServerPlayers, ServerStatus, ServerVersion,
};
use crate::util::{io, server_ping};
use crate::{Error, ErrorKind, Result, State, launcher};
use async_walkdir::WalkDir;
use async_zip::{Compression, ZipEntryBuilder};
use chrono::{DateTime, Local, TimeZone, Utc};
use either::Either;
use enumset::{EnumSet, EnumSetType};
use fs4::tokio::AsyncFileExt;
use futures::StreamExt;
use quartz_nbt::{NbtCompound, NbtTag};
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::io::Cursor;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio_util::compat::FuturesAsyncWriteCompatExt;
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WorldWithProfile {
    pub profile: String,
    #[serde(flatten)]
    pub world: World,
}

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
    pub display_status: DisplayStatus,
    #[serde(flatten)]
    pub details: WorldDetails,
}

impl World {
    pub fn world_type(&self) -> WorldType {
        match self.details {
            WorldDetails::Singleplayer { .. } => WorldType::Singleplayer,
            WorldDetails::Server { .. } => WorldType::Server,
        }
    }

    pub fn world_id(&self) -> &str {
        match &self.details {
            WorldDetails::Singleplayer { path, .. } => path,
            WorldDetails::Server { address, .. } => address,
        }
    }
}

#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum WorldType {
    #[default]
    Singleplayer,
    Server,
}

impl WorldType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Singleplayer => "singleplayer",
            Self::Server => "server",
        }
    }

    pub fn from_string(string: &str) -> Self {
        match string {
            "singleplayer" => Self::Singleplayer,
            "server" => Self::Server,
            _ => Self::Singleplayer,
        }
    }
}

#[derive(Deserialize, Serialize, EnumSetType, Debug, Default)]
#[serde(rename_all = "snake_case")]
#[enumset(serialize_repr = "list")]
pub enum DisplayStatus {
    #[default]
    Normal,
    Hidden,
    Favorite,
}

impl DisplayStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Hidden => "hidden",
            Self::Favorite => "favorite",
        }
    }

    pub fn from_string(string: &str) -> Self {
        match string {
            "normal" => Self::Normal,
            "hidden" => Self::Hidden,
            "favorite" => Self::Favorite,
            _ => Self::Normal,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorldDetails {
    Singleplayer {
        path: String,
        game_mode: SingleplayerGameMode,
        hardcore: bool,
        locked: bool,
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

pub async fn get_recent_worlds(
    limit: usize,
    display_statuses: EnumSet<DisplayStatus>,
) -> Result<Vec<WorldWithProfile>> {
    let state = State::get().await?;
    let profiles_dir = state.directories.profiles_dir();

    let mut profiles = Profile::get_all(&state.pool).await?;
    profiles.sort_by_key(|x| Reverse(x.last_played));

    let mut result = Vec::with_capacity(limit);

    let mut least_recent_time = None;
    for profile in profiles {
        if result.len() >= limit && profile.last_played < least_recent_time {
            break;
        }
        let profile_path = &profile.path;
        let profile_dir = profiles_dir.join(profile_path);
        let profile_worlds =
            get_all_worlds_in_profile(profile_path, &profile_dir).await;
        if let Err(e) = profile_worlds {
            tracing::error!(
                "Failed to get worlds for profile {}: {}",
                profile_path,
                e
            );
            continue;
        }
        for world in profile_worlds? {
            let is_older = least_recent_time.is_none()
                || world.last_played < least_recent_time;
            if result.len() >= limit && is_older {
                continue;
            }
            if !display_statuses.contains(world.display_status) {
                continue;
            }
            if is_older {
                least_recent_time = world.last_played;
            }
            result.push(WorldWithProfile {
                profile: profile_path.clone(),
                world,
            });
        }
        if result.len() > limit {
            result.sort_by_key(|x| Reverse(x.world.last_played));
            result.truncate(limit);
        }
    }

    if result.len() <= limit {
        result.sort_by_key(|x| Reverse(x.world.last_played));
    }
    Ok(result)
}

pub async fn get_profile_worlds(profile_path: &str) -> Result<Vec<World>> {
    get_all_worlds_in_profile(profile_path, &get_full_path(profile_path).await?)
        .await
}

async fn get_all_worlds_in_profile(
    profile_path: &str,
    profile_dir: &Path,
) -> Result<Vec<World>> {
    let mut worlds = vec![];
    get_singleplayer_worlds_in_profile(profile_dir, &mut worlds).await?;
    get_server_worlds_in_profile(profile_path, profile_dir, &mut worlds)
        .await?;

    let state = State::get().await?;
    let attached_data =
        AttachedWorldData::get_all_for_instance(profile_path, &state.pool)
            .await?;
    if !attached_data.is_empty() {
        for world in &mut worlds {
            if let Some(data) = attached_data
                .get(&(world.world_type(), world.world_id().to_owned()))
            {
                attach_world_data_to_world(world, data);
            }
        }
    }

    Ok(worlds)
}

async fn get_singleplayer_worlds_in_profile(
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

pub async fn get_singleplayer_world(
    instance: &str,
    world: &str,
) -> Result<World> {
    let state = State::get().await?;
    let profile_path = state.directories.profiles_dir().join(instance);
    let mut world =
        read_singleplayer_world(get_world_dir(&profile_path, world)).await?;

    if let Some(data) = AttachedWorldData::get_for_world(
        instance,
        world.world_type(),
        world.world_id(),
        &state.pool,
    )
    .await?
    {
        attach_world_data_to_world(&mut world, &data);
    }
    Ok(world)
}

async fn read_singleplayer_world(world_path: PathBuf) -> Result<World> {
    if let Some(_lock) = try_get_world_session_lock(&world_path).await? {
        read_singleplayer_world_maybe_locked(world_path, false).await
    } else {
        read_singleplayer_world_maybe_locked(world_path, true).await
    }
}

async fn read_singleplayer_world_maybe_locked(
    world_path: PathBuf,
    locked: bool,
) -> Result<World> {
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
        display_status: DisplayStatus::Normal,
        details: WorldDetails::Singleplayer {
            path: world_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            game_mode,
            hardcore: level_data.hardcore,
            locked,
        },
    })
}

async fn get_server_worlds_in_profile(
    profile_path: &str,
    instance_dir: &Path,
    worlds: &mut Vec<World>,
) -> Result<()> {
    let servers = servers_data::read(instance_dir).await?;
    if servers.is_empty() {
        return Ok(());
    }

    let state = State::get().await?;
    let join_log = server_join_log::get_joins(profile_path, &state.pool)
        .await
        .ok();

    let first_server_index = worlds.len();
    for (index, server) in servers.into_iter().enumerate() {
        if server.hidden {
            // TODO: Figure out whether we want to hide or show direct connect servers
            continue;
        }
        let world = World {
            name: server.name,
            last_played: join_log
                .as_ref()
                .and_then(|log| {
                    let (host, port) = parse_server_address(&server.ip).ok()?;
                    log.get(&(host.to_owned(), port))
                })
                .copied(),
            icon: server
                .icon
                .and_then(|icon| {
                    Url::parse(&format!("data:image/png;base64,{icon}")).ok()
                })
                .map(Either::Right),
            display_status: DisplayStatus::Normal,
            details: WorldDetails::Server {
                index,
                address: server.ip,
                pack_status: server.accept_textures.into(),
            },
        };
        worlds.push(world);
    }

    if let Some(join_log) = join_log {
        let mut futures = JoinSet::new();
        for (index, world) in worlds.iter().enumerate().skip(first_server_index)
        {
            if world.last_played.is_some() {
                continue;
            }
            if let WorldDetails::Server { address, .. } = &world.details
                && let Ok((host, port)) = parse_server_address(address)
            {
                let host = host.to_owned();
                futures.spawn(async move {
                    resolve_server_address(&host, port)
                        .await
                        .ok()
                        .map(|x| (index, x))
                });
            }
        }
        for (index, address) in futures.join_all().await.into_iter().flatten() {
            worlds[index].last_played = join_log.get(&address).copied();
        }
    }

    Ok(())
}

fn attach_world_data_to_world(world: &mut World, data: &AttachedWorldData) {
    world.display_status = data.display_status;
}

pub async fn set_world_display_status(
    instance: &str,
    world_type: WorldType,
    world_id: &str,
    display_status: DisplayStatus,
) -> Result<()> {
    let state = State::get().await?;
    attached_world_data::set_display_status(
        instance,
        world_type,
        world_id,
        display_status,
        &state.pool,
    )
    .await?;
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
        format!("{formatted_time}_{world}")
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
    static RESERVED_WINDOWS_FILENAMES: LazyLock<Regex> = LazyLock::new(|| {
        RegexBuilder::new(r#"^.*\.|(?:COM|CLOCK\$|CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\..*)?$"#)
            .case_insensitive(true)
            .build()
            .unwrap()
    });
    static COPY_COUNTER_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
        RegexBuilder::new(r#"^(?<name>.*) \((?<count>\d*)\)$"#)
            .case_insensitive(true)
            .unicode(true)
            .build()
            .unwrap()
    });

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
    use crate::Result;
    use crate::util::io;
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

pub async fn get_profile_protocol_version(
    profile: &str,
) -> Result<Option<i32>> {
    let mut profile = super::profile::get(profile).await?.ok_or_else(|| {
        ErrorKind::UnmanagedProfileError(format!(
            "Could not find profile {profile}"
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
    static SIMULTANEOUS_DNS_QUERIES: Semaphore = Semaphore::const_new(24);

    if host.parse::<Ipv4Addr>().is_ok() || host.parse::<Ipv6Addr>().is_ok() {
        return Ok((host.to_owned(), port));
    }

    let _permit = SIMULTANEOUS_DNS_QUERIES.acquire().await?;
    let resolver = hickory_resolver::TokioResolver::builder_tokio()?.build();
    Ok(
        match resolver.srv_lookup(format!("_minecraft._tcp.{host}")).await {
            Err(e)
                if e.proto()
                    .filter(|x| x.kind().is_no_records_found())
                    .is_some() =>
            {
                None
            }
            Err(e) => return Err(e.into()),
            Ok(lookup) => lookup
                .into_iter()
                .next()
                .map(|r| (r.target().to_string(), r.port())),
        }
        .unwrap_or_else(|| (host.to_owned(), port)),
    )
}

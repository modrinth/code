//! Minecraft CLI argument logic
use crate::launcher::quick_play_version::QuickPlayServerVersion;
use crate::launcher::{QuickPlayVersion, parse_rules};
use crate::profile::QuickPlayType;
use crate::state::Credentials;
use crate::{
    state::{MemorySettings, WindowSize},
    util::{io::IOError, platform::classpath_separator},
};
use daedalus::minecraft::LoggingConfiguration;
use daedalus::{
    get_path_from_artifact,
    minecraft::{Argument, ArgumentValue, Library, VersionType},
    modded::SidedDataEntry,
};
use dunce::canonicalize;
use itertools::Itertools;
use std::io::{BufRead, BufReader, ErrorKind};
use std::net::SocketAddr;
use std::{collections::HashMap, path::Path};
use uuid::Uuid;

// Replaces the space separator with a newline character, as to not split the arguments
const TEMPORARY_REPLACE_CHAR: &str = "\n";

pub fn get_class_paths(
    libraries_path: &Path,
    libraries: &[Library],
    launcher_class_path: &[&Path],
    java_arch: &str,
    minecraft_updated: bool,
) -> crate::Result<String> {
    launcher_class_path
        .iter()
        .map(|path| {
            Ok(canonicalize(path)
                .map_err(|_| {
                    crate::ErrorKind::LauncherError(format!(
                        "Specified class path {} does not exist",
                        path.to_string_lossy()
                    ))
                    .as_error()
                })?
                .to_string_lossy()
                .to_string())
        })
        .chain(libraries.iter().filter_map(|library| {
            if let Some(rules) = &library.rules
                && !parse_rules(
                    rules,
                    java_arch,
                    &QuickPlayType::None,
                    minecraft_updated,
                )
            {
                return None;
            }

            if !library.include_in_classpath {
                return None;
            }

            Some(get_lib_path(
                libraries_path,
                &library.name,
                library.natives_os_key_and_classifiers(java_arch).is_some(),
            ))
        }))
        .process_results(|iter| {
            iter.unique().join(classpath_separator(java_arch))
        })
}

pub fn get_class_paths_jar<T: AsRef<str>>(
    libraries_path: &Path,
    libraries: &[T],
    java_arch: &str,
) -> crate::Result<String> {
    let cps = libraries
        .iter()
        .map(|library| get_lib_path(libraries_path, library.as_ref(), false))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(cps.join(classpath_separator(java_arch)))
}

pub fn get_lib_path(
    libraries_path: &Path,
    lib: &str,
    allow_not_exist: bool,
) -> crate::Result<String> {
    let path = libraries_path.join(get_path_from_artifact(lib)?);

    let path = match canonicalize(&path) {
        Ok(p) => p,
        Err(err) if err.kind() == ErrorKind::NotFound && allow_not_exist => {
            path
        }
        Err(err) => {
            return Err(crate::ErrorKind::LauncherError(format!(
                "Could not canonicalize library path {}: {err}",
                path.display()
            ))
            .as_error());
        }
    };

    Ok(path.to_string_lossy().to_string())
}

#[allow(clippy::too_many_arguments)]
pub fn get_jvm_arguments(
    arguments: Option<&[Argument]>,
    natives_path: &Path,
    libraries_path: &Path,
    log_configs_path: &Path,
    class_paths: &str,
    agent_path: &Path,
    version_name: &str,
    memory: MemorySettings,
    custom_args: Vec<String>,
    java_arch: &str,
    quick_play_type: &QuickPlayType,
    quick_play_version: QuickPlayVersion,
    log_config: Option<&LoggingConfiguration>,
    ipc_addr: SocketAddr,
) -> crate::Result<Vec<String>> {
    let mut parsed_arguments = Vec::new();

    if let Some(args) = arguments {
        parse_arguments(
            args,
            &mut parsed_arguments,
            |arg| {
                parse_jvm_argument(
                    arg.to_string(),
                    natives_path,
                    libraries_path,
                    class_paths,
                    version_name,
                    java_arch,
                )
            },
            java_arch,
            quick_play_type,
        )?;
    } else {
        parsed_arguments.push(format!(
            "-Djava.library.path={}",
            canonicalize(natives_path)
                .map_err(|_| crate::ErrorKind::LauncherError(format!(
                    "Specified natives path {} does not exist",
                    natives_path.to_string_lossy()
                ))
                .as_error())?
                .to_string_lossy()
        ));
        parsed_arguments.push("-cp".to_string());
        parsed_arguments.push(class_paths.to_string());
    }

    parsed_arguments.push(format!("-Xmx{}M", memory.maximum));

    if let Some(LoggingConfiguration::Log4j2Xml { argument, file }) = log_config
    {
        let full_path = log_configs_path.join(&file.id);
        let full_path = full_path.to_string_lossy();
        parsed_arguments.push(argument.replace("${path}", &full_path));
    }

    parsed_arguments.push(format!(
        "-javaagent:{}",
        canonicalize(agent_path)
            .map_err(|_| {
                crate::ErrorKind::LauncherError(format!(
                    "Specified Java Agent path {} does not exist",
                    libraries_path.to_string_lossy()
                ))
                .as_error()
            })?
            .to_string_lossy()
    ));

    parsed_arguments
        .push(format!("-Dmodrinth.internal.ipc.host={}", ipc_addr.ip()));
    parsed_arguments
        .push(format!("-Dmodrinth.internal.ipc.port={}", ipc_addr.port()));

    parsed_arguments.push(format!(
        "-Dmodrinth.internal.quickPlay.serverVersion={}",
        serde_json::to_value(quick_play_version.server)?
            .as_str()
            .unwrap()
    ));
    if let QuickPlayType::Server(server) = quick_play_type
        && quick_play_version.server == QuickPlayServerVersion::Injected
    {
        let (host, port) = server.require_resolved()?;
        parsed_arguments.extend_from_slice(&[
            format!("-Dmodrinth.internal.quickPlay.host={host}"),
            format!("-Dmodrinth.internal.quickPlay.port={port}"),
        ]);
    }

    for arg in custom_args {
        if !arg.is_empty() {
            parsed_arguments.push(arg);
        }
    }

    Ok(parsed_arguments)
}

fn parse_jvm_argument(
    mut argument: String,
    natives_path: &Path,
    libraries_path: &Path,
    class_paths: &str,
    version_name: &str,
    java_arch: &str,
) -> crate::Result<String> {
    argument.retain(|c| !c.is_whitespace());
    Ok(argument
        .replace(
            "${natives_directory}",
            &canonicalize(natives_path)
                .map_err(|_| {
                    crate::ErrorKind::LauncherError(format!(
                        "Specified natives path {} does not exist",
                        natives_path.to_string_lossy()
                    ))
                    .as_error()
                })?
                .to_string_lossy(),
        )
        .replace(
            "${library_directory}",
            &canonicalize(libraries_path)
                .map_err(|_| {
                    crate::ErrorKind::LauncherError(format!(
                        "Specified libraries path {} does not exist",
                        libraries_path.to_string_lossy()
                    ))
                    .as_error()
                })?
                .to_string_lossy(),
        )
        .replace("${classpath_separator}", classpath_separator(java_arch))
        .replace("${launcher_name}", "theseus")
        .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
        .replace("${version_name}", version_name)
        .replace("${classpath}", class_paths))
}

#[allow(clippy::too_many_arguments)]
pub async fn get_minecraft_arguments(
    arguments: Option<&[Argument]>,
    legacy_arguments: Option<&str>,
    credentials: &Credentials,
    version: &str,
    asset_index_name: &str,
    game_directory: &Path,
    assets_directory: &Path,
    version_type: &VersionType,
    resolution: WindowSize,
    java_arch: &str,
    quick_play_type: &QuickPlayType,
    quick_play_version: QuickPlayVersion,
) -> crate::Result<Vec<String>> {
    let access_token = credentials.access_token.clone();
    let profile = credentials.maybe_online_profile().await;
    let mut parsed_arguments = Vec::new();

    if let Some(arguments) = arguments {
        parse_arguments(
            arguments,
            &mut parsed_arguments,
            |arg| {
                parse_minecraft_argument(
                    arg,
                    &access_token,
                    &profile.name,
                    profile.id,
                    version,
                    asset_index_name,
                    game_directory,
                    assets_directory,
                    version_type,
                    resolution,
                    quick_play_type,
                )
            },
            java_arch,
            quick_play_type,
        )?;
    } else if let Some(legacy_arguments) = legacy_arguments {
        for x in legacy_arguments.split(' ') {
            parsed_arguments.push(parse_minecraft_argument(
                &x.replace(' ', TEMPORARY_REPLACE_CHAR),
                &access_token,
                &profile.name,
                profile.id,
                version,
                asset_index_name,
                game_directory,
                assets_directory,
                version_type,
                resolution,
                quick_play_type,
            )?);
        }
    }

    if let QuickPlayType::Server(server) = quick_play_type
        && quick_play_version.server == QuickPlayServerVersion::BuiltinLegacy
    {
        let (host, port) = server.require_resolved()?;
        parsed_arguments.extend_from_slice(&[
            "--server".to_string(),
            host.to_string(),
            "--port".to_string(),
            port.to_string(),
        ]);
    }

    Ok(parsed_arguments)
}

#[allow(clippy::too_many_arguments)]
fn parse_minecraft_argument(
    argument: &str,
    access_token: &str,
    username: &str,
    uuid: Uuid,
    version: &str,
    asset_index_name: &str,
    game_directory: &Path,
    assets_directory: &Path,
    version_type: &VersionType,
    resolution: WindowSize,
    quick_play_type: &QuickPlayType,
) -> crate::Result<String> {
    Ok(argument
        .replace("${accessToken}", access_token)
        .replace("${auth_access_token}", access_token)
        .replace("${auth_session}", access_token)
        .replace("${auth_player_name}", username)
        // TODO: add auth xuid eventually
        .replace("${auth_xuid}", "0")
        .replace("${auth_uuid}", &uuid.simple().to_string())
        .replace("${uuid}", &uuid.simple().to_string())
        .replace("${clientid}", "c4502edb-87c6-40cb-b595-64a280cf8906")
        .replace("${user_properties}", "{}")
        .replace("${user_type}", "msa")
        .replace("${version_name}", version)
        .replace("${assets_index_name}", asset_index_name)
        .replace(
            "${game_directory}",
            &canonicalize(game_directory)
                .map_err(|_| {
                    crate::ErrorKind::LauncherError(format!(
                        "Specified game directory {} does not exist",
                        game_directory.to_string_lossy()
                    ))
                    .as_error()
                })?
                .to_string_lossy(),
        )
        .replace(
            "${assets_root}",
            &canonicalize(assets_directory)
                .map_err(|_| {
                    crate::ErrorKind::LauncherError(format!(
                        "Specified assets directory {} does not exist",
                        assets_directory.to_string_lossy()
                    ))
                    .as_error()
                })?
                .to_string_lossy(),
        )
        .replace(
            "${game_assets}",
            &canonicalize(assets_directory)
                .map_err(|_| {
                    crate::ErrorKind::LauncherError(format!(
                        "Specified assets directory {} does not exist",
                        assets_directory.to_string_lossy()
                    ))
                    .as_error()
                })?
                .to_string_lossy(),
        )
        .replace("${version_type}", version_type.as_str())
        .replace("${resolution_width}", &resolution.0.to_string())
        .replace("${resolution_height}", &resolution.1.to_string())
        .replace(
            "${quickPlaySingleplayer}",
            match quick_play_type {
                QuickPlayType::Singleplayer(world) => world,
                _ => "",
            },
        )
        .replace(
            "${quickPlayMultiplayer}",
            &match quick_play_type {
                QuickPlayType::Server(address) => address.to_string(),
                _ => "".to_string(),
            },
        ))
}

fn parse_arguments<F>(
    arguments: &[Argument],
    parsed_arguments: &mut Vec<String>,
    parse_function: F,
    java_arch: &str,
    quick_play_type: &QuickPlayType,
) -> crate::Result<()>
where
    F: Fn(&str) -> crate::Result<String>,
{
    for argument in arguments {
        match argument {
            Argument::Normal(arg) => {
                let parsed =
                    parse_function(&arg.replace(' ', TEMPORARY_REPLACE_CHAR))?;
                for arg in parsed.split(TEMPORARY_REPLACE_CHAR) {
                    parsed_arguments.push(arg.to_string());
                }
            }
            Argument::Ruled { rules, value } => {
                if parse_rules(rules, java_arch, quick_play_type, true) {
                    match value {
                        ArgumentValue::Single(arg) => {
                            parsed_arguments.push(parse_function(
                                &arg.replace(' ', TEMPORARY_REPLACE_CHAR),
                            )?);
                        }
                        ArgumentValue::Many(args) => {
                            for arg in args {
                                parsed_arguments.push(parse_function(
                                    &arg.replace(' ', TEMPORARY_REPLACE_CHAR),
                                )?);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn get_processor_arguments<T: AsRef<str>>(
    libraries_path: &Path,
    arguments: &[T],
    data: &HashMap<String, SidedDataEntry>,
) -> crate::Result<Vec<String>> {
    let mut new_arguments = Vec::new();

    for argument in arguments {
        let trimmed_arg = &argument.as_ref()[1..argument.as_ref().len() - 1];
        if argument.as_ref().starts_with('{') {
            if let Some(entry) = data.get(trimmed_arg) {
                new_arguments.push(if entry.client.starts_with('[') {
                    get_lib_path(
                        libraries_path,
                        &entry.client[1..entry.client.len() - 1],
                        true,
                    )?
                } else {
                    entry.client.clone()
                })
            }
        } else if argument.as_ref().starts_with('[') {
            new_arguments.push(get_lib_path(libraries_path, trimmed_arg, true)?)
        } else {
            new_arguments.push(argument.as_ref().to_string())
        }
    }

    Ok(new_arguments)
}

pub async fn get_processor_main_class(
    path: String,
) -> crate::Result<Option<String>> {
    let main_class = tokio::task::spawn_blocking(move || {
        let zipfile = std::fs::File::open(&path)
            .map_err(|e| IOError::with_path(e, &path))?;
        let mut archive = zip::ZipArchive::new(zipfile).map_err(|_| {
            crate::ErrorKind::LauncherError(format!(
                "Cannot read processor at {path}"
            ))
            .as_error()
        })?;

        let file = archive.by_name("META-INF/MANIFEST.MF").map_err(|_| {
            crate::ErrorKind::LauncherError(format!(
                "Cannot read processor manifest at {path}"
            ))
            .as_error()
        })?;

        let reader = BufReader::new(file);

        for line in reader.lines() {
            let mut line = line.map_err(IOError::from)?;
            line.retain(|c| !c.is_whitespace());

            if line.starts_with("Main-Class:")
                && let Some(class) = line.split(':').nth(1)
            {
                return Ok(Some(class.to_string()));
            }
        }

        Ok::<Option<String>, crate::Error>(None)
    })
    .await??;

    Ok(main_class)
}

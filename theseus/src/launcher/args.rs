use crate::launcher::auth::provider::Credentials;
use crate::launcher::rules::parse_rules;
use crate::launcher::LauncherError;
use daedalus::get_path_from_artifact;
use daedalus::minecraft::{Argument, ArgumentValue, Library, Os, VersionType};
use daedalus::modded::SidedDataEntry;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use uuid::Uuid;

fn get_cp_separator() -> &'static str {
    match super::download::get_os() {
        Os::Osx | Os::Linux | Os::Unknown => ":",
        Os::Windows => ";",
    }
}

pub fn get_class_paths(
    libraries_path: &Path,
    libraries: &[Library],
    client_path: &Path,
) -> Result<String, LauncherError> {
    let mut class_paths = Vec::new();

    for library in libraries {
        if let Some(rules) = &library.rules {
            if !super::rules::parse_rules(rules.as_slice()) {
                continue;
            }
        }

        if !library.include_in_classpath {
            continue;
        }

        class_paths.push(get_lib_path(libraries_path, &library.name)?);
    }

    class_paths.push(
        crate::util::absolute_path(&client_path)
            .map_err(|_| {
                LauncherError::InvalidInput(format!(
                    "Specified class path {} does not exist",
                    client_path.to_string_lossy()
                ))
            })?
            .to_string_lossy()
            .to_string(),
    );

    Ok(class_paths.join(get_cp_separator()))
}

pub fn get_class_paths_jar<T: AsRef<str>>(
    libraries_path: &Path,
    libraries: &[T],
) -> Result<String, LauncherError> {
    let mut class_paths = Vec::new();

    for library in libraries {
        class_paths.push(get_lib_path(libraries_path, library)?)
    }

    Ok(class_paths.join(match super::download::get_os() {
        Os::Osx | Os::Linux | Os::Unknown => ":",
        Os::Windows => ";",
    }))
}

pub fn get_lib_path<T: AsRef<str>>(libraries_path: &Path, lib: T) -> Result<String, LauncherError> {
    let mut path = libraries_path.to_path_buf();

    path.push(get_path_from_artifact(lib.as_ref())?);

    let path = crate::util::absolute_path(&path).map_err(|_| {
        LauncherError::InvalidInput(format!(
            "Library file at path {} does not exist",
            path.to_string_lossy()
        ))
    })?;

    /*if !path.exists() {
        if let Some(parent) = &path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::File::create(&path)?;
    }*/

    Ok(path.to_string_lossy().to_string())
}

pub fn get_jvm_arguments(
    arguments: Option<&[Argument]>,
    natives_path: &Path,
    libraries_path: &Path,
    class_paths: &str,
    version_name: &str,
    memory: i32,
    custom_args: Vec<String>,
) -> Result<Vec<String>, LauncherError> {
    let mut parsed_arguments = Vec::new();

    if let Some(args) = arguments {
        parse_arguments(args, &mut parsed_arguments, |arg| {
            parse_jvm_argument(
                arg,
                natives_path,
                libraries_path,
                class_paths,
                version_name,
            )
        })?;
    } else {
        parsed_arguments.push(format!(
            "-Djava.library.path={}",
            &*crate::util::absolute_path(natives_path)
                .map_err(|_| LauncherError::InvalidInput(format!(
                    "Specified natives path {} does not exist",
                    natives_path.to_string_lossy()
                )))?
                .to_string_lossy()
                .to_string()
        ));
        parsed_arguments.push("-cp".to_string());
        parsed_arguments.push(class_paths.to_string());
    }

    parsed_arguments.push(format!("-Xmx{}M", memory));
    for arg in custom_args {
        if !arg.is_empty() {
            parsed_arguments.push(arg);
        }
    }

    Ok(parsed_arguments)
}

fn parse_jvm_argument(
    argument: &str,
    natives_path: &Path,
    libraries_path: &Path,
    class_paths: &str,
    version_name: &str,
) -> Result<String, LauncherError> {
    let mut argument = argument.to_string();
    argument.retain(|c| !c.is_whitespace());
    Ok(argument
        .replace(
            "${natives_directory}",
            &*crate::util::absolute_path(natives_path)
                .map_err(|_| {
                    LauncherError::InvalidInput(format!(
                        "Specified natives path {} does not exist",
                        natives_path.to_string_lossy()
                    ))
                })?
                .to_string_lossy()
                .to_string(),
        )
        .replace(
            "${library_directory}",
            &*crate::util::absolute_path(libraries_path)
                .map_err(|_| {
                    LauncherError::InvalidInput(format!(
                        "Specified libraries path {} does not exist",
                        libraries_path.to_string_lossy()
                    ))
                })?
                .to_string_lossy()
                .to_string(),
        )
        .replace("${classpath_separator}", get_cp_separator())
        .replace("${launcher_name}", "theseus")
        .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
        .replace("${version_name}", version_name)
        .replace("${classpath}", class_paths))
}

#[allow(clippy::too_many_arguments)]
pub fn get_minecraft_arguments(
    arguments: Option<&[Argument]>,
    legacy_arguments: Option<&str>,
    credentials: &Credentials,
    version: &str,
    asset_index_name: &str,
    game_directory: &Path,
    assets_directory: &Path,
    version_type: &VersionType,
    resolution: (i32, i32),
) -> Result<Vec<String>, LauncherError> {
    if let Some(arguments) = arguments {
        let mut parsed_arguments = Vec::new();

        parse_arguments(arguments, &mut parsed_arguments, |arg| {
            parse_minecraft_argument(
                arg,
                &*credentials.access_token,
                &*credentials.username,
                &credentials.id,
                version,
                asset_index_name,
                game_directory,
                assets_directory,
                version_type,
                resolution,
            )
        })?;

        Ok(parsed_arguments)
    } else if let Some(legacy_arguments) = legacy_arguments {
        Ok(parse_minecraft_argument(
            legacy_arguments,
            &*credentials.access_token,
            &*credentials.username,
            &credentials.id,
            version,
            asset_index_name,
            game_directory,
            assets_directory,
            version_type,
            resolution,
        )?
        .split(' ')
        .into_iter()
        .map(|x| x.to_string())
        .collect())
    } else {
        Ok(Vec::new())
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_minecraft_argument(
    argument: &str,
    access_token: &str,
    username: &str,
    uuid: &Uuid,
    version: &str,
    asset_index_name: &str,
    game_directory: &Path,
    assets_directory: &Path,
    version_type: &VersionType,
    resolution: (i32, i32),
) -> Result<String, LauncherError> {
    Ok(argument
        .replace("${auth_access_token}", access_token)
        .replace("${auth_session}", access_token)
        .replace("${auth_player_name}", username)
        .replace("${auth_uuid}", &*uuid.to_hyphenated().to_string())
        .replace("${user_properties}", "{}")
        .replace("${user_type}", "mojang")
        .replace("${version_name}", version)
        .replace("${assets_index_name}", asset_index_name)
        .replace(
            "${game_directory}",
            &*crate::util::absolute_path(game_directory)
                .map_err(|_| {
                    LauncherError::InvalidInput(format!(
                        "Specified game directory {} does not exist",
                        game_directory.to_string_lossy()
                    ))
                })?
                .to_string_lossy()
                .to_string(),
        )
        .replace(
            "${assets_root}",
            &*crate::util::absolute_path(assets_directory)
                .map_err(|_| {
                    LauncherError::InvalidInput(format!(
                        "Specified assets directory {} does not exist",
                        assets_directory.to_string_lossy()
                    ))
                })?
                .to_string_lossy()
                .to_string(),
        )
        .replace(
            "${game_assets}",
            &*crate::util::absolute_path(assets_directory)
                .map_err(|_| {
                    LauncherError::InvalidInput(format!(
                        "Specified assets directory {} does not exist",
                        assets_directory.to_string_lossy()
                    ))
                })?
                .to_string_lossy()
                .to_string(),
        )
        .replace("${version_type}", version_type.as_str())
        .replace("${resolution_width}", &*resolution.0.to_string())
        .replace("${resolution_height}", &*resolution.1.to_string()))
}

fn parse_arguments<F>(
    arguments: &[Argument],
    parsed_arguments: &mut Vec<String>,
    parse_function: F,
) -> Result<(), LauncherError>
where
    F: Fn(&str) -> Result<String, LauncherError>,
{
    for argument in arguments {
        match argument {
            Argument::Normal(arg) => {
                let parsed = parse_function(arg)?;

                for arg in parsed.split(' ') {
                    parsed_arguments.push(arg.to_string());
                }
            }
            Argument::Ruled { rules, value } => {
                if parse_rules(rules.as_slice()) {
                    match value {
                        ArgumentValue::Single(arg) => {
                            parsed_arguments.push(parse_function(arg)?);
                        }
                        ArgumentValue::Many(args) => {
                            for arg in args {
                                parsed_arguments.push(parse_function(arg)?);
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
) -> Result<Vec<String>, LauncherError> {
    let mut new_arguments = Vec::new();

    for argument in arguments {
        let trimmed_arg = &argument.as_ref()[1..argument.as_ref().len() - 1];
        if argument.as_ref().starts_with('{') {
            if let Some(entry) = data.get(trimmed_arg) {
                new_arguments.push(if entry.client.starts_with('[') {
                    get_lib_path(libraries_path, &entry.client[1..entry.client.len() - 1])?
                } else {
                    entry.client.clone()
                })
            }
        } else if argument.as_ref().starts_with('[') {
            new_arguments.push(get_lib_path(libraries_path, trimmed_arg)?)
        } else {
            new_arguments.push(argument.as_ref().to_string())
        }
    }

    Ok(new_arguments)
}

pub async fn get_processor_main_class(path: String) -> Result<Option<String>, LauncherError> {
    Ok(tokio::task::spawn_blocking(move || {
        let zipfile = std::fs::File::open(&path)?;
        let mut archive = zip::ZipArchive::new(zipfile).map_err(|_| {
            LauncherError::ProcessorError(format!("Cannot read processor at {}", path))
        })?;

        let file = archive.by_name("META-INF/MANIFEST.MF").map_err(|_| {
            LauncherError::ProcessorError(format!("Cannot read processor manifest at {}", path))
        })?;

        let reader = BufReader::new(file);

        for line in reader.lines() {
            let mut line = line?;
            line.retain(|c| !c.is_whitespace());

            if line.starts_with("Main-Class:") {
                if let Some(class) = line.split(':').nth(1) {
                    return Ok(Some(class.to_string()));
                }
            }
        }

        Ok::<Option<String>, LauncherError>(None)
    })
    .await??)
}

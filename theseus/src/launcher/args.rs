use crate::launcher::auth::provider::Credentials;
use crate::launcher::meta::{Argument, ArgumentValue, Library, Os, VersionType};
use crate::launcher::rules::parse_rules;
use crate::launcher::LauncherError;
use std::path::Path;
use uuid::Uuid;

pub fn get_class_paths(
    libraries_path: &Path,
    libraries: &[Library],
    client_path: &Path,
) -> Result<String, LauncherError> {
    let mut class_paths = Vec::new();

    for library in libraries {
        if library.downloads.artifact.is_some() {
            if let Some(rules) = &library.rules {
                if !super::rules::parse_rules(rules.as_slice()) {
                    continue;
                }
            }

            let name_items = library.name.split(':').collect::<Vec<&str>>();

            let package = name_items.get(0).ok_or_else(|| {
                LauncherError::ParseError(format!(
                    "Unable to find package for library {}",
                    &library.name
                ))
            })?;
            let name = name_items.get(1).ok_or_else(|| {
                LauncherError::ParseError(format!(
                    "Unable to find name for library {}",
                    &library.name
                ))
            })?;
            let version = name_items.get(2).ok_or_else(|| {
                LauncherError::ParseError(format!(
                    "Unable to find version for library {}",
                    &library.name
                ))
            })?;

            let mut path = libraries_path.to_path_buf();

            for directory in package.split('.') {
                path.push(directory);
            }

            path.push(name);
            path.push(version);
            path.push(format!("{}-{}.jar", name, version));

            class_paths.push(
                dunce::canonicalize(&path)
                    .map_err(|_| {
                        LauncherError::InvalidInput(format!(
                            "Library file at path {} does not exist",
                            path.to_string_lossy()
                        ))
                    })?
                    .to_string_lossy()
                    .to_string(),
            )
        }
    }

    class_paths.push(
        dunce::canonicalize(&client_path)
            .map_err(|_| {
                LauncherError::InvalidInput(format!(
                    "Specified client path {} does not exist",
                    client_path.to_string_lossy()
                ))
            })?
            .to_string_lossy()
            .to_string(),
    );

    Ok(class_paths.join(match super::download::get_os() {
        Os::Osx | Os::Linux | Os::Unknown => ":",
        Os::Windows => ";",
    }))
}

pub fn get_jvm_arguments(
    arguments: Option<&[Argument]>,
    natives_path: &Path,
    class_paths: &str,
) -> Result<Vec<String>, LauncherError> {
    let mut parsed_arguments = Vec::new();

    if let Some(args) = arguments {
        parse_arguments(args, &mut parsed_arguments, |arg| {
            parse_jvm_argument(arg, natives_path, class_paths)
        })?;
    } else {
        parsed_arguments.push(format!(
            "-Djava.library.path={}",
            &*dunce::canonicalize(natives_path)
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

    Ok(parsed_arguments)
}

fn parse_jvm_argument(
    argument: &str,
    natives_path: &Path,
    class_paths: &str,
) -> Result<String, LauncherError> {
    Ok(argument
        .replace(
            "${natives_directory}",
            &*dunce::canonicalize(natives_path)
                .map_err(|_| {
                    LauncherError::InvalidInput(format!(
                        "Specified natives path {} does not exist",
                        natives_path.to_string_lossy()
                    ))
                })?
                .to_string_lossy()
                .to_string(),
        )
        .replace("${launcher_name}", "theseus")
        .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
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
            &*dunce::canonicalize(game_directory)
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
            &*dunce::canonicalize(assets_directory)
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
            &*dunce::canonicalize(assets_directory)
                .map_err(|_| {
                    LauncherError::InvalidInput(format!(
                        "Specified assets directory {} does not exist",
                        assets_directory.to_string_lossy()
                    ))
                })?
                .to_string_lossy()
                .to_string(),
        )
        .replace("${version_type}", version_type.as_str()))
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

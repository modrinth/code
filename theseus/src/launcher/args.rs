use crate::launcher::meta::{Argument, ArgumentValue, Library, Os, VersionType};
use crate::launcher::rules::parse_rules;
use std::path::Path;
use uuid::Uuid;

pub fn get_class_paths(libraries_path: &Path, libraries: &[Library], client_path: &Path) -> String {
    let mut class_paths = Vec::new();

    for library in libraries {
        if library.downloads.artifact.is_some() {
            if let Some(rules) = &library.rules {
                if !super::rules::parse_rules(rules.as_slice()) {
                    continue;
                }
            }

            let name_items = library.name.split(':').collect::<Vec<&str>>();

            let package = name_items.get(0).unwrap();
            let name = name_items.get(1).unwrap();
            let version = name_items.get(2).unwrap();

            let mut path = libraries_path.to_path_buf();

            for directory in package.split(".") {
                path.push(directory);
            }

            path.push(name);
            path.push(version);
            path.push(format!("{}-{}.jar", name, version));

            class_paths.push(
                std::fs::canonicalize(&path)
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            )
        }
    }

    class_paths.push(
        std::fs::canonicalize(&client_path)
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );

    class_paths.join(match super::download::get_os() {
        Os::Osx | Os::Linux | Os::Unknown => ":",
        Os::Windows => ";",
    })
}

pub fn get_jvm_arguments(
    arguments: Option<&[Argument]>,
    natives_path: &Path,
    class_paths: &str,
) -> Vec<String> {
    let mut parsed_arguments = Vec::new();

    if let Some(args) = arguments {
        parse_arguments(args, &mut parsed_arguments, |arg| {
            parse_jvm_argument(arg, natives_path, class_paths)
        });
    } else {
        parsed_arguments.push(format!(
            "-Djava.library.path={}",
            &*std::fs::canonicalize(natives_path)
                .unwrap()
                .to_string_lossy()
                .to_string()
        ));
        parsed_arguments.push("-cp".to_string());
        parsed_arguments.push(class_paths.to_string());
    }

    parsed_arguments
}

fn parse_jvm_argument(argument: &str, natives_path: &Path, class_paths: &str) -> String {
    argument
        .replace(
            "${natives_directory}",
            &*std::fs::canonicalize(natives_path)
                .unwrap()
                .to_string_lossy()
                .to_string(),
        )
        .replace("${launcher_name}", "theseus")
        .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
        .replace("${classpath}", class_paths)
}

pub fn get_minecraft_arguments(
    arguments: Option<&[Argument]>,
    legacy_arguments: Option<&str>,
    access_token: &str,
    username: &str,
    uuid: &Uuid,
    version: &str,
    asset_index_name: &str,
    game_directory: &Path,
    assets_directory: &Path,
    version_type: &VersionType,
) -> Vec<String> {
    if let Some(arguments) = arguments {
        let mut parsed_arguments = Vec::new();

        parse_arguments(arguments, &mut parsed_arguments, |arg| {
            parse_minecraft_argument(
                arg,
                access_token,
                username,
                uuid,
                version,
                asset_index_name,
                game_directory,
                assets_directory,
                version_type,
            )
        });

        parsed_arguments
    } else if let Some(legacy_arguments) = legacy_arguments {
        parse_minecraft_argument(
            legacy_arguments,
            access_token,
            username,
            uuid,
            version,
            asset_index_name,
            game_directory,
            assets_directory,
            version_type,
        )
        .split(" ")
        .into_iter()
        .map(|x| x.to_string())
        .collect()
    } else {
        Vec::new()
    }
}

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
) -> String {
    argument
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
            &*std::fs::canonicalize(game_directory)
                .unwrap()
                .to_string_lossy()
                .to_string(),
        )
        .replace(
            "${assets_root}",
            &*std::fs::canonicalize(assets_directory)
                .unwrap()
                .to_string_lossy()
                .to_string(),
        )
        .replace(
            "${game_assets}",
            &*std::fs::canonicalize(assets_directory)
                .unwrap()
                .to_string_lossy()
                .to_string(),
        )
        .replace("${version_type}", version_type.as_str())
}

fn parse_arguments<F>(arguments: &[Argument], parsed_arguments: &mut Vec<String>, parse_function: F)
where
    F: Fn(&str) -> String,
{
    for argument in arguments {
        match argument {
            Argument::Normal(arg) => {
                let parsed = parse_function(arg);

                for arg in parsed.split(" ") {
                    parsed_arguments.push(arg.to_string());
                }
            }
            Argument::Ruled { rules, value } => {
                if parse_rules(rules.as_slice()) {
                    match value {
                        ArgumentValue::Single(arg) => {
                            //parsed_arguments.push(parse_function(arg));
                        }
                        ArgumentValue::Many(args) => {
                            for arg in args {
                                //parsed_arguments.push(parse_function(arg));
                            }
                        }
                    }
                }
            }
        }
    }
}

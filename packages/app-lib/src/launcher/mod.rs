//! Logic for launching Minecraft
use crate::data::ModLoader;
use crate::event::emit::{emit_loading, init_or_edit_loading};
use crate::event::{LoadingBarId, LoadingBarType};
use crate::launcher::download::download_log_config;
use crate::launcher::io::IOError;
use crate::profile::QuickPlayType;
use crate::state::{
    Credentials, JavaVersion, ProcessMetadata, ProfileInstallStage,
};
use crate::util::io;
use crate::{State, get_resource_file, process, state as st};
use chrono::Utc;
use daedalus as d;
use daedalus::minecraft::{LoggingSide, RuleAction, VersionInfo};
use daedalus::modded::LoaderVersion;
use regex::Regex;
use serde::Deserialize;
use st::Profile;
use std::fmt::Write;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

mod args;

pub mod download;

// All nones -> disallowed
// 1+ true -> allowed
// 1+ false -> disallowed
#[tracing::instrument]
pub fn parse_rules(
    rules: &[d::minecraft::Rule],
    java_version: &str,
    quick_play_type: &QuickPlayType,
    minecraft_updated: bool,
) -> bool {
    let mut x = rules
        .iter()
        .map(|x| {
            parse_rule(x, java_version, quick_play_type, minecraft_updated)
        })
        .collect::<Vec<Option<bool>>>();

    if rules
        .iter()
        .all(|x| matches!(x.action, RuleAction::Disallow))
    {
        x.push(Some(true))
    }

    !(x.iter().any(|x| x == &Some(false)) || x.iter().all(|x| x.is_none()))
}

// if anything is disallowed, it should NOT be included
// if anything is not disallowed, it shouldn't factor in final result
// if anything is not allowed, it should NOT be included
// if anything is allowed, it should be included
#[tracing::instrument]
pub fn parse_rule(
    rule: &d::minecraft::Rule,
    java_version: &str,
    quick_play_type: &QuickPlayType,
    minecraft_updated: bool,
) -> Option<bool> {
    use d::minecraft::{Rule, RuleAction};

    let res = match rule {
        Rule { os: Some(os), .. } => {
            crate::util::platform::os_rule(os, java_version, minecraft_updated)
        }
        Rule {
            features: Some(features),
            ..
        } => {
            !features.is_demo_user.unwrap_or(true)
                || features.has_custom_resolution.unwrap_or(false)
                || !features.has_quick_plays_support.unwrap_or(true)
                || (features.is_quick_play_singleplayer.unwrap_or(false)
                    && matches!(
                        quick_play_type,
                        QuickPlayType::Singleplayer(_)
                    ))
                || (features.is_quick_play_multiplayer.unwrap_or(false)
                    && matches!(quick_play_type, QuickPlayType::Server(..)))
                || !features.is_quick_play_realms.unwrap_or(true)
        }
        _ => return Some(true),
    };

    match rule.action {
        RuleAction::Allow => {
            if res {
                Some(true)
            } else {
                Some(false)
            }
        }
        RuleAction::Disallow => {
            if res {
                Some(false)
            } else {
                None
            }
        }
    }
}

macro_rules! processor_rules {
    ($dest:expr; $($name:literal : client => $client:expr, server => $server:expr;)+) => {
        $(std::collections::HashMap::insert(
            $dest,
            String::from($name),
            daedalus::modded::SidedDataEntry {
                client: String::from($client),
                server: String::from($server),
            },
        );)+
    }
}

pub async fn get_java_version_from_profile(
    profile: &Profile,
    version_info: &VersionInfo,
) -> crate::Result<Option<JavaVersion>> {
    if let Some(java) = profile.java_path.as_ref() {
        let java =
            crate::api::jre::check_jre(std::path::PathBuf::from(java)).await;

        if let Ok(java) = java {
            return Ok(Some(java));
        }
    }

    let key = version_info
        .java_version
        .as_ref()
        .map_or(8, |it| it.major_version);

    let state = State::get().await?;

    let java_version = JavaVersion::get(key, &state.pool).await?;

    Ok(java_version)
}

pub async fn get_loader_version_from_profile(
    game_version: &str,
    loader: ModLoader,
    loader_version: Option<&str>,
) -> crate::Result<Option<LoaderVersion>> {
    if loader == ModLoader::Vanilla {
        return Ok(None);
    }

    let version = loader_version.unwrap_or("latest");

    let filter = |it: &LoaderVersion| match version {
        "latest" => true,
        "stable" => it.stable,
        id => it.id == *id,
    };

    let versions =
        crate::api::metadata::get_loader_versions(loader.as_meta_str()).await?;

    let loaders = versions.game_versions.into_iter().find(|x| {
        x.id.replace(daedalus::modded::DUMMY_REPLACE_STRING, game_version)
            == game_version
    });

    if let Some(loaders) = loaders {
        let loader_version = loaders.loaders.iter().find(|x| filter(x)).or(
            if version == "stable" {
                loaders.loaders.first()
            } else {
                None
            },
        );

        Ok(loader_version.cloned())
    } else {
        Ok(None)
    }
}

#[tracing::instrument(skip(profile))]

pub async fn install_minecraft(
    profile: &Profile,
    existing_loading_bar: Option<LoadingBarId>,
    repairing: bool,
) -> crate::Result<()> {
    let loading_bar = init_or_edit_loading(
        existing_loading_bar,
        LoadingBarType::MinecraftDownload {
            // If we are downloading minecraft for a profile, provide its name and uuid
            profile_name: profile.name.clone(),
            profile_path: profile.path.clone(),
        },
        100.0,
        "Downloading Minecraft",
    )
    .await?;

    crate::api::profile::edit(&profile.path, |prof| {
        prof.install_stage = ProfileInstallStage::MinecraftInstalling;

        async { Ok(()) }
    })
    .await?;

    let state = State::get().await?;

    let instance_path =
        crate::api::profile::get_full_path(&profile.path).await?;
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
    let minecraft_updated = version_index
        <= minecraft
            .versions
            .iter()
            .position(|x| x.id == "22w16a")
            .unwrap_or(0);

    let mut loader_version = get_loader_version_from_profile(
        &profile.game_version,
        profile.loader,
        profile.loader_version.as_deref(),
    )
    .await?;

    // If no loader version is selected, try to select the stable version!
    if profile.loader != ModLoader::Vanilla && loader_version.is_none() {
        loader_version = get_loader_version_from_profile(
            &profile.game_version,
            profile.loader,
            Some("stable"),
        )
        .await?;

        let loader_version_id = loader_version.clone();
        crate::api::profile::edit(&profile.path, |prof| {
            prof.loader_version = loader_version_id.clone().map(|x| x.id);

            async { Ok(()) }
        })
        .await?;
    }

    let version_jar =
        loader_version.as_ref().map_or(version.id.clone(), |it| {
            format!("{}-{}", version.id.clone(), it.id.clone())
        });

    // Download version info (5)
    let mut version_info = download::download_version_info(
        &state,
        version,
        loader_version.as_ref(),
        Some(repairing),
        Some(&loading_bar),
    )
    .await?;

    let key = version_info
        .java_version
        .as_ref()
        .map_or(8, |it| it.major_version);
    let (java_version, set_java) = if let Some(java_version) =
        get_java_version_from_profile(profile, &version_info).await?
    {
        (std::path::PathBuf::from(java_version.path), false)
    } else {
        let path = crate::api::jre::auto_install_java(key).await?;

        (path, true)
    };

    // Test jre version
    let java_version = crate::api::jre::check_jre(java_version.clone()).await?;

    if set_java {
        java_version.upsert(&state.pool).await?;
    }

    // Download minecraft (5-90)
    download::download_minecraft(
        &state,
        &version_info,
        &loading_bar,
        &java_version.architecture,
        repairing,
        minecraft_updated,
    )
    .await?;

    let client_path = state
        .directories
        .version_dir(&version_jar)
        .join(format!("{version_jar}.jar"));
    if let Some(processors) = &version_info.processors {
        let libraries_dir = state.directories.libraries_dir();

        if let Some(ref mut data) = version_info.data {
            processor_rules! {
                data;
                "SIDE":
                    client => "client",
                    server => "";
                "MINECRAFT_JAR" :
                    client => client_path.to_string_lossy(),
                    server => "";
                "MINECRAFT_VERSION":
                    client => profile.game_version.clone(),
                    server => "";
                "ROOT":
                    client => instance_path.to_string_lossy(),
                    server => "";
                "LIBRARY_DIR":
                    client => libraries_dir.to_string_lossy(),
                    server => "";
            }

            emit_loading(&loading_bar, 0.0, Some("Running forge processors"))?;
            let total_length = processors.len();

            // Forge processors (90-100)
            for (index, processor) in processors.iter().enumerate() {
                if let Some(sides) = &processor.sides {
                    if !sides.contains(&String::from("client")) {
                        continue;
                    }
                }

                let cp = {
                    let mut cp = processor.classpath.clone();
                    cp.push(processor.jar.clone());
                    cp
                };

                let child = Command::new(&java_version.path)
                    .arg("-cp")
                    .arg(args::get_class_paths_jar(
                        &libraries_dir,
                        &cp,
                        &java_version.architecture,
                    )?)
                    .arg(
                        args::get_processor_main_class(args::get_lib_path(
                            &libraries_dir,
                            &processor.jar,
                            false,
                        )?)
                        .await?
                        .ok_or_else(|| {
                            crate::ErrorKind::LauncherError(format!(
                                "Could not find processor main class for {}",
                                processor.jar
                            ))
                        })?,
                    )
                    .args(args::get_processor_arguments(
                        &libraries_dir,
                        &processor.args,
                        data,
                    )?)
                    .output()
                    .await
                    .map_err(|e| IOError::with_path(e, &java_version.path))
                    .map_err(|err| {
                        crate::ErrorKind::LauncherError(format!(
                            "Error running processor: {err}",
                        ))
                    })?;

                if !child.status.success() {
                    return Err(crate::ErrorKind::LauncherError(format!(
                        "Processor error: {}",
                        String::from_utf8_lossy(&child.stderr)
                    ))
                    .as_error());
                }

                emit_loading(
                    &loading_bar,
                    30.0 / total_length as f64,
                    Some(&format!(
                        "Running forge processor {index}/{total_length}"
                    )),
                )?;
            }
        }
    }

    let protocol_version = read_protocol_version_from_jar(client_path).await?;

    crate::api::profile::edit(&profile.path, |prof| {
        prof.install_stage = ProfileInstallStage::Installed;
        prof.protocol_version = protocol_version;

        async { Ok(()) }
    })
    .await?;
    emit_loading(&loading_bar, 1.0, Some("Finished installing"))?;

    Ok(())
}

pub async fn read_protocol_version_from_jar(
    path: PathBuf,
) -> crate::Result<Option<i32>> {
    let zip = async_zip::tokio::read::fs::ZipFileReader::new(path).await?;
    let Some(entry_index) = zip
        .file()
        .entries()
        .iter()
        .position(|x| matches!(x.filename().as_str(), Ok("version.json")))
    else {
        return Ok(None);
    };

    #[derive(Deserialize, Debug)]
    struct VersionData {
        protocol_version: Option<i32>,
    }

    let mut data = vec![];
    zip.reader_with_entry(entry_index)
        .await?
        .read_to_end_checked(&mut data)
        .await?;
    let data: VersionData = serde_json::from_slice(&data)?;

    Ok(data.protocol_version)
}

#[tracing::instrument(skip_all)]
#[allow(clippy::too_many_arguments)]
pub async fn launch_minecraft(
    java_args: &[String],
    env_args: &[(String, String)],
    mc_set_options: &[(String, String)],
    wrapper: &Option<String>,
    memory: &st::MemorySettings,
    resolution: &st::WindowSize,
    credentials: &Credentials,
    post_exit_hook: Option<String>,
    profile: &Profile,
    quick_play_type: &QuickPlayType,
) -> crate::Result<ProcessMetadata> {
    if profile.install_stage == ProfileInstallStage::PackInstalling
        || profile.install_stage == ProfileInstallStage::MinecraftInstalling
    {
        return Err(crate::ErrorKind::LauncherError(
            "Profile is still installing".to_string(),
        )
        .into());
    }

    if profile.install_stage != ProfileInstallStage::Installed {
        install_minecraft(profile, None, false).await?;
    }

    let state = State::get().await?;

    let instance_path =
        crate::api::profile::get_full_path(&profile.path).await?;

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
    let minecraft_updated = version_index
        <= minecraft
            .versions
            .iter()
            .position(|x| x.id == "22w16a")
            .unwrap_or(0);

    let loader_version = get_loader_version_from_profile(
        &profile.game_version,
        profile.loader,
        profile.loader_version.as_deref(),
    )
    .await?;

    if profile.loader != ModLoader::Vanilla && loader_version.is_none() {
        return Err(crate::ErrorKind::LauncherError(format!(
            "No loader version selected for {}",
            profile.loader.as_str()
        ))
        .into());
    }

    let version_jar =
        loader_version.as_ref().map_or(version.id.clone(), |it| {
            format!("{}-{}", version.id.clone(), it.id.clone())
        });

    let mut version_info = download::download_version_info(
        &state,
        version,
        loader_version.as_ref(),
        None,
        None,
    )
    .await?;
    if version_info.logging.is_none() {
        let requires_logging_info = version_index
            <= minecraft
                .versions
                .iter()
                .position(|x| x.id == "13w39a")
                .unwrap_or(0);
        if requires_logging_info {
            version_info = download::download_version_info(
                &state,
                version,
                loader_version.as_ref(),
                Some(true),
                None,
            )
            .await?;
        }
    }

    download_log_config(&state, &version_info, None, false).await?;

    let java_version = get_java_version_from_profile(profile, &version_info)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::LauncherError(
                "Missing correct java installation".to_string(),
            )
        })?;

    // Test jre version
    let java_version =
        crate::api::jre::check_jre(java_version.path.clone().into()).await?;

    let client_path = state
        .directories
        .version_dir(&version_jar)
        .join(format!("{version_jar}.jar"));

    let args = version_info.arguments.clone().unwrap_or_default();
    let mut command = match wrapper {
        Some(hook) => {
            let mut command = Command::new(hook);
            command.arg(&java_version.path);
            command
        }
        None => Command::new(&java_version.path),
    };

    let env_args = Vec::from(env_args);

    // Check if profile has a running profile, and reject running the command if it does
    // Done late so a quick double call doesn't launch two instances
    let existing_processes =
        process::get_by_profile_path(&profile.path).await?;
    if let Some(process) = existing_processes.first() {
        return Err(crate::ErrorKind::LauncherError(format!(
            "Profile {} is already running at path: {}",
            profile.path, process.uuid
        ))
        .as_error());
    }

    let natives_dir = state.directories.version_natives_dir(&version_jar);
    if !natives_dir.exists() {
        io::create_dir_all(&natives_dir).await?;
    }

    let (main_class_keep_alive, main_class_path) =
        get_resource_file!(env "JAVA_JARS_DIR" / "theseus.jar")?;

    command.args(
        args::get_jvm_arguments(
            args.get(&d::minecraft::ArgumentType::Jvm)
                .map(|x| x.as_slice()),
            &natives_dir,
            &state.directories.libraries_dir(),
            &state.directories.log_configs_dir(),
            &args::get_class_paths(
                &state.directories.libraries_dir(),
                version_info.libraries.as_slice(),
                &[&main_class_path, &client_path],
                &java_version.architecture,
                minecraft_updated,
            )?,
            &version_jar,
            *memory,
            Vec::from(java_args),
            &java_version.architecture,
            quick_play_type,
            version_info
                .logging
                .as_ref()
                .and_then(|x| x.get(&LoggingSide::Client)),
        )?
        .into_iter(),
    );

    // The java launcher code requires internal JDK code in Java 25+ in order to support JEP 512
    if java_version.parsed_version >= 25 {
        command.arg("--add-opens=jdk.internal/jdk.internal.misc=ALL-UNNAMED");
    }

    command
        .arg("com.modrinth.theseus.MinecraftLaunch")
        .arg(version_info.main_class.clone())
        .args(
            args::get_minecraft_arguments(
                args.get(&d::minecraft::ArgumentType::Game)
                    .map(|x| x.as_slice()),
                version_info.minecraft_arguments.as_deref(),
                credentials,
                &version.id,
                &version_info.asset_index.id,
                &instance_path,
                &state.directories.assets_dir(),
                &version.type_,
                *resolution,
                &java_version.architecture,
                quick_play_type,
            )?
            .into_iter(),
        )
        .current_dir(instance_path.clone());

    // CARGO-set DYLD_LIBRARY_PATH breaks Minecraft on macOS during testing on playground
    #[cfg(target_os = "macos")]
    if std::env::var("CARGO").is_ok() {
        command.env_remove("DYLD_FALLBACK_LIBRARY_PATH");
    }
    // Java options should be set in instance options (the existence of _JAVA_OPTIONS overwites them)
    command.env_remove("_JAVA_OPTIONS");

    command.envs(env_args);

    // Overwrites the minecraft options.txt file with the settings from the profile
    // Uses 'a:b' syntax which is not quite yaml
    if !mc_set_options.is_empty() {
        let options_path = instance_path.join("options.txt");

        let (mut options_string, input_encoding) = if options_path.exists() {
            io::read_any_encoding_to_string(&options_path).await?
        } else {
            (String::new(), encoding_rs::UTF_8)
        };

        // UTF-16 encodings may be successfully detected and read, but we cannot encode
        // them back, and it's technically possible that the game client strongly expects
        // such encoding
        if input_encoding != input_encoding.output_encoding() {
            return Err(crate::ErrorKind::LauncherError(format!(
                "The instance options.txt file uses an unsupported encoding: {}. \
                Please either turn off instance options that need to modify this file, \
                or convert the file to an encoding that both the game and this app support, \
                such as UTF-8.",
                input_encoding.name()
            ))
            .into());
        }

        for (key, value) in mc_set_options {
            let re = Regex::new(&format!(r"(?m)^{}:.*$", regex::escape(key)))?;
            // check if the regex exists in the file
            if !re.is_match(&options_string) {
                // The key was not found in the file, so append it
                write!(&mut options_string, "\n{key}:{value}").unwrap();
            } else {
                let replaced_string = re
                    .replace_all(&options_string, &format!("{key}:{value}"))
                    .to_string();
                options_string = replaced_string;
            }
        }

        io::write(&options_path, input_encoding.encode(&options_string).0)
            .await?;
    }

    crate::api::profile::edit(&profile.path, |prof| {
        prof.last_played = Some(Utc::now());

        async { Ok(()) }
    })
    .await?;

    // If in tauri, and the 'minimize on launch' setting is enabled, minimize the window
    #[cfg(feature = "tauri")]
    {
        use crate::EventState;

        let window = EventState::get_main_window().await?;
        if let Some(window) = window {
            let settings = crate::state::Settings::get(&state.pool).await?;
            if settings.hide_on_process_start {
                window.minimize()?;
            }
        }
    }

    let _ = state
        .discord_rpc
        .set_activity(&format!("Playing {}", profile.name), true)
        .await;

    let _ = state
        .friends_socket
        .update_status(Some(profile.name.clone()))
        .await;

    // Create Minecraft child by inserting it into the state
    // This also spawns the process and prepares the subsequent processes
    state
        .process_manager
        .insert_new_process(
            &profile.path,
            command,
            post_exit_hook,
            state.directories.profile_logs_dir(&profile.path),
            version_info.logging.is_some(),
            main_class_keep_alive,
            async |process: &ProcessMetadata, stdin| {
                let process_start_time = process.start_time.to_rfc3339();
                let profile_created_time = profile.created.to_rfc3339();
                let profile_modified_time = profile.modified.to_rfc3339();
                let system_properties = [
                    ("modrinth.process.startTime", Some(&process_start_time)),
                    ("modrinth.profile.created", Some(&profile_created_time)),
                    ("modrinth.profile.icon", profile.icon_path.as_ref()),
                    (
                        "modrinth.profile.link.project",
                        profile.linked_data.as_ref().map(|x| &x.project_id),
                    ),
                    (
                        "modrinth.profile.link.version",
                        profile.linked_data.as_ref().map(|x| &x.version_id),
                    ),
                    ("modrinth.profile.modified", Some(&profile_modified_time)),
                    ("modrinth.profile.name", Some(&profile.name)),
                ];
                for (key, value) in system_properties {
                    let Some(value) = value else {
                        continue;
                    };
                    stdin.write_all(b"property\t").await?;
                    stdin.write_all(key.as_bytes()).await?;
                    stdin.write_u8(b'\t').await?;
                    stdin.write_all(value.as_bytes()).await?;
                    stdin.write_u8(b'\n').await?;
                }
                stdin.write_all(b"launch\n").await?;
                stdin.flush().await?;
                Ok(())
            },
        )
        .await
}
